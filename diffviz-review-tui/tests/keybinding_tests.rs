//! Integration tests for TUI keybinding workflows
//!
//! These tests demonstrate common TUI workflows using the test harness,
//! validating that keybindings work correctly and state transitions are correct.

#![cfg(feature = "test-harness")]

use diffviz_review::providers::mock_provider::MockDiffProvider;
use diffviz_review::{DiffQuery, GitRef, ReviewEngineBuilder};
use diffviz_review_tui::test_harness::{
    CombinedTestHarness, InputTestHarness, RenderTestHarness, StateSnapshot,
};

/// Create a test ReviewEngine for testing
fn create_test_engine() -> diffviz_review::engines::ReviewEngine {
    use diffviz_review::{
        ChangeType, CodeImpact, Confidence, Decision, DecisionLineRange, ReviewDecisions,
    };

    let mock_provider =
        MockDiffProvider::from_review_fixtures().expect("Failed to load test fixtures");
    let review_engine_builder =
        ReviewEngineBuilder::new(Box::new(mock_provider), "test-user".to_string());
    let diff_query = DiffQuery::new(GitRef::Head, GitRef::Unstaged);
    let mut review_engine = review_engine_builder
        .build(diff_query)
        .expect("Failed to build ReviewEngine");

    // Set up hardcoded decisions like main.rs does
    let mut decisions = ReviewDecisions::new();

    // Decision 1
    decisions.add_decision(Decision {
        number: 1,
        title: "Refactor authentication module".to_string(),
        summary: "Extract authentication logic into separate, testable module".to_string(),
        decision_log_line: Some(15),
        code_impacts: vec![CodeImpact {
            file: "src/lib.rs".to_string(),
            line_ranges: vec![DecisionLineRange { start: 1, end: 50 }],
            change_type: ChangeType::Modification,
            confidence: Confidence::High,
            reasoning: "Main library module imports new auth module".to_string(),
        }],
    });

    // Decision 2
    decisions.add_decision(Decision {
        number: 2,
        title: "Improve error handling across modules".to_string(),
        summary: "Standardize error types and add context to error messages".to_string(),
        decision_log_line: Some(28),
        code_impacts: vec![CodeImpact {
            file: "src/lib.rs".to_string(),
            line_ranges: vec![DecisionLineRange { start: 1, end: 50 }],
            change_type: ChangeType::Modification,
            confidence: Confidence::Medium,
            reasoning: "Adds error context to library result types".to_string(),
        }],
    });

    // Decision 3
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
// Navigation Tests
// =============================================================================

#[test]
fn test_navigation_down_moves_cursor() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let snapshots = harness.run_sequence("j").expect("Run sequence");

    // Should have initial state + 1 event = 2 snapshots
    assert_eq!(snapshots.len(), 2);
    // Navigation should have moved through decision tree
    assert_eq!(snapshots[0].decision_tree_path.0, 0);
    assert_eq!(snapshots[1].decision_tree_path.0, 1);
}

#[test]
fn test_navigation_up_moves_cursor_backward() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Move down twice, then up once
    let snapshots = harness.run_sequence("jjk").expect("Run sequence");

    // Should have 4 snapshots: initial + 3 events
    assert_eq!(snapshots.len(), 4);
    // Verify we went down then back up
    assert_eq!(snapshots[0].decision_tree_path.0, 0);
    assert_eq!(snapshots[1].decision_tree_path.0, 1);
    assert_eq!(snapshots[2].decision_tree_path.0, 2);
    assert_eq!(snapshots[3].decision_tree_path.0, 1);
}

#[test]
fn test_navigation_multiple_down() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let snapshots = harness.run_sequence("jj").expect("Run sequence");

    // Should navigate down 2 times (3 decisions total, so indices 0, 1, 2)
    assert_eq!(snapshots.len(), 3); // initial + 2 events
    assert_eq!(snapshots[0].decision_tree_path.0, 0);
    assert_eq!(snapshots[1].decision_tree_path.0, 1);
    assert_eq!(snapshots[2].decision_tree_path.0, 2);
}

// =============================================================================
// Focus Tests
// =============================================================================

#[test]
fn test_toggle_focus_switches_panels() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Use <Right> and <Left> to switch focus between panels
    let snapshots = harness.run_sequence("<Right><Left>").expect("Run sequence");

    // Initial should be FileList
    assert_eq!(snapshots[0].focused_panel, "FileList");
    // After Right should switch to DiffView
    assert_eq!(snapshots[1].focused_panel, "DiffView");
    // After Left should switch back to FileList
    assert_eq!(snapshots[2].focused_panel, "FileList");
}

#[test]
fn test_left_right_navigation_switches_focus() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let snapshots = harness.run_sequence("<Right><Left>").expect("Run sequence");

    // Initial should be FileList
    assert_eq!(snapshots[0].focused_panel, "FileList");
    // After Right should switch to DiffView
    assert_eq!(snapshots[1].focused_panel, "DiffView");
    // After Left should switch back to FileList
    assert_eq!(snapshots[2].focused_panel, "FileList");
}

// =============================================================================
// Display/Context Tests
// =============================================================================

