use crate::entities::{DetectorId, Evidence, LineRange, Site, SiteRole, Symptom, SymptomId};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tree_sitter::{Node, Parser};

/// The detector id every cognitive-complexity `Symptom`/`SymptomId` is
/// tagged with.
pub const DETECTOR_ID: &str = "cognitive-complexity";

/// Sonar's default threshold (15) flags fine code; spec.md:186 pins this
/// detector's threshold at 25 instead.
pub const COMPLEXITY_THRESHOLD: usize = 25;

#[derive(Debug, Error)]
pub enum CognitiveComplexityError {
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
}

/// Runs the cognitive-complexity detector (spec.md:179-192) against every
/// `.rs` file found recursively under `root`: every whole-function subtree
/// is scored under the Sonar cognitive-complexity spec (+1 per branch, +1
/// extra per nesting level), and every function scoring `>=
/// COMPLEXITY_THRESHOLD` becomes one `Symptom`, ranked score-descending.
pub fn run_cognitive_complexity(root: &Path) -> Result<Vec<Symptom>, CognitiveComplexityError> {
    let mut scored: Vec<(usize, Symptom)> = Vec::new();

    for file in collect_rust_files(root)? {
        let content =
            fs::read_to_string(&file).map_err(|source| CognitiveComplexityError::Read {
                path: file.clone(),
                source,
            })?;
        let tree = parse_rust(&content, &file)?;
        let mut functions = Vec::new();
        collect_function_items(tree.root_node(), &mut functions);

        let relative = file.strip_prefix(root).unwrap_or(file.as_path());

        for function in functions {
            let mut max_nesting_depth = 0;
            let body = match function.child_by_field_name("body") {
                Some(body) => body,
                None => continue,
            };
            let score = score_node(body, 0, &mut max_nesting_depth);
            if score < COMPLEXITY_THRESHOLD {
                continue;
            }

            let function_length = function.end_position().row - function.start_position().row + 1;
            let qualified_name = qualified_name(function, content.as_bytes());
            let body_text = function.utf8_text(content.as_bytes()).unwrap_or_default();
            let fingerprint = format!("{}::{qualified_name}::{body_text}", relative.display());

            let symptom = Symptom {
                id: SymptomId::new(DetectorId::new(DETECTOR_ID), fingerprint.as_bytes()),
                detector: DetectorId::new(DETECTOR_ID),
                title: format!(
                    "Cognitive complexity {score} (threshold >= {COMPLEXITY_THRESHOLD}) in {qualified_name}"
                ),
                evidence: Evidence::ComplexityScore {
                    score,
                    function_length,
                    max_nesting_depth,
                },
                sites: vec![Site {
                    file: file.clone(),
                    line_ranges: vec![LineRange {
                        start: function.start_position().row + 1,
                        end: function.end_position().row + 1,
                    }],
                    role: SiteRole::MatchSite,
                    note: format!(
                        "Cognitive complexity {score}, max nesting depth {max_nesting_depth}"
                    ),
                }],
            };
            scored.push((score, symptom));
        }
    }

    // Ranked by score descending (spec.md:188); ties broken on the symptom
    // id for a deterministic, rerun-stable order.
    scored.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| a.1.id.to_string().cmp(&b.1.id.to_string()))
    });

    Ok(scored.into_iter().map(|(_, symptom)| symptom).collect())
}

