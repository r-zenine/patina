//! The keybinding registry — single source of truth for key dispatch.
//!
//! One static table feeds five consumers: dispatch (`lookup` via
//! `handle_key_event`), the which-key overlay, the help overlay, the
//! `--describe` manifest, and per-state affordances. Adding a binding here is
//! the whole job; every consumer picks it up.
//!
//! Matching semantics are per-scope (see `lookup`), reproducing the behavior
//! pinned by `dispatch_characterization_tests.rs`:
//! - `Navigation` / `Input` match key code AND modifiers exactly
//! - Leader scopes match key code only (modifiers ignored), and any key
//!   without a row deactivates the leader
//!
//! Truly parametric behavior (typing text) cannot be a finite row; it lives
//! as a coded fallback in `handle_key_event` and is documented by the
//! catch-all docs below so the manifest and affordances stay honest.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::input::UiEvent;
use crate::state::InputMode;

/// Where a binding is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingScope {
    /// Navigation mode with the leader inactive.
    Navigation,
    /// Leader active, no submenu open yet.
    LeaderRoot,
    /// Leader active with a submenu open (`'a'` actions, `'t'` toggles).
    LeaderSubmenu(char),
    /// Instruction / DecisionInstruction text input.
    Input,
}

/// A concrete key pattern: code plus required modifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeySpec {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

const fn plain(code: KeyCode) -> KeySpec {
    KeySpec {
        code,
        modifiers: KeyModifiers::NONE,
    }
}

const fn ctrl(code: KeyCode) -> KeySpec {
    KeySpec {
        code,
        modifiers: KeyModifiers::CONTROL,
    }
}

const fn shift(code: KeyCode) -> KeySpec {
    KeySpec {
        code,
        modifiers: KeyModifiers::SHIFT,
    }
}

/// One binding: keys (aliases) → event, scoped, with display metadata.
#[derive(Debug)]
pub struct Binding {
    pub scope: BindingScope,
    /// Key aliases that trigger this binding.
    pub keys: &'static [KeySpec],
    /// The event produced (cloned on dispatch).
    pub event: UiEvent,
    /// Display strings in input-sequence notation, aligned with `keys`.
    pub notation: &'static [&'static str],
    /// What the binding does — shown in overlays, manifest, affordances.
    pub description: &'static str,
}

/// Doc-only entry for behavior handled by coded fallbacks.
#[derive(Debug)]
pub struct CatchAllDoc {
    pub keys_label: &'static str,
    pub event_label: &'static str,
    pub description: &'static str,
}

/// Leader submenu metadata for overlay titles.
#[derive(Debug)]
pub struct SubmenuDoc {
    pub key: char,
    pub title: &'static str,
}

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

