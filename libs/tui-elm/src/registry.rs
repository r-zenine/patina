//! The keybinding registry: one declarative table drives key dispatch, the
//! which-key and help overlays, the `--describe` manifest, and per-state
//! affordances. Adding a binding row is the whole job; every consumer picks
//! it up, so none of them can drift from dispatch.
//!
//! Matching semantics are per-scope (see [`Registry::lookup`]):
//! - [`BindingScope::Navigation`] / [`BindingScope::Input`] match key code
//!   AND modifiers exactly
//! - Leader scopes match key code only (a modifier held during a leader
//!   chord is ignored), and any key without a row dismisses the leader
//!
//! Truly parametric behavior (typing text) cannot be a finite row; dispatch
//! surfaces it as [`KeyDispatch::TextChar`], and the registry's catch-all
//! docs keep the manifest and affordances honest about it.

use std::fmt;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui_harness::{Affordance, KeyBindingDoc};

/// Where a binding is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingScope {
    /// Modal navigation with the leader inactive.
    Navigation,
    /// Leader active, no submenu open yet.
    LeaderRoot,
    /// Leader active with a submenu open.
    LeaderSubmenu(char),
    /// Text input.
    Input,
}

impl BindingScope {
    /// Manifest label for the scope.
    pub fn label(self) -> String {
        match self {
            BindingScope::Navigation => "Navigation".to_string(),
            BindingScope::LeaderRoot => "Leader".to_string(),
            BindingScope::LeaderSubmenu(c) => format!("Leader:{c}"),
            BindingScope::Input => "Input".to_string(),
        }
    }
}

/// A concrete key pattern: code plus required modifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeySpec {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

/// A key with no modifiers.
pub const fn plain(code: KeyCode) -> KeySpec {
    KeySpec {
        code,
        modifiers: KeyModifiers::NONE,
    }
}

/// A key with Ctrl held.
pub const fn ctrl(code: KeyCode) -> KeySpec {
    KeySpec {
        code,
        modifiers: KeyModifiers::CONTROL,
    }
}

/// A key with Shift held.
pub const fn shift(code: KeyCode) -> KeySpec {
    KeySpec {
        code,
        modifiers: KeyModifiers::SHIFT,
    }
}

/// How overlays and validation treat a row, beyond dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BindingRole {
    /// Ordinary action row.
    #[default]
    Action,
    /// Dismisses the leader menu — overlays render it as an Esc footer
    /// ("cancel"/"back") instead of a menu item.
    DismissLeader,
    /// Opens the named leader submenu — validation uses it to prove every
    /// documented submenu is reachable from the leader root.
    EnterSubmenu(char),
}

/// One binding: keys (aliases) → event, scoped, with display metadata.
#[derive(Debug)]
pub struct Binding<E: 'static> {
    pub scope: BindingScope,
    /// Key aliases that trigger this binding.
    pub keys: &'static [KeySpec],
    /// The event produced (borrowed on dispatch; apps typically clone it).
    pub event: E,
    /// Display strings in input-sequence notation, aligned with `keys`.
    pub notation: &'static [&'static str],
    /// What the binding does — shown in overlays, manifest, affordances.
    pub description: &'static str,
    /// Overlay/validation role of the row.
    pub role: BindingRole,
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

/// Outcome of dispatching one key event against the registry.
#[derive(Debug)]
pub enum KeyDispatch<E: 'static> {
    /// A registry row matched.
    Bound(&'static Binding<E>),
    /// Leader scope, no row: the leader dismisses.
    DismissLeader,
    /// Input scope, no row, plain or shifted character: text input.
    TextChar(char),
}

