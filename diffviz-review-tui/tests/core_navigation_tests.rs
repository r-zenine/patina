//! Phase 1: Core Navigation Steel Thread
//!
//! Tests basic navigation functionality using j/k/arrows/gg/G keybindings.
//! This is the foundation for all other TUI tests - validating that cursor
//! movement through the decision tree works correctly.
//!
//! Test Organization:
//! - Single key navigation (j, k, arrows)
//! - Multi-key sequences (jjj, kkk, jjkk)
//! - Boundary navigation (top, bottom, wraparound)
//! - Jump navigation (gg, G)
//!
//! All tests use InputTestHarness for fast state validation without rendering.

#![cfg(feature = "test-harness")]

use diffviz_review::providers::mock_provider::MockDiffProvider;
use diffviz_review::{
    ChangeType, CodeImpact, Confidence, Decision, DecisionLineRange, DiffQuery, GitRef,
    ReviewDecisions, ReviewEngineBuilder,
};
use diffviz_review_tui::test_harness::InputTestHarness;

// =============================================================================
// Test Setup
// =============================================================================

/// Create a test ReviewEngine with 3 decisions for navigation testing
fn create_test_engine() -> diffviz_review::engines::ReviewEngine {
    let mock_provider =
        MockDiffProvider::from_review_fixtures().expect("Failed to load test fixtures");
    let review_engine_builder =
        ReviewEngineBuilder::new(Box::new(mock_provider), "test-user".to_string());
    let diff_query = DiffQuery::new(GitRef::Head, GitRef::Unstaged);
    let mut review_engine = review_engine_builder
        .build(diff_query)
        .expect("Failed to build ReviewEngine");

    // Set up 3 test decisions for consistent navigation testing
    let mut decisions = ReviewDecisions::new();

    decisions.add_decision(Decision {
        number: 1,
        title: "Decision 1: Navigation Test".to_string(),
        summary: "First decision for testing navigation".to_string(),
        decision_log_line: Some(1),
        code_impacts: vec![CodeImpact {
            file: "src/lib.rs".to_string(),
            line_ranges: vec![DecisionLineRange { start: 1, end: 10 }],
            change_type: ChangeType::Modification,
            confidence: Confidence::High,
            reasoning: "Test impact".to_string(),
        }],
    });

    decisions.add_decision(Decision {
        number: 2,
        title: "Decision 2: Navigation Test".to_string(),
        summary: "Second decision for testing navigation".to_string(),
        decision_log_line: Some(2),
        code_impacts: vec![CodeImpact {
            file: "src/lib.rs".to_string(),
            line_ranges: vec![DecisionLineRange { start: 11, end: 20 }],
            change_type: ChangeType::Modification,
            confidence: Confidence::Medium,
            reasoning: "Test impact".to_string(),
        }],
    });

    decisions.add_decision(Decision {
        number: 3,
        title: "Decision 3: Navigation Test".to_string(),
        summary: "Third decision for testing navigation".to_string(),
        decision_log_line: Some(3),
        code_impacts: vec![CodeImpact {
            file: "src/lib.rs".to_string(),
            line_ranges: vec![DecisionLineRange { start: 21, end: 30 }],
            change_type: ChangeType::Modification,
            confidence: Confidence::Low,
            reasoning: "Test impact".to_string(),
        }],
    });

    review_engine.set_decisions_with_index(decisions);
    review_engine
}

// =============================================================================
// Single Key Navigation Tests
// =============================================================================

#[test]
fn test_navigation_j_moves_down_one_position() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let snapshots = harness.run_sequence("j").expect("Run sequence");

    // Should have initial state + 1 event = 2 snapshots
    assert_eq!(snapshots.len(), 2);
    assert_eq!(snapshots[0].decision_tree_path.0, 0, "Initial position at 0");
    assert_eq!(snapshots[1].decision_tree_path.0, 1, "After 'j', position at 1");
}

#[test]
fn test_navigation_k_moves_up_one_position() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Start at position 1, then move up
    let snapshots = harness.run_sequence("jk").expect("Run sequence");

    assert_eq!(snapshots.len(), 3, "Initial + 2 events");
    assert_eq!(snapshots[0].decision_tree_path.0, 0, "Start at 0");
    assert_eq!(snapshots[1].decision_tree_path.0, 1, "After 'j' at 1");
    assert_eq!(snapshots[2].decision_tree_path.0, 0, "After 'k' back at 0");
}

