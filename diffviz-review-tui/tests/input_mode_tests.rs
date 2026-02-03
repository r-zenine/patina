//! Integration tests for input mode workflows
//!
//! These tests validate text input modes for instruction and edit operations:
//! - Entering instruction mode (Space+i+i)
//! - Entering edit mode (Space+i+e)
//! - Text editing operations (typing, backspace, delete)
//! - Cursor movement (left/right arrows, Home/End)
//! - Word-wise cursor movement (Ctrl+left/right)
//! - Input submission (Enter)
//! - Input cancellation (Esc)
//! - Visual modal rendering
//! - Mode exit behavior and buffer cleanup
//!
//! NOTE: Input modes require being at chunk level (depth 2).
//! Standard navigation to chunk: <Tab>j<Tab>j (expand decision, down to file, expand file, down to chunk)

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

    // Decision with actual fixture file path to enable chunk navigation
    // Uses rust_trait_impl fixture which has old_code and new_code
    decisions.add_decision(Decision {
        number: 1,
        title: "Refactor trait implementation".to_string(),
        summary: "Extract trait and implement for Calculator".to_string(),
        decision_log_line: Some(15),
        code_impacts: vec![CodeImpact {
            file: "src/models/user.rs".to_string(),
            line_ranges: vec![DecisionLineRange { start: 1, end: 20 }],
            change_type: ChangeType::Modification,
            confidence: Confidence::High,
            reasoning: "Trait implementation refactoring".to_string(),
        }],
    });

    // Second decision using different fixture
    decisions.add_decision(Decision {
        number: 2,
        title: "Improve config error handling".to_string(),
        summary: "Add proper error types".to_string(),
        decision_log_line: Some(28),
        code_impacts: vec![CodeImpact {
            file: "src/config/reader.rs".to_string(),
            line_ranges: vec![DecisionLineRange { start: 1, end: 20 }],
            change_type: ChangeType::Modification,
            confidence: Confidence::Medium,
            reasoning: "Error handling improvements".to_string(),
        }],
    });

    review_engine.set_decisions_with_index(decisions);
    review_engine
}

// ============================================================================
// Phase 6: Input Mode Tests - Mode Transitions
// ============================================================================

#[test]
fn test_enter_instruction_mode() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Navigate to chunk (depth 2) and enter instruction mode
    // Sequence: expand decision, down to file, expand file, down to chunk, leader+i+i
    let state = harness
        .run_sequence_final_state("<Tab>j<Tab>j<Space>ii")
        .expect("Entering instruction mode failed");

    assert_eq!(
        state.input_mode, "Instruction",
        "Should be in instruction mode after Space+i+i"
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
fn test_enter_edit_mode() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Navigate to chunk (depth 2) and enter edit mode
    let state = harness
        .run_sequence_final_state("<Tab>j<Tab>j<Space>ie")
        .expect("Entering edit mode failed");

    assert_eq!(
        state.input_mode, "Edit",
        "Should be in edit mode after Space+i+e"
    );
    assert_eq!(
        state.input_buffer, "",
        "Input buffer should be empty on mode entry"
    );
    assert_eq!(
        state.input_cursor, 0,
        "Input cursor should be at start on mode entry"
    );
}

#[test]
fn test_exit_input_mode_with_esc() {
    let engine = create_test_engine();
    let mut harness = InputTestHarness::new(engine);

    // Enter instruction mode then cancel with Esc
    let state = harness
        .run_sequence_final_state("<Tab>j<Tab>j<Space>ii<Esc>")
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
        .run_sequence_final_state("<Tab>j<Tab>j<Space>ii<C-c>")
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
        .run_sequence_final_state("<Tab>j<Tab>j<Space>iihello")
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
        .run_sequence_final_state("<Tab>j<Tab>j<Space>iihello<Space>world")
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
        .run_sequence_final_state("<Tab>j<Tab>j<Space>iiTest!@#")
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
        .run_sequence_final_state("<Tab>j<Tab>j<Space>iihello<Backspace>")
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
        .run_sequence_final_state("<Tab>j<Tab>j<Space>iihello<Backspace><Backspace><Backspace>")
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
        .run_sequence_final_state("<Tab>j<Tab>j<Space>ii<Backspace>")
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
        .run_sequence_final_state("<Tab>j<Tab>j<Space>iihello<Left><Left><Delete>")
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
        .run_sequence_final_state("<Tab>j<Tab>j<Space>iihello<Left><Left>")
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
        .run_sequence_final_state("<Tab>j<Tab>j<Space>iihello<Left><Left><Right>")
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
        .run_sequence_final_state("<Tab>j<Tab>j<Space>iihello<Home>")
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
        .run_sequence_final_state("<Tab>j<Tab>j<Space>iihello<Home><End>")
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
        .run_sequence_final_state("<Tab>j<Tab>j<Space>iihello<Space>world<C-Left>")
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
        .run_sequence_final_state("<Tab>j<Tab>j<Space>iihello<Space>world<Home><C-Right>")
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
        .run_sequence_final_state("<Tab>j<Tab>j<Space>iiworld<Home>hello<Space>")
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
        .run_sequence_final_state("<Tab>j<Tab>j<Space>iihello<Left><Left><Backspace>")
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
        .run_sequence_final_state("<Tab>j<Tab>j<Space>iihello<Enter>")
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
        .run_sequence_with_renders("<Tab>j<Tab>j<Space>ii")
        .expect("Visual rendering failed");

    let output = &results.last().expect("No results").visual;

    // Check for input modal indicators (actual modal content depends on UI implementation)
    assert!(
        output.contains("Instruction") || output.contains("Input"),
        "Visual output should show instruction mode indicator"
    );
}

#[test]
fn test_edit_mode_visual_modal_displays() {
    let engine = create_test_engine();
    let mut harness = CombinedTestHarness::new(engine);

    // Enter edit mode
    let results = harness
        .run_sequence_with_renders("<Tab>j<Tab>j<Space>ie")
        .expect("Visual rendering failed");

    let output = &results.last().expect("No results").visual;

    // Check for input modal indicators
    assert!(
        output.contains("Edit") || output.contains("Input"),
        "Visual output should show edit mode indicator"
    );
}

#[test]
fn test_input_buffer_displays_in_modal() {
    let engine = create_test_engine();
    let mut harness = CombinedTestHarness::new(engine);

    // Type text and check visual output
    let results = harness
        .run_sequence_with_renders("<Tab>j<Tab>j<Space>iihello")
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
        .run_sequence_final_state("<Tab>j<Tab>j<Space>iihello<Space>world<Enter>")
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
        .run_sequence_final_state("<Tab>j<Tab>j<Space>iihello<Esc>")
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
        .run_sequence_final_state("<Tab>j<Tab>j<Space>iifirst<Enter>")
        .expect("First session failed");

    let state = harness
        .run_sequence_final_state("<Space>iisecond<Enter>")
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
        .run_sequence_final_state(
            "<Tab>j<Tab>j<Space>iiworld<Home>hello<Space><End><Backspace><Backspace>!",
        )
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
        .run_sequence_final_state("j<Tab>j<Tab>j<Space>ii<Esc>")
        .expect("Navigation preservation failed");

    // Navigation position should be preserved (decision 2, expanded, file selected)
    assert_eq!(
        state.decision_tree_path.0, 1,
        "Decision position should be preserved"
    );
    assert_eq!(
        state.input_mode, "Navigation",
        "Should return to navigation mode"
    );
}
