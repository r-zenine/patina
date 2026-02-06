//! Phase 2: Panel Management Steel Thread
//!
//! Tests panel focus switching and scrolling functionality.
//! This builds on Phase 1 navigation tests by adding multi-panel coordination.
//!
//! Test Organization:
//! - Panel focus switching (Left/Right/h/l arrows)
//! - Combined navigation + focus switching
//! - Scroll operations (Ctrl+y/e for focused panel)
//! - Inactive panel scrolling (Ctrl+j/k)
//! - Scroll state persistence across focus switches
//! - Page scrolling (Ctrl+b/f)
//!
//! All tests use InputTestHarness for state validation.

#![cfg(feature = "test-harness")]

use diffviz_review::providers::mock_provider::MockDiffProvider;
use diffviz_review::{
    CodeImpact, Decision, DecisionLineRange, DiffQuery, GitRef,
    ReviewDecisions, ReviewEngineBuilder,
};
use diffviz_review_tui::test_harness::InputTestHarness;

// =============================================================================
// Test Setup
// =============================================================================

/// Create a test ReviewEngine with 3 decisions for panel management testing
fn create_test_engine() -> diffviz_review::engines::ReviewEngine {
    let mock_provider =
        MockDiffProvider::from_review_fixtures().expect("Failed to load test fixtures");
    let review_engine_builder =
        ReviewEngineBuilder::new(Box::new(mock_provider), "test-user".to_string());
    let diff_query = DiffQuery::new(GitRef::Head, GitRef::Unstaged);
    let mut review_engine = review_engine_builder
        .build_from_decisions(vec![], diff_query)
        .expect("Failed to build ReviewEngine");

    // Set up 3 test decisions for consistent testing
    let mut decisions = ReviewDecisions::new();

    decisions.add_decision(Decision {
        number: 1,
        title: "Decision 1: Panel Test".to_string(),
        rationale: Some("First decision for testing panel management".to_string()),
        decision_log_line: Some(1),
        code_impacts: vec![CodeImpact {
            file: "src/lib.rs".to_string(),
            line_ranges: vec![DecisionLineRange { start: 1, end: 10 }],
            reasoning: "Test impact".to_string(),
        }],
    });

    decisions.add_decision(Decision {
        number: 2,
        title: "Decision 2: Panel Test".to_string(),
        rationale: Some("Second decision for testing panel management".to_string()),
        decision_log_line: Some(2),
        code_impacts: vec![CodeImpact {
            file: "src/lib.rs".to_string(),
            line_ranges: vec![DecisionLineRange { start: 11, end: 20 }],
            reasoning: "Test impact".to_string(),
        }],
    });

    decisions.add_decision(Decision {
        number: 3,
        title: "Decision 3: Panel Test".to_string(),
        rationale: Some("Third decision for testing panel management".to_string()),
        decision_log_line: Some(3),
        code_impacts: vec![CodeImpact {
            file: "src/lib.rs".to_string(),
            line_ranges: vec![DecisionLineRange { start: 21, end: 30 }],
            reasoning: "Test impact".to_string(),
        }],
    });

    review_engine.set_decisions_with_index(decisions);
    review_engine
}

// =============================================================================
// Panel Focus Switching Tests
// =============================================================================

#[test]
fn test_panel_focus_starts_on_file_list() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let snapshots = harness.run_sequence("").expect("Run sequence");

    // Should have just initial state
    assert_eq!(snapshots.len(), 1);
    let initial_state = &snapshots[0];

    // Initial focus should be on FileList (decision tree)
    assert_eq!(initial_state.focused_panel, "FileList");
}

#[test]
fn test_panel_focus_right_arrow_switches_to_diff_view() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let snapshots = harness.run_sequence("<Right>").expect("Run sequence");

    assert_eq!(snapshots.len(), 2);
    let final_state = &snapshots[1];
    assert_eq!(final_state.focused_panel, "DiffView");
}

#[test]
fn test_panel_focus_l_key_switches_to_diff_view() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let snapshots = harness.run_sequence("l").expect("Run sequence");

    assert_eq!(snapshots.len(), 2);
    let final_state = &snapshots[1];
    assert_eq!(final_state.focused_panel, "DiffView");
}

#[test]
fn test_panel_focus_left_arrow_stays_on_file_list() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let snapshots = harness.run_sequence("<Left>").expect("Run sequence");

    assert_eq!(snapshots.len(), 2);
    let final_state = &snapshots[1];
    // Should stay on FileList since we're already there
    assert_eq!(final_state.focused_panel, "FileList");
}

