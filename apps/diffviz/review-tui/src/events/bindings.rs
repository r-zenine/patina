//! The app's keybinding registry — single source of truth for key dispatch.
//!
//! One static table (a `tui_elm::Registry`) feeds five consumers: dispatch
//! (`REGISTRY.dispatch` via `handle_key_event`), the which-key overlay, the
//! help overlay, the `--describe` manifest, and per-state affordances.
//! Adding a binding here is the whole job; every consumer picks it up.
//!
//! Matching semantics and the coded fallbacks (leader dismissal, text
//! input) live in `tui_elm::Registry` and are pinned by
//! `dispatch_characterization_tests.rs`.

use crossterm::event::KeyCode;
use tui_elm::{
    Binding, BindingRole, BindingScope, CatchAllDoc, Registry, SubmenuDoc, ctrl, plain, shift,
};

use super::input::UiEvent;
use crate::state::InputMode;

pub static SUBMENUS: &[SubmenuDoc] = &[
    SubmenuDoc {
        key: 'a',
        title: "Actions",
    },
    SubmenuDoc {
        key: 't',
        title: "Toggles",
    },
];

pub static INPUT_CATCH_ALL: CatchAllDoc = CatchAllDoc {
    keys_label: "<any character>",
    event_label: "InputChar",
    description: "Insert the character at the cursor",
};

pub static LEADER_CATCH_ALL: CatchAllDoc = CatchAllDoc {
    keys_label: "<any other key>",
    event_label: "DeactivateLeader",
    description: "Close the leader menu",
};

/// The complete registry consumed by dispatch, overlays, and discovery.
pub static REGISTRY: Registry<UiEvent> = Registry {
    bindings: BINDINGS,
    submenus: SUBMENUS,
    leader_label: "Space",
    input_catch_all: Some(&INPUT_CATCH_ALL),
    leader_catch_all: Some(&LEADER_CATCH_ALL),
};

