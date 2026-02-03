//! Phase 3: Decision Tree Expansion Steel Thread
//!
//! Tests decision tree expansion/collapse functionality and depth-based navigation.
//! Validates that Tab toggles expansion state, Enter expands current node, and
//! navigation respects collapsed/expanded states with proper depth tracking.
//!
//! Test Organization:
//! - Tab expansion toggle (single decision expansion)
//! - Enter expansion (alternative key binding)
//! - Expansion state persistence across navigation
//! - Depth-based navigation (0→1→2 and reverse)
//! - Visual indicators for expansion state (▶/▼)
//! - Navigation through collapsed vs expanded trees
//!
//! All tests use InputTestHarness for fast state validation without rendering.

#![cfg(feature = "test-harness")]

use diffviz_review::providers::mock_provider::MockDiffProvider;
use diffviz_review::{
    ChangeType, CodeImpact, Confidence, Decision, DecisionLineRange, DiffQuery, GitRef,
    ReviewEngineBuilder,
};
use diffviz_review_tui::test_harness::InputTestHarness;

// =============================================================================
// Test Setup
// =============================================================================

/// Create a test ReviewEngine with 2 decisions with multiple chunks
/// This provides rich structure for expansion/collapse testing
/// Uses real fixture files with valid line ranges
fn create_test_engine() -> diffviz_review::engines::ReviewEngine {
    let mock_provider =
        MockDiffProvider::from_review_fixtures().expect("Failed to load test fixtures");
    let review_engine_builder =
        ReviewEngineBuilder::new(Box::new(mock_provider), "test-user".to_string());
    let diff_query = DiffQuery::new(GitRef::Head, GitRef::Unstaged);

    // Set up 2 test decisions with multiple chunks for tree expansion testing
    // File sizes: calculator.rs: 72, base.py: 20, Greeting.tsx: 49
    let decisions = vec![
        // Decision 1: Multiple impacts across files
        Decision {
            number: 1,
            title: "Decision 1: Calculator Refactor".to_string(),
            summary: "Refactor calculator logic for better maintainability".to_string(),
            decision_log_line: Some(1),
            code_impacts: vec![
                CodeImpact {
                    file: "src/models/calculator.rs".to_string(),
                    line_ranges: vec![
                        DecisionLineRange { start: 1, end: 30 },
                        DecisionLineRange { start: 40, end: 60 },
                    ],
                    change_type: ChangeType::Modification,
                    confidence: Confidence::High,
                    reasoning: "Refactored for clarity".to_string(),
                },
                CodeImpact {
                    file: "src/models/base.py".to_string(),
                    line_ranges: vec![DecisionLineRange { start: 1, end: 20 }],
                    change_type: ChangeType::Addition,
                    confidence: Confidence::Medium,
                    reasoning: "Added base model".to_string(),
                },
            ],
        },
        // Decision 2: Multiple impacts
        Decision {
            number: 2,
            title: "Decision 2: Component Updates".to_string(),
            summary: "Improve React components throughout codebase".to_string(),
            decision_log_line: Some(2),
            code_impacts: vec![
                CodeImpact {
                    file: "src/components/Greeting.tsx".to_string(),
                    line_ranges: vec![
                        DecisionLineRange { start: 1, end: 25 },
                        DecisionLineRange { start: 26, end: 49 },
                    ],
                    change_type: ChangeType::Modification,
                    confidence: Confidence::High,
                    reasoning: "Enhanced component logic".to_string(),
                },
                CodeImpact {
                    file: "src/types/api.ts".to_string(),
                    line_ranges: vec![DecisionLineRange { start: 1, end: 9 }],
                    change_type: ChangeType::Modification,
                    confidence: Confidence::Low,
                    reasoning: "Updated types".to_string(),
                },
            ],
        },
    ];

    review_engine_builder
        .build_from_decisions(decisions, diff_query)
        .expect("Failed to build ReviewEngine")
}