/// An app's complete keybinding registry.
///
/// Built as a `static` from `static` parts; all methods borrow with
/// `'static` lifetimes so dispatched bindings can outlive the call site.
pub struct Registry<E: 'static> {
    pub bindings: &'static [Binding<E>],
    pub submenus: &'static [SubmenuDoc],
    /// Display label of the leader key in overlay titles (e.g. "Space").
    pub leader_label: &'static str,
    /// Fallback doc for text typed in [`BindingScope::Input`]; `None` for
    /// apps without a text-input scope.
    pub input_catch_all: Option<&'static CatchAllDoc>,
    /// Fallback doc for unbound keys in leader scopes.
    pub leader_catch_all: Option<&'static CatchAllDoc>,
}

impl<E> Registry<E> {
    /// Find the binding for a key in a scope.
    ///
    /// Navigation/Input match code AND modifiers exactly; leader scopes
    /// match code only (a modifier held during a leader chord is ignored).
    pub fn lookup(&self, scope: BindingScope, key: KeyEvent) -> Option<&'static Binding<E>> {
        self.bindings.iter().find(|binding| {
            binding.scope == scope
                && binding.keys.iter().any(|spec| match scope {
                    BindingScope::Navigation | BindingScope::Input => {
                        spec.code == key.code && spec.modifiers == key.modifiers
                    }
                    BindingScope::LeaderRoot | BindingScope::LeaderSubmenu(_) => {
                        spec.code == key.code
                    }
                })
        })
    }

    /// Dispatch one key event: registry lookup plus the two coded fallbacks
    /// that cannot be finite table rows (documented by the catch-all docs):
    /// - leader scopes: any key without a row dismisses the leader
    /// - input scope: any plain/shifted character becomes text input
    pub fn dispatch(&self, scope: BindingScope, key: KeyEvent) -> Option<KeyDispatch<E>> {
        if let Some(binding) = self.lookup(scope, key) {
            return Some(KeyDispatch::Bound(binding));
        }

        match scope {
            BindingScope::LeaderRoot | BindingScope::LeaderSubmenu(_) => {
                Some(KeyDispatch::DismissLeader)
            }
            BindingScope::Input => match key {
                KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                    ..
                } => Some(KeyDispatch::TextChar(c)),
                _ => None,
            },
            BindingScope::Navigation => None,
        }
    }

    /// Rows for one scope, in declaration order (overlays and affordances).
    pub fn bindings_for(&self, scope: BindingScope) -> impl Iterator<Item = &'static Binding<E>> {
        self.bindings.iter().filter(move |b| b.scope == scope)
    }

    /// Doc-only catch-all for a scope, if its fallback behavior exists.
    pub fn catch_all_for(&self, scope: BindingScope) -> Option<&'static CatchAllDoc> {
        match scope {
            BindingScope::Navigation => None,
            BindingScope::LeaderRoot | BindingScope::LeaderSubmenu(_) => self.leader_catch_all,
            BindingScope::Input => self.input_catch_all,
        }
    }
}

impl<E: fmt::Debug> Registry<E> {
    /// Every registry row plus the catch-all fallbacks, as manifest docs
    /// for `ELMApp::describe`.
    pub fn binding_docs(&self) -> Vec<KeyBindingDoc> {
        let mut docs: Vec<KeyBindingDoc> = self
            .bindings
            .iter()
            .map(|binding| KeyBindingDoc {
                mode: binding.scope.label(),
                keys: binding.notation.iter().map(|s| s.to_string()).collect(),
                event: format!("{:?}", binding.event),
                description: binding.description.to_string(),
            })
            .collect();

        for scope in [BindingScope::LeaderRoot, BindingScope::Input] {
            if let Some(catch_all) = self.catch_all_for(scope) {
                docs.push(KeyBindingDoc {
                    mode: scope.label(),
                    keys: vec![catch_all.keys_label.to_string()],
                    event: catch_all.event_label.to_string(),
                    description: catch_all.description.to_string(),
                });
            }
        }

        docs
    }

