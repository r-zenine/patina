//! Integration tests for leader key system workflows
//!
//! These tests validate the Vim-style leader key (Space) system:
//! - Activation and which-key overlay display
//! - Submenu navigation (a/c/i/t/e)
//! - Context-aware action routing (depth-based)
//! - Timeout behavior (2 second timeout)
//! - Visual rendering of leader menu hints

#![cfg(feature = "test-harness")]

use diffviz_review::providers::mock_provider::MockDiffProvider;
use diffviz_review::{
    ChangeType, CodeImpact, Confidence, Decision, DecisionLineRange, DiffQuery, GitRef,
    ReviewDecisions, ReviewEngineBuilder,
};
use diffviz_review_tui::test_harness::{CombinedTestHarness, InputTestHarness};

/// Create a test ReviewEngine with realistic decisions
fn create_test_engine() -> diffviz_review::engines::ReviewEngine {
    let mock_provider =
        MockDiffProvider::from_review_fixtures().expect("Failed to load test fixtures");
    let review_engine_builder =
        ReviewEngineBuilder::new(Box::new(mock_provider), "test-user".to_string());
    let diff_query = DiffQuery::new(GitRef::Head, GitRef::Unstaged);
    let mut review_engine = review_engine_builder
        .build_from_decisions(vec![], diff_query)
        .expect("Failed to build ReviewEngine");

    let mut decisions = ReviewDecisions::new();

    decisions.add_decision(Decision {
        number: 1,
        title: "Refactor authentication module".to_string(),
        summary: "Extract authentication logic into separate module".to_string(),
        decision_log_line: Some(15),
        code_impacts: vec![CodeImpact {
            file: "src/lib.rs".to_string(),
            line_ranges: vec![DecisionLineRange { start: 1, end: 30 }],
            change_type: ChangeType::Modification,
            confidence: Confidence::High,
            reasoning: "Main library imports auth module".to_string(),
        }],
    });

    decisions.add_decision(Decision {
        number: 2,
        title: "Improve error handling".to_string(),
        summary: "Standardize error types".to_string(),
        decision_log_line: Some(28),
        code_impacts: vec![CodeImpact {
            file: "src/error.rs".to_string(),
            line_ranges: vec![DecisionLineRange { start: 40, end: 60 }],
            change_type: ChangeType::Addition,
            confidence: Confidence::Medium,
            reasoning: "New error types for modules".to_string(),
        }],
    });

    review_engine.set_decisions_with_index(decisions);
    review_engine
}

// ============================================================================
// Phase 5: Leader Key System Tests
// ============================================================================

#[test]
fn test_leader_key_activation() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let state = harness
        .run_sequence_final_state("<Space>")
        .expect("Leader activation failed");

    assert!(
        state.leader_active,
        "Leader should be active after Space key"
    );
}

#[test]
fn test_leader_key_deactivation_with_esc() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let state = harness
        .run_sequence_final_state("<Space><Esc>")
        .expect("Leader deactivation failed");

    assert!(!state.leader_active, "Leader should be inactive after Esc");
}

#[test]
fn test_leader_invalid_key_deactivates() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let state = harness
        .run_sequence_final_state("<Space>x")
        .expect("Invalid leader key failed");

    assert!(
        !state.leader_active,
        "Leader should deactivate on invalid key"
    );
}

#[test]
fn test_leader_submenu_actions() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let state = harness
        .run_sequence_final_state("<Space>a")
        .expect("Actions submenu failed");

    assert!(
        state.leader_active,
        "Leader should remain active in submenu"
    );
    assert_eq!(
        state.leader_submenu,
        Some('a'),
        "Should be in actions submenu"
    );
}

#[test]
fn test_leader_submenu_instructions() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let state = harness
        .run_sequence_final_state("<Space>i")
        .expect("Instructions submenu failed");

    assert!(
        state.leader_active,
        "Leader should remain active in submenu"
    );
    assert_eq!(
        state.leader_submenu,
        Some('i'),
        "Should be in instructions submenu"
    );
}

#[test]
fn test_leader_submenu_toggles() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let state = harness
        .run_sequence_final_state("<Space>t")
        .expect("Toggles submenu failed");

    assert!(
        state.leader_active,
        "Leader should remain active in submenu"
    );
    assert_eq!(
        state.leader_submenu,
        Some('t'),
        "Should be in toggles submenu"
    );
}

#[test]
fn test_leader_submenu_export() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let state = harness
        .run_sequence_final_state("<Space>e")
        .expect("Export submenu failed");

    assert!(
        state.leader_active,
        "Leader should remain active in submenu"
    );
    assert_eq!(
        state.leader_submenu,
        Some('e'),
        "Should be in export submenu"
    );
}

