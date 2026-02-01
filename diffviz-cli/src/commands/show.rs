//! Show subcommand implementation (deprecated)
//!
//! This command has been deprecated. The system now uses decision-driven review only.

use anyhow::Result;

use super::CommandExecutor;
use crate::environment::Environment;

/// Show subcommand for displaying file diffs (deprecated)
pub struct ShowCommand;

impl ShowCommand {
    /// Create a new Show command with the specified parameters
    pub fn new(
        _file_path: String,
        _from_commit: Option<String>,
        _to_commit: Option<String>,
        _staged: bool,
        _unstaged: bool,
    ) -> Self {
        Self
    }
}

impl CommandExecutor for ShowCommand {
    fn execute(&self, _environment: Environment) -> Result<()> {
        // The old git-based semantic analysis system has been deprecated.
        // The system now uses decision-driven review only.
        Err(anyhow::anyhow!(
            "The 'show' command has been deprecated. The semantic analysis system now uses decision-driven workflows only.\n\
            Please use the TUI with architectural decisions instead of git-based discovery."
        ))
    }
}