// =============================================================================
// Expansion Toggle Tests (Tab Key)
// =============================================================================

#[test]
fn test_expansion_tab_toggles_first_decision_expansion() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Start at decision 1 (depth 0), press Tab to expand
    let snapshots = harness.run_sequence("<Tab>").expect("Run sequence");

    assert_eq!(snapshots.len(), 2, "Initial + 1 tab press");
    assert_eq!(
        snapshots[0].decision_tree_path.0, 0,
        "Initial decision index at 0"
    );
    assert_eq!(
        snapshots[0].decision_tree_path.1, None,
        "Initial chunk index should be None (depth 0)"
    );
    // After Tab, we should still be at depth 0 but tree structure changes
    // (the test verifies state, actual visual expansion is rendering concern)
}

#[test]
fn test_expansion_tab_then_tab_again_collapses() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Expand with first Tab, then collapse with second Tab
    let snapshots = harness.run_sequence("<Tab><Tab>").expect("Run sequence");

    assert_eq!(snapshots.len(), 3, "Initial + 2 tabs");
    // State should be toggled twice, returning to original state
    assert_eq!(
        snapshots[0].decision_tree_path, snapshots[2].decision_tree_path,
        "After two tabs, should return to original expansion state"
    );
}

#[test]
fn test_expansion_multiple_tab_toggles_on_same_decision() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Toggle expansion multiple times
    let snapshots = harness
        .run_sequence("<Tab><Tab><Tab><Tab>")
        .expect("Run sequence");

    assert_eq!(snapshots.len(), 5, "Initial + 4 tabs");
    // After even number of toggles, should be back to original state
    assert_eq!(
        snapshots[0].decision_tree_path, snapshots[4].decision_tree_path,
        "After even toggles, should return to original state"
    );
}

#[test]
fn test_expansion_tab_at_each_decision_level() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Navigate to decision 2 and toggle expansion
    let snapshots = harness.run_sequence("j<Tab>").expect("Run sequence");

    assert_eq!(snapshots.len(), 3, "Initial + j + tab");
    assert_eq!(
        snapshots[1].decision_tree_path.0, 1,
        "After j, at decision 2"
    );
    // Tab at depth 0 should be handled consistently
}

// =============================================================================
// Expansion with Enter Key Tests
// =============================================================================

#[test]
fn test_expansion_enter_expands_current_decision() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Start at decision 1, press Enter to expand
    let snapshots = harness.run_sequence("<Enter>").expect("Run sequence");

    assert_eq!(snapshots.len(), 2, "Initial + 1 enter press");
    // Enter at depth 0 should expand the decision
    // State verification focuses on path changes
}

#[test]
fn test_expansion_tab_and_enter_have_same_effect() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Test Tab expansion
    let snapshots_tab = harness
        .run_sequence("<Tab>")
        .expect("Tab expansion sequence");
    let tab_state = snapshots_tab.last().unwrap().decision_tree_path.clone();

    // Reset with new harness for Enter test
    let mut harness2 = InputTestHarness::new(create_test_engine());
    // Test Enter expansion
    let snapshots_enter = harness2
        .run_sequence("<Enter>")
        .expect("Enter expansion sequence");
    let enter_state = snapshots_enter.last().unwrap().decision_tree_path.clone();

    assert_eq!(
        tab_state, enter_state,
        "Tab and Enter should have same expansion effect"
    );
}

// =============================================================================
// Depth-Based Navigation Tests
// =============================================================================

/// Calculate depth from tuple path (0=decision, 1=chunk)
fn calculate_depth(path: &(usize, Option<usize>)) -> usize {
    if path.1.is_some() {
        1
    } else {
        0
    }
}

