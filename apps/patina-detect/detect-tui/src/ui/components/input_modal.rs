//! Input modal component for the Fix instruction text entry.
//!
//! Modal chrome (popup framing, cursor rendering) is generic and lives in
//! `tui_design::input_modal`; this module only resolves the triage-domain
//! title text for the current `InputMode`.

use ratatui::{Frame, layout::Rect};
use tui_design::Theme;
use tui_design::input_modal as generic;

use crate::app::TriageData;
use crate::state::{InputMode, UiState};

pub fn render(f: &mut Frame, area: Rect, ui_state: &UiState, data: &TriageData) {
    if !ui_state.is_in_input_mode() {
        return;
    }

    let theme = Theme::mocha();
    let title = match &ui_state.input_mode {
        InputMode::FixInstruction { symptom_id } => {
            let title = data
                .symptoms
                .iter()
                .find(|s| &s.id == symptom_id)
                .map(|s| s.title.as_str())
                .unwrap_or("(symptom no longer available)");
            format!("Fix instruction - {title}")
        }
        InputMode::Navigation => String::new(),
    };

    generic::render(
        f,
        area,
        &title,
        &ui_state.input_buffer,
        ui_state.input_cursor,
        "Describe how to fix this...",
        &theme,
    );
}
