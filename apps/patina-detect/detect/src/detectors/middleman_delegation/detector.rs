use crate::entities::{DetectorId, Evidence, LineRange, Site, SiteRole, Symptom, SymptomId};
use lspkit::{CallHierarchyItem, CallSite, FileLocation, LspClient, Position};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use thiserror::Error;
use tree_sitter::{Node, Parser, Point};

/// Same rust-analyzer indexing-warmup concern as `dead_exports` (see its
/// `INDEXING_WARMUP_BUDGET` doc) — a production run issuing many
/// `prepare_call_hierarchy`/`incoming_calls` requests must ride out warm-up
/// itself, bounded by one shared deadline per run.
const INDEXING_WARMUP_BUDGET: Duration = Duration::from_secs(30);

const INDEXING_POLL_INTERVAL: Duration = Duration::from_millis(300);

/// The detector id every middleman-delegation `Symptom`/`SymptomId` is
/// tagged with.
pub const DETECTOR_ID: &str = "middleman-delegation";

#[derive(Debug, Error)]
pub enum MiddlemanDelegationError {
    #[error("failed to resolve {path} to an absolute path")]
    Canonicalize {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to walk directory {path}")]
    Walk {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to read file {path}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to configure tree-sitter Rust grammar")]
    Language(#[from] tree_sitter::LanguageError),

    #[error("failed to parse {path} as Rust")]
    Parse { path: PathBuf },

    #[error("language server error")]
    Lsp(#[from] lspkit::Error),
}

/// One free function or inherent-impl method found while scanning.
/// `delegate_target` is the simple name of the function/method its body
/// calls, populated only when the whole body is a single delegating call
/// (spec.md:170) — everything else about the candidate is collected
/// regardless, since non-delegating candidates are still needed to resolve
/// other candidates' delegate targets and callers.
struct Candidate {
    file: PathBuf,
    qualified_name: String,
    simple_name: String,
    /// 0-based tree-sitter position of the candidate's own name identifier,
    /// the position `lspkit::LspClient::prepare_call_hierarchy` is queried
    /// against.
    name_point: Point,
    line_range: LineRange,
    delegate_target: Option<String>,
}

/// Runs the middleman-delegation detector (spec.md:165-177) against the Rust
/// crate rooted at `root`: tree-sitter finds functions whose body is a
/// single delegating call (excluding trait-impl and trait-declared methods,
/// which may be satisfying an interface); `lspkit::LspClient::incoming_calls`
/// confirms each has exactly one same-crate caller. Confirmed middlemen that
/// delegate into one another are composed into a single chain per
/// spec.md:176 ("the chain (A → B → C) when middlemen compose").
pub fn run_middleman_delegation(root: &Path) -> Result<Vec<Symptom>, MiddlemanDelegationError> {
    // Same rationale as `dead_exports::run_dead_exports`: `LspClient::start`
    // builds a `file://` URI directly from `root`, which is malformed for a
    // relative path — canonicalize once up front.
    let root = fs::canonicalize(root).map_err(|source| MiddlemanDelegationError::Canonicalize {
        path: root.to_path_buf(),
        source,
    })?;
    let root = root.as_path();

    let mut candidates = Vec::new();
    for file in collect_rust_files(root)? {
        let content =
            fs::read_to_string(&file).map_err(|source| MiddlemanDelegationError::Read {
                path: file.clone(),
                source,
            })?;
        let tree = parse_rust(&content, &file)?;
        collect_candidates(
            tree.root_node(),
            content.as_bytes(),
            &file,
            false,
            &mut candidates,
        );
    }

    let mut by_simple_name: HashMap<&str, Vec<usize>> = HashMap::new();
    for (index, candidate) in candidates.iter().enumerate() {
        by_simple_name
            .entry(candidate.simple_name.as_str())
            .or_default()
            .push(index);
    }

    let client = LspClient::start(root)?;
    let warmup_deadline = Instant::now() + INDEXING_WARMUP_BUDGET;

    let mut confirmed: HashMap<usize, CallSite> = HashMap::new();
    for (index, candidate) in candidates.iter().enumerate() {
        if candidate.delegate_target.is_none() {
            continue;
        }
        let position = Position {
            line: candidate.name_point.row as u32 + 1,
            character: candidate.name_point.column as u32 + 1,
        };
        let at = FileLocation {
            path: candidate.file.clone(),
            position,
        };
        let items = prepare_call_hierarchy_settled(&client, &at, warmup_deadline)?;
        let Some(item) = items.into_iter().next() else {
            continue;
        };
        let callers = incoming_calls_settled(&client, &item, warmup_deadline)?;
        let same_crate_callers: Vec<CallSite> = callers
            .into_iter()
            .filter(|call_site| call_site.item.location.path.starts_with(root))
            .collect();
        if same_crate_callers.len() == 1 {
            confirmed.insert(index, same_crate_callers.into_iter().next().unwrap());
        }
    }

    // Edges among confirmed middlemen only: a confirmed middleman's
    // unambiguously-resolved delegate target, when that target is itself
    // confirmed, composes the chain (spec.md:176).
    let mut edge: HashMap<usize, usize> = HashMap::new();
    for &index in confirmed.keys() {
        let target_name = candidates[index]
            .delegate_target
            .as_ref()
            .expect("confirmed candidates always have a delegate_target");
        if let Some(target_indices) = by_simple_name.get(target_name.as_str())
            && let [target_index] = target_indices[..]
            && confirmed.contains_key(&target_index)
        {
            edge.insert(index, target_index);
        }
    }

    let chained: HashSet<usize> = edge.values().copied().collect();
    let mut heads: Vec<usize> = confirmed
        .keys()
        .copied()
        .filter(|index| !chained.contains(index))
        .collect();
    heads.sort();

    let mut symptoms = Vec::new();
    for head in heads {
        let mut chain_indices = vec![head];
        let mut current = head;
        while let Some(&next) = edge.get(&current) {
            chain_indices.push(next);
            current = next;
        }

        let chain_names: Vec<String> = chain_indices
            .iter()
            .map(|&index| candidates[index].qualified_name.clone())
            .collect();
        let mut fingerprint_parts = chain_names.clone();
        fingerprint_parts.sort();
        let fingerprint = fingerprint_parts.join("|");
        let id = SymptomId::new(DetectorId::new(DETECTOR_ID), fingerprint.as_bytes());

        let caller = confirmed
            .get(&head)
            .expect("head is always a key of confirmed");
        let caller_relative = caller
            .item
            .location
            .path
            .strip_prefix(root)
            .unwrap_or(caller.item.location.path.as_path())
            .to_path_buf();
        let mut sites = vec![Site {
            file: caller_relative,
            line_ranges: vec![LineRange {
                start: caller.item.location.range.start.line as usize,
                end: caller.item.location.range.end.line as usize,
            }],
            role: SiteRole::Caller,
            note: format!("Sole caller of {}", candidates[head].qualified_name),
        }];
        for &index in &chain_indices {
            let candidate = &candidates[index];
            let relative = candidate
                .file
                .strip_prefix(root)
                .unwrap_or(candidate.file.as_path())
                .to_path_buf();
            sites.push(Site {
                file: relative,
                line_ranges: vec![candidate.line_range],
                role: SiteRole::MatchSite,
                note: format!(
                    "Delegates to {}",
                    candidate.delegate_target.as_deref().unwrap_or("<unknown>")
                ),
            });
        }

        symptoms.push(Symptom {
            id,
            detector: DetectorId::new(DETECTOR_ID),
            title: if chain_names.len() > 1 {
                format!("Middleman delegation chain: {}", chain_names.join(" -> "))
            } else {
                format!("Middleman delegation: {}", chain_names[0])
            },
            evidence: Evidence::MiddlemanChain {
                chain: chain_names,
                caller_count: 1,
                body_shape: "single delegating call".to_string(),
            },
            sites,
        });
    }

    symptoms.sort_by_key(|s| s.id.to_string());
    Ok(symptoms)
}

/// Polls `client.prepare_call_hierarchy(at)` until two consecutive reads
/// agree or `deadline` passes — same stability rationale as
/// `dead_exports::references_settled`.
fn prepare_call_hierarchy_settled(
    client: &LspClient,
    at: &FileLocation,
    deadline: Instant,
) -> lspkit::Result<Vec<CallHierarchyItem>> {
    let mut previous: Option<Vec<CallHierarchyItem>> = None;
    loop {
        match client.prepare_call_hierarchy(at) {
            Ok(items) => {
                let past_deadline = Instant::now() >= deadline;
                if previous.as_ref() == Some(&items) || past_deadline {
                    return Ok(items);
                }
                previous = Some(items);
            }
            Err(err) if Instant::now() >= deadline => return Err(err),
            Err(_) => {}
        }
        std::thread::sleep(INDEXING_POLL_INTERVAL);
    }
}

/// Polls `client.incoming_calls(item)` until two consecutive reads agree or
/// `deadline` passes — same stability rationale as
/// `dead_exports::references_settled`.
fn incoming_calls_settled(
    client: &LspClient,
    item: &CallHierarchyItem,
    deadline: Instant,
) -> lspkit::Result<Vec<CallSite>> {
    let mut previous: Option<Vec<CallSite>> = None;
    loop {
        match client.incoming_calls(item) {
            Ok(call_sites) => {
                let past_deadline = Instant::now() >= deadline;
                if previous.as_ref() == Some(&call_sites) || past_deadline {
                    return Ok(call_sites);
                }
                previous = Some(call_sites);
            }
            Err(err) if Instant::now() >= deadline => return Err(err),
            Err(_) => {}
        }
        std::thread::sleep(INDEXING_POLL_INTERVAL);
    }
}

fn parse_rust(content: &str, path: &Path) -> Result<tree_sitter::Tree, MiddlemanDelegationError> {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_rust::LANGUAGE.into())?;
    parser
        .parse(content, None)
        .ok_or_else(|| MiddlemanDelegationError::Parse {
            path: path.to_path_buf(),
        })
}

/// Walks `node`'s subtree collecting one `Candidate` per free function and
/// inherent-impl method. `in_trait_context` skips both trait-declared
/// methods (inside a `trait_item`) and trait-impl methods (inside an
/// `impl Trait for Type` block) — spec.md:172's exclusion, since either may
/// be satisfying an interface rather than being a pointless wrapper.
fn collect_candidates(
    node: Node,
    source: &[u8],
    file: &Path,
    in_trait_context: bool,
    out: &mut Vec<Candidate>,
) {
    let mut child_in_trait_context = in_trait_context;
    if node.kind() == "trait_item" {
        child_in_trait_context = true;
    } else if node.kind() == "impl_item" {
        child_in_trait_context = node.child_by_field_name("trait").is_some();
    }

    if !in_trait_context
        && node.kind() == "function_item"
        && let Some(name_node) = node.child_by_field_name("name")
        && let Ok(name) = name_node.utf8_text(source)
    {
        let delegate_target = node
            .child_by_field_name("body")
            .and_then(|body| body_delegate_target(body, source));
        out.push(Candidate {
            file: file.to_path_buf(),
            qualified_name: qualified_name(node, source),
            simple_name: name.to_string(),
            name_point: name_node.start_position(),
            line_range: LineRange {
                start: node.start_position().row + 1,
                end: node.end_position().row + 1,
            },
            delegate_target,
        });
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_candidates(child, source, file, child_in_trait_context, out);
    }
}

/// Whether `body` (a function's `block`) consists of exactly one statement —
/// a bare call expression tail, a `return <call>;`, or `<call>;` — and, if
/// so, whether that call's own subtree (receiver/arguments, including
/// inside any closures they carry) contains no *other* call — i.e. the body
/// is a single, unadorned pass-through, not a combinator/builder chain
/// (`.iter().filter(f).map(g).collect()`) that merely happens to present
/// one `call_expression` at the top. Revision (contribution 005, decision
/// 1): the original check only looked at the outer node's kind and missed
/// this whole FP class, confirmed against real code in this repo (not just
/// a synthetic case) — see spec.md:170's "single delegating call".
fn body_delegate_target(body: Node, source: &[u8]) -> Option<String> {
    let mut cursor = body.walk();
    let named: Vec<Node> = body.named_children(&mut cursor).collect();
    let [only] = named[..] else {
        return None;
    };

    let mut call = match only.kind() {
        "expression_statement" | "return_expression" => only.named_child(0)?,
        _ => only,
    };
    if call.kind() == "return_expression" {
        call = call.named_child(0)?;
    }
    if call.kind() != "call_expression" {
        return None;
    }
    if count_calls(call) != 1 {
        return None;
    }

    let function = call.child_by_field_name("function")?;
    let text = function.utf8_text(source).ok()?;
    Some(simple_call_name(text))
}

/// Counts `call_expression` nodes in `node`'s subtree, `node` included —
/// used to tell a genuine single pass-through call (count 1) from a
/// combinator/builder chain whose receiver or arguments are themselves
/// built from further calls (count > 1). Descends into closure bodies too,
/// since a computed predicate/mapper (`.filter(|x| x.is_valid())`) is
/// exactly the kind of real work a delegating wrapper shouldn't be doing.
fn count_calls(node: Node) -> usize {
    let mut count = if node.kind() == "call_expression" {
        1
    } else {
        0
    };
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        count += count_calls(child);
    }
    count
}

/// Reduces a call target's text (`bar`, `self.inner.bar`, `Type::bar`) to
/// its final segment, the name candidates are indexed by.
fn simple_call_name(text: &str) -> String {
    text.rsplit(['.', ':'])
        .next()
        .unwrap_or(text)
        .trim()
        .to_string()
}

/// Container-qualified name (mod/impl/trait ancestor names + own name),
/// re-derived locally per `data_clumps`/`dead_exports`'s precedent rather
/// than depending on `diffviz-core`'s `SemanticTree` pipeline.
fn qualified_name(node: Node, source: &[u8]) -> String {
    let mut parts = Vec::new();
    let mut current = node.parent();
    while let Some(p) = current {
        let name = match p.kind() {
            "mod_item" | "trait_item" => p.child_by_field_name("name"),
            "impl_item" => p.child_by_field_name("type"),
            _ => None,
        };
        if let Some(name) = name.and_then(|n| n.utf8_text(source).ok()) {
            parts.push(name.to_string());
        }
        current = p.parent();
    }
    parts.reverse();

    let own_name = node
        .child_by_field_name("name")
        .and_then(|n| n.utf8_text(source).ok())
        .unwrap_or("<anonymous>");
    parts.push(own_name.to_string());
    parts.join("::")
}

fn collect_rust_files(root: &Path) -> Result<Vec<PathBuf>, MiddlemanDelegationError> {
    let mut files = Vec::new();
    visit(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn visit(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), MiddlemanDelegationError> {
    let entries = fs::read_dir(dir).map_err(|source| MiddlemanDelegationError::Walk {
        path: dir.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| MiddlemanDelegationError::Walk {
            path: dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() {
            visit(&path, files)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidates_for(content: &str) -> Vec<Candidate> {
        let tree = parse_rust(content, Path::new("f.rs")).unwrap();
        let mut out = Vec::new();
        collect_candidates(
            tree.root_node(),
            content.as_bytes(),
            Path::new("f.rs"),
            false,
            &mut out,
        );
        out
    }

    #[test]
    fn a_free_function_that_bare_tail_calls_another_is_a_delegate() {
        let candidates = candidates_for("fn foo() { bar() }");
        assert_eq!(candidates[0].delegate_target.as_deref(), Some("bar"));
    }

    #[test]
    fn a_free_function_that_explicitly_returns_a_call_is_a_delegate() {
        let candidates = candidates_for("fn foo() { return bar(); }");
        assert_eq!(candidates[0].delegate_target.as_deref(), Some("bar"));
    }

    #[test]
    fn a_free_function_with_a_semicolon_terminated_call_is_a_delegate() {
        let candidates = candidates_for("fn foo() { bar(); }");
        assert_eq!(candidates[0].delegate_target.as_deref(), Some("bar"));
    }

    #[test]
    fn a_method_call_delegate_resolves_to_its_final_segment() {
        let candidates = candidates_for("fn foo() { self.inner.bar() }");
        assert_eq!(candidates[0].delegate_target.as_deref(), Some("bar"));
    }

    #[test]
    fn a_function_with_more_than_one_statement_is_not_a_delegate() {
        let candidates = candidates_for("fn foo() { let x = 1; bar(x) }");
        assert_eq!(candidates[0].delegate_target, None);
    }

    #[test]
    fn a_function_doing_real_work_is_not_a_delegate() {
        let candidates = candidates_for("fn foo() -> i32 { 42 }");
        assert_eq!(candidates[0].delegate_target, None);
    }

    #[test]
    fn a_combinator_chain_is_not_a_delegate() {
        // Real FP shape found during Phase 10 real-repo verification
        // (contribution 005, decision 1): `determine_line_relevance_with_precedence`
        // -shaped body — one top-level `call_expression` (`.unwrap_or(...)`)
        // whose receiver is itself built from `.iter().filter().map().min()`.
        let candidates = candidates_for(
            "fn foo(xs: &[i32]) -> i32 { \
             xs.iter().filter(|x| **x > 0).map(|x| *x).min().unwrap_or(0) }",
        );
        assert_eq!(candidates[0].delegate_target, None);
    }

    #[test]
    fn a_builder_chain_is_not_a_delegate() {
        // Real FP shape found during Phase 10 real-repo verification:
        // `PathBuf::from(...).join(...)`-shaped body.
        let candidates = candidates_for("fn foo() -> String { String::from(\"a\").add(\"b\") }");
        assert_eq!(candidates[0].delegate_target, None);
    }

    #[test]
    fn a_call_whose_argument_is_itself_a_call_is_not_a_delegate() {
        let candidates = candidates_for("fn foo() { bar(baz()) }");
        assert_eq!(candidates[0].delegate_target, None);
    }

    #[test]
    fn a_lone_method_call_on_a_field_with_a_simple_argument_is_still_a_delegate() {
        // Must not regress the genuine hits found in the real-repo run
        // (e.g. `self.state.get_reviewable_diff(id)`).
        let candidates = candidates_for("fn foo(id: u32) -> i32 { self.state.bar(id) }");
        assert_eq!(candidates[0].delegate_target.as_deref(), Some("bar"));
    }

    #[test]
    fn a_trait_declared_method_is_excluded_entirely() {
        let candidates = candidates_for("trait Greeter { fn greet(&self) { bar() } }");
        assert!(candidates.is_empty());
    }

    #[test]
    fn a_trait_impl_method_is_excluded_entirely() {
        let candidates = candidates_for(
            "trait Greeter { fn greet(&self); }\n\
             struct Thing;\n\
             impl Greeter for Thing { fn greet(&self) { bar() } }",
        );
        assert!(candidates.is_empty());
    }

    #[test]
    fn an_inherent_impl_method_is_a_candidate() {
        let candidates = candidates_for("struct Thing;\nimpl Thing { fn make() { bar() } }");
        assert_eq!(candidates[0].qualified_name, "Thing::make");
        assert_eq!(candidates[0].delegate_target.as_deref(), Some("bar"));
    }
}
