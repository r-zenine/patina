//! Input event handling and keyboard mapping
//!
//! This module handles keyboard input and maps it to UI events that affect
//! navigation, display, and input modes.

use crossterm::event::KeyEvent;
use tui_elm::KeyDispatch;

use super::bindings;
use crate::state::InputMode;

/// UI events that handle navigation and display changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiEvent {
    // Application lifecycle
    Quit,

    // Navigation
    Back,
    NavigateUp,
    NavigateDown,
    NavigateLeft,
    NavigateRight,
    NavigateToTop,
    NavigateToBottom,
    NavigatePageUp,
    NavigatePageDown,

    // Input mode transitions
    EnterInstructionMode,
    ExitInputMode,
    SubmitInput,
    CancelInput,

    // Text input
    InputChar(char),
    DeleteChar,
    DeleteForward,
    MoveCursorLeft,
    MoveCursorRight,
    MoveCursorHome,
    MoveCursorEnd,
    MoveCursorWordLeft,
    MoveCursorWordRight,

    // Review actions (will be converted to business events)
    ToggleApprove,
    ApproveFile,
    SelectCurrent,

    // DrillNav per-chunk toggles (Tab / i)
    ToggleChunkContext,
    ToggleNoteExpansion,

    // Inline reasoning annotations visibility
    ToggleReasoning,

    // Leader key system
    ActivateLeader,
    EnterLeaderSubmenu(char),
    DeactivateLeader,
    LeaderTimeout,

    // Export action
    ExportAll,

    // Help overlay
    ToggleHelp,
}

/// Handle keyboard input and convert to UI events.
///
/// Dispatch is `Registry::dispatch` on the keybinding registry
/// (`super::bindings`): a table lookup plus two coded fallbacks for
/// behavior that cannot be a finite row (both documented by the registry's
/// catch-all docs):
/// - leader scopes: any key without a row silently deactivates the leader
/// - input scope: any plain/shifted character becomes text input
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
