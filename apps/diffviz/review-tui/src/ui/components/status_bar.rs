//! Status bar component showing contextual DrillNav keybinding hints.
//!
//! A one-shot error `status_message` (D7) preempts the hints in red until
//! the next keypress. Hints are contextual per mode (prototype format):
//! Browse advertises drill-in and decision approval progress; Drill
//! advertises h/l only with multiple sibling files, the note toggle only
//! when the focused chunk has a note, and per-file chunk approval progress.
//!
//! The chrome (message-preempts-hints styling, the fixed input-mode status
//! line) is generic and lives in `tui_design::status_bar`; this module only
//! computes the review-domain hint text.

use ratatui::{Frame, layout::Rect};
use tui_design::Theme;
use tui_design::status_bar as generic;

use super::drillnav_common::note_for;
use crate::state::{InputMode, UiState};
use diffviz_review::engines::ReviewEngine;

/// Render the status bar
pub fn render(f: &mut Frame, area: Rect, ui_state: &UiState, review_engine: &ReviewEngine) {
    let theme = Theme::mocha();
    match &ui_state.input_mode {
        InputMode::Navigation => render_navigation_status(f, area, ui_state, review_engine, &theme),
        InputMode::Instruction { .. } => generic::render_input_status(
            f,
            area,
            "Instruction",
            "Enter to submit, Esc to cancel",
            &theme,
        ),
        InputMode::DecisionInstruction { .. } => generic::render_input_status(
            f,
            area,
            "Decision Instruction",
            "Enter to submit, Esc to cancel",
            &theme,
        ),
    }
}

fn render_navigation_status(
    f: &mut Frame,
    area: Rect,
    ui_state: &UiState,
    review_engine: &ReviewEngine,
    theme: &Theme,
) {
    let hints = if ui_state.browse_cursor().is_some() {
        browse_hints(review_engine)
    } else {
        drill_hints(ui_state, review_engine)
    };
    generic::render_hints(f, area, ui_state.status_message(), &hints, theme);
}

fn browse_hints(review_engine: &ReviewEngine) -> String {
    let approved = review_engine.get_approved_decisions_count();
    let total = review_engine.get_all_decisions().len();
    format!(
        "j/k navigate    Enter drill in    n note    a approve ({approved}/{total})    ? help    q quit",
    )
}

fn drill_hints(ui_state: &UiState, review_engine: &ReviewEngine) -> String {
    let (decision_idx, file_idx, cursor) = ui_state
        .drill_position()
        .expect("drill hints requested outside Drill mode");
    let files = &ui_state.drill_index().decisions[decision_idx].files;
    let chunk_ids = &files[file_idx].chunks;

    let total_files = files.len();
    let total_chunks = chunk_ids.len();
    let approved_count = chunk_ids
        .iter()
        .filter(|id| review_engine.state().is_approved(id))
        .count();

    let ctx_label = if ui_state.drill_context_expanded() == Some(true) {
        "collapse ctx"
    } else {
        "expand ctx"
    };
    // Only advertise h/l when there is more than one file to cycle.
    let files_hint = if total_files > 1 {
        format!("file {}/{}    h/l files    ", file_idx + 1, total_files)
    } else {
        String::new()
    };
    // Only advertise the note toggle when the focused chunk has one.
    let note_hint = chunk_ids
        .get(cursor)
        .and_then(|id| note_for(review_engine, id))
        .map(|_| {
            if ui_state.drill_note_expanded() == Some(true) {
                "i collapse note    "
            } else {
                "i expand note    "
            }
        })
        .unwrap_or("");
    format!(
        "{files_hint}j/k chunks    Ctrl-d/u page    Tab {ctx_label}    {note_hint}n note    a approve ({approved_count}/{total_chunks})    Esc back    q quit",
    )
}