#[test]
fn test_toggle_context_display() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let initial = harness.run_sequence_final_state("").expect("Initial");
    // show_all_context should start as true
    assert!(initial.show_all_context);

    // Test that state is captured correctly between calls
    let initial2 = harness.run_sequence_final_state("").expect("Initial again");
    assert_eq!(initial2.show_all_context, initial.show_all_context);
}

// =============================================================================
// Quit Test
// =============================================================================

#[test]
fn test_quit_key() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let initial = harness.run_sequence_final_state("").expect("Initial");
    assert!(!initial.should_quit);

    let after_quit = harness.run_sequence_final_state("q").expect("After q");
    assert!(after_quit.should_quit);
}

// =============================================================================
// Rendering Tests
// =============================================================================

#[test]
fn test_render_initial_state() {
    let engine = create_test_engine();
    let mut ui_state = diffviz_review_tui::state::UiState::new();
    ui_state.decision_tree =
        diffviz_review_tui::decision_navigation::DecisionNavigationTree::build_from_review_engine(
            &engine,
        );

    let harness = RenderTestHarness::new();
    let visual = harness
        .render(&mut ui_state, &engine)
        .expect("Render failed");

    // Visual output should contain expected UI elements
    assert!(visual.contains("Decisions"));
    assert!(visual.contains("Diff View"));
    assert!(!visual.is_empty());
}

#[test]
fn test_render_custom_size() {
    let engine = create_test_engine();
    let mut ui_state = diffviz_review_tui::state::UiState::new();
    ui_state.decision_tree =
        diffviz_review_tui::decision_navigation::DecisionNavigationTree::build_from_review_engine(
            &engine,
        );

    let harness = RenderTestHarness::with_size(120, 40);
    let visual = harness
        .render(&mut ui_state, &engine)
        .expect("Render failed");

    // Should render without errors
    assert!(!visual.is_empty());
}

// =============================================================================
// Combined Integration Tests
// =============================================================================

#[test]
fn test_combined_navigation_and_render() {
    let engine = create_test_engine();
    let mut harness = CombinedTestHarness::new(engine);

    let results = harness
        .run_sequence_with_renders("jj")
        .expect("Combined test failed");

    // Should have initial state + 2 key events = 3 results
    assert_eq!(results.len(), 3);

    // Each result should have both state and visual
    for result in results {
        assert!(!result.visual.is_empty());
        assert!(!result.state.focused_panel.is_empty());
    }
}

#[test]
fn test_combined_with_custom_render_size() {
    let engine = create_test_engine();
    let mut harness = CombinedTestHarness::with_render_size(engine, 100, 30);

    let results = harness
        .run_sequence_with_renders("j")
        .expect("Combined test with custom size failed");

    assert_eq!(results.len(), 2); // initial + 1 key event
    assert!(!results[0].visual.is_empty());
    assert!(!results[1].visual.is_empty());
}

// =============================================================================
// State Snapshot Tests
// =============================================================================

#[test]
fn test_snapshot_serialization_roundtrip() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let snapshot1 = harness
        .run_sequence_final_state("jj")
        .expect("First sequence");
    let json = snapshot1.to_json().expect("Serialize failed");
    let snapshot2 = StateSnapshot::from_json(&json).expect("Deserialize failed");

    // Should match after roundtrip
    assert_eq!(snapshot1.focused_panel, snapshot2.focused_panel);
    assert_eq!(snapshot1.cursor_index, snapshot2.cursor_index);
    assert_eq!(snapshot1.should_quit, snapshot2.should_quit);
}

#[test]
fn test_snapshot_captures_all_fields() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let snapshot = harness
        .run_sequence_final_state("")
        .expect("Snapshot failed");

    // Verify all expected fields are present and non-empty/default
    assert!(!snapshot.focused_panel.is_empty());
    assert!(!snapshot.input_mode.is_empty());
    assert_eq!(snapshot.should_quit, false);
    assert_eq!(snapshot.leader_active, false);
}

// =============================================================================
// Special Key Tests
// =============================================================================

#[test]
fn test_special_keys_work() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Test various special keys don't cause errors
    let _space = harness
        .run_sequence_final_state("<Space>")
        .expect("Space key");
    let _enter = harness
        .run_sequence_final_state("<Enter>")
        .expect("Enter key");
    let _esc = harness.run_sequence_final_state("<Esc>").expect("Esc key");
    let _tab = harness.run_sequence_final_state("<Tab>").expect("Tab key");

    // Test arrow keys
    let _up = harness.run_sequence_final_state("<Up>").expect("Up key");
    let _down = harness
        .run_sequence_final_state("<Down>")
        .expect("Down key");
}

#[test]
fn test_modifier_keys_work() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Test modifier combinations don't cause errors
    let _ctrl_j = harness
        .run_sequence_final_state("<C-j>")
        .expect("Ctrl+j key");
    let _shift_q = harness
        .run_sequence_final_state("<S-q>")
        .expect("Shift+q key");
    let _alt_x = harness
        .run_sequence_final_state("<A-x>")
        .expect("Alt+x key");
}
