use crate::entities::{DetectorId, Evidence, LineRange, Site, SiteRole, Symptom, SymptomId};
use diffviz_core::parsers::descriptor::LanguageDescriptor;
use diffviz_core::parsers::rust::RustDescriptor;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tree_sitter::{Node, Parser};

/// The detector id every Type-2 clones `Symptom`/`SymptomId` is tagged with.
pub const DETECTOR_ID: &str = "type2-clones";

/// Minimum shared subtree size (spec.md:143's "~30 semantic nodes") below
/// which a structurally-identical pair is too small to be worth reporting
/// (e.g. `fn add_one(x: i32) -> i32 { x + 1 }`).
const MIN_SEMANTIC_NODES: usize = 30;

#[derive(Debug, Error)]
pub enum Type2ClonesError {
    #[error("failed to walk directory {path}")]
    Walk {
        path: PathBuf,
        #[source]
        source: ignore::Error,
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
}

/// One `function_item` found while scanning: its location, the normalized
/// structural signature used to match it against other functions, its
/// semantic-node count (for the min-size gate), and whether it lives inside
/// test-only code.
struct FunctionUnit {
    file: PathBuf,
    start_line: usize,
    end_line: usize,
    node_count: usize,
    is_test: bool,
}

/// Runs the Type-2 clones detector (spec.md:134-148) against every `.rs`
/// file found recursively under `root`: every whole-function subtree is
/// hashed on its normalized structure (identifiers/literals collapse to
/// their tree-sitter node kind, so renamed variables/functions don't break a
/// match), grouped by that structure, and every group with 2+ members and at
/// least `MIN_SEMANTIC_NODES` shared nodes becomes one `Symptom`.
pub fn run_type2_clones(root: &Path) -> Result<Vec<Symptom>, Type2ClonesError> {
    let descriptor = RustDescriptor;
    let mut groups: BTreeMap<String, Vec<FunctionUnit>> = BTreeMap::new();

    for file in collect_rust_files(root)? {
        let content = fs::read_to_string(&file).map_err(|source| Type2ClonesError::Read {
            path: file.clone(),
            source,
        })?;
        let tree = parse_rust(&content, &file)?;
        let mut functions = Vec::new();
        collect_function_items(tree.root_node(), &mut functions);

        for function in functions {
            let node_count = count_semantic_nodes(function, &descriptor);
            if node_count < MIN_SEMANTIC_NODES {
                continue;
            }
            let mut signature = String::new();
            structural_signature(function, false, content.as_bytes(), &mut signature);

            groups.entry(signature).or_default().push(FunctionUnit {
                file: file.clone(),
                start_line: function.start_position().row + 1,
                end_line: function.end_position().row + 1,
                node_count,
                is_test: is_in_test_context(function, content.as_bytes()),
            });
        }
    }

    let mut symptoms: Vec<(bool, Symptom)> = Vec::new();
    for (signature, members) in groups {
        if members.len() < 2 {
            continue;
        }

        let files: HashSet<&PathBuf> = members.iter().map(|m| &m.file).collect();
        let cross_file = files.len() > 1;
        let all_test_code = members.iter().all(|m| m.is_test);
        let group_size = members.len();
        let node_count = members[0].node_count;

        let sites = members
            .iter()
            .map(|member| Site {
                file: member.file.clone(),
                line_ranges: vec![LineRange {
                    start: member.start_line,
                    end: member.end_line,
                }],
                role: SiteRole::CloneMember,
                note: format!("One of {group_size} structurally-identical functions"),
            })
            .collect();

        let symptom = Symptom {
            id: SymptomId::new(DetectorId::new(DETECTOR_ID), signature.as_bytes()),
            detector: DetectorId::new(DETECTOR_ID),
            title: format!(
                "Type-2 clone group ({group_size} members, {node_count} semantic nodes)"
            ),
            evidence: Evidence::CloneGroup {
                group_size,
                node_count,
                all_test_code,
            },
            sites,
        };
        symptoms.push((cross_file, symptom));
    }

    // Cross-file groups ranked above same-file groups (spec.md:143); ties
    // broken on the symptom id for a deterministic, rerun-stable order.
    symptoms.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| a.1.id.to_string().cmp(&b.1.id.to_string()))
    });

    Ok(symptoms.into_iter().map(|(_, symptom)| symptom).collect())
}