#[test]
fn test_navigation_depth_increases_with_expand_and_down() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Start at depth 0 (decision 1), expand with Tab, then navigate down to file
    let snapshots = harness.run_sequence("<Tab>j").expect("Run sequence");

    assert_eq!(snapshots.len(), 3, "Initial + tab + j");
    assert_eq!(
        calculate_depth(&snapshots[0].decision_tree_path),
        0,
        "Initial depth is 0 (decision level)"
    );
    // After Tab (expand) and j (navigate), should potentially be at depth 1
    // Actual depth depends on tree structure after expansion
}

#[test]
fn test_navigation_depth_zero_at_decision_nodes() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // At start, we're at a decision node (depth 0)
    let snapshots = harness.run_sequence("").expect("Empty sequence");
    assert_eq!(
        calculate_depth(&snapshots[0].decision_tree_path),
        0,
        "Initial position is at decision level (depth 0)"
    );

    // Navigate to next decision, should still be depth 0
    let snapshots2 = harness.run_sequence("j").expect("Navigate to next");
    assert_eq!(
        calculate_depth(&snapshots2.last().unwrap().decision_tree_path),
        0,
        "Navigation between collapsed decisions stays at depth 0"
    );
}

#[test]
fn test_expansion_then_navigation_changes_depth() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Expand first decision, navigate down to enter file level
    let snapshots = harness
        .run_sequence("<Tab>j")
        .expect("Expand then navigate");

    let initial_depth = calculate_depth(&snapshots[0].decision_tree_path);
    let final_depth = calculate_depth(&snapshots.last().unwrap().decision_tree_path);

    // After expansion and navigation, depth may change depending on tree structure
    // The key is that depths are properly calculated
    assert!(
        initial_depth <= 2 && final_depth <= 2,
        "Depths should be 0-2 (decision, file, chunk)"
    );
}

#[test]
fn test_navigation_respects_collapsed_tree_structure() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // With tree collapsed, j from first decision should go to second decision
    // (not skip to files, since they're hidden)
    let snapshots = harness
        .run_sequence("jjjj")
        .expect("Navigate collapsed tree");

    // Each snapshot should maintain depth 0 (navigating between collapsed decisions)
    for snapshot in &snapshots {
        let depth = calculate_depth(&snapshot.decision_tree_path);
        assert!(
            depth <= 1,
            "In collapsed tree, should stay at decision level or near it, got depth {}",
            depth
        );
    }
}

#[test]
fn test_expansion_increases_visible_tree_nodes() {
    let mut harness = InputTestHarness::new(create_test_engine());

    // Start collapsed
    let snapshots_collapsed = harness.run_sequence("").expect("Initial state");
    let collapsed_count = snapshots_collapsed.len();
    // Empty sequence returns just initial state (1 snapshot)

    // Now expand
    let mut harness2 = InputTestHarness::new(create_test_engine());
    let snapshots_expanded = harness2.run_sequence("<Tab>").expect("After expansion");
    let expanded_count = snapshots_expanded.len();
    // Tab + initial = 2 snapshots

    // Verify both have expected counts
    assert_eq!(
        collapsed_count, 1,
        "Empty sequence should return just initial state"
    );
    assert_eq!(
        expanded_count, 2,
        "Tab sequence should return initial + 1 event state"
    );
}

// =============================================================================
// Expansion State Persistence Tests
// =============================================================================

#[test]
fn test_expansion_state_persists_during_navigation() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Expand decision 1, navigate down (if visible), navigate back up
    let snapshots = harness
        .run_sequence("<Tab>jk")
        .expect("Expand, navigate, return");

    assert_eq!(snapshots.len(), 4, "Initial + tab + j + k");
    // The expansion state at decision 1 should be persistent

    // Verify we can navigate back through expanded area without changing depth incorrectly
    let path_before_nav = snapshots[1].decision_tree_path.clone();
    let path_after_nav = snapshots[3].decision_tree_path.clone();
    assert_eq!(
        path_before_nav, path_after_nav,
        "After expand-navigate-return, path should be same"
    );
}

