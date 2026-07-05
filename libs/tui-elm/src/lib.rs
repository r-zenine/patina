//! tui-elm — the interaction layer between `tui-harness` (ELM runtime,
//! headless testing, agent discovery) and `tui-design` (theme, stylesheet).
//!
//! An app declares one static [`Registry`] of keybindings; that single
//! table then drives:
//! - key dispatch with per-scope matching semantics ([`Registry::dispatch`])
//! - the which-key leader overlay ([`which_key::render`])
//! - the full help overlay ([`help::render`])
//! - `--describe` manifest docs ([`Registry::binding_docs`])
//! - per-state affordances ([`Registry::affordances`])
//! - structural invariants asserted in tests ([`Registry::validate`])
//!
//! [`LeaderState`] provides the Spacemacs-style leader menu state machine
//! (activation, submenus, display timeout) that the overlay renders.

pub mod help;
pub mod leader;
pub mod registry;
pub mod which_key;

pub use help::HelpText;
pub use leader::LeaderState;
pub use registry::{
    Binding, BindingRole, BindingScope, CatchAllDoc, KeyDispatch, KeySpec, Registry, SubmenuDoc,
    ctrl, plain, shift,
};
