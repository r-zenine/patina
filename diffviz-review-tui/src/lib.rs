//! Clean TUI for code review built on ReviewEngine architecture
//!
//! This crate provides a terminal user interface for reviewing code changes
//! using the ReviewEngine business logic and RenderableDiff display system.
//! It's designed from the ground up for ReviewableDiffId navigation and
//! clean separation between UI and business logic.

pub mod app;
pub mod command;
pub mod decision_navigation;
pub mod diff;
pub mod events;
pub mod formatting;
pub mod navigation;
pub mod state;
pub mod theme;
pub mod ui;

#[cfg(feature = "test-harness")]
pub mod test_harness;

// Re-export main types for easy access
pub use app::ReviewTuiApp;
pub use decision_navigation::DecisionNavigationTree;
pub use state::{FocusPanel, InputMode, UiState};

#[cfg(feature = "test-harness")]
pub use app::HeadlessApp;
#[cfg(feature = "test-harness")]
pub use test_harness::{InputTestHarness, StateSnapshot};

/// Result type used throughout the TUI
pub type Result<T> = anyhow::Result<T>;
