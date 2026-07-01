//! DrillNav navigation contract (plan-drillnav-main-tui, Phase 0)
//!
//! Encodes the Browse/Drill state machine behavior as failing tests:
//! browse j/k bounds, Enter drill-in, Esc back with cursor restore, h/l
//! sibling-file cycling with wraparound and per-file retention, Tab context
//! expansion, i note expansion, g/G jumps, Ctrl-d/u viewport paging, plus
//! the retained bindings (Space leader, ?, q).
//!
//! Fixture map (see drillnav_common): decision idx 0 has files
//! [reader.rs (2 chunks), calculator.rs (7 chunks)]; decision idx 2 has
//! 3 files; 3 decisions total.

#![cfg(feature = "test-harness")]

mod drillnav_common;

use drillnav_common::create_drillnav_engine;

use diffviz_review_tui::test_harness::InputTestHarness;

// =============================================================================
// Browse mode
// =============================================================================

#[test]
fn test_initial_state_is_browse_at_first_decision() {
    let mut harness = InputTestHarness::new(create_drillnav_engine());

    let snapshot = harness.run_sequence_final_state("").expect("Run sequence");

    assert_eq!(snapshot.nav_mode, "Browse");
    assert_eq!(snapshot.browse_cursor, Some(0));
    assert_eq!(snapshot.drill_decision, None);
    assert_eq!(snapshot.drill_file, None);
    assert_eq!(snapshot.drill_chunk, None);
    assert_eq!(snapshot.status_message, None);
}

#[test]
fn test_browse_j_moves_down_and_clamps_at_last_decision() {
    let mut harness = InputTestHarness::new(create_drillnav_engine());

    let snapshots = harness.run_sequence("jjjj").expect("Run sequence");

    assert_eq!(snapshots[1].browse_cursor, Some(1));
    assert_eq!(snapshots[2].browse_cursor, Some(2));
    // 3 decisions: further j presses clamp at the last card
    assert_eq!(snapshots[3].browse_cursor, Some(2));
    assert_eq!(snapshots[4].browse_cursor, Some(2));
}

#[test]
fn test_browse_k_moves_up_and_clamps_at_first_decision() {
    let mut harness = InputTestHarness::new(create_drillnav_engine());

    let snapshots = harness.run_sequence("jjkkk").expect("Run sequence");

    assert_eq!(snapshots[2].browse_cursor, Some(2));
    assert_eq!(snapshots[3].browse_cursor, Some(1));
    assert_eq!(snapshots[4].browse_cursor, Some(0));
    assert_eq!(snapshots[5].browse_cursor, Some(0));
}

#[test]
fn test_browse_h_l_do_not_change_browse_position() {
    let mut harness = InputTestHarness::new(create_drillnav_engine());

    let snapshot = harness
        .run_sequence_final_state("jhl")
        .expect("Run sequence");

    assert_eq!(snapshot.nav_mode, "Browse");
    assert_eq!(snapshot.browse_cursor, Some(1));
}

#[test]
fn test_browse_g_and_shift_g_jump_to_first_and_last_decision() {
    let mut harness = InputTestHarness::new(create_drillnav_engine());

    let snapshot = harness
        .run_sequence_final_state("<S-G>")
        .expect("Run sequence");
    assert_eq!(snapshot.browse_cursor, Some(2));

    let mut harness = InputTestHarness::new(create_drillnav_engine());
    let snapshot = harness
        .run_sequence_final_state("jjg")
        .expect("Run sequence");
    assert_eq!(snapshot.browse_cursor, Some(0));
}

// =============================================================================
// Drill-in / back
// =============================================================================

#[test]
fn test_enter_drills_into_decision_under_cursor() {
    let mut harness = InputTestHarness::new(create_drillnav_engine());

    let snapshot = harness
        .run_sequence_final_state("<Enter>")
        .expect("Run sequence");

    assert_eq!(snapshot.nav_mode, "Drill");
    assert_eq!(snapshot.drill_decision, Some(0));
    assert_eq!(snapshot.drill_file, Some(0));
    assert_eq!(snapshot.drill_chunk, Some(0));
    assert_eq!(snapshot.browse_cursor, None);
}

