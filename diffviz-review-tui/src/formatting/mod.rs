//! RenderableDiff formatting for TUI display
//!
//! This module handles the conversion from ReviewableDiff to RenderableDiff
//! and then to ratatui display components, providing clean git-diff style
//! formatting with syntax highlighting support.

pub mod diff_formatter;

pub use diff_formatter::{FormattedDiff, TuiDiffFormatter};