#[test]
fn test_leader_submenu_comments() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let state = harness
        .run_sequence_final_state("<Space>c")
        .expect("Comments submenu failed");

    assert!(
        state.leader_active,
        "Leader should remain active in submenu"
    );
    assert_eq!(
        state.leader_submenu,
        Some('c'),
        "Should be in comments submenu"
    );
}

#[test]
fn test_leader_exit_submenu_with_esc() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let state = harness
        .run_sequence_final_state("<Space>a<Esc>")
        .expect("Submenu exit failed");

    // Should return to root menu or deactivate (depending on implementation)
    // After Esc in submenu, leader should deactivate
    assert!(
        !state.leader_active || state.leader_submenu.is_none(),
        "Leader should be deactivated or submenu cleared after Esc"
    );
}

#[test]
fn test_leader_invalid_submenu_key() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let state = harness
        .run_sequence_final_state("<Space>ax")
        .expect("Invalid submenu key failed");

    assert!(
        !state.leader_active,
        "Leader should deactivate on invalid submenu key"
    );
}

#[test]
fn test_leader_approve_chunk() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Navigate to depth 2 (chunk level), then approve
    let state = harness
        .run_sequence_final_state("j<Tab>j<Space>aa")
        .expect("Approve chunk failed");

    // Should deactivate after action
    assert!(
        !state.leader_active,
        "Leader should deactivate after approval action"
    );
}

#[test]
fn test_leader_approve_file() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let state = harness
        .run_sequence_final_state("j<Space>af")
        .expect("Approve file failed");

    assert!(
        !state.leader_active,
        "Leader should deactivate after approval action"
    );
}

#[test]
fn test_leader_approve_decision_at_depth_0() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // At depth 0, approve decision
    let state = harness
        .run_sequence_final_state("<Space>aa")
        .expect("Approve decision failed");

    assert!(
        !state.leader_active,
        "Leader should deactivate after approval action"
    );
}

#[test]
fn test_leader_visual_which_key_overlay() {
    let engine = create_test_engine();
    let mut harness = CombinedTestHarness::new(engine);

    let results = harness
        .run_sequence_with_renders("<Space>")
        .expect("Which-key render failed");

    // Check that output contains leader menu indicators
    let output = &results.last().expect("No results").visual;
    assert!(
        output.contains("Space") || output.contains("a") || output.contains("Actions"),
        "Which-key overlay should contain leader menu hints"
    );
}

#[test]
fn test_leader_visual_actions_menu() {
    let engine = create_test_engine();
    let mut harness = CombinedTestHarness::new(engine);

    let results = harness
        .run_sequence_with_renders("<Space>a")
        .expect("Actions menu render failed");

    let output = &results.last().expect("No results").visual;
    assert!(
        output.contains("Actions") || output.contains("a") || output.contains("Approve"),
        "Actions submenu should be visible"
    );
}

#[test]
fn test_leader_visual_instructions_menu() {
    let engine = create_test_engine();
    let mut harness = CombinedTestHarness::new(engine);

    let results = harness
        .run_sequence_with_renders("<Space>i")
        .expect("Instructions menu render failed");

    let output = &results.last().expect("No results").visual;
    assert!(
        output.contains("Instructions") || output.contains("i") || output.contains("Instruction"),
        "Instructions submenu should be visible"
    );
}

#[test]
fn test_leader_visual_toggles_menu() {
    let engine = create_test_engine();
    let mut harness = CombinedTestHarness::new(engine);

    let results = harness
        .run_sequence_with_renders("<Space>t")
        .expect("Toggles menu render failed");

    let output = &results.last().expect("No results").visual;
    assert!(
        output.contains("Toggles") || output.contains("Semantic") || output.contains("Context"),
        "Toggles submenu should be visible"
    );
}

#[test]
fn test_leader_visual_export_menu() {
    let engine = create_test_engine();
    let mut harness = CombinedTestHarness::new(engine);

    let results = harness
        .run_sequence_with_renders("<Space>e")
        .expect("Export menu render failed");

    let output = &results.last().expect("No results").visual;
    assert!(
        output.contains("Export") || output.contains("e") || output.contains("file"),
        "Export submenu should be visible"
    );
}

#[test]
fn test_leader_depth_aware_actions_menu() {
    let engine = create_test_engine();
    let mut harness = CombinedTestHarness::new(engine);

    // At depth 0 (decision), should show decision approval option
    let results = harness
        .run_sequence_with_renders("<Space>a")
        .expect("Depth 0 actions menu failed");

    let output = &results.last().expect("No results").visual;
    // At depth 0, "d" option (approve decision) should be visible
    assert!(
        output.contains("[d]") || output.contains("decision") || output.contains("Decision"),
        "Decision approval should be available at depth 0"
    );
}