#[test]
fn test_esc_backs_out_and_restores_browse_cursor() {
    let mut harness = InputTestHarness::new(create_drillnav_engine());

    let snapshot = harness
        .run_sequence_final_state("j<Enter><Esc>")
        .expect("Run sequence");

    assert_eq!(snapshot.nav_mode, "Browse");
    assert_eq!(snapshot.browse_cursor, Some(1));
    assert_eq!(snapshot.drill_decision, None);
}

#[test]
fn test_esc_in_browse_does_not_quit_or_move() {
    let mut harness = InputTestHarness::new(create_drillnav_engine());

    let snapshot = harness
        .run_sequence_final_state("<Esc>")
        .expect("Run sequence");

    assert_eq!(snapshot.nav_mode, "Browse");
    assert_eq!(snapshot.browse_cursor, Some(0));
    assert!(!snapshot.should_quit);
}

// =============================================================================
// Drill mode: chunk cursor (j/k, g/G)
// =============================================================================

#[test]
fn test_drill_j_k_moves_chunk_cursor_with_bounds() {
    let mut harness = InputTestHarness::new(create_drillnav_engine());

    // Decision 0, file 0 (reader.rs) has exactly 2 chunks.
    let snapshots = harness.run_sequence("<Enter>jjk").expect("Run sequence");

    assert_eq!(snapshots[1].drill_chunk, Some(0));
    assert_eq!(snapshots[2].drill_chunk, Some(1));
    // Clamped at the last chunk
    assert_eq!(snapshots[3].drill_chunk, Some(1));
    assert_eq!(snapshots[4].drill_chunk, Some(0));
}

#[test]
fn test_drill_g_and_shift_g_jump_to_first_and_last_chunk() {
    let mut harness = InputTestHarness::new(create_drillnav_engine());

    // Decision 0, file 1 (calculator.rs) has 7 chunks.
    let snapshot = harness
        .run_sequence_final_state("<Enter>l<S-G>")
        .expect("Run sequence");
    assert_eq!(snapshot.drill_chunk, Some(6));

    let mut harness = InputTestHarness::new(create_drillnav_engine());
    let snapshot = harness
        .run_sequence_final_state("<Enter>l<S-G>g")
        .expect("Run sequence");
    assert_eq!(snapshot.drill_chunk, Some(0));
}

// =============================================================================
// Drill mode: sibling files (h/l)
// =============================================================================

#[test]
fn test_drill_l_cycles_files_with_wraparound() {
    let mut harness = InputTestHarness::new(create_drillnav_engine());

    // Decision 0 has 2 files: l moves 0 → 1, l again wraps 1 → 0.
    let snapshots = harness.run_sequence("<Enter>ll").expect("Run sequence");

    assert_eq!(snapshots[1].drill_file, Some(0));
    assert_eq!(snapshots[2].drill_file, Some(1));
    assert_eq!(snapshots[3].drill_file, Some(0));
}

#[test]
fn test_drill_h_cycles_files_backward_with_wraparound() {
    let mut harness = InputTestHarness::new(create_drillnav_engine());

    // Decision idx 2 has 3 files: h from file 0 wraps to file 2.
    let snapshot = harness
        .run_sequence_final_state("jj<Enter>h")
        .expect("Run sequence");

    assert_eq!(snapshot.drill_decision, Some(2));
    assert_eq!(snapshot.drill_file, Some(2));
}

#[test]
fn test_per_file_chunk_cursor_retained_when_cycling_siblings() {
    let mut harness = InputTestHarness::new(create_drillnav_engine());

    // Move to chunk 1 in file 0, visit file 1, come back: cursor still 1.
    let snapshots = harness.run_sequence("<Enter>jlh").expect("Run sequence");

    assert_eq!(snapshots[2].drill_chunk, Some(1));
    // Fresh file starts at its own cursor (0)
    assert_eq!(snapshots[3].drill_file, Some(1));
    assert_eq!(snapshots[3].drill_chunk, Some(0));
    // Back on file 0 with position retained
    assert_eq!(snapshots[4].drill_file, Some(0));
    assert_eq!(snapshots[4].drill_chunk, Some(1));
}

// =============================================================================
// Drill mode: expansion toggles (Tab, i)
// =============================================================================

