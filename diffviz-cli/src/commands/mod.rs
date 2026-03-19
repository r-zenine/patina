//! CLI subcommand implementations
//!
//! This module contains the implementation for each diffviz CLI subcommand,
//! following clean separation of concerns and the Environment pattern.

pub mod debug;
pub mod review;
pub mod templates;
pub mod validate;

use crate::environment::Environment;
use anyhow::Result;

/// Common functionality shared across commands
pub trait CommandExecutor {
    /// Execute the command with the given environment
    fn execute(&self, environment: Environment) -> Result<()>;
}
