//! Clean TUI for code review built on ReviewEngine architecture
//!
//! This crate provides a terminal user interface for reviewing code changes
//! using the ReviewEngine business logic and RenderableDiff display system.
//! It's designed from the ground up for ReviewableDiffId navigation and
//! clean separation between UI and business logic.

pub mod app;
pub mod command;
pub mod error;
pub mod events;
pub mod state;
pub mod state_snapshot;
pub mod theme_ext;
pub mod ui;

#[cfg(feature = "test-harness")]
pub mod test_harness;

// Re-export main types for easy access
pub use app::ReviewTuiApp;
pub use error::ReviewTuiError;
pub use state::{InputMode, UiState};
pub use state_snapshot::StateSnapshot;

#[cfg(feature = "test-harness")]
pub use test_harness::InputTestHarness;

pub(crate) type Result<T> = anyhow::Result<T>;
