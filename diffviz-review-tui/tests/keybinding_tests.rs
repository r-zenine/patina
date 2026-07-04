//! Integration tests for DrillNav keybinding workflows (plan-drillnav-main-tui
//! Phase 2).
//!
//! These tests drive the full key → state → render pipeline: the DrillNav key
//! table produces the expected visual output (browse cards, pinned drill
//! header, note rows), the harness renders at arbitrary sizes, and snapshots
//! round-trip with the v2 DrillNav fields.
//!
//! State-machine bounds and approval cascades live in the drillnav_* contract
//! suites; this file focuses on keys driving the rendered UI.

#![cfg(feature = "test-harness")]

mod drillnav_common;

use drillnav_common::create_drillnav_engine;

use diffviz_review_tui::test_harness::{CombinedTestHarness, InputTestHarness, StateSnapshot};

// =============================================================================
// Browse rendering
// =============================================================================

#[test]
fn test_browse_renders_decision_cards() {
    let mut harness = CombinedTestHarness::with_render_size(create_drillnav_engine(), 120, 40);

    let results = harness.run_sequence_with_renders("").expect("Render");
    let visual = &results[0].visual;

    assert!(visual.contains("#1 Refactor calculator model module"));
    assert!(visual.contains("#2 Improve error handling in network client"));
    assert!(visual.contains("#3 Add structured logging throughout application"));
    // Card metadata comes from the drill index: files and chunk progress
    assert!(visual.contains("2 files · 0/9 chunks approved"));
    assert!(visual.contains("1 file · 0/1 chunks approved"));
    // File preview rows at Body tier
    assert!(visual.contains("~ src/config/reader.rs"));
    assert!(visual.contains("~ src/models/calculator.rs"));
}

#[test]
fn test_browse_status_bar_advertises_drill_in_and_approval_progress() {
    let mut harness = CombinedTestHarness::with_render_size(create_drillnav_engine(), 120, 40);

    let results = harness.run_sequence_with_renders("").expect("Render");
    let visual = &results[0].visual;

    assert!(visual.contains("Enter drill in"));
    assert!(visual.contains("a approve (0/3)"));
    assert!(visual.contains("n note"));
}

// =============================================================================
// Drill-in / back rendering
// =============================================================================

#[test]
fn test_enter_renders_drill_view_and_esc_returns_to_browse() {
    let mut harness = CombinedTestHarness::with_render_size(create_drillnav_engine(), 120, 40);

    let results = harness
        .run_sequence_with_renders("<Enter><Esc>")
        .expect("Run sequence");

    // Drilled in: pinned header shows the file and its impact reasoning
    let drill_visual = &results[1].visual;
    assert!(drill_visual.contains("~ src/config/reader.rs"));
    assert!(drill_visual.contains("Configuration reader updates"));
    // Browse card labels are gone from the drill view
    assert!(!drill_visual.contains("#1 Refactor calculator model module"));

    // Esc restores the browse cards
    let browse_visual = &results[2].visual;
    assert!(browse_visual.contains("#1 Refactor calculator model module"));
}

#[test]
fn test_drill_status_bar_is_contextual_for_multi_file_decision() {
    let mut harness = CombinedTestHarness::with_render_size(create_drillnav_engine(), 120, 40);

    let results = harness
        .run_sequence_with_renders("<Enter>")
        .expect("Run sequence");
    let visual = &results[1].visual;

    assert!(visual.contains("file 1/2"));
    assert!(visual.contains("h/l files"));
    assert!(visual.contains("j/k chunks"));
    assert!(visual.contains("Tab expand ctx"));
    assert!(visual.contains("Esc back"));
    // reader.rs has 2 chunks, none approved
    assert!(visual.contains("a approve (0/2)"));
}

#[test]
fn test_drill_status_bar_omits_file_cycling_for_single_file_decision() {
    let mut harness = CombinedTestHarness::with_render_size(create_drillnav_engine(), 120, 40);

    // Decision idx 1 has a single file: no h/l hint, no dot pagination
    let results = harness
        .run_sequence_with_renders("j<Enter>")
        .expect("Run sequence");
    let visual = &results[2].visual;

    assert!(!visual.contains("h/l files"));
    assert!(!visual.contains("●"));
    assert!(visual.contains("~ src/network/client.rs"));
}

// =============================================================================
// Note rendering (n to append, i to expand)
// =============================================================================

const LONG_NOTE: &str = "this note is long enough that the collapsed single row \
                         cannot hold it and the renderer must truncate it";

#[test]
fn test_note_renders_truncated_and_i_expands_it() {
    let mut harness = CombinedTestHarness::with_render_size(create_drillnav_engine(), 120, 40);

    let sequence = format!("<Enter>n{LONG_NOTE}<Enter>");
    let results = harness
        .run_sequence_with_renders(&sequence)
        .expect("Run sequence");
    let collapsed = &results.last().expect("results").visual;

    // Collapsed: one row, author-prefixed, ellipsis marks the truncation
    assert!(collapsed.contains("test-user: this note is long enough"));
    assert!(collapsed.contains("…"));
    assert!(!collapsed.contains("truncate it"));

    // i expands the note to its full wrapped rows
    let results = harness.run_sequence_with_renders("i").expect("Expand note");
    let expanded = &results.last().expect("results").visual;
    assert!(expanded.contains("truncate it"));
    assert!(!expanded.contains("…"));
}

