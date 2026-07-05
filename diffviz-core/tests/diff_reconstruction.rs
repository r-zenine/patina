//! Property test: replaying a rendered diff's lines must reconstruct both input
//! sources exactly.
//!
//! This is the invariant that would have caught the Myers-diff line-dropping bug
//! (see bugs.md, `tests/bug_myers_diff_drops_duplicate_lines.rs`) on day one:
//! filtering out Deleted lines from the rendered output must reproduce the new
//! source, and filtering out Added lines must reproduce the old source.
//!
//! `#[ignore]`d until `plan-core-hardening` Phase 2 replaces the hand-rolled Myers
//! engine with `similar` — today this fails whenever the generated bodies contain
//! duplicate/repeated lines, which is exactly the bug's trigger condition.

use diffviz_core::ReviewableDiff;
use diffviz_core::ast_diff::SourceCode;
use diffviz_core::common::ProgrammingLanguage;
use diffviz_core::decision_based_diff::create_reviewable_diff_from_range;
use diffviz_core::parsers::RustParser;
use diffviz_core::renderable_diff::{ChangeType, RenderableDiff, RenderableLine};
use proptest::prelude::*;

fn reconstruct_new<'a>(lines: &'a [RenderableLine<'a>]) -> Vec<&'a str> {
    lines
        .iter()
        .filter(|l| !matches!(l.primary_change_type(), Some(ChangeType::Deleted)))
        .map(|l| l.content)
        .collect()
}

fn reconstruct_old<'a>(lines: &'a [RenderableLine<'a>]) -> Vec<&'a str> {
    lines
        .iter()
        .filter(|l| !matches!(l.primary_change_type(), Some(ChangeType::Added)))
        .map(|l| l.content)
        .collect()
}

/// A line body safe to embed inside a Rust fn without disturbing its brace/paren
/// structure: no newlines, no braces or parens (unbalanced parens can make
/// tree-sitter's error recovery split the function into more than one node), so
/// the wrapping function stays exactly one top-level semantic unit regardless of
/// what content proptest generates. Drawn from a small alphabet so duplicate
/// lines (the Myers snake-loop's trigger condition) are common rather than rare.
fn line_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("a".to_string()),
        Just("b".to_string()),
        Just(String::new()),
        Just("stmt".to_string()),
    ]
}

fn render_modification(old_source: &SourceCode, new_source: &SourceCode) -> Vec<ReviewableDiff> {
    let parser = RustParser::new();
    create_reviewable_diff_from_range(
        "src/lib.rs",
        1,
        2,
        Some(old_source),
        new_source,
        ProgrammingLanguage::Rust,
        &parser,
    )
    .expect("diff creation should succeed")
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    #[ignore = "fails today: Myers snake loop drops lines on duplicate-line inputs (fixed by plan-core-hardening Phase 2)"]
    fn replaying_ops_reconstructs_both_sources(
        old_lines in prop::collection::vec(line_strategy(), 0..8),
        new_lines in prop::collection::vec(line_strategy(), 0..8),
    ) {
        let old_full = format!("fn f() {{\n{}\n}}\n", old_lines.join("\n"));
        let new_full = format!("fn f() {{\n{}\n}}\n", new_lines.join("\n"));

        let old = SourceCode::new(old_full.clone());
        let new = SourceCode::new(new_full.clone());

        let diffs = render_modification(&old, &new);
        prop_assert_eq!(diffs.len(), 1);
        let renderable = RenderableDiff::try_from(&diffs[0]).expect("rendering should succeed");

        let expected_old: Vec<&str> = old_full.lines().collect();
        let expected_new: Vec<&str> = new_full.lines().collect();

        prop_assert_eq!(reconstruct_old(&renderable.lines), expected_old);
        prop_assert_eq!(reconstruct_new(&renderable.lines), expected_new);
    }
}
