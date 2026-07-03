//! Input event handling and keyboard mapping
//!
//! This module handles keyboard input and maps it to UI events that affect
//! navigation, display, and input modes.

use crate::state::InputMode;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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

/// Handle keyboard input and convert to UI events
pub fn handle_key_event(
    key: KeyEvent,
    input_mode: &InputMode,
    leader_active: bool,
    leader_submenu: Option<char>,
) -> Option<UiEvent> {
    match input_mode {
        InputMode::Navigation => {
            if leader_active {
                handle_leader_keys(key, leader_submenu)
            } else {
                handle_navigation_keys(key)
            }
        }
        InputMode::Instruction { .. } | InputMode::DecisionInstruction { .. } => {
            handle_input_mode_keys(key)
        }
    }
}

/// Handle keys in navigation mode (the DrillNav key table)
fn handle_navigation_keys(key: KeyEvent) -> Option<UiEvent> {
    match key {
        // Application controls
        KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(UiEvent::Quit),
        KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => Some(UiEvent::Quit),

        // Help overlay
        KeyEvent {
            code: KeyCode::Char('?'),
            modifiers: KeyModifiers::SHIFT,
            ..
        } => Some(UiEvent::ToggleHelp),

        // Leader key activation
        KeyEvent {
            code: KeyCode::Char(' '),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(UiEvent::ActivateLeader),

        // Back / escape navigation level
        KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(UiEvent::Back),

        // Navigation - vim-style
        KeyEvent {
            code: KeyCode::Char('h'),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Left,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(UiEvent::NavigateLeft),
        KeyEvent {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(UiEvent::NavigateDown),
        KeyEvent {
            code: KeyCode::Char('k'),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(UiEvent::NavigateUp),
        KeyEvent {
            code: KeyCode::Char('l'),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Right,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(UiEvent::NavigateRight),

        // Navigation - extended
        KeyEvent {
            code: KeyCode::Char('g'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(UiEvent::NavigateToTop),
        KeyEvent {
            code: KeyCode::Char('G'),
            modifiers: KeyModifiers::SHIFT,
            ..
        } => Some(UiEvent::NavigateToBottom),
        KeyEvent {
            code: KeyCode::Char('u'),
            modifiers: KeyModifiers::CONTROL,
            ..
        }
        | KeyEvent {
            code: KeyCode::PageUp,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(UiEvent::NavigatePageUp),
        KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::CONTROL,
            ..
        }
        | KeyEvent {
            code: KeyCode::PageDown,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(UiEvent::NavigatePageDown),

        // Per-chunk toggles: Tab expands context, i expands the note
        KeyEvent {
            code: KeyCode::Tab,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(UiEvent::ToggleChunkContext),
        KeyEvent {
            code: KeyCode::Char('i'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(UiEvent::ToggleNoteExpansion),

        // Review actions: Enter drills in, a toggles approval, n opens the
        // note input for the focused decision (Browse) or chunk (Drill)
        KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(UiEvent::SelectCurrent),
        KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(UiEvent::ToggleApprove),
        KeyEvent {
            code: KeyCode::Char('n'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(UiEvent::EnterInstructionMode),

        _ => None,
    }
}

/// Handle keys in leader key mode
fn handle_leader_keys(key: KeyEvent, submenu: Option<char>) -> Option<UiEvent> {
    match (submenu, key.code) {
        // First level - entering submenus
        (None, KeyCode::Char('a')) => Some(UiEvent::EnterLeaderSubmenu('a')),
        (None, KeyCode::Char('t')) => Some(UiEvent::EnterLeaderSubmenu('t')),

        // Actions submenu (Space + a + ?)
        (Some('a'), KeyCode::Char('a')) => Some(UiEvent::ToggleApprove),
        (Some('a'), KeyCode::Char('d')) => Some(UiEvent::ToggleApprove),
        (Some('a'), KeyCode::Char('f')) => Some(UiEvent::ApproveFile),

        // Toggles submenu (Space + t + ?)
        (Some('t'), KeyCode::Char('r')) => Some(UiEvent::ToggleReasoning),

        // Cancel leader mode
        (_, KeyCode::Esc) => Some(UiEvent::DeactivateLeader),

        // Unknown key - deactivate leader silently
        _ => Some(UiEvent::DeactivateLeader),
    }
}

/// Handle keys in input modes (comment, instruction, edit)
fn handle_input_mode_keys(key: KeyEvent) -> Option<UiEvent> {
    match key {
        // Exit input mode
        KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(UiEvent::CancelInput),
        KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => Some(UiEvent::CancelInput),

        // Submit input - using plain Enter since Ctrl+Enter doesn't work reliably
        KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(UiEvent::SubmitInput),

        // Text input
        KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::SHIFT,
            ..
        } => Some(UiEvent::InputChar(c)),

        // Text editing
        KeyEvent {
            code: KeyCode::Backspace,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(UiEvent::DeleteChar),
        KeyEvent {
            code: KeyCode::Delete,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(UiEvent::DeleteForward),

        // Cursor movement
        KeyEvent {
            code: KeyCode::Left,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(UiEvent::MoveCursorLeft),
        KeyEvent {
            code: KeyCode::Right,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(UiEvent::MoveCursorRight),
        KeyEvent {
            code: KeyCode::Home,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(UiEvent::MoveCursorHome),
        KeyEvent {
            code: KeyCode::End,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(UiEvent::MoveCursorEnd),
        KeyEvent {
            code: KeyCode::Left,
            modifiers: KeyModifiers::CONTROL,
            ..
        } => Some(UiEvent::MoveCursorWordLeft),
        KeyEvent {
            code: KeyCode::Right,
            modifiers: KeyModifiers::CONTROL,
            ..
        } => Some(UiEvent::MoveCursorWordRight),

        _ => None,
    }
}
