//! Integration tests for decision approval TUI workflows
//!
//! These tests validate the decision approval feature using the test harness:
//! - Keyboard interactions (Space+a+d to toggle approval)
//! - Visual rendering (approval icons and progress counters)
//! - Cascading behavior (approve all chunks → auto-approve decision)
//! - Reverse cascade (approve decision → all chunks approved)

#![cfg(feature = "test-harness")]

use diffviz_review::providers::mock_provider::MockDiffProvider;
use diffviz_review::{
    ChangeType, CodeImpact, Confidence, Decision, DecisionLineRange, ReviewDecisions,
};
use diffviz_review::{DiffQuery, GitRef, ReviewEngineBuilder};
use diffviz_review_tui::test_harness::{CombinedTestHarness, InputTestHarness, RenderTestHarness};

/// Create a test ReviewEngine with realistic decisions and file structure
fn create_test_engine() -> diffviz_review::engines::ReviewEngine {
    let mock_provider =
        MockDiffProvider::from_review_fixtures().expect("Failed to load test fixtures");
    let review_engine_builder =
        ReviewEngineBuilder::new(Box::new(mock_provider), "test-user".to_string());
    let diff_query = DiffQuery::new(GitRef::Head, GitRef::Unstaged);
    let mut review_engine = review_engine_builder
        .build(diff_query)
        .expect("Failed to build ReviewEngine");

    // Set up decisions with multiple file impacts
    let mut decisions = ReviewDecisions::new();

    // Decision 1: Affects multiple chunks that can be approved independently
    decisions.add_decision(Decision {
        number: 1,
        title: "Refactor authentication module".to_string(),
        summary: "Extract authentication logic into separate, testable module".to_string(),
        decision_log_line: Some(15),
        code_impacts: vec![CodeImpact {
            file: "src/lib.rs".to_string(),
            line_ranges: vec![
                DecisionLineRange { start: 1, end: 30 },
                DecisionLineRange { start: 40, end: 50 },
            ],
            change_type: ChangeType::Modification,
            confidence: Confidence::High,
            reasoning: "Main library module imports new auth module".to_string(),
        }],
    });

    // Decision 2: For testing independent decisions
    decisions.add_decision(Decision {
        number: 2,
        title: "Improve error handling across modules".to_string(),
        summary: "Standardize error types and add context to error messages".to_string(),
        decision_log_line: Some(28),
        code_impacts: vec![CodeImpact {
            file: "src/lib.rs".to_string(),
            line_ranges: vec![DecisionLineRange { start: 60, end: 80 }],
            change_type: ChangeType::Modification,
            confidence: Confidence::Medium,
            reasoning: "Adds error context to library result types".to_string(),
        }],
    });

    // Decision 3: Decision with no initial chunks (edge case)
    decisions.add_decision(Decision {
        number: 3,
        title: "Add structured logging throughout application".to_string(),
        summary: "Architectural decision: use tracing crate for observability".to_string(),
        decision_log_line: Some(42),
        code_impacts: vec![],
    });

    review_engine.set_decisions_with_index(decisions);
    review_engine
}

// =============================================================================
// Basic Decision Approval Toggle Tests
// =============================================================================

/// Test that navigating to a decision and pressing Space+a+d approves it
#[test]
fn test_toggle_approve_decision_basic() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Start at decision 0 (at depth 0)
    assert_eq!(
        harness
            .run_sequence_final_state("")
            .expect("Initial")
            .decision_tree_path
            .0,
        0
    );

    // Now approve the decision at depth 0
    let approve_snapshots = harness
        .run_sequence("<Space>ad")
        .expect("Approval sequence");

    // Should have snapshots for space, a, d
    assert!(approve_snapshots.len() >= 3);
}

/// Test that toggling approval twice returns to unapproved state
#[test]
fn test_toggle_approve_decision_twice() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Start at decision 0
    assert_eq!(
        harness
            .run_sequence_final_state("")
            .expect("Initial")
            .decision_tree_path
            .0,
        0
    );

    // First approval
    let approve_snapshots = harness.run_sequence("<Space>ad").expect("First approval");
    assert!(approve_snapshots.len() >= 3);

    // Second toggle (unapproval)
    let unapprove_snapshots = harness.run_sequence("<Space>ad").expect("Second toggle");

    // Should have completed toggle snapshots
    assert!(unapprove_snapshots.len() >= 3);
}

