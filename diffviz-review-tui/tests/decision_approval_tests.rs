//! Approval visuals for DrillNav (plan-drillnav-main-tui Phase 2).
//!
//! The drillnav_approval_tests suite owns the state contract (cascade
//! semantics, engine effects, error surfacing in the snapshot). This suite
//! validates what the reviewer *sees*: the ✓ badge on approved cards, live
//! progress counts in cards and status bar, the pinned header badge, and the
//! red status message rendered when approval has no target.

#![cfg(feature = "test-harness")]

mod drillnav_common;

use drillnav_common::{create_drillnav_engine, create_empty_engine};

use diffviz_review_tui::test_harness::CombinedTestHarness;

// =============================================================================
// Browse: decision approval cascades into card visuals
// =============================================================================

#[test]
fn test_approving_decision_shows_badge_and_full_chunk_progress() {
    let mut harness = CombinedTestHarness::with_render_size(create_drillnav_engine(), 120, 40);

    let results = harness.run_sequence_with_renders("a").expect("Run");

    let before = &results[0].visual;
    assert!(!before.contains("✓"));
    assert!(before.contains("2 files · 0/9 chunks approved"));
    assert!(before.contains("a approve (0/3)"));

    // Approval cascades to every chunk of the decision (9/9) and the card
    // label row carries the ✓ badge.
    let after = &results[1].visual;
    let label_row = after
        .lines()
        .find(|l| l.contains("#1 Refactor calculator model module"))
        .expect("decision 1 card rendered");
    assert!(label_row.contains("✓"));
    assert!(after.contains("2 files · 9/9 chunks approved"));
    assert!(after.contains("a approve (1/3)"));
}

#[test]
fn test_toggling_approval_twice_clears_badge_and_progress() {
    let mut harness = CombinedTestHarness::with_render_size(create_drillnav_engine(), 120, 40);

    let results = harness.run_sequence_with_renders("aa").expect("Run");
    let visual = &results[2].visual;

    assert!(!visual.contains("✓"));
    assert!(visual.contains("2 files · 0/9 chunks approved"));
    assert!(visual.contains("a approve (0/3)"));
}

#[test]
fn test_approving_multiple_decisions_updates_overall_count() {
    let mut harness = CombinedTestHarness::with_render_size(create_drillnav_engine(), 120, 40);

    let results = harness.run_sequence_with_renders("ajaja").expect("Run");
    let visual = &results.last().expect("results").visual;

    assert!(visual.contains("a approve (3/3)"));
    for label in [
        "#1 Refactor calculator model module",
        "#2 Improve error handling",
        "#3 Add structured logging",
    ] {
        let row = visual
            .lines()
            .find(|l| l.contains(label))
            .unwrap_or_else(|| panic!("{label} card rendered"));
        assert!(row.contains("✓"), "{label} should carry the ✓ badge");
    }
}

// =============================================================================
// Drill: chunk approval visuals
// =============================================================================

#[test]
fn test_approving_chunk_shows_badge_and_per_file_progress() {
    let mut harness = CombinedTestHarness::with_render_size(create_drillnav_engine(), 120, 40);

    let results = harness.run_sequence_with_renders("<Enter>a").expect("Run");

    let before = &results[1].visual;
    assert!(!before.contains("✓"));
    assert!(before.contains("a approve (0/2)"));

    let after = &results[2].visual;
    assert!(after.contains("✓"));
    assert!(after.contains("a approve (1/2)"));
}

#[test]
fn test_partial_chunk_approval_shows_in_browse_card_progress() {
    let mut harness = CombinedTestHarness::with_render_size(create_drillnav_engine(), 120, 40);

    // Approve one chunk of decision 1, then back out to browse.
    let results = harness
        .run_sequence_with_renders("<Enter>a<Esc>")
        .expect("Run");
    let visual = &results.last().expect("results").visual;

    assert!(visual.contains("2 files · 1/9 chunks approved"));
    // Partial approval: the decision itself is not approved, no card badge.
    let label_row = visual
        .lines()
        .find(|l| l.contains("#1 Refactor calculator model module"))
        .expect("decision 1 card rendered");
    assert!(!label_row.contains("✓"));
}

#[test]
fn test_decision_approved_in_browse_badges_the_drill_header() {
    let mut harness = CombinedTestHarness::with_render_size(create_drillnav_engine(), 120, 40);

    let results = harness.run_sequence_with_renders("a<Enter>").expect("Run");
    let visual = &results.last().expect("results").visual;

    let header_row = visual
        .lines()
        .find(|l| l.contains("~ src/config/reader.rs"))
        .expect("pinned header rendered");
    assert!(
        header_row.contains("✓"),
        "approved decision's drill header should carry the ✓ badge"
    );
}

// =============================================================================
// Error surfacing (D7): approval without a target
// =============================================================================

#[test]
fn test_approve_with_no_decisions_renders_status_message() {
    let mut harness = CombinedTestHarness::with_render_size(create_empty_engine(), 120, 40);

    let results = harness.run_sequence_with_renders("aj").expect("Run");

    // 'a' with nothing to approve: red one-shot message replaces the hints
    let error_visual = &results[1].visual;
    assert!(error_visual.contains("Nothing to approve"));
    assert!(!error_visual.contains("Enter drill in"));

    // The next keypress clears it and the hints return
    let cleared_visual = &results[2].visual;
    assert!(!cleared_visual.contains("Nothing to approve"));
    assert!(cleared_visual.contains("Enter drill in"));
}