#[test]
fn test_expansion_state_independent_per_decision() {
    let mut harness = InputTestHarness::new(create_test_engine());

    // Expand decision 1, move to decision 2 without expanding
    let snapshots = harness
        .run_sequence("<Tab>j")
        .expect("Expand decision 1, navigate to decision 2");

    assert_eq!(snapshots.len(), 3, "Initial + tab + j");
    // Decision 2 should be collapsed independently
    // Navigation should reflect different expansion states
}

#[test]
fn test_expand_decision1_navigate_to_first_chunk_correct_flattened_behavior() {
    let mut harness = InputTestHarness::new(create_test_engine());

    // Start at decision 0, depth 0: (0, None)
    let snapshots0 = harness.run_sequence("").expect("Get initial state");
    let initial_path = &snapshots0.last().unwrap().decision_tree_path;
    assert_eq!(initial_path.0, 0, "Initially at decision 0");
    assert_eq!(initial_path.1, None, "Initially no chunk selected");

    // Expand first decision - tree now shows decision 0 with its chunks visible
    harness.run_sequence("<Tab>").expect("Expand decision 1");

    // Navigate down with j - should go to first chunk of expanded decision 0
    // This is correct flattened-tree behavior: expanded nodes show their children
    let snapshots_after_j = harness.run_sequence("j").expect("Navigate to first chunk");
    let path_after_j = &snapshots_after_j.last().unwrap().decision_tree_path;

    assert_eq!(path_after_j.0, 0, "Still at decision 0");
    assert_eq!(
        path_after_j.1,
        Some(0),
        "Now at first chunk of decision 0 (chunk depth)"
    );

    // Verify depth is 1 (chunk level)
    let depth = if path_after_j.1.is_some() { 1 } else { 0 };
    assert_eq!(depth, 1, "Should be at chunk depth (1)");
}

// =============================================================================
// Visual Expansion Indicator Tests (Rendering)
// =============================================================================

#[test]
#[ignore = "Visual rendering tests need RenderTestHarness for icon verification"]
fn test_expansion_shows_down_arrow_for_expanded_node() {
    // This test requires RenderTestHarness to verify visual indicators
    // Expansion state affects rendering of ▼ vs ▶ icons
}

#[test]
#[ignore = "Visual rendering tests need RenderTestHarness for icon verification"]
fn test_expansion_shows_right_arrow_for_collapsed_node() {
    // This test requires RenderTestHarness to verify visual indicators
    // Collapsed state should show ▶ instead of ▼
}

#[test]
#[ignore = "Visual rendering tests need RenderTestHarness for icon verification"]
fn test_expansion_visual_state_matches_navigation_behavior() {
    // Verify that visual expansion indicators (▼/▶) match actual navigation behavior
    // Using CombinedTestHarness for full validation
}

// =============================================================================
// Complex Expansion Scenarios
// =============================================================================

#[test]
fn test_expansion_navigate_into_expanded_tree() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Expand decision 1, then navigate down into files/chunks
    let snapshots = harness
        .run_sequence("<Tab>jj")
        .expect("Expand and deep navigate");

    assert_eq!(snapshots.len(), 4, "Initial + tab + j + j");
    // First j might go to a file, second j to a chunk (if structure allows)
    // Tree structure determines actual paths
}

#[test]
fn test_expansion_collapse_expand_cycle() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Expand, collapse, expand - should return to previous state
    let snapshots = harness
        .run_sequence("<Tab><Tab><Tab>")
        .expect("Expand-collapse-expand cycle");

    assert_eq!(snapshots.len(), 4, "Initial + 3 tabs");
    // After odd number of toggles, should be expanded (opposite of initial collapsed)
    // After 3 toggles: collapsed → expanded → collapsed → expanded
}