    /// Keys meaningful in `scope`, for `ELMApp::affordances`.
    pub fn affordances(&self, scope: BindingScope) -> Vec<Affordance> {
        let mut affordances: Vec<Affordance> = self
            .bindings_for(scope)
            .map(|binding| Affordance {
                keys: binding.notation.iter().map(|s| s.to_string()).collect(),
                event: format!("{:?}", binding.event),
                description: binding.description.to_string(),
            })
            .collect();

        if let Some(catch_all) = self.catch_all_for(scope) {
            affordances.push(Affordance {
                keys: vec![catch_all.keys_label.to_string()],
                event: catch_all.event_label.to_string(),
                description: catch_all.description.to_string(),
            });
        }

        affordances
    }

    /// Assert the registry's structural invariants; call from an app test.
    ///
    /// Panics with a descriptive message when:
    /// - a (scope, key) pair resolves to more than one row
    /// - a row has an empty description or misaligned notation
    /// - a submenu has rows but no [`SubmenuDoc`], a [`SubmenuDoc`] has no
    ///   rows, or a submenu is not reachable from the leader root via a
    ///   [`BindingRole::EnterSubmenu`] row
    pub fn validate(&self) {
        for (i, a) in self.bindings.iter().enumerate() {
            for b in self.bindings.iter().skip(i + 1) {
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

        for binding in self.bindings {
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

        for binding in self.bindings {
            if let BindingScope::LeaderSubmenu(c) = binding.scope {
                assert!(
                    self.submenus.iter().any(|s| s.key == c),
                    "submenu '{c}' has bindings but no SubmenuDoc"
                );
            }
        }
        for submenu in self.submenus {
            assert!(
                self.bindings
                    .iter()
                    .any(|b| b.scope == BindingScope::LeaderSubmenu(submenu.key)),
                "SubmenuDoc '{}' has no bindings",
                submenu.key
            );
            assert!(
                self.bindings
                    .iter()
                    .any(|b| b.scope == BindingScope::LeaderRoot
                        && b.role == BindingRole::EnterSubmenu(submenu.key)),
                "submenu '{}' is not reachable from the leader root",
                submenu.key
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum TestEvent {
        Up,
        OpenMenu,
        Pick,
        CloseMenu,
        Submit,
    }

    static CATCH_ALL: CatchAllDoc = CatchAllDoc {
        keys_label: "<any character>",
        event_label: "Char",
        description: "Type the character",
    };

    static ROWS: &[Binding<TestEvent>] = &[
        Binding {
            scope: BindingScope::Navigation,
            keys: &[plain(KeyCode::Char('k')), plain(KeyCode::Up)],
            event: TestEvent::Up,
            notation: &["k", "<Up>"],
            description: "Move up",
            role: BindingRole::Action,
        },
        Binding {
            scope: BindingScope::LeaderRoot,
            keys: &[plain(KeyCode::Char('m'))],
            event: TestEvent::OpenMenu,
            notation: &["m"],
            description: "Menu",
            role: BindingRole::EnterSubmenu('m'),
        },
        Binding {
            scope: BindingScope::LeaderSubmenu('m'),
            keys: &[plain(KeyCode::Char('p'))],
            event: TestEvent::Pick,
            notation: &["p"],
            description: "Pick",
            role: BindingRole::Action,
        },
        Binding {
            scope: BindingScope::LeaderSubmenu('m'),
            keys: &[plain(KeyCode::Esc)],
            event: TestEvent::CloseMenu,
            notation: &["<Esc>"],
            description: "Back",
            role: BindingRole::DismissLeader,
        },
        Binding {
            scope: BindingScope::Input,
            keys: &[plain(KeyCode::Enter)],
            event: TestEvent::Submit,
            notation: &["<Enter>"],
            description: "Submit",
            role: BindingRole::Action,
        },
    ];

    static REGISTRY: Registry<TestEvent> = Registry {
        bindings: ROWS,
        submenus: &[SubmenuDoc {
            key: 'm',
            title: "Menu",
        }],
        leader_label: "Space",
        input_catch_all: Some(&CATCH_ALL),
        leader_catch_all: None,
    };

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn navigation_matches_modifiers_exactly() {
        assert!(
            REGISTRY
                .lookup(BindingScope::Navigation, key(KeyCode::Char('k')))
                .is_some()
        );
        assert!(
            REGISTRY
                .lookup(
                    BindingScope::Navigation,
                    KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL)
                )
                .is_none()
        );
    }

    #[test]
    fn leader_scopes_ignore_modifiers() {
        let ctrl_m = KeyEvent::new(KeyCode::Char('m'), KeyModifiers::CONTROL);
        let binding = REGISTRY.lookup(BindingScope::LeaderRoot, ctrl_m).unwrap();
        assert_eq!(binding.event, TestEvent::OpenMenu);
    }

    #[test]
    fn unbound_leader_key_dismisses() {
        match REGISTRY.dispatch(BindingScope::LeaderRoot, key(KeyCode::Char('z'))) {
            Some(KeyDispatch::DismissLeader) => {}
            other => panic!("expected DismissLeader, got {other:?}"),
        }
    }

    #[test]
    fn unbound_input_char_becomes_text() {
        match REGISTRY.dispatch(BindingScope::Input, key(KeyCode::Char('x'))) {
            Some(KeyDispatch::TextChar('x')) => {}
            other => panic!("expected TextChar('x'), got {other:?}"),
        }
        assert!(
            REGISTRY
                .dispatch(
                    BindingScope::Input,
                    KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL)
                )
                .is_none()
        );
    }

    #[test]
    fn unbound_navigation_key_is_none() {
        assert!(
            REGISTRY
                .dispatch(BindingScope::Navigation, key(KeyCode::Char('z')))
                .is_none()
        );
    }

    #[test]
    fn docs_cover_rows_and_catch_alls() {
        let docs = REGISTRY.binding_docs();
        assert_eq!(docs.len(), ROWS.len() + 1);
        let last = docs.last().unwrap();
        assert_eq!(last.mode, "Input");
        assert_eq!(last.event, "Char");

        let affordances = REGISTRY.affordances(BindingScope::LeaderSubmenu('m'));
        assert_eq!(affordances.len(), 2);
        assert_eq!(affordances[0].event, "Pick");
    }

    #[test]
    fn registry_invariants_hold() {
        REGISTRY.validate();
    }

    #[test]
    #[should_panic(expected = "duplicate binding")]
    fn validate_rejects_duplicate_keys() {
        static DUP: &[Binding<TestEvent>] = &[
            Binding {
                scope: BindingScope::Navigation,
                keys: &[plain(KeyCode::Char('k'))],
                event: TestEvent::Up,
                notation: &["k"],
                description: "Move up",
                role: BindingRole::Action,
            },
            Binding {
                scope: BindingScope::Navigation,
                keys: &[plain(KeyCode::Char('k'))],
                event: TestEvent::Submit,
                notation: &["k"],
                description: "Submit",
                role: BindingRole::Action,
            },
        ];
        let registry = Registry {
            bindings: DUP,
            submenus: &[],
            leader_label: "Space",
            input_catch_all: None,
            leader_catch_all: None,
        };
        registry.validate();
    }

    #[test]
    #[should_panic(expected = "not reachable")]
    fn validate_rejects_unreachable_submenu() {
        static ORPHAN: &[Binding<TestEvent>] = &[Binding {
            scope: BindingScope::LeaderSubmenu('m'),
            keys: &[plain(KeyCode::Char('p'))],
            event: TestEvent::Pick,
            notation: &["p"],
            description: "Pick",
            role: BindingRole::Action,
        }];
        let registry = Registry {
            bindings: ORPHAN,
            submenus: &[SubmenuDoc {
                key: 'm',
                title: "Menu",
            }],
            leader_label: "Space",
            input_catch_all: None,
            leader_catch_all: None,
        };
        registry.validate();
    }
}