#[test]
fn test_panel_focus_h_key_stays_on_file_list() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let snapshots = harness.run_sequence("h").expect("Run sequence");

    assert_eq!(snapshots.len(), 2);
    let final_state = &snapshots[1];
    assert_eq!(final_state.focused_panel, "FileList");
}

#[test]
fn test_panel_focus_right_then_left_returns_to_file_list() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let snapshots = harness.run_sequence("<Right><Left>").expect("Run sequence");

    assert_eq!(snapshots.len(), 3);
    assert_eq!(snapshots[0].focused_panel, "FileList"); // Initial
    assert_eq!(snapshots[1].focused_panel, "DiffView"); // After right
    assert_eq!(snapshots[2].focused_panel, "FileList"); // After left
}

#[test]
fn test_panel_focus_l_then_h_returns_to_file_list() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let snapshots = harness.run_sequence("lh").expect("Run sequence");

    assert_eq!(snapshots.len(), 3);
    assert_eq!(snapshots[0].focused_panel, "FileList"); // Initial
    assert_eq!(snapshots[1].focused_panel, "DiffView"); // After l
    assert_eq!(snapshots[2].focused_panel, "FileList"); // After h
}

// =============================================================================
// Combined Navigation + Focus Switching Tests
// =============================================================================

#[test]
fn test_navigate_then_switch_focus_preserves_position() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let snapshots = harness.run_sequence("jj<Right>").expect("Run sequence");

    assert_eq!(snapshots.len(), 4);
    let final_state = &snapshots[3];

    // Position should be preserved (moved down 2)
    assert_eq!(final_state.decision_tree_path.0, 2);
    // Focus should have switched
    assert_eq!(final_state.focused_panel, "DiffView");
}

#[test]
fn test_switch_focus_navigate_switch_back_preserves_position() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let snapshots = harness
        .run_sequence("<Right>j<Left>")
        .expect("Run sequence");

    assert_eq!(snapshots.len(), 4);

    // After switching right, j in DiffView doesn't affect tree position
    // Tree position should stay at 0 (j moves cursor in DiffView, not tree)
    assert_eq!(snapshots[2].decision_tree_path.0, 0);
    assert_eq!(snapshots[2].focused_panel, "DiffView");

    // Switching back should preserve tree position (still 0)
    assert_eq!(snapshots[3].decision_tree_path.0, 0);
    assert_eq!(snapshots[3].focused_panel, "FileList");
}

#[test]
fn test_multiple_focus_switches_with_navigation() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let snapshots = harness
        .run_sequence("j<Right>j<Left>k")
        .expect("Run sequence");

    assert_eq!(snapshots.len(), 6);

    // Track the sequence
    assert_eq!(snapshots[1].decision_tree_path.0, 1); // After j in FileList
    assert_eq!(snapshots[2].focused_panel, "DiffView"); // After right
                                                        // After j in DiffView, tree position stays same (j affects cursor in DiffView)
    assert_eq!(snapshots[3].decision_tree_path.0, 1);
    assert_eq!(snapshots[4].focused_panel, "FileList"); // After left
    assert_eq!(snapshots[5].decision_tree_path.0, 0); // After k in FileList (back to 0)
}

// =============================================================================
// Scroll Operations Tests
// =============================================================================

#[test]
#[ignore = "Scroll operations need investigation - need to understand scroll_offset behavior"]
fn test_scroll_down_ctrl_e_increases_offset() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let snapshots = harness.run_sequence("<C-e>").expect("Run sequence");

    assert_eq!(snapshots.len(), 2);
    let final_state = &snapshots[1];

    // Scroll offset should increase
    assert!(final_state.scroll_offset > 0);
}

#[test]
#[ignore = "Scroll operations need investigation - need to understand scroll_offset behavior"]
fn test_scroll_up_ctrl_y_from_scrolled_position() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let snapshots = harness
        .run_sequence("<C-e><C-e><C-y>")
        .expect("Run sequence");

    assert_eq!(snapshots.len(), 4);

    // Should scroll down twice then up once
    let after_two_down = snapshots[2].scroll_offset;
    let after_one_up = snapshots[3].scroll_offset;

    assert!(after_one_up < after_two_down);
}

#[test]
#[ignore = "Scroll operations need investigation - need to understand scroll_offset behavior"]
fn test_scroll_up_ctrl_y_at_top_stays_at_zero() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let snapshots = harness.run_sequence("<C-y>").expect("Run sequence");

    assert_eq!(snapshots.len(), 2);
    let final_state = &snapshots[1];

    // Should stay at 0 when already at top
    assert_eq!(final_state.scroll_offset, 0);
}

