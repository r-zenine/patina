//! UI rendering and component modules.

pub mod components;
pub mod icons;
pub mod layout;

use crate::app::TriageData;
use crate::events::bindings::REGISTRY;
use crate::state::UiState;
use ratatui::Frame;
use ratatui::widgets::Paragraph;
use tui_design::{Theme, stylesheet};
use tui_elm::HelpText;

/// Main UI drawing function: full-width DrillNav view over a status bar,
/// with modal/overlay layers on top.
pub fn draw(f: &mut Frame, ui_state: &UiState, data: &TriageData) {
    let theme = Theme::mocha();

    f.render_widget(
        Paragraph::new("").style(stylesheet::terminal_floor(&theme)),
        f.area(),
    );

    let chunks = layout::create_main_layout(f.area());

    if ui_state.browse_cursor().is_some() {
        components::drillnav_browse::render(f, chunks.content, ui_state, data);
    } else {
        components::drillnav_drill::render(f, chunks.content, ui_state, data);
    }

    components::status_bar::render(f, chunks.status_bar, ui_state, data);

    if ui_state.is_in_input_mode() {
        components::input_modal::render(f, f.area(), ui_state, data);
    }

    tui_elm::which_key::render(f, &REGISTRY, &ui_state.leader, &theme);
    if ui_state.show_help {
        tui_elm::help::render(
            f,
            &REGISTRY,
            &theme,
            &HelpText {
                title: "Keybindings Help (? to close)",
                input_section: "TEXT INPUT (fix instruction)",
            },
        );
    }
}
