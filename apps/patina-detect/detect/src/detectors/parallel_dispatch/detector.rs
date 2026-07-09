use crate::entities::{DetectorId, Evidence, LineRange, Site, SiteRole, Symptom, SymptomId};
use lspkit::{FileLocation, Hover, Location, LspClient, Position};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use thiserror::Error;
use tree_sitter::{Node, Parser, Point};

/// Same rust-analyzer indexing-warmup concern as `dead_exports` (see its
/// `INDEXING_WARMUP_BUDGET` doc) — a production run issuing many
/// `definition`/`hover` calls right after spawn can transiently under- or
/// over-report before the crate graph finishes indexing.
const INDEXING_WARMUP_BUDGET: Duration = Duration::from_secs(30);
const INDEXING_POLL_INTERVAL: Duration = Duration::from_millis(300);

/// The detector id every parallel-dispatch `Symptom`/`SymptomId` is tagged
/// with.
pub const DETECTOR_ID: &str = "parallel-dispatch";

/// Scrutinee types this detector never groups on: std enums that legitimately
/// recur across many unrelated match sites are not the "missing polymorphism"
/// smell spec.md:212-224 targets. Checked against the type annotation's bare
/// name (pre-generic, pre-reference) before any LSP call, so a candidate
/// typed `Option<Shape>` — which isn't a plain `type_identifier` anyway — or
/// simply named `Option` never reaches `hover`/`definition`.
const STD_TYPE_DENYLIST: &[&str] = &["Option", "Result", "Cow", "Ordering"];

/// An enum matched in at least this many sites...
const MIN_SITE_COUNT: usize = 3;
/// ...spread across at least this many distinct files — spec.md's FP
/// control for this detector.
const MIN_FILE_COUNT: usize = 2;

