//! Event handling system for the TUI
//!
//! This module provides a clean separation between UI events (navigation, input)
//! and business events (review operations that need ReviewEngine handling).

pub mod business;
pub mod input;

pub use business::{ui_event_to_business_event, BusinessEvent};
pub use input::{handle_key_event, UiEvent};
