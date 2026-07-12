//! The app's keybinding registry — single source of truth for key dispatch.
//!
//! One static table (a `tui_elm::Registry`) feeds five consumers: dispatch
//! (`REGISTRY.dispatch` via `handle_key_event`), the which-key overlay, the
//! help overlay, the `--describe` manifest, and per-state affordances.
//! Adding a binding here is the whole job; every consumer picks it up.

use crossterm::event::KeyCode;
use tui_elm::{
    Binding, BindingRole, BindingScope, CatchAllDoc, Registry, SubmenuDoc, ctrl, plain, shift,
};

use super::input::UiEvent;
use crate::tui::state::InputMode;

pub static SUBMENUS: &[SubmenuDoc] = &[SubmenuDoc {
    key: 'd',
    title: "Dismiss",
}];

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
        keys: &[plain(KeyCode::Char('j')), plain(KeyCode::Down)],
        event: UiEvent::NavigateDown,
        notation: &["j", "<Down>"],
        description: "Move down (Browse: symptoms, Drill: sites)",
        role: BindingRole::Action,
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: &[plain(KeyCode::Char('k')), plain(KeyCode::Up)],
        event: UiEvent::NavigateUp,
        notation: &["k", "<Up>"],
        description: "Move up (Browse: symptoms, Drill: sites)",
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
        keys: &[plain(KeyCode::Enter)],
        event: UiEvent::SelectCurrent,
        notation: &["<Enter>"],
        description: "Drill into the focused symptom",
        role: BindingRole::Action,
    },
    // ------------------------------------------------------------------
    // Leader root menu (Space)
    // ------------------------------------------------------------------
    Binding {
        scope: BindingScope::LeaderRoot,
        keys: &[plain(KeyCode::Char('d'))],
        event: UiEvent::EnterLeaderSubmenu('d'),
        notation: &["d"],
        description: "Dismiss",
        role: BindingRole::EnterSubmenu('d'),
    },
    Binding {
        scope: BindingScope::LeaderRoot,
        keys: &[plain(KeyCode::Char('f'))],
        event: UiEvent::EnterFixInstructionMode,
        notation: &["f"],
        description: "Fix (write an instruction)",
        role: BindingRole::Action,
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
    // Dismiss submenu (Space + d)
    // ------------------------------------------------------------------
    Binding {
        scope: BindingScope::LeaderSubmenu('d'),
        keys: &[plain(KeyCode::Char('f'))],
        event: UiEvent::DismissFalsePositive,
        notation: &["f"],
        description: "False positive",
        role: BindingRole::Action,
    },
    Binding {
        scope: BindingScope::LeaderSubmenu('d'),
        keys: &[plain(KeyCode::Char('i'))],
        event: UiEvent::DismissIntentional,
        notation: &["i"],
        description: "Intentional",
        role: BindingRole::Action,
    },
    Binding {
        scope: BindingScope::LeaderSubmenu('d'),
        keys: &[plain(KeyCode::Char('a'))],
        event: UiEvent::DismissAcceptedDebt,
        notation: &["a"],
        description: "Accepted debt",
        role: BindingRole::Action,
    },
    Binding {
        scope: BindingScope::LeaderSubmenu('d'),
        keys: &[plain(KeyCode::Esc)],
        event: UiEvent::DeactivateLeader,
        notation: &["<Esc>"],
        description: "Back",
        role: BindingRole::DismissLeader,
    },
    // ------------------------------------------------------------------
    // Text input (Fix instruction)
    // ------------------------------------------------------------------
    Binding {
        scope: BindingScope::Input,
        keys: &[plain(KeyCode::Esc), ctrl(KeyCode::Char('c'))],
        event: UiEvent::CancelInput,
        notation: &["<Esc>", "<C-c>"],
        description: "Cancel the fix instruction",
        role: BindingRole::Action,
    },
    Binding {
        scope: BindingScope::Input,
        keys: &[plain(KeyCode::Enter)],
        event: UiEvent::SubmitInput,
        notation: &["<Enter>"],
        description: "Submit the fix instruction",
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
        InputMode::FixInstruction { .. } => BindingScope::Input,
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
