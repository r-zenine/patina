//! Input event handling and keyboard mapping
//!
//! This module handles keyboard input and maps it to UI events that affect
//! navigation, display, and input modes.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::bindings::{self, BindingScope};
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
/// Dispatch is a lookup in the keybinding registry (`super::bindings`), plus
/// two coded fallbacks for behavior that cannot be a finite table row (both
/// documented by the registry's catch-all docs):
/// - leader scopes: any key without a row silently deactivates the leader
/// - input scope: any plain/shifted character becomes text input
pub fn handle_key_event(
    key: KeyEvent,
    input_mode: &InputMode,
    leader_active: bool,
    leader_submenu: Option<char>,
) -> Option<UiEvent> {
    let scope = bindings::scope_of(input_mode, leader_active, leader_submenu);

    if let Some(binding) = bindings::lookup(scope, key) {
        return Some(binding.event.clone());
    }

    match scope {
        BindingScope::LeaderRoot | BindingScope::LeaderSubmenu(_) => {
            Some(UiEvent::DeactivateLeader)
        }
        BindingScope::Input => match key {
            KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                ..
            } => Some(UiEvent::InputChar(c)),
            _ => None,
        },
        BindingScope::Navigation => None,
    }
}