fn parse_rust(content: &str, path: &Path) -> Result<tree_sitter::Tree, CognitiveComplexityError> {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_rust::LANGUAGE.into())?;
    parser
        .parse(content, None)
        .ok_or_else(|| CognitiveComplexityError::Parse {
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

/// Sonar cognitive-complexity scorer (spec.md:183-185: "+1 per branch, +1
/// extra per nesting level"). `nesting` is the current nesting level;
/// `max_nesting_depth` is updated to the deepest nesting level any
/// incrementing construct's body reaches.
///
/// Deliberately excluded from v1 (mirrors the spec's own "Deliberately
/// excluded from v1" call-out for extraction-candidate analysis,
/// spec.md:189-191): nested function/closure nesting-level increments,
/// `if let`/`while let` scrutinee handling (tree-sitter-rust represents
/// these as ordinary `if_expression`/`while_expression` with a `let_condition`
/// in the condition field — not scored differently here), and labeled
/// break/continue increments. None of this phase's fixtures exercise those
/// shapes.
fn score_node(node: Node, nesting: usize, max_nesting_depth: &mut usize) -> usize {
    match node.kind() {
        "if_expression" => score_if(node, nesting, max_nesting_depth),
        "while_expression" => {
            let mut total = 1 + nesting;
            if let Some(cond) = node.child_by_field_name("condition") {
                total += score_node(cond, nesting, max_nesting_depth);
            }
            if let Some(body) = node.child_by_field_name("body") {
                *max_nesting_depth = (*max_nesting_depth).max(nesting + 1);
                total += score_node(body, nesting + 1, max_nesting_depth);
            }
            total
        }
        "for_expression" => {
            let mut total = 1 + nesting;
            if let Some(value) = node.child_by_field_name("value") {
                total += score_node(value, nesting, max_nesting_depth);
            }
            if let Some(body) = node.child_by_field_name("body") {
                *max_nesting_depth = (*max_nesting_depth).max(nesting + 1);
                total += score_node(body, nesting + 1, max_nesting_depth);
            }
            total
        }
        "loop_expression" => {
            let mut total = 1 + nesting;
            if let Some(body) = node.child_by_field_name("body") {
                *max_nesting_depth = (*max_nesting_depth).max(nesting + 1);
                total += score_node(body, nesting + 1, max_nesting_depth);
            }
            total
        }
        "match_expression" => score_match(node, nesting, max_nesting_depth),
        "binary_expression" => score_binary(node, nesting, max_nesting_depth, None),
        _ => {
            let mut total = 0;
            let mut cursor = node.walk();
            for child in node.named_children(&mut cursor) {
                total += score_node(child, nesting, max_nesting_depth);
            }
            total
        }
    }
}

/// `if`/`else if`/`else` handling. `alternative` (when present) wraps an
/// `else_clause` node whose single child is either another `if_expression`
/// (an else-if — a flat +1 continuation at the *same* nesting level, not a
/// deeper one) or a `block` (a plain else — flat +1, its body nests one
/// level deeper than the `if`).
fn score_if(node: Node, nesting: usize, max_nesting_depth: &mut usize) -> usize {
    let mut total = 1 + nesting;
    if let Some(cond) = node.child_by_field_name("condition") {
        total += score_node(cond, nesting, max_nesting_depth);
    }
    if let Some(consequence) = node.child_by_field_name("consequence") {
        *max_nesting_depth = (*max_nesting_depth).max(nesting + 1);
        total += score_node(consequence, nesting + 1, max_nesting_depth);
    }
    if let Some(else_clause) = node.child_by_field_name("alternative")
        && let Some(inner) = else_clause.named_child(0)
    {
        if inner.kind() == "if_expression" {
            total += score_if(inner, nesting, max_nesting_depth);
        } else {
            total += 1;
            *max_nesting_depth = (*max_nesting_depth).max(nesting + 1);
            total += score_node(inner, nesting + 1, max_nesting_depth);
        }
    }
    total
}

/// `match` handling: the match itself contributes once (not once per arm,
/// per spec.md's ranking-by-score-not-arm-count design); each arm's result
/// expression nests one level deeper than the match.
fn score_match(node: Node, nesting: usize, max_nesting_depth: &mut usize) -> usize {
    let mut total = 1 + nesting;
    if let Some(scrutinee) = node.child_by_field_name("value") {
        total += score_node(scrutinee, nesting, max_nesting_depth);
    }
    if let Some(body) = node.child_by_field_name("body") {
        *max_nesting_depth = (*max_nesting_depth).max(nesting + 1);
        let mut cursor = body.walk();
        for arm in body.named_children(&mut cursor) {
            if arm.kind() != "match_arm" {
                continue;
            }
            if let Some(value) = arm.child_by_field_name("value") {
                total += score_node(value, nesting + 1, max_nesting_depth);
            }
        }
    }
    total
}

/// Logical-operator run detection (Sonar's B3 rule): a maximal run of the
/// *same* `&&`/`||` operator counts once; a change in operator starts a new
/// run. `run_op` is the enclosing run's operator, if this node is being
/// visited as part of one; comparison/arithmetic operators never start a
/// run and are scored as plain operand traversal.
fn score_binary(
    node: Node,
    nesting: usize,
    max_nesting_depth: &mut usize,
    run_op: Option<&'static str>,
) -> usize {
    let op = node
        .child_by_field_name("operator")
        .map(|o| o.kind())
        .unwrap_or_default();
    let is_logical = op == "&&" || op == "||";
    let (Some(left), Some(right)) = (
        node.child_by_field_name("left"),
        node.child_by_field_name("right"),
    ) else {
        return 0;
    };

    if !is_logical {
        return score_operand(left, nesting, max_nesting_depth, None)
            + score_operand(right, nesting, max_nesting_depth, None);
    }

    let this_run: &'static str = if op == "&&" { "&&" } else { "||" };
    let mut total = if run_op == Some(this_run) { 0 } else { 1 };
    total += score_operand(left, nesting, max_nesting_depth, Some(this_run));
    total += score_operand(right, nesting, max_nesting_depth, Some(this_run));
    total
}

fn score_operand(
    node: Node,
    nesting: usize,
    max_nesting_depth: &mut usize,
    run_op: Option<&'static str>,
) -> usize {
    if node.kind() == "binary_expression" {
        score_binary(node, nesting, max_nesting_depth, run_op)
    } else {
        score_node(node, nesting, max_nesting_depth)
    }
}

/// Container-qualified name for the fingerprint (decision D007): walks
/// ancestors collecting `mod_item`/`impl_item`/`trait_item` names, re-derived
/// locally rather than depending on `diffviz-core`'s `SemanticTree`
/// pipeline (matching `type2_clones`'s precedent of a self-contained
/// detector module).
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

fn collect_rust_files(root: &Path) -> Result<Vec<PathBuf>, CognitiveComplexityError> {
    let mut files = Vec::new();
    visit(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn visit(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), CognitiveComplexityError> {
    let entries = fs::read_dir(dir).map_err(|source| CognitiveComplexityError::Walk {
        path: dir.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| CognitiveComplexityError::Walk {
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

    fn score_of(content: &str) -> (usize, usize) {
        let tree = parse_rust(content, Path::new("f.rs")).unwrap();
        let mut functions = Vec::new();
        collect_function_items(tree.root_node(), &mut functions);
        let body = functions[0].child_by_field_name("body").unwrap();
        let mut max_depth = 0;
        let score = score_node(body, 0, &mut max_depth);
        (score, max_depth)
    }

    #[test]
    fn a_flat_function_with_no_branches_scores_zero() {
        let (score, _) = score_of("fn f() -> i32 { let x = 1; x + 1 }");
        assert_eq!(score, 0);
    }

    #[test]
    fn a_single_if_scores_one() {
        let (score, _) = score_of("fn f(a: i32) { if a > 0 { } }");
        assert_eq!(score, 1);
    }

    #[test]
    fn a_nested_if_adds_the_nesting_increment() {
        // outer if: 1+0=1; inner if: 1+1=2; total=3
        let (score, _) = score_of("fn f(a: i32, b: i32) { if a > 0 { if b > 0 { } } }");
        assert_eq!(score, 3);
    }

    #[test]
    fn an_else_if_is_flat_at_the_same_nesting_level_as_the_if() {
        // if: 1+0=1; else-if: 1+0=1 (same nesting, not nested); total=2
        let (score, _) =
            score_of("fn f(a: i32) -> i32 { if a > 0 { 1 } else if a < 0 { 2 } else { 3 } }");
        // else block itself: flat +1 => total = 1 (if) + 1 (else-if) + 1 (else) = 3
        assert_eq!(score, 3);
    }

    #[test]
    fn a_run_of_the_same_logical_operator_counts_once() {
        let (score, _) = score_of("fn f(a: bool, b: bool, c: bool) { if a && b && c { } }");
        // if: 1; && run (a && b && c is left-associative, same op throughout): 1
        assert_eq!(score, 2);
    }

    #[test]
    fn a_change_in_logical_operator_starts_a_new_run() {
        let (score, _) = score_of("fn f(a: bool, b: bool, c: bool) { if a && b || c { } }");
        // if: 1; && run: 1; || run: 1
        assert_eq!(score, 3);
    }

    #[test]
    fn a_match_counts_once_regardless_of_arm_count() {
        let (score, _) = score_of("fn f(x: i32) -> i32 { match x { 0 => 1, 1 => 2, _ => 0 } }");
        assert_eq!(score, 1);
    }

    #[test]
    fn deeply_nested_high_complexity_fixture_scores_at_or_above_threshold() {
        let (score, _) = score_of(
            r#"
            fn deeply_nested(a: i32, b: i32, c: i32) -> i32 {
                let mut result = 0;
                if a > 0 {
                    for i in 0..a {
                        while i < b {
                            if c > 0 && b > 0 {
                                if a == b {
                                    for _ in 0..a {
                                        result += 1;
                                    }
                                } else if a == c {
                                    result += 2;
                                } else {
                                    result += 3;
                                }
                            }
                        }
                    }
                }
                result
            }
            "#,
        );
        assert!(score >= COMPLEXITY_THRESHOLD, "got {score}");
    }

    #[test]
    fn qualified_name_includes_enclosing_impl_and_mod() {
        let content = "mod outer { impl Foo { fn bar() {} } }";
        let tree = parse_rust(content, Path::new("f.rs")).unwrap();
        let mut functions = Vec::new();
        collect_function_items(tree.root_node(), &mut functions);
        assert_eq!(
            qualified_name(functions[0], content.as_bytes()),
            "outer::Foo::bar"
        );
    }
}
