//! UI rendering and component modules

pub mod components;
pub mod layout;

use crate::state::UiState;
use diffviz_review::engines::ReviewEngine;
use ratatui::Frame;
use ratatui::widgets::Paragraph;
use tui_design::{Theme, stylesheet};

/// Main UI drawing function: full-width DrillNav view over a status bar,
/// with modal/overlay layers on top.
pub fn draw(f: &mut Frame, ui_state: &UiState, review_engine: &ReviewEngine) {
    let theme = Theme::mocha();

    // Terminal floor (crust) under everything.
    f.render_widget(
        Paragraph::new("").style(stylesheet::terminal_floor(&theme)),
        f.area(),
    );

    let chunks = layout::create_main_layout(f.area());

    if ui_state.browse_cursor().is_some() {
        components::drillnav_browse::render(f, chunks.content, ui_state, review_engine);
    } else {
        components::drillnav_drill::render(f, chunks.content, ui_state, review_engine);
    }

    components::status_bar::render(f, chunks.status_bar, ui_state, review_engine);

    // Render overlays (in order - last rendered is on top)
    if ui_state.is_in_input_mode() {
        components::input_modal::render(f, f.area(), ui_state, review_engine);
    }

    components::which_key::render(f, ui_state);
    components::help_overlay::render(f, ui_state);
}
