//! Review subcommand implementation (deprecated)
//!
//! This command has been deprecated. The system now uses decision-driven review only.

use anyhow::Result;

use super::CommandExecutor;
use crate::environment::Environment;

/// Review subcommand for launching the interactive TUI (deprecated)
pub struct ReviewCommand;

impl ReviewCommand {
    /// Create a new Review command
    pub fn new(
        _file_filter: Option<String>,
        _from_commit: Option<String>,
        _to_commit: Option<String>,
    ) -> Self {
        Self
    }
}

impl CommandExecutor for ReviewCommand {
    fn execute(&self, _environment: Environment) -> Result<()> {
        // The old git-based review workflow has been deprecated.
        // The system now uses decision-driven review only.
        Err(anyhow::anyhow!(
            "The 'review' command has been deprecated. The review system now uses decision-driven workflows only.\n\
            Please use the TUI with architectural decisions instead of git-based discovery."
        ))
    }
}
