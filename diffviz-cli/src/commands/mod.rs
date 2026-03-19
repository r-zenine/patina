//! CLI subcommand implementations
//!
//! This module contains the implementation for each diffviz CLI subcommand,
//! following clean separation of concerns and the Environment pattern.

pub mod debug;
pub mod review;

use crate::environment::Environment;
use anyhow::Result;

/// Common functionality shared across commands
pub trait CommandExecutor {
    /// Execute the command with the given environment
    fn execute(&self, environment: Environment) -> Result<()>;
}
