//! Status bar component showing contextual DrillNav keybinding hints.
//!
//! Chrome (message-preempts-hints styling, fixed input-mode status line) is
//! generic and lives in `tui_design::status_bar`; this module only computes
//! the triage-domain hint text.

use ratatui::{Frame, layout::Rect};
use tui_design::Theme;
use tui_design::status_bar as generic;

use crate::app::TriageData;
use crate::state::{InputMode, UiState};

pub fn render(f: &mut Frame, area: Rect, ui_state: &UiState, data: &TriageData) {
    let theme = Theme::mocha();
    match &ui_state.input_mode {
        InputMode::Navigation => render_navigation_status(f, area, ui_state, data, &theme),
        InputMode::FixInstruction { .. } => generic::render_input_status(
            f,
            area,
            "Fix instruction",
            "Enter to submit, Esc to cancel",
            &theme,
        ),
    }
}

fn render_navigation_status(
    f: &mut Frame,
    area: Rect,
    ui_state: &UiState,
    data: &TriageData,
    theme: &Theme,
) {
    let hints = if ui_state.browse_cursor().is_some() {
        browse_hints(data)
    } else {
        drill_hints(ui_state, data)
    };
    generic::render_hints(f, area, ui_state.status_message(), &hints, theme);
}

fn browse_hints(data: &TriageData) -> String {
    format!(
        "j/k navigate    Enter drill in    Space actions ({} untriaged)    ? help    q quit",
        data.symptoms.len()
    )
}

fn drill_hints(ui_state: &UiState, data: &TriageData) -> String {
    let (symptom_idx, cursor) = ui_state
        .drill_position()
        .expect("drill hints requested outside Drill mode");
    let total_sites = data.symptoms[symptom_idx].sites.len();
    format!(
        "j/k sites ({}/{})    Ctrl-d/u page    Space actions    Esc back    q quit",
        cursor + 1,
        total_sites
    )
}
