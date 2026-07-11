use crate::entities::{DetectorId, Evidence, LineRange, Site, SiteRole, Symptom, SymptomId};
use ast_grep_config::{GlobalRules, RuleConfig, RuleConfigError, from_yaml_string};
use ast_grep_core::{Doc, NodeMatch};
use ast_grep_language::{LanguageExt, SupportLang};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// The detector id every house-rule `Symptom`/`SymptomId` is tagged with.
pub const DETECTOR_ID: &str = "house-rules";

/// The house-rule pack YAML, embedded at compile time so the binary is
/// self-contained. The same file is also valid input for the standalone
/// `ast-grep` CLI (`ast-grep scan -r rules/house-rules.yaml <path>`).
pub const RULE_PACK_YAML: &str = include_str!("../../../rules/house-rules.yaml");

#[derive(Debug, Error)]
pub enum HouseRulesError {
    #[error("failed to parse house-rules rule pack")]
    RulePack(#[from] RuleConfigError),

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
}

/// Runs the house-rules pack (embedded `RULE_PACK_YAML`) against every `.rs`
/// file found recursively under `root`, returning one `Symptom` per match.
pub fn run_house_rules(root: &Path) -> Result<Vec<Symptom>, HouseRulesError> {
    let globals = GlobalRules::default();
    let rules: Vec<RuleConfig<SupportLang>> = from_yaml_string(RULE_PACK_YAML, &globals)?;

    let mut symptoms = Vec::new();
    for file in collect_rust_files(root)? {
        let content = fs::read_to_string(&file).map_err(|source| HouseRulesError::Read {
            path: file.clone(),
            source,
        })?;
        let grep = SupportLang::Rust.ast_grep(&content);

        for rule in &rules {
            for m in grep.root().find_all(&rule.matcher) {
                if !passes_post_filter(&rule.id, &m) {
                    continue;
                }
                let matched_snippet = m.text().to_string();
                let start_line = m.start_pos().line() + 1;
                let end_line = m.end_pos().line() + 1;
                let fingerprint = fingerprint_bytes(&rule.id, &matched_snippet);

                symptoms.push(Symptom {
                    id: SymptomId::new(DetectorId::new(DETECTOR_ID), &fingerprint),
                    detector: DetectorId::new(DETECTOR_ID),
                    title: rule.message.clone(),
                    evidence: Evidence::RuleMatch {
                        rule_id: rule.id.clone(),
                        matched_snippet: matched_snippet.clone(),
                    },
                    sites: vec![Site {
                        file: file.clone(),
                        line_ranges: vec![LineRange {
                            start: start_line,
                            end: end_line,
                        }],
                        role: SiteRole::MatchSite,
                        note: rule.message.clone(),
                    }],
                });
            }
        }
    }

    Ok(symptoms)
}

/// Rejects matches that ast-grep's YAML matcher language can't distinguish
/// on its own, using the small amount of structural context (ancestors,
/// captured metavariables, sibling comments) available on the matched node.
fn passes_post_filter<D: Doc>(rule_id: &str, m: &NodeMatch<D>) -> bool {
    match rule_id {
        "no-format-in-error-value" => format_in_error_value_is_true_positive(m),
        "no-let-underscore-result" => let_underscore_result_is_true_positive(m),
        "no-catchall-default-arm" => catchall_default_arm_is_true_positive(m),
        "no-ok-discarding-errors" | "no-unwrap-or-default" => !receiver_is_infallible_decode(m),
        _ => true,
    }
}

/// `.ok()` / `.unwrap_or_default()` on tree-sitter's `Node::utf8_text()` is
/// not a real error-swallow: `utf8_text` only fails on non-UTF8 byte
/// boundaries, which cannot happen for nodes carved from already-valid-UTF8
/// parsed source. Cosmetic name-extraction helpers use this pervasively.
fn receiver_is_infallible_decode<D: Doc>(m: &NodeMatch<D>) -> bool {
    m.get_env()
        .get_match("EXPR")
        .is_some_and(|expr| expr.text().contains(".utf8_text("))
}

/// A catch-all arm is only a silent default when its value is a *bare*
/// default: a literal, unit, a plain path (e.g. a unit enum variant), or a
/// constructor call fed nothing but literals (`Vec::new()`,
/// `Default::default()`, `String::from("x")`). Arms that carry captured
/// information forward (`Category::Other(reason)`) or do real fallback work
/// (blocks, returns, dispatch on non-literal values) are excluded.
fn catchall_default_arm_is_true_positive<D: Doc>(m: &NodeMatch<D>) -> bool {
    let Some(value) = m.field("value") else {
        return true;
    };
    is_bare_default_expression(&value)
}

fn is_bare_default_expression<D: Doc>(node: &ast_grep_core::Node<D>) -> bool {
    match node.kind().as_ref() {
        // Real fallback work, not a silent default.
        "block" | "return_expression" | "if_expression" | "match_expression" | "try_expression"
        | "await_expression" | "closure_expression" => false,
        // A call is a bare default only if nothing non-literal flows into it:
        // `Vec::new()` / `String::from("x")` yes, `Other(reason)` / `e.into()` no.
        "call_expression" => {
            let function_is_plain_path = node
                .field("function")
                .is_some_and(|f| matches!(f.kind().as_ref(), "identifier" | "scoped_identifier"));
            let args_are_literals = node
                .field("arguments")
                .is_none_or(|args| args.children().all(|a| is_literal_or_punct(&a)));
            function_is_plain_path && args_are_literals
        }
        // Literals, unit, plain paths, empty collections, macros like vec![].
        _ => true,
    }
}

fn is_literal_or_punct<D: Doc>(node: &ast_grep_core::Node<D>) -> bool {
    let kind = node.kind();
    kind.ends_with("_literal") || !node.is_named()
}

/// `format!` inside a `field_initializer` is only a stringly-typed error
/// value when the enclosing struct/enum literal is itself an error type
/// (this repo's convention: `*Error` naming, see root `CLAUDE.md`'s error
/// handling section). Fields on unrelated diagnostic/UI types (e.g.
/// `Symptom { title: format!(...) }`) are excluded.
fn format_in_error_value_is_true_positive<D: Doc>(m: &NodeMatch<D>) -> bool {
    let Some(struct_lit) = m.ancestors().find(|n| n.kind() == "struct_expression") else {
        return true;
    };
    let Some(name) = struct_lit.field("name") else {
        return true;
    };
    name.text().contains("Error")
}

/// `let _ = $EXPR;` only swallows a `Result` when `$EXPR` actually returns
/// one. Ast-grep has no type information, so this narrowly excludes the
/// audit's confirmed non-Result false positive (`String::from_utf8_lossy`,
/// which returns `Cow<str>`) and best-effort cleanup calls the author has
/// already justified with a preceding comment.
fn let_underscore_result_is_true_positive<D: Doc>(m: &NodeMatch<D>) -> bool {
    if let Some(expr) = m.get_env().get_match("EXPR")
        && expr.text().contains("from_utf8_lossy")
    {
        return false;
    }
    if let Some(prev) = m.prev()
        && prev.kind().contains("comment")
    {
        return false;
    }
    true
}

/// Builds this detector's fingerprint bytes: rule id + whitespace-normalized
/// match text. Whitespace-insensitive so trivial reformatting (rustfmt
/// re-wrapping a line, for instance) doesn't change the `SymptomId` — only
/// the matched code's actual token content does.
fn fingerprint_bytes(rule_id: &str, matched_snippet: &str) -> Vec<u8> {
    let normalized = matched_snippet
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    format!("{rule_id}:{normalized}").into_bytes()
}

fn collect_rust_files(root: &Path) -> Result<Vec<PathBuf>, HouseRulesError> {
    let mut files = Vec::new();
    visit(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn visit(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), HouseRulesError> {
    let entries = fs::read_dir(dir).map_err(|source| HouseRulesError::Walk {
        path: dir.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| HouseRulesError::Walk {
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

    fn diffviz_core_src() -> PathBuf {
        // Runs from apps/patina-detect/detect, so climb to the repo root.
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../libs/diffviz-core/src")
    }

    #[test]
    fn finds_unwrap_or_default_in_diffviz_core() {
        let symptoms = run_house_rules(&diffviz_core_src()).expect("detector run failed");
        assert!(
            symptoms
                .iter()
                .any(|s| matches!(&s.evidence, Evidence::RuleMatch { rule_id, .. } if rule_id == "no-unwrap-or-default")),
            "expected at least one no-unwrap-or-default symptom, found: {:#?}",
            symptoms
        );
    }

    #[test]
    fn finds_allow_dead_code_in_diffviz_core() {
        let symptoms = run_house_rules(&diffviz_core_src()).expect("detector run failed");
        assert!(
            symptoms
                .iter()
                .any(|s| matches!(&s.evidence, Evidence::RuleMatch { rule_id, .. } if rule_id == "no-allow-dead-code")),
            "expected at least one no-allow-dead-code symptom, found: {:#?}",
            symptoms
        );
    }

    #[test]
    fn finds_catchall_default_arm_in_diffviz_core() {
        let symptoms = run_house_rules(&diffviz_core_src()).expect("detector run failed");
        assert!(
            symptoms
                .iter()
                .any(|s| matches!(&s.evidence, Evidence::RuleMatch { rule_id, .. } if rule_id == "no-catchall-default-arm")),
            "expected at least one no-catchall-default-arm symptom, found: {:#?}",
            symptoms
        );
    }

    #[test]
    fn does_not_flag_catchall_panic_arm() {
        let dir = write_fixture(
            "fn f(x: i32) { match x { 1 => (), _ => panic!(\"unexpected value\") } }",
        );
        let symptoms = run_house_rules(dir.path()).expect("detector run failed");
        assert!(
            symptoms
                .iter()
                .all(|s| !matches!(&s.evidence, Evidence::RuleMatch { rule_id, .. } if rule_id == "no-catchall-default-arm")),
            "expected no no-catchall-default-arm symptom for a panic!() test-assertion arm, found: {:#?}",
            symptoms
        );
    }

    #[test]
    fn does_not_flag_catchall_named_variant_construction() {
        let dir = write_fixture(
            "enum Category { Known, Other(String) }\nfn f(x: &str) -> Category { match x { \"known\" => Category::Known, _ => Category::Other(x.to_string()) } }",
        );
        let symptoms = run_house_rules(dir.path()).expect("detector run failed");
        assert!(
            symptoms
                .iter()
                .all(|s| !matches!(&s.evidence, Evidence::RuleMatch { rule_id, .. } if rule_id == "no-catchall-default-arm")),
            "expected no no-catchall-default-arm symptom for an explicit named-variant catch-all, found: {:#?}",
            symptoms
        );
    }

    #[test]
    fn still_flags_catchall_zero_arg_constructor_default() {
        let dir = write_fixture(
            "fn f(x: i32) -> Vec<i32> { match x { 1 => vec![1], _ => Vec::new() } }\nfn g(x: i32) -> String { match x { 1 => \"a\".into(), _ => Default::default() } }",
        );
        let symptoms = run_house_rules(dir.path()).expect("detector run failed");
        let count = symptoms
            .iter()
            .filter(|s| matches!(&s.evidence, Evidence::RuleMatch { rule_id, .. } if rule_id == "no-catchall-default-arm"))
            .count();
        assert_eq!(
            count, 2,
            "expected both Vec::new() and Default::default() catch-alls flagged, found: {:#?}",
            symptoms
        );
    }

    #[test]
    fn does_not_flag_catchall_doing_fallback_work() {
        let dir = write_fixture(
            "fn f(x: i32) -> i32 { match x { 1 => 1, _ => { let y = x * 2; y } } }\nfn g(x: i32) -> Result<i32, String> { match x { 1 => Ok(1), _ => Err(format!(\"bad: {x}\")) } }",
        );
        let symptoms = run_house_rules(dir.path()).expect("detector run failed");
        assert!(
            symptoms
                .iter()
                .all(|s| !matches!(&s.evidence, Evidence::RuleMatch { rule_id, .. } if rule_id == "no-catchall-default-arm")),
            "expected no no-catchall-default-arm symptom for arms doing real fallback work, found: {:#?}",
            symptoms
        );
    }

    #[test]
    fn does_not_flag_ok_on_utf8_text() {
        let dir = write_fixture(
            "fn name(node: tree_sitter::Node, src: &[u8]) -> Option<String> { node.utf8_text(src).ok().map(str::to_string) }",
        );
        let symptoms = run_house_rules(dir.path()).expect("detector run failed");
        assert!(
            symptoms
                .iter()
                .all(|s| !matches!(&s.evidence, Evidence::RuleMatch { rule_id, .. } if rule_id == "no-ok-discarding-errors")),
            "expected no no-ok-discarding-errors symptom for utf8_text().ok(), found: {:#?}",
            symptoms
        );
    }

    #[test]
    fn does_not_flag_unwrap_or_default_on_utf8_text() {
        let dir = write_fixture(
            "fn name(node: tree_sitter::Node, src: &[u8]) -> String { node.utf8_text(src).unwrap_or_default().to_string() }",
        );
        let symptoms = run_house_rules(dir.path()).expect("detector run failed");
        assert!(
            symptoms
                .iter()
                .all(|s| !matches!(&s.evidence, Evidence::RuleMatch { rule_id, .. } if rule_id == "no-unwrap-or-default")),
            "expected no no-unwrap-or-default symptom for utf8_text().unwrap_or_default(), found: {:#?}",
            symptoms
        );
    }

    #[test]
    fn still_flags_ok_on_genuine_fallible_call() {
        let dir = write_fixture("fn f() -> Option<Vec<u8>> { std::fs::read(\"config\").ok() }");
        let symptoms = run_house_rules(dir.path()).expect("detector run failed");
        assert!(
            symptoms
                .iter()
                .any(|s| matches!(&s.evidence, Evidence::RuleMatch { rule_id, .. } if rule_id == "no-ok-discarding-errors")),
            "expected a no-ok-discarding-errors symptom for a genuine I/O Result, found: {:#?}",
            symptoms
        );
    }

    #[test]
    fn finds_format_in_error_value_in_diffviz_core() {
        let symptoms = run_house_rules(&diffviz_core_src()).expect("detector run failed");
        assert!(
            symptoms
                .iter()
                .any(|s| matches!(&s.evidence, Evidence::RuleMatch { rule_id, .. } if rule_id == "no-format-in-error-value")),
            "expected at least one no-format-in-error-value symptom, found: {:#?}",
            symptoms
        );
    }

    #[test]
    fn does_not_flag_format_in_non_error_struct_field() {
        let dir = write_fixture(
            "struct Symptom { title: String }\nfn f(name: &str) -> Symptom { Symptom { title: format!(\"Found: {name}\") } }",
        );
        let symptoms = run_house_rules(dir.path()).expect("detector run failed");
        assert!(
            symptoms
                .iter()
                .all(|s| !matches!(&s.evidence, Evidence::RuleMatch { rule_id, .. } if rule_id == "no-format-in-error-value")),
            "expected no no-format-in-error-value symptom for a format! field on a non-error diagnostic type, found: {:#?}",
            symptoms
        );
    }

    #[test]
    fn still_flags_format_in_error_struct_field() {
        let dir = write_fixture(
            "struct ParseError { message: String }\nfn f(e: &str) -> ParseError { ParseError { message: format!(\"failed: {e}\") } }",
        );
        let symptoms = run_house_rules(dir.path()).expect("detector run failed");
        assert!(
            symptoms
                .iter()
                .any(|s| matches!(&s.evidence, Evidence::RuleMatch { rule_id, .. } if rule_id == "no-format-in-error-value")),
            "expected a no-format-in-error-value symptom for a format! field on a *Error type, found: {:#?}",
            symptoms
        );
    }

    /// `no-let-underscore-result`, `no-stringly-typed-map-err`, and
    /// `no-todo-unimplemented` have zero live instances in diffviz-core
    /// today (confirmed by grep before writing these tests, and by the
    /// audit output showing no such symptoms). Rather than fake a fixture
    /// inside diffviz-core itself, these three are proven against a
    /// synthetic temp file — the acceptance criterion is "the detector
    /// finds it", not "diffviz-core currently violates every rule".
    fn write_fixture(content: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        std::fs::write(dir.path().join("fixture.rs"), content).expect("failed to write fixture");
        dir
    }

    #[test]
    fn finds_let_underscore_result_in_a_fixture() {
        let dir = write_fixture("fn f() -> Result<(), ()> { let _ = Ok::<(), ()>(()); Ok(()) }");
        let symptoms = run_house_rules(dir.path()).expect("detector run failed");
        assert!(
            symptoms
                .iter()
                .any(|s| matches!(&s.evidence, Evidence::RuleMatch { rule_id, .. } if rule_id == "no-let-underscore-result")),
            "expected a no-let-underscore-result symptom, found: {:#?}",
            symptoms
        );
    }

    #[test]
    fn does_not_flag_let_underscore_from_utf8_lossy() {
        let dir = write_fixture("fn f(bytes: &[u8]) { let _ = String::from_utf8_lossy(bytes); }");
        let symptoms = run_house_rules(dir.path()).expect("detector run failed");
        assert!(
            symptoms
                .iter()
                .all(|s| !matches!(&s.evidence, Evidence::RuleMatch { rule_id, .. } if rule_id == "no-let-underscore-result")),
            "expected no no-let-underscore-result symptom for a from_utf8_lossy call, found: {:#?}",
            symptoms
        );
    }

    #[test]
    fn does_not_flag_let_underscore_with_justifying_comment() {
        let dir = write_fixture(
            "fn f() { // best-effort cleanup, failure here is not actionable\n    let _ = std::fs::remove_file(\"tmp\"); }",
        );
        let symptoms = run_house_rules(dir.path()).expect("detector run failed");
        assert!(
            symptoms
                .iter()
                .all(|s| !matches!(&s.evidence, Evidence::RuleMatch { rule_id, .. } if rule_id == "no-let-underscore-result")),
            "expected no no-let-underscore-result symptom when a justifying comment precedes the let, found: {:#?}",
            symptoms
        );
    }

    #[test]
    fn finds_stringly_typed_map_err_in_a_fixture() {
        let dir = write_fixture(
            "fn f() -> Result<(), String> { std::fs::read(\"x\").map_err(|e| e.to_string())?; Ok(()) }",
        );
        let symptoms = run_house_rules(dir.path()).expect("detector run failed");
        assert!(
            symptoms
                .iter()
                .any(|s| matches!(&s.evidence, Evidence::RuleMatch { rule_id, .. } if rule_id == "no-stringly-typed-map-err")),
            "expected a no-stringly-typed-map-err symptom, found: {:#?}",
            symptoms
        );
    }

    #[test]
    fn finds_todo_unimplemented_in_a_fixture() {
        let dir = write_fixture("fn f() { todo!() }\nfn g() { unimplemented!() }");
        let symptoms = run_house_rules(dir.path()).expect("detector run failed");
        assert!(
            symptoms
                .iter()
                .any(|s| matches!(&s.evidence, Evidence::RuleMatch { rule_id, .. } if rule_id == "no-todo-unimplemented")),
            "expected a no-todo-unimplemented symptom, found: {:#?}",
            symptoms
        );
    }

    #[test]
    fn rerunning_with_no_code_change_produces_identical_symptom_ids() {
        let first = run_house_rules(&diffviz_core_src()).expect("first run failed");
        let second = run_house_rules(&diffviz_core_src()).expect("second run failed");

        let first_ids: Vec<_> = first.iter().map(|s| s.id.to_string()).collect();
        let second_ids: Vec<_> = second.iter().map(|s| s.id.to_string()).collect();
        assert_eq!(first_ids, second_ids);
    }
}
