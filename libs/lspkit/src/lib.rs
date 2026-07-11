//! Pure LSP client infrastructure: spawn a language server, resolve positions to
//! semantic facts (definitions, references, call hierarchy, hover). No dependency
//! on any review/diff-orchestration domain layer.

mod composite;
mod native;
mod transport;

use std::collections::HashMap;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to spawn language server")]
    Spawn(#[from] std::io::Error),

    #[error("Language server exited before responding")]
    ServerExited,

    #[error("Malformed LSP message: {0}")]
    Protocol(String),

    #[error("Server returned error {code}: {message}")]
    ServerError { code: i64, message: String },

    #[error("Request timed out after {0:?}")]
    Timeout(std::time::Duration),

    #[error("No symbol found at {path}:{line}:{character}")]
    NoSymbolAt {
        path: PathBuf,
        line: u32,
        character: u32,
    },
}

pub type Result<T> = std::result::Result<T, Error>;

// ---- Core position/location types (1-based, matching editor conventions) ----

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    pub path: PathBuf,
    pub range: Range,
}

/// A file + position, the common input shape for nearly every query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileLocation {
    pub path: PathBuf,
    pub position: Position,
}

// ---- Symbol data ----

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Method,
    Struct,
    Enum,
    EnumMember,
    Field,
    Interface,
    Module,
    TypeParameter,
    /// impl blocks, per rust-analyzer's own labeling
    Object,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentSymbol {
    pub name: String,
    pub kind: SymbolKind,
    /// resolved signature/type, e.g. "fn(path: &str) -> Self"
    pub detail: Option<String>,
    pub range: Range,
    pub children: Vec<DocumentSymbol>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hover {
    pub signature: String,
    pub docs: Option<String>,
}

/// Returned by `prepare_call_hierarchy`; feed it into `incoming_calls`/`outgoing_calls`.
/// Carries enough identity for the server to resolve it back without the caller
/// re-deriving a position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallHierarchyItem {
    pub name: String,
    pub kind: SymbolKind,
    pub location: Location,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallSite {
    pub item: CallHierarchyItem,
    /// where in `item` the call actually happens
    pub call_ranges: Vec<Range>,
}

// ---- Composite result types ----

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeekResult {
    pub location: Location,
    pub signature: String,
    pub docs: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlastRadius {
    pub callers: Vec<CallSite>,
    pub callees: Vec<CallSite>,
    pub other_references: Vec<Location>,
    pub total_impact: usize,
}

// ---- LspClient ----

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Safety net only. rust-analyzer sends `experimental/serverStatus` for
/// every workspace we've observed once the client advertises
/// `serverStatusNotification`, so `start` normally returns as soon as the
/// server reports `quiescent: true` — this bound just stops a future/odd
/// server version that never sends the notification from hanging `start`
/// forever.
const INDEXING_FALLBACK_TIMEOUT: Duration = Duration::from_secs(60);

type PendingRequests = Mutex<HashMap<i64, Sender<serde_json::Value>>>;

/// Tracks rust-analyzer's `experimental/serverStatus` notifications so
/// `start` can block until the server reports `quiescent: true` — its "all
/// background analysis done" signal. Counting `$/progress` streams instead
/// is racy: the phases (`Fetching`, `Roots Scanned`, `cachePriming`, ...)
/// run back-to-back with gaps between one stream's `end` and the next's
/// `begin`, so "a stream ended and none are outstanding" can be observed
/// ~500ms after spawn while indexing hasn't actually started — every
/// subsequent `references()` then answers from an empty index.
#[derive(Default)]
struct IndexingState {
    /// Latest `quiescent` value reported by `experimental/serverStatus`.
    quiescent: bool,
}

type IndexingSignal = Arc<(Mutex<IndexingState>, Condvar)>;

pub struct LspClient {
    #[allow(dead_code)]
    workspace_root: PathBuf,
    process: Child,
    stdin: Mutex<BufWriter<std::process::ChildStdin>>,
    next_id: AtomicI64,
    pending: Arc<PendingRequests>,
    _reader_thread: JoinHandle<()>,
}

impl LspClient {
    /// Spawns the server and blocks through the initialize handshake.
    pub fn start(workspace_root: &Path) -> Result<Self> {
        let mut process = Command::new("rust-analyzer")
            .current_dir(workspace_root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        let stdin = process.stdin.take().ok_or(Error::ServerExited)?;
        let stdout = process.stdout.take().ok_or(Error::ServerExited)?;

        let pending: Arc<PendingRequests> = Arc::new(Mutex::new(HashMap::new()));
        let indexing: IndexingSignal =
            Arc::new((Mutex::new(IndexingState::default()), Condvar::new()));
        let reader_thread = {
            let pending = Arc::clone(&pending);
            let indexing = Arc::clone(&indexing);
            let mut reader = BufReader::new(stdout);
            std::thread::spawn(move || {
                while let Ok(message) = transport::read_message(&mut reader) {
                    let Some(id) = message.get("id").and_then(serde_json::Value::as_i64) else {
                        // Notifications and server-initiated requests (e.g.
                        // window/logMessage, client/registerCapability) carry
                        // no response we need to dispatch.
                        // `experimental/serverStatus` is the exception: its
                        // `quiescent` flag is how rust-analyzer tells us all
                        // background analysis is done, so `start` can wait on
                        // it instead of guessing with a timeout.
                        if message.get("method").and_then(serde_json::Value::as_str)
                            == Some("experimental/serverStatus")
                        {
                            track_server_status(&indexing, &message);
                        }
                        continue;
                    };
                    if let Some(sender) = pending.lock().unwrap().remove(&id) {
                        let _ = sender.send(message);
                    }
                }
            })
        };

        let stdin = Mutex::new(BufWriter::new(stdin));
        let next_id = AtomicI64::new(0);

        let root_uri = format!("file://{}", workspace_root.display());
        let init_id = next_id.fetch_add(1, Ordering::SeqCst);
        send_request(
            &stdin,
            &pending,
            init_id,
            "initialize",
            serde_json::json!({
                "processId": std::process::id(),
                "rootUri": root_uri,
                "capabilities": {
                    "experimental": { "serverStatusNotification": true },
                    // Without this, rust-analyzer answers
                    // `textDocument/documentSymbol` with flat `SymbolInformation`
                    // entries instead of nested `DocumentSymbol` trees, and
                    // `sibling_methods` needs the nesting to find an impl
                    // block's children.
                    "textDocument": {
                        "documentSymbol": { "hierarchicalDocumentSymbolSupport": true },
                    },
                },
                // Analyze with every cargo feature enabled: code behind a
                // non-default feature gate (e.g. a `#[cfg(feature = "...")]`
                // test harness and the integration tests exercising it) is
                // otherwise cfg'd out of the crate graph entirely, and
                // `references` silently reports 0 for symbols only such code
                // uses.
                "initializationOptions": {
                    "cargo": { "features": "all" },
                },
            }),
        )?;

        {
            let mut writer = stdin.lock().unwrap();
            transport::write_message(
                &mut *writer,
                &serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "initialized",
                    "params": {},
                }),
            )?;
        }

        wait_for_indexing(&indexing);

        Ok(Self {
            workspace_root: workspace_root.to_path_buf(),
            process,
            stdin,
            next_id,
            pending,
            _reader_thread: reader_thread,
        })
    }

    /// Send a JSON-RPC request and block for its response (or a timeout).
    pub(crate) fn call(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        send_request(&self.stdin, &self.pending, id, method, params)
    }
}

/// Send one JSON-RPC request over `stdin` and block until its response
/// arrives via the background reader thread (or `REQUEST_TIMEOUT` elapses).
fn send_request(
    stdin: &Mutex<BufWriter<std::process::ChildStdin>>,
    pending: &PendingRequests,
    id: i64,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value> {
    let (tx, rx) = mpsc::channel();
    pending.lock().unwrap().insert(id, tx);

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params,
    });
    {
        let mut writer = stdin.lock().unwrap();
        transport::write_message(&mut *writer, &request)?;
    }

    let response = rx
        .recv_timeout(REQUEST_TIMEOUT)
        .map_err(|_| Error::Timeout(REQUEST_TIMEOUT))?;

    if let Some(error) = response.get("error") {
        return Err(Error::ServerError {
            code: error
                .get("code")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0),
            message: error
                .get("message")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .to_string(),
        });
    }
    Ok(response
        .get("result")
        .cloned()
        .unwrap_or(serde_json::Value::Null))
}

