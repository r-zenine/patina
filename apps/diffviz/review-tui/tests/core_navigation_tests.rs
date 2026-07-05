//! Core navigation visuals for DrillNav (plan-drillnav-main-tui Phase 2).
//!
//! The drillnav_navigation_tests suite owns the state-machine contract
//! (cursor bounds, wraparound, per-file retention). This suite validates the
//! *rendered* navigation feedback: the accent bar follows the browse cursor,
//! dot pagination tracks h/l cycling, the pinned header swaps per file,
//! scroll_into_view keeps the focused card visible, and Ctrl-d/u actually
//! move the drill viewport.

#![cfg(feature = "test-harness")]

mod drillnav_common;

use drillnav_common::create_drillnav_engine;

use diffviz_review_tui::test_harness::CombinedTestHarness;

/// The rendered row containing `needle`, if any.
fn line_with<'a>(visual: &'a str, needle: &str) -> Option<&'a str> {
    visual.lines().find(|l| l.contains(needle))
}

/// Whether the card row containing `needle` carries the focus accent bar.
fn is_focused(visual: &str, needle: &str) -> bool {
    line_with(visual, needle)
        .unwrap_or_else(|| panic!("no rendered line contains {needle:?}"))
        .starts_with('▌')
}

// =============================================================================
// Browse: accent bar follows the cursor
// =============================================================================

#[test]
fn test_accent_bar_starts_on_first_decision_card() {
    let mut harness = CombinedTestHarness::with_render_size(create_drillnav_engine(), 120, 40);

    let results = harness.run_sequence_with_renders("").expect("Render");
    let visual = &results[0].visual;

    assert!(is_focused(visual, "#1 Refactor calculator model module"));
    assert!(!is_focused(visual, "#2 Improve error handling"));
    assert!(!is_focused(visual, "#3 Add structured logging"));
}

#[test]
fn test_accent_bar_follows_j_and_k() {
    let mut harness = CombinedTestHarness::with_render_size(create_drillnav_engine(), 120, 40);

    let results = harness.run_sequence_with_renders("jk").expect("Run");

    let after_j = &results[1].visual;
    assert!(!is_focused(after_j, "#1 Refactor calculator model module"));
    assert!(is_focused(after_j, "#2 Improve error handling"));

    let after_k = &results[2].visual;
    assert!(is_focused(after_k, "#1 Refactor calculator model module"));
    assert!(!is_focused(after_k, "#2 Improve error handling"));
}

#[test]
fn test_browse_scrolls_focused_card_into_view_on_small_terminal() {
    // 10 rows can't fit all three cards; jumping to the last decision must
    // scroll its card into the viewport.
    let mut harness = CombinedTestHarness::with_render_size(create_drillnav_engine(), 120, 10);

    let results = harness.run_sequence_with_renders("<S-G>").expect("Run");

    let initial = &results[0].visual;
    assert!(initial.contains("#1 Refactor calculator model module"));
    assert!(!initial.contains("#3 Add structured logging"));

    let at_bottom = &results[1].visual;
    assert!(at_bottom.contains("#3 Add structured logging"));
    assert!(is_focused(at_bottom, "#3 Add structured logging"));
}

// =============================================================================
// Drill: pinned header and dot pagination track h/l
// =============================================================================

#[test]
fn test_dot_pagination_tracks_file_cycling() {
    let mut harness = CombinedTestHarness::with_render_size(create_drillnav_engine(), 120, 40);

    let results = harness.run_sequence_with_renders("<Enter>lh").expect("Run");

    // File 0 of 2 focused: active dot first
    assert!(results[1].visual.contains("● ○"));
    // l → file 1: active dot second
    assert!(results[2].visual.contains("○ ●"));
    // h → back to file 0
    assert!(results[3].visual.contains("● ○"));
}

#[test]
fn test_pinned_header_swaps_with_sibling_file() {
    let mut harness = CombinedTestHarness::with_render_size(create_drillnav_engine(), 120, 40);

    let results = harness.run_sequence_with_renders("<Enter>l").expect("Run");

    let file0 = &results[1].visual;
    assert!(file0.contains("~ src/config/reader.rs"));
    assert!(file0.contains("Configuration reader updates"));

    let file1 = &results[2].visual;
    assert!(file1.contains("~ src/models/calculator.rs"));
    assert!(file1.contains("Calculator model structure refactoring"));
    assert!(!file1.contains("~ src/config/reader.rs"));
}

#[test]
fn test_h_from_first_file_wraps_to_last_sibling() {
    let mut harness = CombinedTestHarness::with_render_size(create_drillnav_engine(), 120, 40);

    // Decision idx 2 has 3 files; h from file 0 wraps to file 2.
    let results = harness
        .run_sequence_with_renders("jj<Enter>h")
        .expect("Run");
    let visual = &results.last().expect("results").visual;

    assert!(visual.contains("○ ○ ●"));
    // File index 2 in lexicographic order: src/types/api.ts
    assert!(visual.contains("~ src/types/api.ts"));
}

#[test]
fn test_drill_renders_chunk_separators_for_multi_chunk_file() {
    let mut harness = CombinedTestHarness::with_render_size(create_drillnav_engine(), 120, 40);

    let results = harness.run_sequence_with_renders("<Enter>").expect("Run");

    // reader.rs has 2 chunks → exactly one ··· separator row between cards
    let visual = &results[1].visual;
    let separators = visual.lines().filter(|l| l.trim() == "···").count();
    assert_eq!(separators, 1);
}

// =============================================================================
// Drill: viewport paging (Ctrl-d/u)
// =============================================================================

#[test]
fn test_ctrl_d_moves_drill_viewport_and_ctrl_u_restores_it() {
    // calculator.rs (7 chunk cards) overflows a 24-row viewport.
    let mut harness = CombinedTestHarness::with_render_size(create_drillnav_engine(), 120, 24);

    let results = harness
        .run_sequence_with_renders("<Enter>l<C-d><C-u>")
        .expect("Run");

    let unpaged = &results[2].visual;
    let paged = &results[3].visual;
    let restored = &results[4].visual;

    assert_ne!(
        unpaged, paged,
        "Ctrl-d should scroll the drill viewport content"
    );
    assert_eq!(unpaged, restored, "Ctrl-u should page back to the top");
    // The pinned header must survive paging
    assert!(paged.contains("~ src/models/calculator.rs"));
}

#[test]
fn test_page_offset_is_clamped_at_content_end() {
    let mut harness = CombinedTestHarness::with_render_size(create_drillnav_engine(), 120, 24);

    // Page far past the end: rendering clamps, so one more Ctrl-d changes nothing.
    let results = harness
        .run_sequence_with_renders("<Enter>l<C-d><C-d><C-d><C-d><C-d><C-d><C-d><C-d>")
        .expect("Run");

    let far = &results[results.len() - 2].visual;
    let further = &results[results.len() - 1].visual;
    assert_eq!(far, further, "paging past the end must be clamped");
}