#[test]
fn test_tab_toggles_context_expansion_on_focused_chunk() {
    let mut harness = InputTestHarness::new(create_drillnav_engine());

    let snapshots = harness
        .run_sequence("<Enter><Tab><Tab>")
        .expect("Run sequence");

    assert_eq!(snapshots[1].drill_context_expanded, Some(false));
    assert_eq!(snapshots[2].drill_context_expanded, Some(true));
    assert_eq!(snapshots[3].drill_context_expanded, Some(false));
}

#[test]
fn test_context_expansion_is_per_chunk() {
    let mut harness = InputTestHarness::new(create_drillnav_engine());

    // Expand chunk 0, move to chunk 1: chunk 1 is not expanded; moving back
    // to chunk 0 shows it still expanded.
    let snapshots = harness
        .run_sequence("<Enter><Tab>jk")
        .expect("Run sequence");

    assert_eq!(snapshots[2].drill_context_expanded, Some(true));
    assert_eq!(snapshots[3].drill_context_expanded, Some(false));
    assert_eq!(snapshots[4].drill_context_expanded, Some(true));
}

#[test]
fn test_i_toggles_note_expansion_on_focused_chunk() {
    let mut harness = InputTestHarness::new(create_drillnav_engine());

    let snapshots = harness.run_sequence("<Enter>ii").expect("Run sequence");

    assert_eq!(snapshots[1].drill_note_expanded, Some(false));
    assert_eq!(snapshots[2].drill_note_expanded, Some(true));
    assert_eq!(snapshots[3].drill_note_expanded, Some(false));
}

// =============================================================================
// Drill mode: viewport paging (Ctrl-d/u)
// =============================================================================

#[test]
fn test_ctrl_d_pages_drill_viewport_down_and_ctrl_u_back() {
    let mut harness = InputTestHarness::new(create_drillnav_engine());

    // calculator.rs (7 chunk cards) overflows the default viewport.
    let snapshots = harness
        .run_sequence("<Enter>l<C-d><C-u>")
        .expect("Run sequence");

    assert_eq!(snapshots[2].drill_page_offset, Some(0));
    let paged = snapshots[3].drill_page_offset.expect("in drill mode");
    assert!(paged > 0, "Ctrl-d should page the drill viewport down");
    assert_eq!(snapshots[4].drill_page_offset, Some(0));
}

#[test]
fn test_ctrl_u_clamps_at_zero() {
    let mut harness = InputTestHarness::new(create_drillnav_engine());

    let snapshot = harness
        .run_sequence_final_state("<Enter><C-u>")
        .expect("Run sequence");

    assert_eq!(snapshot.drill_page_offset, Some(0));
}

// =============================================================================
// Retained global bindings (q, ?, Space leader)
// =============================================================================

#[test]
fn test_q_quits_from_browse_and_drill() {
    let mut harness = InputTestHarness::new(create_drillnav_engine());
    let snapshot = harness.run_sequence_final_state("q").expect("Run sequence");
    assert!(snapshot.should_quit);

    let mut harness = InputTestHarness::new(create_drillnav_engine());
    let snapshot = harness
        .run_sequence_final_state("<Enter>q")
        .expect("Run sequence");
    assert!(snapshot.should_quit);
}

#[test]
fn test_help_overlay_toggles_in_both_modes() {
    let mut harness = InputTestHarness::new(create_drillnav_engine());
    let snapshot = harness
        .run_sequence_final_state("<S-?>")
        .expect("Run sequence");
    assert!(snapshot.show_help);

    let mut harness = InputTestHarness::new(create_drillnav_engine());
    let snapshot = harness
        .run_sequence_final_state("<Enter><S-?>")
        .expect("Run sequence");
    assert!(snapshot.show_help);
}

#[test]
fn test_space_activates_leader_in_both_modes() {
    let mut harness = InputTestHarness::new(create_drillnav_engine());
    let snapshot = harness
        .run_sequence_final_state("<Space>")
        .expect("Run sequence");
    assert!(snapshot.leader_active);

    let mut harness = InputTestHarness::new(create_drillnav_engine());
    let snapshot = harness
        .run_sequence_final_state("<Enter><Space>")
        .expect("Run sequence");
    assert!(snapshot.leader_active);
}