/// Updates `indexing` from one `experimental/serverStatus` notification.
/// The flag is level-triggered — rust-analyzer re-sends the full status on
/// every change, so `quiescent` can flip back to `false` (e.g. a workspace
/// reload) and the latest value always wins.
fn track_server_status(indexing: &IndexingSignal, message: &serde_json::Value) {
    let Some(quiescent) = message
        .pointer("/params/quiescent")
        .and_then(serde_json::Value::as_bool)
    else {
        return;
    };
    let (lock, condvar) = &**indexing;
    let mut state = lock.lock().unwrap();
    state.quiescent = quiescent;
    if quiescent {
        condvar.notify_all();
    }
}

/// Blocks until rust-analyzer reports `quiescent: true`, bounded by
/// [`INDEXING_FALLBACK_TIMEOUT`] as a safety net.
fn wait_for_indexing(indexing: &IndexingSignal) {
    let (lock, condvar) = &**indexing;
    let deadline = Instant::now() + INDEXING_FALLBACK_TIMEOUT;
    let mut state = lock.lock().unwrap();
    while !state.quiescent {
        let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
            break;
        };
        let (guard, timeout) = condvar.wait_timeout(state, remaining).unwrap();
        state = guard;
        if timeout.timed_out() {
            break;
        }
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}
