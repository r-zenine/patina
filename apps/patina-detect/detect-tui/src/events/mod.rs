//! Event handling system for the triage TUI: UI events (navigation, input)
//! separated from business events (baseline operations).

pub mod bindings;
pub mod business;
pub mod input;

pub use bindings::{BINDINGS, REGISTRY, SUBMENUS};
pub use business::{BusinessEvent, ui_event_to_business_event};
pub use input::{UiEvent, handle_key_event};
pub use tui_elm::{Binding, BindingScope};