#[derive(Debug, Error)]
pub enum ParallelDispatchError {
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

/// A `match` site whose scrutinee is a bare identifier referring to a
/// same-function parameter with an explicit, non-generic named type
/// (spec.md's "resolve scrutinee type via LSP definition/hover" — v1 scopes
/// to this common, unambiguous shape; field-access/method-call/let-bound
/// scrutinees are left for a later pass, see decision log).
struct CandidateMatchSite {
    file: PathBuf,
    line_range: LineRange,
    arm_count: usize,
    /// 0-based tree-sitter position of the parameter's type annotation — the
    /// position `definition`/`hover` are queried against, mirroring Phase
    /// 12's `describe(shape: Shape)` fixture precedent.
    type_point: Point,
    type_name: String,
}

/// Runs the parallel-dispatch detector (spec.md:212-224) against the Rust
/// crate rooted at `root`: every `match` expression whose scrutinee is a
/// same-function parameter with a plain named type is collected via
/// tree-sitter, then that type is resolved through a real `rust-analyzer`
/// process (`lspkit::LspClient::hover` to confirm it's an enum,
/// `LspClient::definition` to resolve its declaration and compute a
/// qualified name). Enums matched in `>= 3` sites across `>= 2` files are
/// reported; std types (`Option`, `Result`, ...) are excluded outright.
pub fn run_parallel_dispatch(root: &Path) -> Result<Vec<Symptom>, ParallelDispatchError> {
    // `LspClient::start` builds a `file://` URI directly from `root`, which
    // is malformed for a relative path — canonicalize once up front so every
    // downstream file path (candidates, LSP query positions) is absolute.
    let root = fs::canonicalize(root).map_err(|source| ParallelDispatchError::Canonicalize {
        path: root.to_path_buf(),
        source,
    })?;
    let root = root.as_path();

    let mut candidates = Vec::new();
    for file in collect_rust_files(root)? {
        let content = fs::read_to_string(&file).map_err(|source| ParallelDispatchError::Read {
            path: file.clone(),
            source,
        })?;
        let tree = parse_rust(&content, &file)?;
        collect_match_sites(tree.root_node(), content.as_bytes(), &file, &mut candidates);
    }

    let client = LspClient::start(root)?;
    let warmup_deadline = Instant::now() + INDEXING_WARMUP_BUDGET;
    let mut decl_cache: HashMap<PathBuf, (String, tree_sitter::Tree)> = HashMap::new();
    let mut groups: HashMap<String, Vec<(PathBuf, LineRange, usize)>> = HashMap::new();

    for candidate in &candidates {
        if STD_TYPE_DENYLIST.contains(&candidate.type_name.as_str()) {
            continue;
        }

        let position = Position {
            line: candidate.type_point.row as u32 + 1,
            character: candidate.type_point.column as u32 + 1,
        };
        let at = FileLocation {
            path: candidate.file.clone(),
            position,
        };

        // A single candidate's LSP calls failing (stale rust-analyzer-side
        // build metadata, a type from an unindexed dependency, ...) must not
        // abort every other candidate's evidence gathering — skip and
        // continue, mirroring `dead_exports`/`near_duplicate_structs`.
        let hover = match hover_settled(&client, &at, warmup_deadline) {
            Ok(hover) => hover,
            Err(err) => {
                eprintln!(
                    "parallel-dispatch: skipping {}:{} — hover() failed: {err}",
                    candidate.file.display(),
                    candidate.line_range.start
                );
                continue;
            }
        };
        let Some(hover) = hover else { continue };
        if !hover.signature.contains("enum ") {
            // Struct, primitive, or other non-enum type — not this
            // detector's target.
            continue;
        }

        let definitions = match definition_settled(&client, &at, warmup_deadline) {
            Ok(defs) => defs,
            Err(err) => {
                eprintln!(
                    "parallel-dispatch: skipping {}:{} — definition() failed: {err}",
                    candidate.file.display(),
                    candidate.line_range.start
                );
                continue;
            }
        };
        let Some(decl) = definitions.into_iter().next() else {
            continue;
        };

        let Some(qualified) = resolve_qualified_enum_name(&decl, &mut decl_cache)? else {
            continue;
        };

        groups.entry(qualified).or_default().push((
            candidate.file.clone(),
            candidate.line_range,
            candidate.arm_count,
        ));
    }

    let mut symptoms = Vec::new();
    for (qualified_name, occurrences) in groups {
        let file_count = occurrences
            .iter()
            .map(|(file, _, _)| file)
            .collect::<HashSet<_>>()
            .len();
        if occurrences.len() < MIN_SITE_COUNT || file_count < MIN_FILE_COUNT {
            continue;
        }

        let arm_counts: Vec<usize> = occurrences.iter().map(|(_, _, arms)| *arms).collect();
        let mut sites: Vec<Site> = occurrences
            .iter()
            .map(|(file, range, arms)| Site {
                file: file
                    .strip_prefix(root)
                    .unwrap_or(file.as_path())
                    .to_path_buf(),
                line_ranges: vec![*range],
                role: SiteRole::MatchSite,
                note: format!("{arms}-arm match on {qualified_name}"),
            })
            .collect();
        sites.sort_by(|a, b| {
            (&a.file, a.line_ranges[0].start).cmp(&(&b.file, b.line_ranges[0].start))
        });

        let id = SymptomId::new(DetectorId::new(DETECTOR_ID), qualified_name.as_bytes());
        symptoms.push(Symptom {
            id,
            detector: DetectorId::new(DETECTOR_ID),
            title: format!("Parallel dispatch on {qualified_name}"),
            evidence: Evidence::ParallelDispatch {
                enum_name: qualified_name,
                site_count: occurrences.len(),
                file_count,
                arm_counts,
            },
            sites,
        });
    }

    symptoms.sort_by_key(|s| s.id.to_string());
    Ok(symptoms)
}

/// Polls `client.hover(at)` until two consecutive reads agree or `deadline`
/// passes — same stability concern as `dead_exports::references_settled`,
/// adapted to hover's `Option<Hover>` shape (no monotonic-growth invariant to
/// lean on, just retry-until-stable-or-timeout).
fn hover_settled(
    client: &LspClient,
    at: &FileLocation,
    deadline: Instant,
) -> lspkit::Result<Option<Hover>> {
    let mut previous: Option<Option<Hover>> = None;
    loop {
        match client.hover(at) {
            Ok(hover) => {
                let past_deadline = Instant::now() >= deadline;
                if previous.as_ref() == Some(&hover) || past_deadline {
                    return Ok(hover);
                }
                previous = Some(hover);
            }
            Err(err) if Instant::now() >= deadline => return Err(err),
            Err(_) => {}
        }
        std::thread::sleep(INDEXING_POLL_INTERVAL);
    }
}

/// Polls `client.definition(at)` until two consecutive reads agree or
/// `deadline` passes. See [`hover_settled`].
fn definition_settled(
    client: &LspClient,
    at: &FileLocation,
    deadline: Instant,
) -> lspkit::Result<Vec<Location>> {
    let mut previous: Option<Vec<Location>> = None;
    loop {
        match client.definition(at) {
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

/// Resolves an enum's declaration `Location` to its container-qualified name
/// by re-parsing the declaring file and walking up from the declaration
/// point to the nearest `enum_item` ancestor. Returns `None` (not an error)
/// when the location can't be read/parsed (e.g. it resolves outside the
/// workspace into a dependency's vendored source) or doesn't land on an
/// enum — both are "this candidate doesn't apply", not failures.
fn resolve_qualified_enum_name(
    decl: &Location,
    cache: &mut HashMap<PathBuf, (String, tree_sitter::Tree)>,
) -> Result<Option<String>, ParallelDispatchError> {
    if !cache.contains_key(&decl.path) {
        let Ok(content) = fs::read_to_string(&decl.path) else {
            return Ok(None);
        };
        let Ok(tree) = parse_rust(&content, &decl.path) else {
            return Ok(None);
        };
        cache.insert(decl.path.clone(), (content, tree));
    }
    let (content, tree) = cache.get(&decl.path).expect("just inserted above");

    let point = Point {
        row: (decl.range.start.line as usize).saturating_sub(1),
        column: (decl.range.start.character as usize).saturating_sub(1),
    };
    let Some(node) = tree.root_node().descendant_for_point_range(point, point) else {
        return Ok(None);
    };
    let Some(enum_node) = find_ancestor_enum(node) else {
        return Ok(None);
    };
    Ok(Some(qualified_name(enum_node, content.as_bytes())))
}

fn find_ancestor_enum(node: Node) -> Option<Node> {
    let mut current = Some(node);
    while let Some(n) = current {
        if n.kind() == "enum_item" {
            return Some(n);
        }
        current = n.parent();
    }
    None
}

/// Walks `node`'s subtree collecting one [`CandidateMatchSite`] per `match`
/// expression whose scrutinee is a bare identifier bound to the immediately
/// enclosing function's parameter list. `scope` maps parameter name to its
/// type annotation's position/name and is rebuilt on entering each
/// `function_item`, so nested functions/closures never see an outer
/// function's parameters.
fn collect_match_sites(node: Node, source: &[u8], file: &Path, out: &mut Vec<CandidateMatchSite>) {
    collect_match_sites_scoped(node, source, file, &HashMap::new(), out);
}

fn collect_match_sites_scoped(
    node: Node,
    source: &[u8],
    file: &Path,
    scope: &HashMap<String, (Point, String)>,
    out: &mut Vec<CandidateMatchSite>,
) {
    let child_scope;
    let scope = if node.kind() == "function_item" {
        child_scope = parameter_types(node, source);
        &child_scope
    } else {
        scope
    };

    if node.kind() == "match_expression"
        && let Some(site) = match_site_for(node, source, file, scope)
    {
        out.push(site);
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_match_sites_scoped(child, source, file, scope, out);
    }
}

/// Extracts `(param name -> (type annotation position, type name))` for each
/// of `function_node`'s parameters carrying a plain named type
/// (`type_identifier` — excludes `&Type`, `Option<Type>`, tuples, `self`),
/// the v1 scope-narrowing this detector applies (see decision log).
fn parameter_types(function_node: Node, source: &[u8]) -> HashMap<String, (Point, String)> {
    let mut map = HashMap::new();
    let Some(params) = function_node.child_by_field_name("parameters") else {
        return map;
    };
    let mut cursor = params.walk();
    for param in params.named_children(&mut cursor) {
        if param.kind() != "parameter" {
            continue;
        }
        let Some(pattern) = param.child_by_field_name("pattern") else {
            continue;
        };
        if pattern.kind() != "identifier" {
            continue;
        }
        let Ok(name) = pattern.utf8_text(source) else {
            continue;
        };
        let Some(type_node) = param.child_by_field_name("type") else {
            continue;
        };
        if type_node.kind() != "type_identifier" {
            continue;
        }
        let Ok(type_name) = type_node.utf8_text(source) else {
            continue;
        };
        map.insert(
            name.to_string(),
            (type_node.start_position(), type_name.to_string()),
        );
    }
    map
}

fn match_site_for(
    match_node: Node,
    source: &[u8],
    file: &Path,
    scope: &HashMap<String, (Point, String)>,
) -> Option<CandidateMatchSite> {
    let scrutinee = match_node.child_by_field_name("value")?;
    if scrutinee.kind() != "identifier" {
        return None;
    }
    let name = scrutinee.utf8_text(source).ok()?;
    let (type_point, type_name) = scope.get(name)?;

    let body = match_node.child_by_field_name("body")?;
    let mut cursor = body.walk();
    let arm_count = body
        .named_children(&mut cursor)
        .filter(|arm| arm.kind() == "match_arm")
        .count();

    Some(CandidateMatchSite {
        file: file.to_path_buf(),
        line_range: LineRange {
            start: match_node.start_position().row + 1,
            end: match_node.end_position().row + 1,
        },
        arm_count,
        type_point: *type_point,
        type_name: type_name.clone(),
    })
}

/// Container-qualified name (mod/impl/trait ancestor names + own name),
/// re-derived locally per `dead_exports`/`data_clumps`/`type2_clones`'s
/// precedent rather than depending on `diffviz-core`'s `SemanticTree`
/// pipeline.
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

fn parse_rust(content: &str, path: &Path) -> Result<tree_sitter::Tree, ParallelDispatchError> {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_rust::LANGUAGE.into())?;
    parser
        .parse(content, None)
        .ok_or_else(|| ParallelDispatchError::Parse {
            path: path.to_path_buf(),
        })
}

fn collect_rust_files(root: &Path) -> Result<Vec<PathBuf>, ParallelDispatchError> {
    let mut files = Vec::new();
    visit(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn visit(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), ParallelDispatchError> {
    let entries = fs::read_dir(dir).map_err(|source| ParallelDispatchError::Walk {
        path: dir.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| ParallelDispatchError::Walk {
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

    fn matches_for(content: &str) -> Vec<CandidateMatchSite> {
        let tree = parse_rust(content, Path::new("f.rs")).unwrap();
        let mut out = Vec::new();
        collect_match_sites(
            tree.root_node(),
            content.as_bytes(),
            Path::new("f.rs"),
            &mut out,
        );
        out
    }

    #[test]
    fn a_match_on_a_plain_typed_parameter_is_a_candidate() {
        let sites = matches_for(
            "fn describe(shape: Shape) -> &'static str {\n\
             match shape {\n\
             Shape::Circle => \"circle\",\n\
             Shape::Square => \"square\",\n\
             }\n\
             }",
        );
        assert_eq!(sites.len(), 1);
        assert_eq!(sites[0].type_name, "Shape");
        assert_eq!(sites[0].arm_count, 2);
    }

    #[test]
    fn a_match_on_a_reference_typed_parameter_is_not_a_candidate() {
        let sites = matches_for(
            "fn describe(shape: &Shape) -> &'static str {\n\
             match shape {\n\
             _ => \"x\",\n\
             }\n\
             }",
        );
        assert!(sites.is_empty());
    }

    #[test]
    fn a_match_on_a_field_access_scrutinee_is_not_a_candidate() {
        let sites = matches_for(
            "fn describe(self_: Thing) -> &'static str {\n\
             match self_.shape {\n\
             _ => \"x\",\n\
             }\n\
             }",
        );
        assert!(sites.is_empty());
    }

    #[test]
    fn a_match_on_an_optiontyped_parameter_is_not_a_candidate() {
        // `Option<i32>` isn't a plain `type_identifier`, so it never even
        // reaches the denylist check — proven here at the tree-sitter layer.
        let sites = matches_for(
            "fn handle(opt: Option<i32>) -> i32 {\n\
             match opt {\n\
             Some(x) => x,\n\
             None => 0,\n\
             }\n\
             }",
        );
        assert!(sites.is_empty());
    }

    #[test]
    fn a_nested_function_does_not_see_the_outer_scope() {
        let sites = matches_for(
            "fn outer(shape: Shape) -> i32 {\n\
             fn inner() -> i32 {\n\
             match shape {\n\
             _ => 0,\n\
             }\n\
             }\n\
             inner()\n\
             }",
        );
        assert!(sites.is_empty());
    }

    #[test]
    fn qualified_name_includes_enclosing_module() {
        let content = "mod shapes { pub enum Shape { Circle } }";
        let tree = parse_rust(content, Path::new("f.rs")).unwrap();
        let mut cursor = tree.root_node().walk();
        let enum_node = find_enum(tree.root_node(), &mut cursor).unwrap();
        assert_eq!(
            qualified_name(enum_node, content.as_bytes()),
            "shapes::Shape"
        );
    }

    fn find_enum<'a>(node: Node<'a>, cursor: &mut tree_sitter::TreeCursor<'a>) -> Option<Node<'a>> {
        if node.kind() == "enum_item" {
            return Some(node);
        }
        for child in node.children(cursor) {
            if let Some(found) = find_enum(child, &mut child.walk()) {
                return Some(found);
            }
        }
        None
    }
}
