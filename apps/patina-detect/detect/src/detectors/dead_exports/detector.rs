use crate::entities::{DetectorId, Evidence, LineRange, Site, SiteRole, Symptom, SymptomId};
use lspkit::{FileLocation, Location, LspClient, Position};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use thiserror::Error;
use tree_sitter::{Node, Parser, Point};

/// `rust-analyzer` answers `initialize` before it has finished indexing the
/// crate graph (`libs/lspkit/tests/references_integration.rs` documents the
/// same race for a single call): calls made right after `LspClient::start`
/// can transiently fail outright ("file not found"), or worse, *succeed*
/// with an incomplete result (an empty `references()` that looks exactly
/// like a genuinely dead symbol, until indexing catches up a moment later).
/// A production detector issuing many calls in one run can't rely on a
/// test-level retry (each retry there restarts the whole client, racing
/// indexing again every time) — it must ride out the warm-up itself, bounded
/// by one shared deadline so indexing only has to settle once per run.
const INDEXING_WARMUP_BUDGET: Duration = Duration::from_secs(30);

/// Delay between successive polls while riding out indexing warm-up (see
/// [`INDEXING_WARMUP_BUDGET`]).
const INDEXING_POLL_INTERVAL: Duration = Duration::from_millis(300);

/// The detector id every dead-exports `Symptom`/`SymptomId` is tagged with.
pub const DETECTOR_ID: &str = "dead-exports";

