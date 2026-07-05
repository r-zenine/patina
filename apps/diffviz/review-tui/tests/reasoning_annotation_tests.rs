//! Integration tests for inline reasoning annotation feature
//!
//! These tests define the full contract for the `<Space>tr` toggle and the
//! inline decision reasoning annotations rendered inside DrillNav chunk cards.
//!
//! **Test groups:**
//! - Group 1 (toggle state): `show_reasoning` on `UiState`/`StateSnapshot`
//! - Group 2 (annotation injection, Phase 3): annotation lines rendered inside
//!   `drillnav_drill.rs` chunk cards at the correct trigger line, mauve, no
//!   sigil, per design contribution 004. The old diff_view title-bar `◆`
//!   badge (cross-decision membership) was dropped, not ported — see D2 of
//!   that design.

#![cfg(feature = "test-harness")]

use diffviz_review::providers::mock_provider::MockDiffProvider;
use diffviz_review::{
    CodeImpact, Decision, DecisionLineRange, DiffQuery, GitRef, ReviewEngineBuilder,
};
use diffviz_review_tui::test_harness::{CombinedTestHarness, InputTestHarness};

/// Reasoning text that is unique and cannot appear in the fixture source code
const DECISION_REASONING: &str =
    "Extract subtract method from Calculator to reduce coupling with arithmetic core";

/// Create a test engine where Decision 1 maps to `src/models/calculator.rs`.
///
/// The file has 72 lines in the mock fixture — line range 1-72 covers the whole file.
/// Decision 2 has no code impacts (edge case: decisions without diffs).
fn create_test_engine_with_decisions() -> diffviz_review::engines::ReviewEngine {
    let mock_provider =
        MockDiffProvider::from_review_fixtures().expect("Failed to load test fixtures");
    let review_engine_builder =
        ReviewEngineBuilder::new(Box::new(mock_provider), "test-user".to_string());
    let diff_query = DiffQuery::new(GitRef::Head, GitRef::Unstaged);

    let decisions = vec![
        Decision {
            number: 1,
            title: "Refactor calculator implementation".to_string(),
            rationale: Some("Add subtract method to Calculator struct".to_string()),
            code_impacts: vec![CodeImpact {
                file: "src/models/calculator.rs".to_string(),
                line_ranges: vec![DecisionLineRange { start: 1, end: 72 }],
                reasoning: DECISION_REASONING.to_string(),
            }],
        },
        Decision {
            number: 2,
            title: "Architectural logging decision".to_string(),
            rationale: Some("No code changes — pure architectural decision".to_string()),
            code_impacts: vec![],
        },
    ];

    review_engine_builder
        .build_from_decisions(decisions, diff_query)
        .expect("Failed to build ReviewEngine")
}

// =============================================================================
// Group 1 — Toggle State
//
// These tests access `snapshot.show_reasoning`, which does not exist yet.
// They will fail to COMPILE until Phase 1 adds `show_reasoning: bool` to
// `UiState` and `StateSnapshot`. This compile failure is the intended "red"
// state for TDD Phase 0.
// =============================================================================

/// `show_reasoning` defaults to `false` when no input is provided.
#[test]
fn show_reasoning_defaults_to_false() {
    let engine = create_test_engine_with_decisions();
    let mut harness = InputTestHarness::new(engine);

    let state = harness.run_sequence_final_state("").expect("Initial state");

    assert!(
        !state.show_reasoning,
        "show_reasoning should default to false"
    );
}

/// `<Space>tr` turns `show_reasoning` on.
#[test]
fn space_tr_toggles_reasoning_on() {
    let engine = create_test_engine_with_decisions();
    let mut harness = InputTestHarness::new(engine);

    let state = harness
        .run_sequence_final_state("<Space>tr")
        .expect("Toggle reasoning on");

    assert!(
        state.show_reasoning,
        "show_reasoning should be true after <Space>tr"
    );
}

/// A second `<Space>tr` turns `show_reasoning` back off.
#[test]
fn space_tr_toggles_reasoning_off_again() {
    let engine = create_test_engine_with_decisions();
    let mut harness = InputTestHarness::new(engine);

    let state = harness
        .run_sequence_final_state("<Space>tr<Space>tr")
        .expect("Toggle reasoning twice");

    assert!(
        !state.show_reasoning,
        "show_reasoning should be false after toggling twice"
    );
}

// =============================================================================
// Group 2 — Annotation Injection (Visual, DrillNav drill view)
//
// Navigation: `<Enter>` drills into Decision 1's first (only) chunk.
// =============================================================================

/// When `show_reasoning` is on in the drill view, the reasoning text from the
/// decision's `CodeImpact.reasoning` should appear in the rendered chunk card.
#[test]
fn annotation_lines_appear_when_reasoning_on() {
    let engine = create_test_engine_with_decisions();
    // Use 120×40 so the 79-char DECISION_REASONING fits without wrapping
    // (at default 80×24 the drill card is too narrow)
    let mut harness = CombinedTestHarness::with_render_size(engine, 120, 40);

    let results = harness
        .run_sequence_with_renders("<Enter><Space>tr")
        .expect("Drill into chunk and enable reasoning");
    let visual = results.last().expect("Should have results").visual.clone();

    assert!(
        visual.contains(DECISION_REASONING),
        "Annotation with reasoning text should appear when show_reasoning=true.\nLooked for: {DECISION_REASONING:?}\nVisual output:\n{visual}"
    );
}

/// When `show_reasoning` is off (default), no reasoning text should appear
/// in the rendered chunk card.
#[test]
fn annotation_lines_absent_when_reasoning_off() {
    let engine = create_test_engine_with_decisions();
    let mut harness = CombinedTestHarness::new(engine);

    let results = harness
        .run_sequence_with_renders("<Enter>")
        .expect("Drill into chunk");
    let visual = results.last().expect("Should have results").visual.clone();

    assert!(
        !visual.contains(DECISION_REASONING),
        "Annotation text should NOT appear when show_reasoning=false.\nVisual output:\n{visual}"
    );
}

/// When reasoning is on, the annotation line (`D1`) should appear in the
/// rendered output before the last line of the chunk card — i.e. inline at
/// the trigger line, not appended at the end.
#[test]
fn annotation_appears_at_correct_position() {
    let engine = create_test_engine_with_decisions();
    // Use 120×40 so the 79-char DECISION_REASONING fits without wrapping
    let mut harness = CombinedTestHarness::with_render_size(engine, 120, 40);

    let results = harness
        .run_sequence_with_renders("<Enter><Space>tr")
        .expect("Drill into chunk and enable reasoning");
    let visual = results.last().expect("Should have results").visual.clone();

    assert!(
        visual.contains("D1"),
        "Annotation label 'D1' should appear in the chunk card when reasoning is on.\nVisual output:\n{visual}"
    );

    let lines: Vec<&str> = visual.lines().collect();
    let reasoning_line_idx = lines.iter().position(|l| l.contains(DECISION_REASONING));

    assert!(
        reasoning_line_idx.is_some(),
        "Reasoning text should appear in visual output"
    );

    let idx = reasoning_line_idx.unwrap();
    assert!(
        idx < lines.len().saturating_sub(1),
        "Reasoning annotation should not be the last line of output (it should appear inline in the chunk card, not appended at the end)"
    );
}
