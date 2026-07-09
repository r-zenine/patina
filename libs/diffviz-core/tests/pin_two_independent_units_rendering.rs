//! Resolves the caveat carried by decision D006 in
//! `.plans/plan-patina-detect/decision-log.yaml`: `pin_added_boundary_rendering.rs`
//! proves `old_source: None` (audit mode) works for a *single* unit, but
//! patina-detect's Type-2 clones detector (Phase 4) needs to render two
//! independent clone-group members side by side — two separate `new_source`
//! calls, neither "old" for the other. That call shape was never exercised.
//! This test drives `create_reviewable_diff_from_range` twice, independently,
//! against two structurally-identical-but-differently-named functions, and
//! pins that each call renders correctly and in isolation from the other
//! (no cross-contamination between the two independent parses/renders).

use diffviz_core::ast_diff::SourceCode;
use diffviz_core::common::ProgrammingLanguage;
use diffviz_core::decision_based_diff::create_reviewable_diff_from_range;
use diffviz_core::parsers::RustParser;
use diffviz_core::renderable_diff::{ChangeType, RenderableDiff};

#[test]
fn two_independent_new_source_calls_render_side_by_side_without_cross_contamination() {
    let parser = RustParser::new();

    let member_a_source = "fn compute_total(items: &[i32]) -> i32 {\n    items.iter().sum()\n}\n";
    let member_b_source = "fn compute_sum(values: &[i32]) -> i32 {\n    values.iter().sum()\n}\n";

    let new_source_a = SourceCode::new(member_a_source.to_string());
    let new_source_b = SourceCode::new(member_b_source.to_string());

    // Two entirely independent calls — neither source is ever passed as the
    // other's `old_source`. This is the call shape D006 left unverified.
    let diffs_a = create_reviewable_diff_from_range(
        "src/a.rs",
        1,
        3,
        None,
        &new_source_a,
        ProgrammingLanguage::Rust,
        &parser,
    )
    .expect("member a diff creation should succeed");

    let diffs_b = create_reviewable_diff_from_range(
        "src/b.rs",
        1,
        3,
        None,
        &new_source_b,
        ProgrammingLanguage::Rust,
        &parser,
    )
    .expect("member b diff creation should succeed");

    assert_eq!(diffs_a.len(), 1, "member a is one semantic unit");
    assert_eq!(diffs_b.len(), 1, "member b is one semantic unit");

    let renderable_a = RenderableDiff::try_from(&diffs_a[0]).expect("member a should render");
    let renderable_b = RenderableDiff::try_from(&diffs_b[0]).expect("member b should render");

    // Each renders as a pure addition on its own terms — the presence of the
    // other member's render never leaks in (no shared numbering, no shared
    // annotations).
    for renderable in [&renderable_a, &renderable_b] {
        for (i, line) in renderable.lines.iter().enumerate() {
            assert_eq!(
                line.line_number,
                i + 1,
                "each render numbers from 1 independently"
            );
            assert_eq!(
                line.primary_change_type(),
                Some(&ChangeType::Added),
                "each independently-rendered member is a pure addition"
            );
        }
    }

    // The two renders are distinct content, proving they weren't diffed
    // against each other despite being structurally near-identical.
    let text_a: Vec<_> = renderable_a.lines.iter().map(|l| &l.content).collect();
    let text_b: Vec<_> = renderable_b.lines.iter().map(|l| &l.content).collect();
    assert_ne!(
        text_a, text_b,
        "the two independent renders keep their own content"
    );
}