#[derive(Debug, Error)]
pub enum DeadExportsError {
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

/// One `pub` function or struct field found while scanning, surviving the
/// mechanical exclusion list (trait-impl methods, derive-tagged struct
/// fields, `main`) — spec.md:158-160.
struct Candidate {
    file: PathBuf,
    qualified_name: String,
    /// 0-based tree-sitter position of the candidate's own name identifier,
    /// the position `lspkit::LspClient::references` is queried against.
    name_point: Point,
    line_range: LineRange,
}

/// Runs the dead-exports detector (spec.md:150-163) against the Rust crate
/// rooted at `root`: every `pub` function and struct field enumerated from
/// tree-sitter is checked via a real `rust-analyzer` process (`lspkit::
/// LspClient::references`) for reference sites outside its own declaration.
/// Trait-impl methods, derive-heavy struct fields, and bin entry points are
/// excluded outright (spec.md's mechanical exclusion list); a candidate
/// whose only references live in test code is tagged `test_only` rather
/// than dropped.
pub fn run_dead_exports(root: &Path) -> Result<Vec<Symptom>, DeadExportsError> {
    // `LspClient::start` builds a `file://` URI directly from `root`
    // (`libs/lspkit/src/native.rs`'s `to_file_uri`), which is malformed for
    // a relative path (e.g. the CLI's own default, `libs/diffviz-core`) —
    // canonicalize once up front so every downstream file path (candidates,
    // LSP query positions) is absolute.
    let root = fs::canonicalize(root).map_err(|source| DeadExportsError::Canonicalize {
        path: root.to_path_buf(),
        source,
    })?;
    let root = root.as_path();

    let mut candidates = Vec::new();
    for file in collect_rust_files(root)? {
        let content = fs::read_to_string(&file).map_err(|source| DeadExportsError::Read {
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

    let client = LspClient::start(root)?;
    let warmup_deadline = Instant::now() + INDEXING_WARMUP_BUDGET;
    let mut file_cache: HashMap<PathBuf, (String, tree_sitter::Tree)> = HashMap::new();
    let mut symptoms = Vec::new();

    for candidate in &candidates {
        let position = Position {
            line: candidate.name_point.row as u32 + 1,
            character: candidate.name_point.column as u32 + 1,
        };
        let at = FileLocation {
            path: candidate.file.clone(),
            position,
        };
        // A single candidate's `references()` call failing (e.g. a
        // dependency's stale rust-analyzer-side build metadata, unrelated to
        // the candidate itself — see `near_duplicate_structs::detector`'s
        // identical fix) must not abort every other candidate's evidence
        // gathering. Skip and continue rather than `?`.
        let references = match references_settled(&client, &at, warmup_deadline) {
            Ok(references) => references,
            Err(err) => {
                eprintln!(
                    "dead-exports: skipping {} — references() failed: {err}",
                    candidate.qualified_name
                );
                continue;
            }
        };

        let Some((reference_count, test_only)) = classify(&references, &mut file_cache)? else {
            // Referenced from production code outside its own declaration —
            // not a finding.
            continue;
        };

        let relative = candidate
            .file
            .strip_prefix(root)
            .unwrap_or(candidate.file.as_path())
            .to_path_buf();
        let id = SymptomId::new(
            DetectorId::new(DETECTOR_ID),
            candidate.qualified_name.as_bytes(),
        );

        symptoms.push(Symptom {
            id,
            detector: DetectorId::new(DETECTOR_ID),
            title: if test_only {
                format!("Test-only export: {}", candidate.qualified_name)
            } else {
                format!("Dead export: {}", candidate.qualified_name)
            },
            evidence: Evidence::DeadExport {
                qualified_name: candidate.qualified_name.clone(),
                reference_count,
                test_only,
            },
            sites: vec![Site {
                file: relative,
                line_ranges: vec![candidate.line_range],
                role: SiteRole::MatchSite,
                note: if test_only {
                    "Only referenced from test code".to_string()
                } else {
                    "No references found outside its own declaration".to_string()
                },
            }],
        });
    }

    symptoms.sort_by_key(|s| s.id.to_string());
    Ok(symptoms)
}

/// Polls `client.references(at, false)` until two consecutive reads agree
/// (indexing has caught up — results only grow monotonically as indexing
/// completes, never shrink) or `deadline` passes, whichever comes first. A
/// bare retry-on-error isn't enough: an in-progress index can return `Ok([])`
/// just as easily as an error, and that empty result is indistinguishable
/// from a genuinely dead symbol without this stability check. See
/// [`INDEXING_WARMUP_BUDGET`].
fn references_settled(
    client: &LspClient,
    at: &FileLocation,
    deadline: Instant,
) -> lspkit::Result<Vec<Location>> {
    let mut previous: Option<Vec<Location>> = None;
    loop {
        match client.references(at, false) {
            Ok(locations) => {
                let past_deadline = Instant::now() >= deadline;
                if previous.as_ref() == Some(&locations) || past_deadline {
                    return Ok(locations);
                }
                previous = Some(locations);
            }
            Err(err) if Instant::now() >= deadline => return Err(err),
            Err(_) => {}
        }
        std::thread::sleep(INDEXING_POLL_INTERVAL);
    }
}

/// Classifies a candidate's `references()` result into a reportable finding,
/// or `None` when it's genuinely used by production code (not a finding).
/// Zero references means the candidate is dead; references that all resolve
/// to test-context code mean it's test-only (still reported, per spec.md's
/// "production code only tests exercise is its own finding" — not dropped).
fn classify(
    references: &[Location],
    file_cache: &mut HashMap<PathBuf, (String, tree_sitter::Tree)>,
) -> Result<Option<(usize, bool)>, DeadExportsError> {
    if references.is_empty() {
        return Ok(Some((0, false)));
    }

    for reference in references {
        let point = Point {
            row: (reference.range.start.line as usize).saturating_sub(1),
            column: (reference.range.start.character as usize).saturating_sub(1),
        };
        if !is_reference_test_only(&reference.path, point, file_cache)? {
            return Ok(None);
        }
    }

    Ok(Some((references.len(), true)))
}

/// Whether the reference at `point` in `path` sits inside test-context code
/// (`#[test]`/`#[cfg(test)]`), lazily parsing and caching each referenced
/// file at most once per detector run.
fn is_reference_test_only(
    path: &Path,
    point: Point,
    file_cache: &mut HashMap<PathBuf, (String, tree_sitter::Tree)>,
) -> Result<bool, DeadExportsError> {
    if !file_cache.contains_key(path) {
        let content = fs::read_to_string(path).map_err(|source| DeadExportsError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let tree = parse_rust(&content, path)?;
        file_cache.insert(path.to_path_buf(), (content, tree));
    }
    let (content, tree) = file_cache.get(path).expect("just inserted above");
    let node = tree.root_node().descendant_for_point_range(point, point);
    Ok(node.is_some_and(|n| is_in_test_context(n, content.as_bytes())))
}

fn parse_rust(content: &str, path: &Path) -> Result<tree_sitter::Tree, DeadExportsError> {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_rust::LANGUAGE.into())?;
    parser
        .parse(content, None)
        .ok_or_else(|| DeadExportsError::Parse {
            path: path.to_path_buf(),
        })
}

/// Walks `node`'s subtree collecting one `Candidate` per `pub` free
/// function/method and `pub` struct field, applying spec.md:158-160's
/// mechanical exclusion list: `in_trait_impl` skips methods declared inside
/// an `impl Trait for Type` block (they're referenced through the trait, not
/// directly); a struct carrying any `#[derive(...)]` attribute excludes all
/// of its fields; a function named `main` (a bin entry point) is always
/// excluded.
fn collect_candidates(
    node: Node,
    source: &[u8],
    file: &Path,
    in_trait_impl: bool,
    out: &mut Vec<Candidate>,
) {
    let mut child_in_trait_impl = in_trait_impl;
    if node.kind() == "impl_item" {
        child_in_trait_impl = node.child_by_field_name("trait").is_some();
    }

    if !in_trait_impl
        && node.kind() == "function_item"
        && has_pub_visibility(node, source)
        && let Some(name_node) = node.child_by_field_name("name")
        && let Ok(name) = name_node.utf8_text(source)
        && name != "main"
    {
        out.push(Candidate {
            file: file.to_path_buf(),
            qualified_name: qualified_name(node, source),
            name_point: name_node.start_position(),
            line_range: LineRange {
                start: node.start_position().row + 1,
                end: node.end_position().row + 1,
            },
        });
    }

    if node.kind() == "struct_item"
        && !has_preceding_attribute(node, source, "derive")
        && let Some(body) = node.child_by_field_name("body")
        && body.kind() == "field_declaration_list"
    {
        let struct_name = qualified_name(node, source);
        let mut cursor = body.walk();
        for field in body.named_children(&mut cursor) {
            if field.kind() != "field_declaration" || !has_pub_visibility(field, source) {
                continue;
            }
            if let Some(field_name_node) = field.child_by_field_name("name")
                && let Ok(field_name) = field_name_node.utf8_text(source)
            {
                out.push(Candidate {
                    file: file.to_path_buf(),
                    qualified_name: format!("{struct_name}::{field_name}"),
                    name_point: field_name_node.start_position(),
                    line_range: LineRange {
                        start: field.start_position().row + 1,
                        end: field.end_position().row + 1,
                    },
                });
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_candidates(child, source, file, child_in_trait_impl, out);
    }
}

/// Whether `node` (a `function_item` or `field_declaration`) carries a
/// direct `pub` `visibility_modifier` child. `tree-sitter-rust` does not
/// expose this as a named field, so this scans direct children (mirrors
/// `diffviz-core`'s `LanguageDescriptor::extract_visibility` default).
fn has_pub_visibility(node: Node, source: &[u8]) -> bool {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "visibility_modifier" {
            return child
                .utf8_text(source)
                .is_ok_and(|text| text.starts_with("pub"));
        }
    }
    false
}

/// Whether `node` is immediately preceded (among its parent's children) by
/// an `attribute_item` whose text contains `keyword` — e.g. `"derive"` for
/// `#[derive(Debug)]` or `"test"` for `#[test]`/`#[cfg(test)]`.
fn has_preceding_attribute(node: Node, source: &[u8], keyword: &str) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    let mut cursor = parent.walk();
    let siblings: Vec<Node> = parent.children(&mut cursor).collect();
    let Some(index) = siblings.iter().position(|s| s.id() == node.id()) else {
        return false;
    };
    for sibling in siblings[..index].iter().rev() {
        if sibling.kind() != "attribute_item" {
            break;
        }
        if sibling
            .utf8_text(source)
            .is_ok_and(|text| text.contains(keyword))
        {
            return true;
        }
    }
    false
}

/// Whether `node` (or any ancestor) is immediately preceded by a `#[test]`
/// or `#[cfg(test)]` attribute — covers both a directly-annotated `#[test]`
/// function and a function nested inside a `#[cfg(test)] mod tests { ... }`.
fn is_in_test_context(node: Node, source: &[u8]) -> bool {
    let mut current = node;
    loop {
        if has_preceding_attribute(current, source, "test") {
            return true;
        }
        match current.parent() {
            Some(parent) => current = parent,
            None => return false,
        }
    }
}

/// Container-qualified name (mod/impl/trait ancestor names + own name),
/// re-derived locally per `data_clumps`/`type2_clones`'s precedent rather
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

fn collect_rust_files(root: &Path) -> Result<Vec<PathBuf>, DeadExportsError> {
    let mut files = Vec::new();
    visit(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn visit(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), DeadExportsError> {
    let entries = fs::read_dir(dir).map_err(|source| DeadExportsError::Walk {
        path: dir.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| DeadExportsError::Walk {
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

    fn names(candidates: &[Candidate]) -> Vec<&str> {
        candidates
            .iter()
            .map(|c| c.qualified_name.as_str())
            .collect()
    }

    #[test]
    fn a_pub_free_function_is_a_candidate() {
        let candidates = candidates_for("pub fn foo() {}");
        assert_eq!(names(&candidates), vec!["foo"]);
    }

    #[test]
    fn a_private_function_is_not_a_candidate() {
        let candidates = candidates_for("fn foo() {}");
        assert!(candidates.is_empty());
    }

    #[test]
    fn a_function_named_main_is_excluded_even_when_pub() {
        let candidates = candidates_for("pub fn main() {}");
        assert!(candidates.is_empty());
    }

    #[test]
    fn a_trait_impl_method_is_excluded() {
        let candidates = candidates_for(
            "trait Greeter { fn greet(&self) -> String; }\n\
             struct Thing;\n\
             impl Greeter for Thing { fn greet(&self) -> String { \"hi\".to_string() } }",
        );
        assert!(names(&candidates).iter().all(|n| !n.contains("greet")));
    }

    #[test]
    fn an_inherent_impl_method_is_a_candidate() {
        let candidates =
            candidates_for("struct Thing;\nimpl Thing { pub fn make() -> Thing { Thing } }");
        assert_eq!(names(&candidates), vec!["Thing::make"]);
    }

    #[test]
    fn a_pub_field_on_a_derive_tagged_struct_is_excluded() {
        let candidates = candidates_for("#[derive(Debug)]\npub struct Config { pub name: String }");
        assert!(candidates.is_empty());
    }

    #[test]
    fn a_pub_field_on_a_plain_struct_is_a_candidate() {
        let candidates = candidates_for("pub struct Config { pub name: String }");
        assert_eq!(names(&candidates), vec!["Config::name"]);
    }

    #[test]
    fn a_private_field_is_not_a_candidate() {
        let candidates = candidates_for("pub struct Config { name: String }");
        assert!(candidates.is_empty());
    }

    fn first_function_item(content: &str) -> tree_sitter::Tree {
        parse_rust(content, Path::new("f.rs")).unwrap()
    }

    fn find_function_item(node: Node) -> Node {
        if node.kind() == "function_item" {
            return node;
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "function_item" {
                return child;
            }
        }
        panic!("no function_item found")
    }

    #[test]
    fn a_function_directly_annotated_test_is_in_test_context() {
        let content = "#[test]\nfn checks_something() { assert!(true); }";
        let tree = first_function_item(content);
        let node = find_function_item(tree.root_node());
        assert!(is_in_test_context(node, content.as_bytes()));
    }

    #[test]
    fn a_function_outside_any_test_module_is_not_in_test_context() {
        let content = "fn production_code() {}";
        let tree = first_function_item(content);
        let node = find_function_item(tree.root_node());
        assert!(!is_in_test_context(node, content.as_bytes()));
    }

    #[test]
    fn classify_with_no_references_is_dead() {
        let mut cache = HashMap::new();
        let result = classify(&[], &mut cache).unwrap();
        assert_eq!(result, Some((0, false)));
    }
}