fn parse_rust(content: &str, path: &Path) -> Result<tree_sitter::Tree, Type2ClonesError> {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_rust::LANGUAGE.into())?;
    parser
        .parse(content, None)
        .ok_or_else(|| Type2ClonesError::Parse {
            path: path.to_path_buf(),
        })
}

fn collect_function_items<'a>(node: Node<'a>, out: &mut Vec<Node<'a>>) {
    if node.kind() == "function_item" {
        out.push(node);
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_function_items(child, out);
    }
}

/// Normalized structural signature: a pre-order walk of named-node kinds,
/// skipping comments. Because tree-sitter node kinds for identifiers and
/// literals (`identifier`, `integer_literal`, ...) never carry the actual
/// text, this already collapses "compute_score"/"compute_rating" and
/// "0"/"100" to the same signature for *local, renamable* names — no
/// separate placeholder-substitution step is needed for those.
///
/// It is deliberately *not* blanket-blind to identifier text, though: a
/// leaf sitting in a [`is_reference_position`] — a method/field name, a
/// macro name, or a `Type::name` path segment — names something defined
/// elsewhere (an external API surface), not a locally renamable binding, so
/// its text is kept as part of the signature. Two functions that share
/// control-flow shape but call different APIs (`get_file_hash()` vs.
/// `get_content_snapshot()`, `JavaParser::new()` vs. `CParser::new()`,
/// `assert!` vs. `assert_eq!`) no longer collapse into one false clone
/// match; a genuinely renamed local variable used repeatedly through a
/// function body (e.g. `values`/`numbers` as the receiver of `.len()`)
/// still does, because receiver/argument positions aren't reference
/// positions.
fn structural_signature(node: Node, retain_text: bool, source: &[u8], out: &mut String) {
    if matches!(
        node.kind(),
        "line_comment" | "block_comment" | "doc_comment"
    ) {
        return;
    }
    let is_identifier_leaf = matches!(
        node.kind(),
        "identifier" | "field_identifier" | "type_identifier"
    );
    out.push_str(node.kind());
    if is_identifier_leaf && retain_text {
        out.push(':');
        if let Ok(text) = node.utf8_text(source) {
            out.push_str(text);
        }
    }
    out.push('|');

    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            let child = cursor.node();
            if child.is_named() {
                let child_retain = is_reference_position(node.kind(), cursor.field_name());
                structural_signature(child, child_retain, source, out);
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
}

/// Whether a child in `field_name` position under a `parent_kind` node
/// names something defined outside the function being fingerprinted (a
/// method/field name, a macro name, or a `Type`/`variant` path segment)
/// rather than a locally renamable variable/parameter/loop binding.
fn is_reference_position(parent_kind: &str, field_name: Option<&str>) -> bool {
    matches!(
        (parent_kind, field_name),
        ("field_expression", Some("field"))
            | ("macro_invocation", Some("macro"))
            | ("scoped_identifier", Some("path"))
            | ("scoped_identifier", Some("name"))
    )
}

/// Counts named nodes in `node`'s subtree that carry real structural
/// meaning — i.e. everything except `descriptor`'s `trivial_kinds` (which
/// already excludes identifiers, literals, type tokens, comments, and
/// visibility/operator tokens). This is the "~30 semantic nodes" gate from
/// spec.md:143.
fn count_semantic_nodes(node: Node, descriptor: &RustDescriptor) -> usize {
    let trivial = descriptor.trivial_kinds();
    let mut count = 0;
    let mut stack = vec![node];
    while let Some(n) = stack.pop() {
        if !trivial.contains(&n.kind()) {
            count += 1;
        }
        let mut cursor = n.walk();
        stack.extend(n.named_children(&mut cursor));
    }
    count
}

/// Whether `node` (or any ancestor) is immediately preceded by a `#[test]`
/// or `#[cfg(test)]` attribute — covers both a directly-annotated `#[test]`
/// function and a function nested inside a `#[cfg(test)] mod tests { ... }`.
fn is_in_test_context(node: Node, source: &[u8]) -> bool {
    let mut current = node;
    loop {
        if has_preceding_test_attribute(current, source) {
            return true;
        }
        match current.parent() {
            Some(parent) => current = parent,
            None => return false,
        }
    }
}

fn has_preceding_test_attribute(node: Node, source: &[u8]) -> bool {
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
            .is_ok_and(|text| text.contains("test"))
        {
            return true;
        }
    }
    false
}

