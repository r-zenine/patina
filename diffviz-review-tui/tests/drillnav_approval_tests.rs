//! DrillNav approval contract (plan-drillnav-main-tui, Phase 0)
//!
//! `a` is a direct top-level binding: in Browse it toggles approval of the
//! decision under the cursor (cascading to all its chunks, D4/D5 of the old
//! leader table are gone); in Drill it toggles approval of the focused chunk.
//! Engine failures surface as a one-shot `status_message` (D7) instead of
//! aborting the app, cleared on the next keypress.
//!
//! Business state is asserted on the engine reclaimed via
//! `into_review_engine()`; UI state via StateSnapshot v2.

#![cfg(feature = "test-harness")]

mod drillnav_common;

use drillnav_common::{chunks_for_file, create_drillnav_engine, create_empty_engine, drive_app};

use diffviz_review_tui::test_harness::InputTestHarness;

// =============================================================================
// Browse: decision approval cascades
// =============================================================================

#[test]
fn test_a_in_browse_approves_decision_and_cascades_to_chunks() {
    // Decision number 1 (browse cursor 0) spans 9 chunks across 2 files.
    let engine = drive_app(create_drillnav_engine(), "a");

    assert!(engine.is_decision_approved(1));
    assert_eq!(engine.decision_approval_progress(1), (9, 9));
    // Sibling decisions untouched
    assert!(!engine.is_decision_approved(2));
    assert!(!engine.is_decision_approved(3));
}

#[test]
fn test_a_in_browse_toggles_decision_approval_off() {
    let engine = drive_app(create_drillnav_engine(), "aa");

    assert!(!engine.is_decision_approved(1));
    assert_eq!(engine.decision_approval_progress(1), (0, 9));
}

#[test]
fn test_a_approves_the_decision_under_the_cursor() {
    // Browse cursor 2 → decision number 3 (5 chunks across 3 files).
    let engine = drive_app(create_drillnav_engine(), "jja");

    assert!(engine.is_decision_approved(3));
    assert_eq!(engine.decision_approval_progress(3), (5, 5));
    assert!(!engine.is_decision_approved(1));
}

// =============================================================================
// Drill: chunk approval
// =============================================================================

#[test]
fn test_a_in_drill_approves_only_the_focused_chunk() {
    // Drill into decision 1, file 0 (reader.rs), chunk 0.
    let engine = drive_app(create_drillnav_engine(), "<Enter>a");

    assert_eq!(engine.decision_approval_progress(1), (1, 9));
    assert!(!engine.is_decision_approved(1));
}

#[test]
fn test_a_in_drill_toggles_chunk_approval_off() {
    let engine = drive_app(create_drillnav_engine(), "<Enter>aa");

    assert_eq!(engine.decision_approval_progress(1), (0, 9));
}

#[test]
fn test_chunk_approval_follows_the_chunk_cursor() {
    // File 1 (calculator.rs, 7 chunks): approve chunk 0 and chunk 1.
    let engine = drive_app(create_drillnav_engine(), "<Enter>laja");

    let chunks = chunks_for_file(&engine, 1, "src/models/calculator.rs");
    assert!(engine.state().is_approved(&chunks[0]));
    assert!(engine.state().is_approved(&chunks[1]));
    assert!(!engine.state().is_approved(&chunks[2]));
    assert_eq!(engine.decision_approval_progress(1), (2, 9));
}

#[test]
fn test_approving_every_chunk_completes_the_decision() {
    // Decision number 2 has a single chunk: approving it from Drill should
    // complete the decision via the engine's reverse cascade.
    let engine = drive_app(create_drillnav_engine(), "j<Enter>a");

    assert_eq!(engine.decision_approval_progress(2), (1, 1));
    assert!(engine.is_decision_approved(2));
}

// =============================================================================
// Error surfacing (D7): one-shot status message
// =============================================================================

#[test]
fn test_failed_approval_surfaces_status_message_and_next_key_clears_it() {
    // An empty review has nothing to approve: 'a' must not abort the app,
    // it must surface a one-shot error in the status bar.
    let mut harness = InputTestHarness::new(create_empty_engine());

    let snapshots = harness.run_sequence("aj").expect("Run sequence");

    assert!(
        snapshots[1].status_message.is_some(),
        "failed approval should surface a status message"
    );
    assert_eq!(
        snapshots[2].status_message, None,
        "next keypress should clear the one-shot message"
    );
}

#[test]
fn test_successful_approval_sets_no_status_message() {
    let mut harness = InputTestHarness::new(create_drillnav_engine());

    let snapshot = harness.run_sequence_final_state("a").expect("Run sequence");

    assert_eq!(snapshot.status_message, None);
}