#[test]
fn test_navigation_down_arrow_moves_down() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let snapshots = harness.run_sequence("<Down>").expect("Run sequence");

    assert_eq!(snapshots.len(), 2);
    assert_eq!(snapshots[0].decision_tree_path.0, 0);
    assert_eq!(snapshots[1].decision_tree_path.0, 1, "Arrow down moves cursor");
}

#[test]
fn test_navigation_up_arrow_moves_up() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let snapshots = harness.run_sequence("j<Up>").expect("Run sequence");

    assert_eq!(snapshots.len(), 3);
    assert_eq!(snapshots[2].decision_tree_path.0, 0, "Arrow up moves cursor");
}

// =============================================================================
// Multi-Key Sequence Tests
// =============================================================================

#[test]
fn test_navigation_multiple_j_moves_down_multiple_positions() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let snapshots = harness.run_sequence("jjj").expect("Run sequence");

    // Should have initial + 3 events = 4 snapshots
    assert_eq!(snapshots.len(), 4);
    assert_eq!(snapshots[0].decision_tree_path.0, 0);
    assert_eq!(snapshots[1].decision_tree_path.0, 1);
    assert_eq!(snapshots[2].decision_tree_path.0, 2);
    // With 3 decisions collapsed, max position is 2 (indices 0, 1, 2)
    assert_eq!(snapshots[3].decision_tree_path.0, 2, "After 'jjj' at bottom (position 2)");
}

#[test]
fn test_navigation_multiple_k_moves_up_multiple_positions() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Move down 3, then up 2
    let snapshots = harness.run_sequence("jjjkk").expect("Run sequence");

    assert_eq!(snapshots.len(), 6);
    assert_eq!(snapshots[3].decision_tree_path.0, 2, "After 'jjj' at bottom (position 2)");
    assert_eq!(snapshots[4].decision_tree_path.0, 1, "After first 'k'");
    assert_eq!(snapshots[5].decision_tree_path.0, 0, "After second 'k'");
}

#[test]
fn test_navigation_mixed_sequence_j_and_k() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let snapshots = harness.run_sequence("jjkj").expect("Run sequence");

    assert_eq!(snapshots.len(), 5);
    assert_eq!(snapshots[0].decision_tree_path.0, 0);
    assert_eq!(snapshots[1].decision_tree_path.0, 1);
    assert_eq!(snapshots[2].decision_tree_path.0, 2);
    assert_eq!(snapshots[3].decision_tree_path.0, 1, "After 'k' back to 1");
    assert_eq!(snapshots[4].decision_tree_path.0, 2, "After 'j' to 2");
}

#[test]
fn test_navigation_mixed_arrows_and_vim_keys() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let snapshots = harness.run_sequence("j<Down>k<Up>").expect("Run sequence");

    assert_eq!(snapshots.len(), 5);
    assert_eq!(snapshots[0].decision_tree_path.0, 0);
    assert_eq!(snapshots[1].decision_tree_path.0, 1, "After 'j'");
    assert_eq!(snapshots[2].decision_tree_path.0, 2, "After '<Down>'");
    assert_eq!(snapshots[3].decision_tree_path.0, 1, "After 'k'");
    assert_eq!(snapshots[4].decision_tree_path.0, 0, "After '<Up>'");
}

// =============================================================================
// Boundary Navigation Tests
// =============================================================================

#[test]
fn test_navigation_k_at_top_stays_at_top() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Try to move up from initial position (should stay at 0)
    let snapshots = harness.run_sequence("k").expect("Run sequence");

    assert_eq!(snapshots.len(), 2);
    assert_eq!(snapshots[0].decision_tree_path.0, 0, "Start at top");
    assert_eq!(snapshots[1].decision_tree_path.0, 0, "Stay at top after 'k'");
}

#[test]
fn test_navigation_multiple_k_at_top_stays_at_top() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let snapshots = harness.run_sequence("kkk").expect("Run sequence");

    assert_eq!(snapshots.len(), 4);
    for snapshot in &snapshots {
        assert_eq!(
            snapshot.decision_tree_path.0, 0,
            "Should stay at top regardless of 'k' presses"
        );
    }
}