fn collect_rust_files(root: &Path) -> Result<Vec<PathBuf>, Type2ClonesError> {
    let mut files = Vec::new();
    let mut builder = ignore::WalkBuilder::new(root);
    builder.add_custom_ignore_filename(crate::detectors::IGNORE_FILE_NAME);
    for entry in builder.build() {
        let entry = entry.map_err(|source| Type2ClonesError::Walk {
            path: root.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "rs") && path.is_file() {
            files.push(path.to_path_buf());
        }
    }
    files.sort();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn first_function_signature(content: &str) -> String {
        let tree = parse_rust(content, Path::new("f.rs")).unwrap();
        let mut functions = Vec::new();
        collect_function_items(tree.root_node(), &mut functions);
        let mut signature = String::new();
        structural_signature(functions[0], false, content.as_bytes(), &mut signature);
        signature
    }

    #[test]
    fn renamed_identifiers_and_literals_produce_the_same_signature() {
        let a =
            "fn compute(values: &[i32]) -> i32 { let mut total = 0; total += values[0]; total }";
        let b = "fn tally(items: &[i32]) -> i32 { let mut sum = 1; sum += items[9]; sum }";
        assert_eq!(first_function_signature(a), first_function_signature(b));
    }

    #[test]
    fn a_different_control_flow_shape_produces_a_different_signature() {
        let a = "fn f(x: i32) -> i32 { if x > 0 { x } else { 0 } }";
        let b = "fn f(x: i32) -> i32 { let mut y = x; while y > 0 { y -= 1; } y }";
        assert_ne!(first_function_signature(a), first_function_signature(b));
    }

    #[test]
    fn calling_a_different_method_on_a_same_shaped_receiver_produces_a_different_signature() {
        let a = "fn f(values: &[i32]) -> usize { values.len() }";
        let b = "fn f(values: &[i32]) -> usize { values.count() }";
        assert_ne!(
            first_function_signature(a),
            first_function_signature(b),
            "different method names on an otherwise identical shape must not collapse to one clone"
        );
    }

    #[test]
    fn a_renamed_receiver_calling_the_same_method_still_produces_the_same_signature() {
        let a = "fn f(values: &[i32]) -> usize { values.len() }";
        let b = "fn f(numbers: &[i32]) -> usize { numbers.len() }";
        assert_eq!(
            first_function_signature(a),
            first_function_signature(b),
            "a renamed local receiver calling the same method must still match (true Type-2 clone)"
        );
    }

    #[test]
    fn different_macro_names_produce_a_different_signature() {
        let a = "fn f(x: i32) { assert!(x > 0); }";
        let b = "fn f(x: i32) { assert_eq!(x, 0); }";
        assert_ne!(first_function_signature(a), first_function_signature(b));
    }

    #[test]
    fn different_scoped_type_paths_produce_a_different_signature() {
        let a = "fn f() -> Foo { Foo::new() }";
        let b = "fn f() -> Foo { Bar::new() }";
        assert_ne!(
            first_function_signature(a),
            first_function_signature(b),
            "JavaParser::new() vs CParser::new()-shaped calls must not collapse to one clone"
        );
    }

    #[test]
    fn trivial_function_is_well_below_the_min_size_gate() {
        let descriptor = RustDescriptor;
        let tree = parse_rust("fn add_one(x: i32) -> i32 { x + 1 }", Path::new("f.rs")).unwrap();
        let mut functions = Vec::new();
        collect_function_items(tree.root_node(), &mut functions);
        assert!(count_semantic_nodes(functions[0], &descriptor) < MIN_SEMANTIC_NODES);
    }

    #[test]
    fn a_function_directly_annotated_test_is_in_test_context() {
        let content = "#[test]\nfn checks_something() { assert!(true); }";
        let tree = parse_rust(content, Path::new("f.rs")).unwrap();
        let mut functions = Vec::new();
        collect_function_items(tree.root_node(), &mut functions);
        assert!(is_in_test_context(functions[0], content.as_bytes()));
    }

    #[test]
    fn a_function_outside_any_test_module_is_not_in_test_context() {
        let content = "fn production_code() {}";
        let tree = parse_rust(content, Path::new("f.rs")).unwrap();
        let mut functions = Vec::new();
        collect_function_items(tree.root_node(), &mut functions);
        assert!(!is_in_test_context(functions[0], content.as_bytes()));
    }
}
