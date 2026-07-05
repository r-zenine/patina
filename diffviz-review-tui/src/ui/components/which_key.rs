//! Which-key style overlay for the leader menu.
//!
//! Thin wrapper over `tui_elm::which_key`: the overlay is rendered from the
//! keybinding registry (`events::bindings::REGISTRY`), so it cannot drift
//! from dispatch.

use ratatui::Frame;
use tui_design::Theme;

use crate::events::bindings::REGISTRY;
use crate::state::UiState;

/// Render the which-key overlay when the leader is active.
pub fn render(f: &mut Frame, ui_state: &UiState) {
    tui_elm::which_key::render(f, &REGISTRY, &ui_state.leader, &Theme::mocha());
}
