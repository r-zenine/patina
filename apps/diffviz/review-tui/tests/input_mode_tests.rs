//! Integration tests for input mode workflows
//!
//! These tests validate text input modes for instruction and edit operations:
//! - Entering note input via the direct `n` binding (DrillNav, D4)
//! - Text editing operations (typing, backspace, delete)
//! - Cursor movement (left/right arrows, Home/End)
//! - Word-wise cursor movement (Ctrl+left/right)
//! - Input submission (Enter)
//! - Input cancellation (Esc)
//! - Visual modal rendering
//! - Mode exit behavior and buffer cleanup
//!
//! NOTE: `n` targets the focused chunk while drilled in (Instruction mode)
//! and the decision under the cursor while browsing (DecisionInstruction).
//! Standard navigation to a chunk: <Enter> (drill into the first decision).

#![cfg(feature = "test-harness")]

use diffviz_review::providers::mock_provider::MockDiffProvider;
use diffviz_review::{
    CodeImpact, Decision, DecisionLineRange, DiffQuery, GitRef, ReviewEngineBuilder,
};
use diffviz_review_tui::test_harness::{CombinedTestHarness, InputTestHarness};

/// Create a test ReviewEngine with realistic decisions
fn create_test_engine() -> diffviz_review::engines::ReviewEngine {
    let mock_provider =
        MockDiffProvider::from_review_fixtures().expect("Failed to load test fixtures");
    let review_engine_builder =
        ReviewEngineBuilder::new(Box::new(mock_provider), "test-user".to_string());
    let diff_query = DiffQuery::new(GitRef::Head, GitRef::Unstaged);

    let decisions = vec![
        Decision {
            number: 1,
            title: "Refactor trait implementation".to_string(),
            rationale: Some("Extract trait and implement for Calculator".to_string()),
            code_impacts: vec![CodeImpact {
                file: "src/models/calculator.rs".to_string(),
                line_ranges: vec![DecisionLineRange { start: 1, end: 20 }],
                reasoning: "Trait implementation refactoring".to_string(),
            }],
        },
        Decision {
            number: 2,
            title: "Improve config error handling".to_string(),
            rationale: Some("Add proper error types".to_string()),
            code_impacts: vec![CodeImpact {
                file: "src/config/reader.rs".to_string(),
                line_ranges: vec![DecisionLineRange { start: 1, end: 7 }],
                reasoning: "Error handling improvements".to_string(),
            }],
        },
    ];

    review_engine_builder
        .build_from_decisions(decisions, diff_query)
        .expect("Failed to build ReviewEngine")
}

// ============================================================================
// Phase 6: Input Mode Tests - Mode Transitions
// ============================================================================

#[test]
fn test_enter_instruction_mode() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Navigate to chunk (depth 1) and enter instruction mode
    // Sequence: expand decision, down to file, expand file, down to chunk, leader+i+i
    let state = harness
        .run_sequence_final_state("<Enter>n")
        .expect("Entering instruction mode failed");

    assert_eq!(
        state.input_mode, "Instruction",
        "Should be in instruction mode after n in Drill"
    );
    assert_eq!(
        state.input_buffer, "",
        "Input buffer should be empty on mode entry"
    );
    assert_eq!(
        state.input_cursor, 0,
        "Input cursor should be at start on mode entry"
    );
    assert!(
        !state.leader_active,
        "Leader should deactivate after entering input mode"
    );
}

#[test]
fn test_exit_input_mode_with_esc() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Enter instruction mode then cancel with Esc
    let state = harness
        .run_sequence_final_state("<Enter>n<Esc>")
        .expect("Exiting input mode failed");

    assert_eq!(
        state.input_mode, "Navigation",
        "Should return to navigation mode after Esc"
    );
    assert_eq!(
        state.input_buffer, "",
        "Input buffer should be cleared on exit"
    );
    assert_eq!(
        state.input_cursor, 0,
        "Input cursor should be reset on exit"
    );
}

#[test]
fn test_exit_input_mode_with_ctrl_c() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Enter instruction mode then cancel with Ctrl+C
    let state = harness
        .run_sequence_final_state("<Enter>n<C-c>")
        .expect("Exiting input mode with Ctrl+C failed");

    assert_eq!(
        state.input_mode, "Navigation",
        "Should return to navigation mode after Ctrl+C"
    );
}

// ============================================================================
// Phase 6: Input Mode Tests - Text Input
// ============================================================================

#[test]
fn test_type_text_in_instruction_mode() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Enter instruction mode and type text
    let state = harness
        .run_sequence_final_state("<Enter>nhello")
        .expect("Typing in instruction mode failed");

    assert_eq!(
        state.input_mode, "Instruction",
        "Should remain in input mode"
    );
    assert_eq!(
        state.input_buffer, "hello",
        "Input buffer should contain typed text"
    );
    assert_eq!(
        state.input_cursor, 5,
        "Cursor should be at end after typing 'hello'"
    );
}