/// The registry. Order matters only for readability; `lookup` rejects
/// duplicate matches by construction (see the invariant test).
pub static BINDINGS: &[Binding] = &[
    // ------------------------------------------------------------------
    // Navigation
    // ------------------------------------------------------------------
    Binding {
        scope: BindingScope::Navigation,
        keys: &[plain(KeyCode::Char('q')), ctrl(KeyCode::Char('c'))],
        event: UiEvent::Quit,
        notation: &["q", "<C-c>"],
        description: "Quit the application",
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: &[shift(KeyCode::Char('?'))],
        event: UiEvent::ToggleHelp,
        notation: &["?"],
        description: "Toggle this help overlay",
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: &[plain(KeyCode::Char(' '))],
        event: UiEvent::ActivateLeader,
        notation: &["<Space>"],
        description: "Open the leader menu",
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: &[plain(KeyCode::Esc)],
        event: UiEvent::Back,
        notation: &["<Esc>"],
        description: "Back to Browse",
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: &[plain(KeyCode::Char('h')), plain(KeyCode::Left)],
        event: UiEvent::NavigateLeft,
        notation: &["h", "<Left>"],
        description: "Drill: switch to previous sibling file",
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: &[plain(KeyCode::Char('j')), plain(KeyCode::Down)],
        event: UiEvent::NavigateDown,
        notation: &["j", "<Down>"],
        description: "Move down (Browse: decisions, Drill: chunks)",
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: &[plain(KeyCode::Char('k')), plain(KeyCode::Up)],
        event: UiEvent::NavigateUp,
        notation: &["k", "<Up>"],
        description: "Move up (Browse: decisions, Drill: chunks)",
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: &[plain(KeyCode::Char('l')), plain(KeyCode::Right)],
        event: UiEvent::NavigateRight,
        notation: &["l", "<Right>"],
        description: "Drill: switch to next sibling file",
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: &[plain(KeyCode::Char('g'))],
        event: UiEvent::NavigateToTop,
        notation: &["g"],
        description: "Jump to first",
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: &[shift(KeyCode::Char('G'))],
        event: UiEvent::NavigateToBottom,
        notation: &["G"],
        description: "Jump to last",
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: &[ctrl(KeyCode::Char('u')), plain(KeyCode::PageUp)],
        event: UiEvent::NavigatePageUp,
        notation: &["<C-u>", "<PageUp>"],
        description: "Page up",
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: &[ctrl(KeyCode::Char('d')), plain(KeyCode::PageDown)],
        event: UiEvent::NavigatePageDown,
        notation: &["<C-d>", "<PageDown>"],
        description: "Page down",
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: &[plain(KeyCode::Tab)],
        event: UiEvent::ToggleChunkContext,
        notation: &["<Tab>"],
        description: "Expand/collapse context lines of the chunk",
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: &[plain(KeyCode::Char('i'))],
        event: UiEvent::ToggleNoteExpansion,
        notation: &["i"],
        description: "Expand/collapse the chunk's note",
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: &[plain(KeyCode::Enter)],
        event: UiEvent::SelectCurrent,
        notation: &["<Enter>"],
        description: "Drill into the focused decision",
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: &[plain(KeyCode::Char('a'))],
        event: UiEvent::ToggleApprove,
        notation: &["a"],
        description: "Toggle approve (decision or chunk)",
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: &[plain(KeyCode::Char('n'))],
        event: UiEvent::EnterInstructionMode,
        notation: &["n"],
        description: "Add or append a note",
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
    },
    Binding {
        scope: BindingScope::LeaderRoot,
        keys: &[plain(KeyCode::Char('t'))],
        event: UiEvent::EnterLeaderSubmenu('t'),
        notation: &["t"],
        description: "Toggles",
    },
    Binding {
        scope: BindingScope::LeaderRoot,
        keys: &[plain(KeyCode::Esc)],
        event: UiEvent::DeactivateLeader,
        notation: &["<Esc>"],
        description: "Cancel",
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
    },
    Binding {
        scope: BindingScope::LeaderSubmenu('a'),
        keys: &[plain(KeyCode::Char('f'))],
        event: UiEvent::ApproveFile,
        notation: &["f"],
        description: "Approve all chunks in the file",
    },
    Binding {
        scope: BindingScope::LeaderSubmenu('a'),
        keys: &[plain(KeyCode::Esc)],
        event: UiEvent::DeactivateLeader,
        notation: &["<Esc>"],
        description: "Back",
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
    },
    Binding {
        scope: BindingScope::LeaderSubmenu('t'),
        keys: &[plain(KeyCode::Esc)],
        event: UiEvent::DeactivateLeader,
        notation: &["<Esc>"],
        description: "Back",
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
    },
    Binding {
        scope: BindingScope::Input,
        keys: &[plain(KeyCode::Enter)],
        event: UiEvent::SubmitInput,
        notation: &["<Enter>"],
        description: "Submit the note",
    },
    Binding {
        scope: BindingScope::Input,
        keys: &[plain(KeyCode::Backspace)],
        event: UiEvent::DeleteChar,
        notation: &["<Backspace>"],
        description: "Delete the character before the cursor",
    },
    Binding {
        scope: BindingScope::Input,
        keys: &[plain(KeyCode::Delete)],
        event: UiEvent::DeleteForward,
        notation: &["<Delete>"],
        description: "Delete the character under the cursor",
    },
    Binding {
        scope: BindingScope::Input,
        keys: &[plain(KeyCode::Left)],
        event: UiEvent::MoveCursorLeft,
        notation: &["<Left>"],
        description: "Move the cursor left",
    },
    Binding {
        scope: BindingScope::Input,
        keys: &[plain(KeyCode::Right)],
        event: UiEvent::MoveCursorRight,
        notation: &["<Right>"],
        description: "Move the cursor right",
    },
    Binding {
        scope: BindingScope::Input,
        keys: &[plain(KeyCode::Home)],
        event: UiEvent::MoveCursorHome,
        notation: &["<Home>"],
        description: "Move the cursor to the start",
    },
    Binding {
        scope: BindingScope::Input,
        keys: &[plain(KeyCode::End)],
        event: UiEvent::MoveCursorEnd,
        notation: &["<End>"],
        description: "Move the cursor to the end",
    },
    Binding {
        scope: BindingScope::Input,
        keys: &[ctrl(KeyCode::Left)],
        event: UiEvent::MoveCursorWordLeft,
        notation: &["<C-Left>"],
        description: "Move the cursor one word left",
    },
    Binding {
        scope: BindingScope::Input,
        keys: &[ctrl(KeyCode::Right)],
        event: UiEvent::MoveCursorWordRight,
        notation: &["<C-Right>"],
        description: "Move the cursor one word right",
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

/// Find the binding for a key in a scope.
///
/// Navigation/Input match code AND modifiers exactly; leader scopes match
/// code only (characterized behavior — a modifier held during a leader
/// chord is ignored).
pub fn lookup(scope: BindingScope, key: KeyEvent) -> Option<&'static Binding> {
    BINDINGS.iter().find(|binding| {
        binding.scope == scope
            && binding.keys.iter().any(|spec| match scope {
                BindingScope::Navigation | BindingScope::Input => {
                    spec.code == key.code && spec.modifiers == key.modifiers
                }
                BindingScope::LeaderRoot | BindingScope::LeaderSubmenu(_) => spec.code == key.code,
            })
    })
}

/// Rows for one scope, in declaration order (overlays and affordances).
pub fn bindings_for(scope: BindingScope) -> impl Iterator<Item = &'static Binding> {
    BINDINGS.iter().filter(move |b| b.scope == scope)
}