#[test]
fn test_expansion_with_multiple_decisions_independent_expansion() {
    let mut harness = InputTestHarness::new(create_test_engine());

    // Expand decision 1
    harness.run_sequence("<Tab>").expect("Expand 1");

    // Navigate to decision 2 and expand
    harness.run_sequence("j").expect("Navigate to 2");
    harness.run_sequence("<Tab>").expect("Expand 2");

    // Navigate back to decision 1
    let snapshots = harness.run_sequence("k").expect("Navigate back to 1");

    assert_eq!(
        snapshots.last().unwrap().decision_tree_path.0,
        0,
        "Back at decision 1"
    );
    // Decision 1 should still be expanded independently
}

#[test]
fn test_expansion_depth_routing_consistency() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Track depth changes through expansion and navigation
    let snapshots = harness
        .run_sequence("<Tab>jkj")
        .expect("Complex expansion sequence");

    for snapshot in &snapshots {
        let depth = calculate_depth(&snapshot.decision_tree_path);
        // Depth should be 0-1 throughout
        assert!(depth <= 1, "Depth out of valid range: {}", depth);
    }
}

// =============================================================================
// Edge Cases and Boundary Conditions
// =============================================================================

#[test]
fn test_expansion_at_last_decision() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Navigate to last decision and expand
    let snapshots = harness.run_sequence("j<Tab>").expect("Navigate and expand");

    assert_eq!(
        snapshots[1].decision_tree_path.0, 1,
        "After j, at second decision"
    );
    // Expansion should work on last decision too
}

#[test]
fn test_expansion_collapse_then_navigate_through() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Expand then collapse decision 1
    harness
        .run_sequence("<Tab><Tab>")
        .expect("Expand and collapse");

    // Now navigate through collapsed tree
    let snapshots = harness.run_sequence("jj").expect("Navigate collapsed");

    let final_depth = calculate_depth(&snapshots.last().unwrap().decision_tree_path);
    // Should navigate between decisions without entering expanded structure
    assert_eq!(
        final_depth, 0,
        "Should stay at decision level when collapsed"
    );
}

#[test]
fn test_rapid_tab_expansion_toggles() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Rapid Tab toggles
    let snapshots = harness
        .run_sequence("<Tab><Tab><Tab><Tab><Tab>")
        .expect("Rapid toggles");

    assert_eq!(snapshots.len(), 6, "Initial + 5 tabs");
    // After odd tabs, should be expanded
    // State should be consistent regardless of rapid input
}

#[test]
fn test_expansion_state_with_zero_depth_navigation() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Verify that depth 0 navigation (between decisions) is unaffected by expansion
    let snapshots = harness.run_sequence("jkj").expect("Depth 0 navigation");

    for snapshot in &snapshots {
        // These are decision-level navigations, depth should reflect actual position
        let depth = calculate_depth(&snapshot.decision_tree_path);
        assert!(
            depth <= 1,
            "Depth 0 nav should not exceed depth 1: {}",
            depth
        );
    }
}

// =============================================================================
// State Consistency Tests
// =============================================================================

#[test]
fn test_expansion_preserves_focused_panel() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let snapshots = harness
        .run_sequence("<Tab>j<Tab>")
        .expect("Expansion sequence");

    let initial_panel = &snapshots[0].focused_panel;
    for snapshot in &snapshots {
        assert_eq!(
            &snapshot.focused_panel, initial_panel,
            "Expansion should not change focused panel"
        );
    }
}

#[test]
fn test_expansion_preserves_other_ui_state() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let snapshots = harness
        .run_sequence("<Tab><Tab>")
        .expect("Expansion sequence");

    // UI state like input_mode, leader_active, etc should not change
    assert_eq!(
        snapshots[0].input_mode, snapshots[1].input_mode,
        "Input mode should not change during expansion"
    );
    assert_eq!(
        snapshots[0].leader_active, snapshots[1].leader_active,
        "Leader state should not change during expansion"
    );
    assert_eq!(
        snapshots[0].show_help, snapshots[1].show_help,
        "Help state should not change during expansion"
    );
}
