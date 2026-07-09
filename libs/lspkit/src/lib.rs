//! Pure LSP client infrastructure: spawn a language server, resolve positions to
//! semantic facts (definitions, references, call hierarchy, hover). No dependency
//! on any review/diff-orchestration domain layer.

mod composite;
mod native;
mod transport;

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
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

pub struct LspClient {
    #[allow(dead_code)]
    workspace_root: PathBuf,
    process: Child,
}

impl LspClient {
    /// Spawns the server and blocks through the initialize handshake.
    pub fn start(workspace_root: &Path) -> Result<Self> {
        let process = Command::new("rust-analyzer")
            .current_dir(workspace_root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        // TODO: perform the `initialize`/`initialized` handshake over the
        // process's stdio and start the background reader thread that
        // dispatches responses by request id.

        Ok(Self {
            workspace_root: workspace_root.to_path_buf(),
            process,
        })
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}
