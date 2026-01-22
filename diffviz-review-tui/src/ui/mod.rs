//! UI rendering and component modules

pub mod components;
pub mod layout;

use crate::state::UiState;
use diffviz_review::engines::ReviewEngine;
use ratatui::Frame;

/// Main UI drawing function
pub fn draw(f: &mut Frame, ui_state: &UiState, review_engine: &ReviewEngine) {
    let chunks = layout::create_main_layout(f.area());

    // Render main components - tree explorer is now primary view for all decision levels
    components::decision_tree::render(f, chunks.file_list, ui_state, review_engine);

    components::diff_view::render(f, chunks.diff_view, ui_state, review_engine);
    components::status_bar::render(f, chunks.status_bar, ui_state, review_engine);

    // Render overlays (in order - last rendered is on top)
    if ui_state.is_in_input_mode() {
        components::input_modal::render(f, f.area(), ui_state);
    }

    // Render decision detail modal if active
    components::decision_detail_modal::render(f, f.area(), ui_state, review_engine);

    components::which_key::render(f, ui_state);
    components::help_overlay::render(f, ui_state);
}