#[test]
fn test_type_text_with_spaces() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Type text with spaces
    let state = harness
        .run_sequence_final_state("<Enter>nhello<Space>world")
        .expect("Typing with spaces failed");

    assert_eq!(
        state.input_buffer, "hello world",
        "Input buffer should contain text with spaces"
    );
    assert_eq!(state.input_cursor, 11, "Cursor should be at end");
}

#[test]
fn test_type_special_characters() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Type text with punctuation and special chars
    let state = harness
        .run_sequence_final_state("<Enter>nTest!@#")
        .expect("Typing special characters failed");

    assert_eq!(
        state.input_buffer, "Test!@#",
        "Input buffer should handle special characters"
    );
}

// ============================================================================
// Phase 6: Input Mode Tests - Backspace and Delete
// ============================================================================

#[test]
fn test_backspace_deletes_char_before_cursor() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Type text then backspace
    let state = harness
        .run_sequence_final_state("<Enter>nhello<Backspace>")
        .expect("Backspace failed");

    assert_eq!(
        state.input_buffer, "hell",
        "Backspace should delete last character"
    );
    assert_eq!(
        state.input_cursor, 4,
        "Cursor should move back after backspace"
    );
}

#[test]
fn test_multiple_backspaces() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Type text then multiple backspaces
    let state = harness
        .run_sequence_final_state("<Enter>nhello<Backspace><Backspace><Backspace>")
        .expect("Multiple backspaces failed");

    assert_eq!(state.input_buffer, "he", "Should delete 3 characters");
    assert_eq!(state.input_cursor, 2, "Cursor should be at position 2");
}

#[test]
fn test_backspace_at_start_does_nothing() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Backspace at start of empty buffer
    let state = harness
        .run_sequence_final_state("<Enter>n<Backspace>")
        .expect("Backspace at start failed");

    assert_eq!(
        state.input_buffer, "",
        "Buffer should remain empty when backspacing at start"
    );
    assert_eq!(state.input_cursor, 0, "Cursor should remain at 0");
}

#[test]
#[ignore = "Feature not implemented: DeleteForward event handler"]
fn test_delete_forward() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Type text, move cursor to middle, delete forward
    let state = harness
        .run_sequence_final_state("<Enter>nhello<Left><Left><Delete>")
        .expect("Delete forward failed");

    assert_eq!(
        state.input_buffer, "helllo",
        "Delete should remove character after cursor"
    );
    assert_eq!(
        state.input_cursor, 3,
        "Cursor should not move after delete forward"
    );
}

// ============================================================================
// Phase 6: Input Mode Tests - Cursor Movement
// ============================================================================

#[test]
fn test_move_cursor_left() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Type text then move cursor left
    let state = harness
        .run_sequence_final_state("<Enter>nhello<Left><Left>")
        .expect("Move cursor left failed");

    assert_eq!(state.input_buffer, "hello", "Buffer should be unchanged");
    assert_eq!(state.input_cursor, 3, "Cursor should move to position 3");
}

#[test]
fn test_move_cursor_right() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Type text, move left, then move right
    let state = harness
        .run_sequence_final_state("<Enter>nhello<Left><Left><Right>")
        .expect("Move cursor right failed");

    assert_eq!(state.input_buffer, "hello", "Buffer should be unchanged");
    assert_eq!(state.input_cursor, 4, "Cursor should move to position 4");
}

#[test]
fn test_move_cursor_to_home() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Type text then move to start with Home
    let state = harness
        .run_sequence_final_state("<Enter>nhello<Home>")
        .expect("Move cursor to home failed");

    assert_eq!(state.input_buffer, "hello", "Buffer should be unchanged");
    assert_eq!(state.input_cursor, 0, "Cursor should be at start");
}

#[test]
fn test_move_cursor_to_end() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Type text, move to start, then move to end with End
    let state = harness
        .run_sequence_final_state("<Enter>nhello<Home><End>")
        .expect("Move cursor to end failed");

    assert_eq!(state.input_buffer, "hello", "Buffer should be unchanged");
    assert_eq!(state.input_cursor, 5, "Cursor should be at end");
}

#[test]
#[ignore = "Feature not implemented: MoveCursorWordLeft event handler"]
fn test_move_cursor_word_left() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Type multi-word text then move by word
    let state = harness
        .run_sequence_final_state("<Enter>nhello<Space>world<C-Left>")
        .expect("Move cursor word left failed");

    assert_eq!(
        state.input_buffer, "hello world",
        "Buffer should be unchanged"
    );
    // Cursor should jump to start of "world"
    assert_eq!(
        state.input_cursor, 6,
        "Cursor should jump to previous word boundary"
    );
}