/// The rows. Order matters only for readability; `Registry::validate`
/// rejects duplicate matches (see the invariant test).
pub static BINDINGS: &[Binding<UiEvent>] = &[
    // ------------------------------------------------------------------
    // Navigation
    // ------------------------------------------------------------------
    Binding {
        scope: BindingScope::Navigation,
        keys: &[plain(KeyCode::Char('q')), ctrl(KeyCode::Char('c'))],
        event: UiEvent::Quit,
        notation: &["q", "<C-c>"],
        description: "Quit the application",
        role: BindingRole::Action,
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: &[shift(KeyCode::Char('?'))],
        event: UiEvent::ToggleHelp,
        notation: &["?"],
        description: "Toggle this help overlay",
        role: BindingRole::Action,
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: &[plain(KeyCode::Char(' '))],
        event: UiEvent::ActivateLeader,
        notation: &["<Space>"],
        description: "Open the leader menu",
        role: BindingRole::Action,
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: &[plain(KeyCode::Esc)],
        event: UiEvent::Back,
        notation: &["<Esc>"],
        description: "Back to Browse",
        role: BindingRole::Action,
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: &[plain(KeyCode::Char('h')), plain(KeyCode::Left)],
        event: UiEvent::NavigateLeft,
        notation: &["h", "<Left>"],
        description: "Drill: switch to previous sibling file",
        role: BindingRole::Action,
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: &[plain(KeyCode::Char('j')), plain(KeyCode::Down)],
        event: UiEvent::NavigateDown,
        notation: &["j", "<Down>"],
        description: "Move down (Browse: decisions, Drill: chunks)",
        role: BindingRole::Action,
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: &[plain(KeyCode::Char('k')), plain(KeyCode::Up)],
        event: UiEvent::NavigateUp,
        notation: &["k", "<Up>"],
        description: "Move up (Browse: decisions, Drill: chunks)",
        role: BindingRole::Action,
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: &[plain(KeyCode::Char('l')), plain(KeyCode::Right)],
        event: UiEvent::NavigateRight,
        notation: &["l", "<Right>"],
        description: "Drill: switch to next sibling file",
        role: BindingRole::Action,
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: &[plain(KeyCode::Char('g'))],
        event: UiEvent::NavigateToTop,
        notation: &["g"],
        description: "Jump to first",
        role: BindingRole::Action,
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: &[shift(KeyCode::Char('G'))],
        event: UiEvent::NavigateToBottom,
        notation: &["G"],
        description: "Jump to last",
        role: BindingRole::Action,
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: &[ctrl(KeyCode::Char('u')), plain(KeyCode::PageUp)],
        event: UiEvent::NavigatePageUp,
        notation: &["<C-u>", "<PageUp>"],
        description: "Page up",
        role: BindingRole::Action,
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: &[ctrl(KeyCode::Char('d')), plain(KeyCode::PageDown)],
        event: UiEvent::NavigatePageDown,
        notation: &["<C-d>", "<PageDown>"],
        description: "Page down",
        role: BindingRole::Action,
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: &[plain(KeyCode::Tab)],
        event: UiEvent::ToggleChunkContext,
        notation: &["<Tab>"],
        description: "Expand/collapse context lines of the chunk",
        role: BindingRole::Action,
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: &[plain(KeyCode::Char('i'))],
        event: UiEvent::ToggleNoteExpansion,
        notation: &["i"],
        description: "Expand/collapse the chunk's note",
        role: BindingRole::Action,
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: &[plain(KeyCode::Enter)],
        event: UiEvent::SelectCurrent,
        notation: &["<Enter>"],
        description: "Drill into the focused decision",
        role: BindingRole::Action,
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: &[plain(KeyCode::Char('a'))],
        event: UiEvent::ToggleApprove,
        notation: &["a"],
        description: "Toggle approve (decision or chunk)",
        role: BindingRole::Action,
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: &[plain(KeyCode::Char('n'))],
        event: UiEvent::EnterInstructionMode,
        notation: &["n"],
        description: "Add or append a note",
        role: BindingRole::Action,
    },
    // ------------------------------------------------------------------
    // Leader root menu (Space)
    // ------------------------------------------------------------------
    Binding {
        scope: BindingScope::LeaderRoot,
        keys: &[plain(KeyCode::Char('a'))],
        event: UiEvent::EnterLeaderSubmenu('a'),
        notation: &["a"],
        description: "Actions",
        role: BindingRole::EnterSubmenu('a'),
    },
    Binding {
        scope: BindingScope::LeaderRoot,
        keys: &[plain(KeyCode::Char('t'))],
        event: UiEvent::EnterLeaderSubmenu('t'),
        notation: &["t"],
        description: "Toggles",
        role: BindingRole::EnterSubmenu('t'),
    },
    Binding {
        scope: BindingScope::LeaderRoot,
        keys: &[plain(KeyCode::Esc)],
        event: UiEvent::DeactivateLeader,
        notation: &["<Esc>"],
        description: "Cancel",
        role: BindingRole::DismissLeader,
    },
    // ------------------------------------------------------------------
    // Actions submenu (Space + a)
    // ------------------------------------------------------------------
    Binding {
        scope: BindingScope::LeaderSubmenu('a'),
        keys: &[plain(KeyCode::Char('a')), plain(KeyCode::Char('d'))],
        event: UiEvent::ToggleApprove,
        notation: &["a", "d"],
        description: "Toggle approve (decision or chunk)",
        role: BindingRole::Action,
    },
    Binding {
        scope: BindingScope::LeaderSubmenu('a'),
        keys: &[plain(KeyCode::Char('f'))],
        event: UiEvent::ApproveFile,
        notation: &["f"],
        description: "Approve all chunks in the file",
        role: BindingRole::Action,
    },
    Binding {
        scope: BindingScope::LeaderSubmenu('a'),
        keys: &[plain(KeyCode::Esc)],
        event: UiEvent::DeactivateLeader,
        notation: &["<Esc>"],
        description: "Back",
        role: BindingRole::DismissLeader,
    },
    // ------------------------------------------------------------------
    // Toggles submenu (Space + t)
    // ------------------------------------------------------------------
    Binding {
        scope: BindingScope::LeaderSubmenu('t'),
        keys: &[plain(KeyCode::Char('r'))],
        event: UiEvent::ToggleReasoning,
        notation: &["r"],
        description: "Toggle reasoning annotations",
        role: BindingRole::Action,
    },
    Binding {
        scope: BindingScope::LeaderSubmenu('t'),
        keys: &[plain(KeyCode::Esc)],
        event: UiEvent::DeactivateLeader,
        notation: &["<Esc>"],
        description: "Back",
        role: BindingRole::DismissLeader,
    },
    // ------------------------------------------------------------------
    // Text input (Instruction / DecisionInstruction)
    // ------------------------------------------------------------------
    Binding {
        scope: BindingScope::Input,
        keys: &[plain(KeyCode::Esc), ctrl(KeyCode::Char('c'))],
        event: UiEvent::CancelInput,
        notation: &["<Esc>", "<C-c>"],
        description: "Cancel the note",
        role: BindingRole::Action,
    },
    Binding {
        scope: BindingScope::Input,
        keys: &[plain(KeyCode::Enter)],
        event: UiEvent::SubmitInput,
        notation: &["<Enter>"],
        description: "Submit the note",
        role: BindingRole::Action,
    },
    Binding {
        scope: BindingScope::Input,
        keys: &[plain(KeyCode::Backspace)],
        event: UiEvent::DeleteChar,
        notation: &["<Backspace>"],
        description: "Delete the character before the cursor",
        role: BindingRole::Action,
    },
    Binding {
        scope: BindingScope::Input,
        keys: &[plain(KeyCode::Delete)],
        event: UiEvent::DeleteForward,
        notation: &["<Delete>"],
        description: "Delete the character under the cursor",
        role: BindingRole::Action,
    },
    Binding {
        scope: BindingScope::Input,
        keys: &[plain(KeyCode::Left)],
        event: UiEvent::MoveCursorLeft,
        notation: &["<Left>"],
        description: "Move the cursor left",
        role: BindingRole::Action,
    },
    Binding {
        scope: BindingScope::Input,
        keys: &[plain(KeyCode::Right)],
        event: UiEvent::MoveCursorRight,
        notation: &["<Right>"],
        description: "Move the cursor right",
        role: BindingRole::Action,
    },
    Binding {
        scope: BindingScope::Input,
        keys: &[plain(KeyCode::Home)],
        event: UiEvent::MoveCursorHome,
        notation: &["<Home>"],
        description: "Move the cursor to the start",
        role: BindingRole::Action,
    },
    Binding {
        scope: BindingScope::Input,
        keys: &[plain(KeyCode::End)],
        event: UiEvent::MoveCursorEnd,
        notation: &["<End>"],
        description: "Move the cursor to the end",
        role: BindingRole::Action,
    },
    Binding {
        scope: BindingScope::Input,
        keys: &[ctrl(KeyCode::Left)],
        event: UiEvent::MoveCursorWordLeft,
        notation: &["<C-Left>"],
        description: "Move the cursor one word left",
        role: BindingRole::Action,
    },
    Binding {
        scope: BindingScope::Input,
        keys: &[ctrl(KeyCode::Right)],
        event: UiEvent::MoveCursorWordRight,
        notation: &["<C-Right>"],
        description: "Move the cursor one word right",
        role: BindingRole::Action,
    },
];

/// Compute the active binding scope from dispatch inputs.
pub fn scope_of(
    input_mode: &InputMode,
    leader_active: bool,
    leader_submenu: Option<char>,
) -> BindingScope {
    match input_mode {
        InputMode::Navigation => match (leader_active, leader_submenu) {
            (false, _) => BindingScope::Navigation,
            (true, None) => BindingScope::LeaderRoot,
            (true, Some(c)) => BindingScope::LeaderSubmenu(c),
        },
        InputMode::Instruction { .. } | InputMode::DecisionInstruction { .. } => {
            BindingScope::Input
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Structural invariants (no duplicate keys, full documentation,
    /// submenu reachability) are asserted by the framework.
    #[test]
    fn registry_invariants_hold() {
        REGISTRY.validate();
    }
}
