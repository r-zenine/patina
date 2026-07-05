//! Help overlay component showing all keybindings.
//!
//! Thin wrapper over `tui_elm::help`: the overlay is rendered from the
//! keybinding registry (`events::bindings::REGISTRY`), so it cannot drift
//! from dispatch.

use ratatui::Frame;
use tui_design::Theme;
use tui_elm::HelpText;

use crate::events::bindings::REGISTRY;
use crate::state::UiState;

pub fn render(f: &mut Frame, ui_state: &UiState) {
    if !ui_state.show_help {
        return;
    }

    tui_elm::help::render(
        f,
        &REGISTRY,
        &Theme::mocha(),
        &HelpText {
            title: "Keybindings Help (? to close)",
            input_section: "TEXT INPUT (note editing)",
        },
    );
}