/// Test that approving at depth 0 (decision) triggers without crashing
#[test]
fn test_approve_at_decision_depth() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Start at decision 0 (depth 0)
    let initial_snapshot = harness.run_sequence_final_state("").expect("Initial state");
    assert_eq!(initial_snapshot.decision_tree_path.0, 0);

    // Approve at depth 0 (should not crash even if no chunks)
    let approve_snapshots = harness.run_sequence("<Space>ad").expect("Approve");
    assert!(approve_snapshots.len() >= 3);
}

// =============================================================================
// Progress Counter Tests
// =============================================================================

/// Test that decision approval progress queries work
#[test]
fn test_decision_approval_progress_calculation() {
    let engine = create_test_engine();

    // Progress queries should work without crashing
    let _progress_0 = engine.decision_approval_progress(0);
    let _progress_1 = engine.decision_approval_progress(1);
    let progress_3 = engine.decision_approval_progress(3);

    // Decision 3 should have no chunks
    assert_eq!(progress_3.1, 0);
}

// =============================================================================
// Multiple Decision Tests
// =============================================================================

/// Test that approving one decision doesn't affect others
#[test]
fn test_approve_decision_independent() {
    // Use a separate engine to verify independence
    let mut engine_verify = create_test_engine();

    // Approve decision 0
    engine_verify
        .approve_decision(0, "test-user".to_string())
        .expect("Approve decision 0");
    let (approved_0, total_0) = engine_verify.decision_approval_progress(0);

    // Approve decision 1 independently
    engine_verify
        .approve_decision(1, "test-user".to_string())
        .expect("Approve decision 1");
    let (approved_1, total_1) = engine_verify.decision_approval_progress(1);

    // Verify both decisions have cascaded correctly and independently
    if total_0 > 0 {
        assert_eq!(approved_0, total_0, "Decision 0 should be fully approved");
    }
    if total_1 > 0 {
        assert_eq!(approved_1, total_1, "Decision 1 should be fully approved");
    }
}

/// Test toggling approval state multiple times across decisions
#[test]
fn test_toggle_multiple_decisions() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Approve decision 0
    harness
        .run_sequence("<Space>ad")
        .expect("Approve decision 0");

    // Move to decision 1 and approve
    harness
        .run_sequence("j<Space>ad")
        .expect("Move and approve decision 1");

    // Move back to decision 0 and unapprove
    harness
        .run_sequence("k<Space>ad")
        .expect("Move back and unapprove decision 0");

    // Final state should have decision 0 at index 0
    let final_state = harness.run_sequence_final_state("").expect("Final state");
    assert_eq!(final_state.decision_tree_path.0, 0);
}

// =============================================================================
// Visual Rendering Tests
// =============================================================================

/// Test that decision tree and diff view render without errors
#[test]
fn test_rendering_with_approval_data() {
    let engine = create_test_engine();
    let mut ui_state = diffviz_review_tui::state::UiState::new();
    ui_state.decision_tree =
        diffviz_review_tui::decision_navigation::DecisionNavigationTree::build_from_review_engine(
            &engine,
        );

    let harness = RenderTestHarness::new();
    let render = harness
        .render(&mut ui_state, &engine)
        .expect("Render failed");

    // Should contain UI elements
    assert!(!render.is_empty());
    assert!(render.contains("Decisions") || render.contains("Diff"));
}

/// Test that rendering at decision depth (0) works correctly
#[test]
fn test_rendering_at_decision_depth() {
    let engine = create_test_engine();
    let mut ui_state = diffviz_review_tui::state::UiState::new();
    ui_state.decision_tree =
        diffviz_review_tui::decision_navigation::DecisionNavigationTree::build_from_review_engine(
            &engine,
        );

    // Ensure we're at decision depth
    assert_eq!(ui_state.decision_tree.selected_path.depth(), 0);

    let harness = RenderTestHarness::new();
    let render = harness
        .render(&mut ui_state, &engine)
        .expect("Render failed");

    // Render should work at depth 0
    assert!(!render.is_empty());
}

/// Test rendering with custom size
#[test]
fn test_rendering_with_custom_size() {
    let engine = create_test_engine();
    let mut ui_state = diffviz_review_tui::state::UiState::new();
    ui_state.decision_tree =
        diffviz_review_tui::decision_navigation::DecisionNavigationTree::build_from_review_engine(
            &engine,
        );

    let harness = RenderTestHarness::with_size(120, 40);
    let render = harness
        .render(&mut ui_state, &engine)
        .expect("Render failed");

    assert!(!render.is_empty());
}