#[test]
fn test_reopening_note_preloads_existing_text_for_editing() {
    let mut harness = CombinedTestHarness::with_render_size(create_drillnav_engine(), 120, 40);

    // Write the initial note and submit it.
    harness
        .run_sequence_with_renders("<Enter>nFirst note<Enter>")
        .expect("Run sequence");

    // Reopening the note input shows the existing text pre-filled, not blank.
    let results = harness.run_sequence_with_renders("n").expect("Reopen note");
    let modal_visual = &results.last().expect("results").visual;
    assert!(modal_visual.contains("First note"));
    assert!(modal_visual.contains("Edit test-user's note"));

    // Editing the pre-filled text and resubmitting replaces the note in
    // place rather than appending a duplicate copy of the original text.
    let results = harness
        .run_sequence_with_renders(" - edited<Enter>i")
        .expect("Edit and submit");
    let expanded_visual = &results.last().expect("results").visual;
    assert!(expanded_visual.contains("First note - edited"));
    assert_eq!(expanded_visual.matches("First note").count(), 1);
}

// =============================================================================
// Render sizes
// =============================================================================

#[test]
fn test_renders_at_small_and_large_terminal_sizes() {
    for (w, h) in [(80, 24), (200, 60)] {
        let mut harness = CombinedTestHarness::with_render_size(create_drillnav_engine(), w, h);
        let results = harness
            .run_sequence_with_renders("<Enter>jl<Esc>")
            .expect("Run sequence");
        for result in &results {
            assert!(!result.visual.is_empty(), "empty visual at {w}x{h}");
        }
    }
}

// =============================================================================
// Key table robustness
// =============================================================================

#[test]
fn test_quit_key() {
    let mut harness = InputTestHarness::new(create_drillnav_engine());

    let initial = harness.run_sequence_final_state("").expect("Initial");
    assert!(!initial.should_quit);

    let after_quit = harness.run_sequence_final_state("q").expect("After q");
    assert!(after_quit.should_quit);
}

#[test]
fn test_special_keys_do_not_error() {
    let mut harness = InputTestHarness::new(create_drillnav_engine());

    for key in ["<Space>", "<Enter>", "<Esc>", "<Tab>", "<Up>", "<Down>"] {
        harness
            .run_sequence_final_state(key)
            .unwrap_or_else(|e| panic!("{key} errored: {e}"));
    }
}

#[test]
fn test_modifier_keys_do_not_error() {
    let mut harness = InputTestHarness::new(create_drillnav_engine());

    for key in ["<C-d>", "<C-u>", "<S-G>", "<A-x>"] {
        harness
            .run_sequence_final_state(key)
            .unwrap_or_else(|e| panic!("{key} errored: {e}"));
    }
}

#[test]
fn test_chunk_navigation_resets_stale_page_offset() {
    // Regression: Ctrl-d accumulates a raw page offset that used to survive
    // j/k cursor moves, so the render's scroll clamp kept pinning near the
    // bottom of the content even after the focused chunk changed — the
    // focus indicator appeared to vanish. Moving the cursor must clear it.
    let mut harness = InputTestHarness::new(create_drillnav_engine());

    let paged = harness
        .run_sequence_final_state("<Enter><C-d><C-d><C-d>")
        .expect("Run sequence");
    assert!(paged.drill_page_offset.unwrap_or(0) > 0);

    let after_down = harness.run_sequence_final_state("j").expect("Run sequence");
    assert_eq!(after_down.drill_page_offset, Some(0));

    let paged_again = harness
        .run_sequence_final_state("<C-d><C-d>")
        .expect("Run sequence");
    assert!(paged_again.drill_page_offset.unwrap_or(0) > 0);

    let after_up = harness.run_sequence_final_state("k").expect("Run sequence");
    assert_eq!(after_up.drill_page_offset, Some(0));
}

// =============================================================================
// Snapshot v2 round-trip
// =============================================================================

#[test]
fn test_snapshot_roundtrip_preserves_drillnav_fields() {
    let mut harness = InputTestHarness::new(create_drillnav_engine());

    let snapshot = harness
        .run_sequence_final_state("<Enter>jl")
        .expect("Run sequence");
    let json = snapshot.to_json().expect("Serialize");
    let restored = StateSnapshot::from_json(&json).expect("Deserialize");

    assert_eq!(restored.nav_mode, "Drill");
    assert_eq!(restored.drill_decision, snapshot.drill_decision);
    assert_eq!(restored.drill_file, snapshot.drill_file);
    assert_eq!(restored.drill_chunk, snapshot.drill_chunk);
    assert_eq!(restored.browse_cursor, None);
    assert_eq!(restored.should_quit, snapshot.should_quit);
}
