//! Integration tests for inline reasoning annotation feature
//!
//! These tests define the full contract for the `<Space>tr` toggle and the
//! inline decision reasoning annotations injected into the diff view.
//!
//! Strategy: TDD — all tests are written before any implementation.
//!
//! **Test groups and their phase dependencies:**
//! - Group 1 (toggle state): requires Phase 1 — will fail to compile until
//!   `show_reasoning` is added to `UiState` and `StateSnapshot`
//! - Group 2 (title badge): requires Phase 3 — will fail at runtime until
//!   the badge logic is added to `diff_view.rs`
//! - Group 3 (annotation injection): requires Phase 1 + 2 + 3 — will fail
//!   at runtime until the full pipeline is wired up

#![cfg(feature = "test-harness")]

use diffviz_review::providers::mock_provider::MockDiffProvider;
use diffviz_review::{CodeImpact, Decision, DecisionLineRange, DiffQuery, GitRef, ReviewEngineBuilder};
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

    let state = harness
        .run_sequence_final_state("")
        .expect("Initial state");

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
// Group 2 — Title Badge
//
// These tests validate the visual ◆ badge in the diff panel title.
// They will fail at runtime until Phase 3 wires the badge into `diff_view.rs`.
//
// Navigation: `<Tab>j` expands Decision 1 and moves to its first chunk.
// =============================================================================

/// When at diff depth (depth 1) with decisions mapped and `show_reasoning` off,
/// the diff panel title should contain the `◆ D1` badge.
#[test]
fn title_badge_shown_when_reasoning_off_and_decisions_exist() {
    let engine = create_test_engine_with_decisions();
    let mut harness = CombinedTestHarness::new(engine);

    // Navigate to depth 1: expand Decision 1 (<Tab>), move to its first chunk (j)
    let results = harness
        .run_sequence_with_renders("<Tab>j")
        .expect("Navigate to chunk");
    let visual = results.last().expect("Should have results").visual.clone();

    // show_reasoning is false (default), decisions exist → badge should appear
    assert!(
        visual.contains("◆ D"),
        "Title badge '◆ D' should appear when at chunk depth with decisions and show_reasoning=false.\nVisual output:\n{visual}"
    );
}

/// When `show_reasoning` is toggled on at diff depth, the title badge should
/// not appear (annotations are visible in the diff body instead).
#[test]
fn title_badge_hidden_when_reasoning_on() {
    let engine = create_test_engine_with_decisions();
    let mut harness = CombinedTestHarness::new(engine);

    // Navigate to depth 1 then toggle reasoning on
    let results = harness
        .run_sequence_with_renders("<Tab>j<Space>tr")
        .expect("Navigate to chunk and enable reasoning");
    let visual = results.last().expect("Should have results").visual.clone();

    // show_reasoning is true → badge should be hidden (annotations shown instead)
    assert!(
        !visual.contains("◆ D"),
        "Title badge '◆ D' should be hidden when show_reasoning=true.\nVisual output:\n{visual}"
    );
}

// =============================================================================
// Group 3 — Annotation Injection (Visual)
//
// These tests validate that annotation lines appear in the diff body.
// They require Phase 1 (toggle), Phase 2 (widget), and Phase 3 (data flow).
// =============================================================================

/// When `show_reasoning` is on at diff depth, the reasoning text from the
/// decision's `CodeImpact.reasoning` should appear in the rendered diff body.
#[test]
fn annotation_lines_appear_when_reasoning_on() {
    let engine = create_test_engine_with_decisions();
    let mut harness = CombinedTestHarness::new(engine);

    // Navigate to depth 1 then toggle reasoning on
    let results = harness
        .run_sequence_with_renders("<Tab>j<Space>tr")
        .expect("Navigate to chunk and enable reasoning");
    let visual = results.last().expect("Should have results").visual.clone();

    assert!(
        visual.contains(DECISION_REASONING),
        "Annotation with reasoning text should appear when show_reasoning=true.\nLooked for: {DECISION_REASONING:?}\nVisual output:\n{visual}"
    );
}

/// When `show_reasoning` is off (default), no reasoning text should appear
/// in the rendered diff body.
#[test]
fn annotation_lines_absent_when_reasoning_off() {
    let engine = create_test_engine_with_decisions();
    let mut harness = CombinedTestHarness::new(engine);

    // Navigate to depth 1 but do NOT toggle reasoning
    let results = harness
        .run_sequence_with_renders("<Tab>j")
        .expect("Navigate to chunk");
    let visual = results.last().expect("Should have results").visual.clone();

    assert!(
        !visual.contains(DECISION_REASONING),
        "Annotation text should NOT appear when show_reasoning=false.\nVisual output:\n{visual}"
    );
}

/// When reasoning is on, the annotation line (`◆ D1`) should appear in the
/// rendered output before or at the first changed line of the diff.
///
/// This test validates positioning: the annotation must not appear after all
/// diff content (it should be inline at the trigger line, not appended at end).
#[test]
fn annotation_appears_at_correct_position() {
    let engine = create_test_engine_with_decisions();
    let mut harness = CombinedTestHarness::new(engine);

    let results = harness
        .run_sequence_with_renders("<Tab>j<Space>tr")
        .expect("Navigate to chunk and enable reasoning");
    let visual = results.last().expect("Should have results").visual.clone();

    // The annotation (◆ D1) and the reasoning text must both appear
    assert!(
        visual.contains("◆ D1"),
        "Annotation gutter symbol '◆ D1' should appear in diff body when reasoning is on.\nVisual output:\n{visual}"
    );

    // The annotation must appear BEFORE the end of the diff content.
    // Heuristic: the reasoning text must appear somewhere before the last few lines.
    // We verify it's not appended at the very end by checking it's not the last line.
    let lines: Vec<&str> = visual.lines().collect();
    let reasoning_line_idx = lines.iter().position(|l| l.contains(DECISION_REASONING));

    assert!(
        reasoning_line_idx.is_some(),
        "Reasoning text should appear in visual output"
    );

    let idx = reasoning_line_idx.unwrap();
    assert!(
        idx < lines.len().saturating_sub(1),
        "Reasoning annotation should not be the last line of output (it should appear inline in the diff, not appended at the end)"
    );
}