#[test]
#[ignore = "Feature not implemented: MoveCursorWordRight event handler"]
fn test_move_cursor_word_right() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Type multi-word text, move to start, then move by word
    let state = harness
        .run_sequence_final_state("<Enter>nhello<Space>world<Home><C-Right>")
        .expect("Move cursor word right failed");

    assert_eq!(
        state.input_buffer, "hello world",
        "Buffer should be unchanged"
    );
    // Cursor should jump to end of "hello"
    assert_eq!(
        state.input_cursor, 5,
        "Cursor should jump to next word boundary"
    );
}

// ============================================================================
// Phase 6: Input Mode Tests - Text Editing at Cursor
// ============================================================================

#[test]
fn test_insert_text_at_cursor_position() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Type text, move cursor to middle, insert new text
    let state = harness
        .run_sequence_final_state("<Enter>nworld<Home>hello<Space>")
        .expect("Insert at cursor failed");

    assert_eq!(
        state.input_buffer, "hello world",
        "Text should be inserted at cursor position"
    );
    assert_eq!(
        state.input_cursor, 6,
        "Cursor should be after inserted text"
    );
}

#[test]
fn test_backspace_in_middle_of_text() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Type text, move to middle, backspace
    let state = harness
        .run_sequence_final_state("<Enter>nhello<Left><Left><Backspace>")
        .expect("Backspace in middle failed");

    assert_eq!(
        state.input_buffer, "helo",
        "Backspace should delete character before cursor in middle"
    );
    assert_eq!(
        state.input_cursor, 2,
        "Cursor should move back after backspace"
    );
}

// ============================================================================
// Phase 6: Input Mode Tests - Submit Input
// ============================================================================

#[test]
#[ignore = "Submit requires ReviewEngine integration with actual file content"]
fn test_submit_input_with_enter() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Type text and submit with Enter
    let state = harness
        .run_sequence_final_state("<Enter>nhello<Enter>")
        .expect("Submit input failed");

    // After submission, should return to navigation mode
    assert_eq!(
        state.input_mode, "Navigation",
        "Should return to navigation after submit"
    );
    assert_eq!(
        state.input_buffer, "",
        "Buffer should be cleared after submit"
    );
    assert_eq!(state.input_cursor, 0, "Cursor should be reset after submit");
}

// ============================================================================
// Phase 6: Input Mode Tests - Visual Rendering
// ============================================================================

#[test]
fn test_instruction_mode_visual_modal_displays() {
    let engine = create_test_engine();
    let mut harness = CombinedTestHarness::new(engine);

    // Enter instruction mode
    let results = harness
        .run_sequence_with_renders("<Enter>n")
        .expect("Visual rendering failed");

    let output = &results.last().expect("No results").visual;

    // Check for input modal indicators (actual modal content depends on UI implementation)
    assert!(
        output.contains("Instruction") || output.contains("Input"),
        "Visual output should show instruction mode indicator"
    );
}

#[test]
fn test_input_buffer_displays_in_modal() {
    let engine = create_test_engine();
    let mut harness = CombinedTestHarness::new(engine);

    // Type text and check visual output
    let results = harness
        .run_sequence_with_renders("<Enter>nhello")
        .expect("Visual rendering failed");

    let output = &results.last().expect("No results").visual;

    // Visual output should contain the typed text
    assert!(
        output.contains("hello"),
        "Visual output should display input buffer content"
    );
}

// ============================================================================
// Phase 6: Input Mode Tests - Integration Workflows
// ============================================================================

#[test]
#[ignore = "Submit requires ReviewEngine integration with actual file content"]
fn test_navigate_enter_input_type_submit_workflow() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Full workflow: navigate → expand → enter input mode → type → submit
    let state = harness
        .run_sequence_final_state("<Enter>nhello<Space>world<Enter>")
        .expect("Full workflow failed");

    assert_eq!(
        state.input_mode, "Navigation",
        "Should be back in navigation after workflow"
    );
    assert_eq!(
        state.input_buffer, "",
        "Buffer should be cleared after workflow"
    );
}

#[test]
fn test_navigate_enter_input_cancel_workflow() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Workflow with cancellation
    let state = harness
        .run_sequence_final_state("<Enter>nhello<Esc>")
        .expect("Cancel workflow failed");

    assert_eq!(
        state.input_mode, "Navigation",
        "Should be back in navigation after cancel"
    );
    assert_eq!(
        state.input_buffer, "",
        "Buffer should be cleared after cancel"
    );
}