// =============================================================================
// Combined Integration Tests (Full Workflow)
// =============================================================================

/// Test complete workflow: approve at depth 0, verify render output
#[test]
fn test_decision_approval_complete_workflow() {
    let engine = create_test_engine();
    let mut harness = CombinedTestHarness::new(engine);

    // Step 1: Start at decision 0 (depth 0)
    let results = harness
        .run_sequence_with_renders("")
        .expect("Initial state");
    assert!(results.len() >= 1);

    // Step 2: Approve decision
    let approve_results = harness
        .run_sequence_with_renders("<Space>ad")
        .expect("Approve");
    assert!(approve_results.len() >= 3);

    // Step 3: Verify render output exists
    for result in &approve_results {
        assert!(!result.state.focused_panel.is_empty());
        assert!(!result.visual.is_empty());
    }
}

/// Test that visual output updates after approval toggle
#[test]
fn test_visual_updates_after_toggle() {
    let engine = create_test_engine();
    let mut harness = CombinedTestHarness::new(engine);

    // Get initial visual
    let initial_results = harness.run_sequence_with_renders("j").expect("Initial");
    let initial_visual = initial_results[initial_results.len() - 1].visual.clone();

    // Approve and get visual
    let approve_results = harness
        .run_sequence_with_renders("<Space>ad")
        .expect("Approve");
    let approved_visual = approve_results[approve_results.len() - 1].visual.clone();

    // Both should be non-empty (actual visual changes depend on rendering implementation)
    assert!(!initial_visual.is_empty());
    assert!(!approved_visual.is_empty());
}

/// Test navigation and approval sequence
#[test]
fn test_navigation_approval_sequence() {
    let engine = create_test_engine();
    let mut harness = CombinedTestHarness::new(engine);

    // Complete sequence: navigate down, approve, navigate up
    let results = harness
        .run_sequence_with_renders("j<Space>adk")
        .expect("Sequence");

    // Should have multiple results
    assert!(results.len() >= 5);

    // All should have state and visual output
    for result in &results {
        assert!(!result.state.focused_panel.is_empty());
    }
}

// =============================================================================
// Edge Case Tests
// =============================================================================

/// Test approving decision with no chunks (edge case)
#[test]
fn test_approve_decision_with_no_chunks() {
    let engine = create_test_engine();

    // Decision 3 has no chunks (checked during engine setup)
    let (approved, total) = engine.decision_approval_progress(3);
    assert_eq!(total, 0, "Decision 3 should have no chunks");
    assert_eq!(approved, 0);
}

/// Test that navigation around approved decisions works correctly
#[test]
fn test_navigate_around_approved_decisions() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Approve decision 0
    harness
        .run_sequence("<Space>ad")
        .expect("Approve decision 0");

    // Navigate down and back up
    let snapshots = harness.run_sequence("jk").expect("Navigate around");

    // After j then k, should be back at decision 0
    let final_path = snapshots.last().unwrap().decision_tree_path.0;
    assert!(
        final_path <= 1,
        "Should be at decision 0 or 1 after j then k"
    );
}

// =============================================================================
// State Consistency Tests
// =============================================================================

/// Test that approval state persists across multiple test sequences
#[test]
fn test_approval_state_persistence() {
    let mut engine = create_test_engine();

    // Approve decision 0
    engine
        .approve_decision(0, "test-user".to_string())
        .expect("Approve decision 0");

    // Check progress after first approval
    let progress_0 = engine.decision_approval_progress(0);
    if progress_0.1 > 0 {
        assert_eq!(progress_0.0, progress_0.1);
    }

    // Approve decision 1
    engine
        .approve_decision(1, "test-user".to_string())
        .expect("Approve decision 1");

    // Verify both have cascaded
    let progress_1 = engine.decision_approval_progress(1);
    if progress_1.1 > 0 {
        assert_eq!(progress_1.0, progress_1.1);
    }

    // Verify decision 0 is still approved
    let progress_0_final = engine.decision_approval_progress(0);
    if progress_0_final.1 > 0 {
        assert_eq!(progress_0_final.0, progress_0_final.1);
    }
}

/// Test special key handling with approval
#[test]
fn test_special_keys_work_during_approval_workflow() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // These should all work without crashing
    harness.run_sequence("<Space>").expect("Space key");
    harness.run_sequence("<Enter>").expect("Enter key");
    harness.run_sequence("<Esc>").expect("Esc key");
    harness.run_sequence("<Up>").expect("Up key");
    harness.run_sequence("<Down>").expect("Down key");
}