/// Doc-only catch-all for a scope, if its fallback behavior exists.
pub fn catch_all_for(scope: BindingScope) -> Option<&'static CatchAllDoc> {
    match scope {
        BindingScope::Navigation => None,
        BindingScope::LeaderRoot | BindingScope::LeaderSubmenu(_) => Some(&LEADER_CATCH_ALL),
        BindingScope::Input => Some(&INPUT_CATCH_ALL),
    }
}

/// Manifest label for a scope.
pub fn scope_label(scope: BindingScope) -> String {
    match scope {
        BindingScope::Navigation => "Navigation".to_string(),
        BindingScope::LeaderRoot => "Leader".to_string(),
        BindingScope::LeaderSubmenu(c) => format!("Leader:{c}"),
        BindingScope::Input => "Input".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every (scope, KeySpec) pair must resolve to at most one binding.
    #[test]
    fn no_duplicate_key_rows() {
        for (i, a) in BINDINGS.iter().enumerate() {
            for b in BINDINGS.iter().skip(i + 1) {
                if a.scope != b.scope {
                    continue;
                }
                for spec in a.keys {
                    assert!(
                        !b.keys.contains(spec),
                        "duplicate binding for {spec:?} in {:?}: {:?} and {:?}",
                        a.scope,
                        a.event,
                        b.event
                    );
                }
            }
        }
    }

    /// Overlays, manifest, and affordances render these — none may be empty
    /// or misaligned.
    #[test]
    fn rows_are_fully_documented() {
        for binding in BINDINGS {
            assert!(
                !binding.description.is_empty(),
                "{:?} has no description",
                binding.event
            );
            assert_eq!(
                binding.keys.len(),
                binding.notation.len(),
                "{:?}: notation must align with keys",
                binding.event
            );
        }
    }

    /// Every submenu referenced by a binding row has a SubmenuDoc title.
    #[test]
    fn submenus_are_documented() {
        for binding in BINDINGS {
            if let BindingScope::LeaderSubmenu(c) = binding.scope {
                assert!(
                    SUBMENUS.iter().any(|s| s.key == c),
                    "submenu '{c}' has bindings but no SubmenuDoc"
                );
            }
        }
        for submenu in SUBMENUS {
            assert!(
                BINDINGS
                    .iter()
                    .any(|b| b.scope == BindingScope::LeaderSubmenu(submenu.key)),
                "SubmenuDoc '{}' has no bindings",
                submenu.key
            );
            assert!(
                BINDINGS.iter().any(|b| b.scope == BindingScope::LeaderRoot
                    && b.event == UiEvent::EnterLeaderSubmenu(submenu.key)),
                "submenu '{}' is not reachable from the leader root",
                submenu.key
            );
        }
    }
}
