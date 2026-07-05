//! DrillNav note-entry contract (plan-drillnav-main-tui, Phase 0)
//!
//! `n` is the direct note-entry binding (D4): in Drill it opens the
//! instruction input modal targeting the focused chunk; in Browse it targets
//! the decision under the cursor. Submission routes through the engine's
//! single-note model — a chunk has exactly one note and adding appends to it.
//! Esc cancels input and restores navigation exactly where it was.

#![cfg(feature = "test-harness")]

mod drillnav_common;

use drillnav_common::{chunks_for_file, create_drillnav_engine, drive_app};

use diffviz_review_tui::test_harness::InputTestHarness;

// =============================================================================
// Opening the input modal
// =============================================================================

#[test]
fn test_n_in_drill_opens_chunk_note_input() {
    let mut harness = InputTestHarness::new(create_drillnav_engine());

    let snapshot = harness
        .run_sequence_final_state("<Enter>n")
        .expect("Run sequence");

    assert_eq!(snapshot.input_mode, "Instruction");
    assert_eq!(snapshot.input_buffer, "");
}

#[test]
fn test_n_in_browse_opens_decision_note_input() {
    let mut harness = InputTestHarness::new(create_drillnav_engine());

    let snapshot = harness.run_sequence_final_state("n").expect("Run sequence");

    assert_eq!(snapshot.input_mode, "DecisionInstruction");
}

#[test]
fn test_typed_characters_land_in_the_note_buffer() {
    let mut harness = InputTestHarness::new(create_drillnav_engine());

    // 'j' inside input mode must be text, not navigation.
    let snapshot = harness
        .run_sequence_final_state("<Enter>njob done")
        .expect("Run sequence");

    assert_eq!(snapshot.input_mode, "Instruction");
    assert_eq!(snapshot.input_buffer, "job done");
    assert_eq!(snapshot.drill_chunk, Some(0));
}

// =============================================================================
// Submission: single-note model (engine state)
// =============================================================================

#[test]
fn test_submit_attaches_note_to_focused_chunk() {
    // Drill into decision 1, cycle to calculator.rs (unambiguous chunk
    // ordering), note the first chunk.
    let engine = drive_app(create_drillnav_engine(), "<Enter>lnneeds docs<Enter>");

    let chunks = chunks_for_file(&engine, 1, "src/models/calculator.rs");
    let notes = engine
        .state()
        .get_instructions(&chunks[0])
        .expect("focused chunk should have a note");
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].content, "needs docs");
    // Other chunks untouched
    assert!(
        engine
            .state()
            .get_instructions(&chunks[1])
            .is_none_or(|v| v.is_empty())
    );
}

#[test]
fn test_second_submit_appends_to_the_single_note() {
    let engine = drive_app(
        create_drillnav_engine(),
        "<Enter>lnfirst pass<Enter>nsecond pass<Enter>",
    );

    let chunks = chunks_for_file(&engine, 1, "src/models/calculator.rs");
    let notes = engine
        .state()
        .get_instructions(&chunks[0])
        .expect("chunk should have a note");
    assert_eq!(
        notes.len(),
        1,
        "single-note model: appends never grow the list"
    );
    assert!(notes[0].content.contains("first pass"));
    assert!(notes[0].content.contains("second pass"));
}

#[test]
fn test_submit_returns_to_navigation_preserving_drill_position() {
    let mut harness = InputTestHarness::new(create_drillnav_engine());

    let snapshot = harness
        .run_sequence_final_state("<Enter>ljnok<Enter>")
        .expect("Run sequence");

    assert_eq!(snapshot.input_mode, "Navigation");
    assert_eq!(snapshot.nav_mode, "Drill");
    assert_eq!(snapshot.drill_file, Some(1));
    assert_eq!(snapshot.drill_chunk, Some(1));
}

// =============================================================================
// Cancel
// =============================================================================

#[test]
fn test_esc_cancels_note_input_and_restores_drill_position() {
    let mut harness = InputTestHarness::new(create_drillnav_engine());

    // Esc in input mode cancels the modal — it must NOT back out of the drill.
    let snapshot = harness
        .run_sequence_final_state("<Enter>ljndraft<Esc>")
        .expect("Run sequence");

    assert_eq!(snapshot.input_mode, "Navigation");
    assert_eq!(snapshot.input_buffer, "");
    assert_eq!(snapshot.nav_mode, "Drill");
    assert_eq!(snapshot.drill_file, Some(1));
    assert_eq!(snapshot.drill_chunk, Some(1));
}

#[test]
fn test_cancelled_note_adds_nothing_to_the_engine() {
    let engine = drive_app(create_drillnav_engine(), "<Enter>lndraft<Esc>");

    let chunks = chunks_for_file(&engine, 1, "src/models/calculator.rs");
    assert!(
        engine
            .state()
            .get_instructions(&chunks[0])
            .is_none_or(|v| v.is_empty())
    );
}
