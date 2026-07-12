//! Input event handling and keyboard mapping.

use crossterm::event::KeyEvent;
use tui_elm::KeyDispatch;

use super::bindings;
use crate::tui::state::InputMode;

/// UI events that handle navigation, display, and triage actions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiEvent {
    // Application lifecycle
    Quit,

    // Navigation
    Back,
    NavigateUp,
    NavigateDown,
    NavigatePageUp,
    NavigatePageDown,
    SelectCurrent,

    // Input mode transitions
    EnterFixInstructionMode,
    ExitInputMode,
    SubmitInput,
    CancelInput,

    // Text input
    InputChar(char),
    DeleteChar,
    MoveCursorLeft,
    MoveCursorRight,
    MoveCursorHome,
    MoveCursorEnd,

    // Triage verdicts (will be converted to business events)
    DismissFalsePositive,
    DismissIntentional,
    DismissAcceptedDebt,

    // Leader key system
    ActivateLeader,
    EnterLeaderSubmenu(char),
    DeactivateLeader,
    LeaderTimeout,

    // Help overlay
    ToggleHelp,
}

/// Handle keyboard input and convert to UI events.
///
/// Dispatch is `Registry::dispatch` on the keybinding registry
/// (`super::bindings`): a table lookup plus two coded fallbacks —
/// leader scopes silently deactivate the leader on any unbound key, and
/// the input scope turns any plain/shifted character into text input.
pub fn handle_key_event(
    key: KeyEvent,
    input_mode: &InputMode,
    leader_active: bool,
    leader_submenu: Option<char>,
) -> Option<UiEvent> {
    let scope = bindings::scope_of(input_mode, leader_active, leader_submenu);

    match bindings::REGISTRY.dispatch(scope, key)? {
        KeyDispatch::Bound(binding) => Some(binding.event.clone()),
        KeyDispatch::DismissLeader => Some(UiEvent::DeactivateLeader),
        KeyDispatch::TextChar(c) => Some(UiEvent::InputChar(c)),
    }
}