#[test]
fn test_navigation_j_past_bottom_behavior() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Move down many times to test bottom boundary
    // With 3 decisions and their files/chunks, we should have multiple items in tree
    let snapshots = harness.run_sequence("jjjjjjjjjjjjjjjjjjjj").expect("Run sequence");

    // Get the last position reached
    let max_position = snapshots.last().unwrap().decision_tree_path.0;

    // Verify we didn't go negative or wrap to 0
    assert!(
        max_position > 0,
        "Should not wrap to 0 or go negative when at bottom"
    );

    // Verify position stabilized (last few snapshots should have same position)
    let last_5: Vec<_> = snapshots.iter().rev().take(5).collect();
    let positions: Vec<_> = last_5.iter().map(|s| s.decision_tree_path.0).collect();
    assert!(
        positions.windows(2).all(|w| w[0] == w[1]),
        "Position should stabilize at bottom, got: {:?}",
        positions
    );
}

#[test]
fn test_navigation_to_bottom_and_back_to_top() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Move down to bottom, then all the way back up
    let snapshots = harness
        .run_sequence("jjjjjjjjjjkkkkkkkkkkkkk")
        .expect("Run sequence");

    // Last snapshot should be back at position 0
    assert_eq!(
        snapshots.last().unwrap().decision_tree_path.0,
        0,
        "Should return to top after enough 'k' presses"
    );
}

// =============================================================================
// Jump Navigation Tests
// =============================================================================

#[test]
#[ignore = "Jump to top (gg) not yet implemented"]
fn test_navigation_gg_jumps_to_top() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Move down, then jump to top with gg
    let snapshots = harness.run_sequence("jjjgg").expect("Run sequence");

    assert_eq!(
        snapshots.last().unwrap().decision_tree_path.0,
        0,
        "gg should jump to top"
    );
}

#[test]
#[ignore = "Jump to bottom (G) not yet implemented"]
fn test_navigation_shift_g_jumps_to_bottom() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Start at top, jump to bottom with G
    let snapshots = harness.run_sequence("<S-g>").expect("Run sequence");

    let last_pos = snapshots.last().unwrap().decision_tree_path.0;
    assert!(last_pos > 0, "G should jump to bottom (pos > 0)");

    // Verify we're actually at the bottom by trying to go down
    let snapshots2 = harness.run_sequence("j").expect("Run sequence");
    assert_eq!(
        snapshots2.last().unwrap().decision_tree_path.0,
        last_pos,
        "Should stay at bottom after G"
    );
}

#[test]
#[ignore = "Jump navigation (gg/G) not yet implemented"]
fn test_navigation_gg_then_shift_g_covers_full_range() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Jump to bottom, then to top, then to bottom again
    let snapshots = harness.run_sequence("<S-g>gg<S-g>").expect("Run sequence");

    let pos_after_g = snapshots[1].decision_tree_path.0;
    let pos_after_gg = snapshots[2].decision_tree_path.0;
    let pos_after_g2 = snapshots[3].decision_tree_path.0;

    assert!(pos_after_g > 0, "After G should be at bottom");
    assert_eq!(pos_after_gg, 0, "After gg should be at top");
    assert_eq!(pos_after_g2, pos_after_g, "After G again should be at bottom");
}

// =============================================================================
// State Consistency Tests
// =============================================================================

#[test]
fn test_navigation_preserves_focused_panel() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let snapshots = harness.run_sequence("jjkk").expect("Run sequence");

    // All snapshots should maintain the same focused panel during navigation
    let initial_panel = &snapshots[0].focused_panel;
    for snapshot in &snapshots {
        assert_eq!(
            &snapshot.focused_panel, initial_panel,
            "Navigation should not change focused panel"
        );
    }
}

#[test]
fn test_navigation_updates_decision_tree_path_only() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let snapshots = harness.run_sequence("jk").expect("Run sequence");

    // Verify only decision_tree_path changes, other state remains stable
    assert_eq!(
        snapshots[0].input_mode, snapshots[1].input_mode,
        "Input mode should not change"
    );
    assert_eq!(
        snapshots[0].input_mode, snapshots[2].input_mode,
        "Input mode should not change"
    );
    assert_eq!(
        snapshots[0].leader_active, snapshots[1].leader_active,
        "Leader state should not change"
    );
    assert_eq!(
        snapshots[0].show_help, snapshots[1].show_help,
        "Help state should not change"
    );
}

#[test]
fn test_navigation_sequence_creates_expected_snapshot_count() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let sequence = "jjjkkjk";
    let snapshots = harness.run_sequence(sequence).expect("Run sequence");

    // Should have initial state + 7 events = 8 snapshots
    let expected_count = sequence.len() + 1;
    assert_eq!(
        snapshots.len(),
        expected_count,
        "Each key should produce one snapshot plus initial"
    );
}