#[test]
#[ignore = "Submit requires ReviewEngine integration with actual file content"]
fn test_multiple_input_mode_sessions() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Enter input mode, type, submit, then do it again
    harness
        .run_sequence_final_state("<Enter>nfirst<Enter>")
        .expect("First session failed");

    let state = harness
        .run_sequence_final_state("nsecond<Enter>")
        .expect("Second session failed");

    assert_eq!(
        state.input_mode, "Navigation",
        "Should be in navigation after multiple sessions"
    );
    assert_eq!(
        state.input_buffer, "",
        "Buffer should be clean for new session"
    );
}

#[test]
fn test_edit_text_complex_operations() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Complex editing: type → move cursor → insert → delete → edit
    let state = harness
        .run_sequence_final_state("<Enter>nworld<Home>hello<Space><End><Backspace><Backspace>!")
        .expect("Complex editing failed");

    assert_eq!(
        state.input_buffer, "hello wor!",
        "Complex edits should produce correct result"
    );
}

#[test]
fn test_input_mode_preserves_navigation_state() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Navigate to specific position, enter input mode, exit
    let state = harness
        .run_sequence_final_state("j<Enter>n<Esc>")
        .expect("Navigation preservation failed");

    // Drill position should be preserved (drilled into decision idx 1)
    assert_eq!(
        state.drill_decision,
        Some(1),
        "Drill position should be preserved"
    );
    assert_eq!(
        state.input_mode, "Navigation",
        "Should return to navigation mode"
    );
}

// ============================================================================
// Bug Reproduction Tests
// ============================================================================

#[test]
fn submit_instruction_via_build_from_decisions_returns_to_navigation() {
    let mock_provider =
        MockDiffProvider::from_review_fixtures().expect("Failed to load test fixtures");
    let review_engine_builder =
        ReviewEngineBuilder::new(Box::new(mock_provider), "test-user".to_string());
    let diff_query = DiffQuery::new(GitRef::Head, GitRef::Unstaged);

    let decisions = vec![Decision {
        number: 1,
        title: "Refactor trait implementation".to_string(),
        rationale: Some("Extract trait and implement for Calculator".to_string()),
        code_impacts: vec![CodeImpact {
            file: "src/models/calculator.rs".to_string(),
            line_ranges: vec![DecisionLineRange { start: 1, end: 20 }],
            reasoning: "Trait implementation refactoring".to_string(),
        }],
    }];

    let engine = review_engine_builder
        .build_from_decisions(decisions, diff_query)
        .expect("Failed to build ReviewEngine");

    let mut harness = InputTestHarness::new(engine);

    let state = harness
        .run_sequence_final_state("<Enter>nhello<Enter>")
        .expect("Submitting instruction should not crash");

    assert_eq!(
        state.input_mode, "Navigation",
        "Should return to Navigation mode after submitting instruction"
    );
}

// ============================================================================
// Decision-level instruction mode
// ============================================================================

#[test]
fn test_enter_decision_instruction_mode_at_depth_0() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // In Browse mode, n should enter DecisionInstruction mode
    let state = harness
        .run_sequence_final_state("n")
        .expect("Entering decision instruction mode failed");

    assert_eq!(
        state.input_mode, "DecisionInstruction",
        "Should be in DecisionInstruction mode at depth 0"
    );
    assert_eq!(state.input_buffer, "");
    assert_eq!(state.input_cursor, 0);
    assert!(!state.leader_active);
}

#[test]
fn test_decision_instruction_mode_accepts_text() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let state = harness
        .run_sequence_final_state("ncheck error handling")
        .expect("Typing in decision instruction mode failed");

    assert_eq!(state.input_mode, "DecisionInstruction");
    assert_eq!(state.input_buffer, "check error handling");
}

#[test]
fn test_decision_instruction_mode_submit_exits_to_navigation() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let state = harness
        .run_sequence_final_state("nmy instruction<Enter>")
        .expect("Submitting decision instruction failed");

    assert_eq!(state.input_mode, "Navigation");
    assert_eq!(state.input_buffer, "");
}

#[test]
fn test_decision_instruction_mode_cancel_exits_to_navigation() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    let state = harness
        .run_sequence_final_state("nmy instruction<Esc>")
        .expect("Cancelling decision instruction failed");

    assert_eq!(state.input_mode, "Navigation");
    assert_eq!(state.input_buffer, "");
}

#[test]
fn test_chunk_level_still_enters_regular_instruction_mode() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Drilled into a chunk, n should still use regular Instruction mode
    let state = harness
        .run_sequence_final_state("<Enter>n")
        .expect("Entering instruction mode at chunk level failed");

    assert_eq!(
        state.input_mode, "Instruction",
        "Should use regular Instruction mode at chunk level (depth 1)"
    );
}