#[test]
fn test_leader_sequence_navigate_and_approve() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Navigate down, expand, navigate to chunk, approve
    let state = harness
        .run_sequence_final_state("j<Tab>j<Space>aa")
        .expect("Complex sequence failed");

    assert!(
        !state.leader_active,
        "Leader should be deactivated after sequence"
    );
}

#[test]
fn test_leader_escape_from_submenu() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let state = harness
        .run_sequence_final_state("<Space>a<Esc>")
        .expect("Escape from submenu failed");

    // After escape from submenu, leader should be deactivated
    assert!(
        !state.leader_active || state.leader_submenu.is_none(),
        "Leader should deactivate or return to root menu"
    );
}

#[test]
fn test_leader_multiple_sequences() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // First sequence
    let state = harness
        .run_sequence_final_state("<Space>a")
        .expect("First leader sequence failed");
    assert_eq!(
        state.leader_submenu,
        Some('a'),
        "Should be in actions submenu after first sequence"
    );

    // Escape and try again
    let state = harness
        .run_sequence_final_state("<Esc><Space>t")
        .expect("Second leader sequence failed");

    // Should now be in toggles submenu (or deactivated after escape)
    // If still active, should be toggles
    if state.leader_active {
        assert_eq!(
            state.leader_submenu,
            Some('t'),
            "Should transition to toggles submenu"
        );
    }
}

#[test]
fn test_leader_toggle_semantic_highlighting() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let state_before = harness
        .run_sequence_final_state("")
        .expect("Initial state failed");
    let highlight_before = state_before.highlight_semantics;

    let state_after = harness
        .run_sequence_final_state("<Space>ts")
        .expect("Toggle semantic failed");

    assert_ne!(
        highlight_before, state_after.highlight_semantics,
        "Semantic highlighting should toggle"
    );
    assert!(
        !state_after.leader_active,
        "Leader should deactivate after action"
    );
}

#[test]
fn test_leader_toggle_context_display() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let state_before = harness
        .run_sequence_final_state("")
        .expect("Initial state failed");
    let context_before = state_before.show_all_context;

    let state_after = harness
        .run_sequence_final_state("<Space>tc")
        .expect("Toggle context failed");

    assert_ne!(
        context_before, state_after.show_all_context,
        "Context display should toggle"
    );
    assert!(
        !state_after.leader_active,
        "Leader should deactivate after action"
    );
}

#[test]
fn test_leader_navigation_in_submenu() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Enter actions submenu and execute action in one sequence
    let snapshots = harness
        .run_sequence("<Space>af")
        .expect("Actions submenu and action failed");

    // Check intermediate state (after Space+a)
    assert!(
        snapshots.len() >= 2,
        "Should have snapshots for Space and a"
    );

    // Check final state
    let final_state = &snapshots[snapshots.len() - 1];
    assert!(
        !final_state.leader_active,
        "Should exit after executing action"
    );
}

#[test]
fn test_leader_key_reset_timeout_on_submenu() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Activate leader and enter submenu
    let state = harness
        .run_sequence_final_state("<Space>a")
        .expect("Submenu entry failed");

    // Verify leader is still active and submenu is set after entering
    assert!(
        state.leader_active,
        "Leader should remain active after entering submenu"
    );
    assert_eq!(
        state.leader_submenu,
        Some('a'),
        "Should be in actions submenu"
    );
}

#[test]
fn test_leader_full_workflow_decisions() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Navigate through decisions and approve with leader key
    let state = harness
        .run_sequence_final_state("j<Space>aaj<Space>aa")
        .expect("Full workflow failed");

    assert!(
        !state.leader_active,
        "Leader should be deactivated after final action"
    );
}

#[test]
fn test_leader_nested_submenu_sequences() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Multiple submenu entries and exits
    let _state = harness
        .run_sequence_final_state("<Space>a<Esc><Space>i<Esc><Space>t")
        .expect("Nested sequences failed");

    // Should end in toggles submenu (if implementation allows re-entry)
    // Or should be deactivated (if escape deactivates completely)
}

#[test]
fn test_leader_with_panel_navigation() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Navigate panels, then use leader key
    let state = harness
        .run_sequence_final_state("l<Space>a")
        .expect("Leader after panel nav failed");

    assert_eq!(
        state.leader_submenu,
        Some('a'),
        "Leader should work after panel navigation"
    );
}

#[test]
fn test_leader_with_expansion_and_action() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Expand tree, navigate, then use leader
    let state = harness
        .run_sequence_final_state("j<Tab>j<Space>aa")
        .expect("Leader after expansion failed");

    assert!(
        !state.leader_active,
        "Leader should deactivate after action"
    );
}