#[test]
#[ignore = "Page scroll operations need investigation"]
fn test_scroll_page_down_ctrl_f() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let snapshots = harness.run_sequence("<C-f>").expect("Run sequence");

    assert_eq!(snapshots.len(), 2);
    let final_state = &snapshots[1];

    // Page down should scroll by larger amount
    assert!(final_state.scroll_offset > 0);
}

#[test]
#[ignore = "Page scroll operations need investigation"]
fn test_scroll_page_up_ctrl_b() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let snapshots = harness.run_sequence("<C-f><C-b>").expect("Run sequence");

    assert_eq!(snapshots.len(), 3);

    // Should return to original position
    assert_eq!(snapshots[2].scroll_offset, 0);
}

// =============================================================================
// Inactive Panel Scrolling Tests
// =============================================================================

#[test]
#[ignore = "Inactive panel scrolling needs investigation - need to understand how it tracks separate scroll state"]
fn test_inactive_panel_scroll_down_ctrl_j() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Focus on FileList, scroll the inactive DiffView panel
    let snapshots = harness.run_sequence("<C-j>").expect("Run sequence");

    assert_eq!(snapshots.len(), 2);
    let final_state = &snapshots[1];

    // Focus should remain on FileList
    assert_eq!(final_state.focused_panel, "FileList");
    // TODO: Need to understand how inactive panel scroll is tracked
}

#[test]
#[ignore = "Inactive panel scrolling needs investigation"]
fn test_inactive_panel_scroll_up_ctrl_k() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let snapshots = harness.run_sequence("<C-j><C-k>").expect("Run sequence");

    assert_eq!(snapshots.len(), 3);

    // Focus should remain on FileList throughout
    assert_eq!(snapshots[1].focused_panel, "FileList");
    assert_eq!(snapshots[2].focused_panel, "FileList");
}

// =============================================================================
// Scroll State Persistence Tests
// =============================================================================

#[test]
#[ignore = "Scroll state persistence needs investigation - need to understand if scroll state is per-panel"]
fn test_scroll_state_persists_across_focus_switch() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Scroll in FileList, switch to DiffView, switch back
    let snapshots = harness
        .run_sequence("<C-e><Right><Left>")
        .expect("Run sequence");

    assert_eq!(snapshots.len(), 4);

    let after_scroll = snapshots[1].scroll_offset;
    let after_return = snapshots[3].scroll_offset;

    // Scroll state should be preserved when returning to FileList
    assert_eq!(after_scroll, after_return);
}

#[test]
#[ignore = "Scroll state independence needs investigation"]
fn test_panels_have_independent_scroll_state() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Scroll FileList, switch to DiffView, scroll DiffView, switch back
    let snapshots = harness
        .run_sequence("<C-e><Right><C-e><C-e><Left>")
        .expect("Run sequence");

    assert_eq!(snapshots.len(), 6);

    // Each panel should maintain its own scroll position
    // This test needs deeper understanding of how scroll state is tracked per panel
}

// =============================================================================
// State Consistency Tests
// =============================================================================

#[test]
fn test_focus_switching_preserves_navigation_position() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let snapshots = harness
        .run_sequence("jj<Right><Left>")
        .expect("Run sequence");

    assert_eq!(snapshots.len(), 5);

    // Navigation position should be preserved throughout
    assert_eq!(snapshots[2].decision_tree_path.0, 2); // After jj
    assert_eq!(snapshots[3].decision_tree_path.0, 2); // After right
    assert_eq!(snapshots[4].decision_tree_path.0, 2); // After left
}

#[test]
fn test_focus_switching_only_affects_focused_panel() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let snapshots = harness.run_sequence("<Right>").expect("Run sequence");

    assert_eq!(snapshots.len(), 2);
    let initial_state = &snapshots[0];
    let final_state = &snapshots[1];

    // Only focused_panel should change
    assert_ne!(initial_state.focused_panel, final_state.focused_panel);

    // Everything else should stay the same
    assert_eq!(
        initial_state.decision_tree_path,
        final_state.decision_tree_path
    );
    assert_eq!(initial_state.input_mode, final_state.input_mode);
}

#[test]
fn test_navigation_works_in_both_panels() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Navigate in FileList
    let snapshots = harness.run_sequence("j").expect("Run sequence");
    assert_eq!(snapshots[1].decision_tree_path.0, 1);

    // Switch to DiffView and navigate
    // Note: k in DiffView affects cursor, not tree position
    let snapshots = harness.run_sequence("<Right>k").expect("Run sequence");
    assert_eq!(snapshots[1].focused_panel, "DiffView");
    // Tree position should stay at 1 (where we left it)
    assert_eq!(snapshots[2].decision_tree_path.0, 1);
    // k in DiffView doesn't change tree position
    assert_eq!(snapshots[2].focused_panel, "DiffView");
}
