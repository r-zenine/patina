//! Command system for side effects.
//!
//! Verdict persistence (`Baseline::record_verdict`) is a synchronous domain
//! operation on the triage baseline, not I/O the update logic needs to
//! defer — it's the same "accepted pragmatic compromise" `diffviz-review-tui`
//! documents for `self.engine.approve()`. The `Command` type still exists so
//! any future side effect (e.g. exporting findings to a file) has a place to
//! go without ad-hoc I/O in a handler.

use crate::tui::Result;

#[derive(Debug, Clone)]
pub enum Command {
    /// Write content to a file.
    WriteFile { path: String, content: String },

    /// Show a message to the user.
    ShowMessage { message: String },

    /// Batch multiple commands together.
    Batch(Vec<Command>),

    /// No operation - do nothing.
    None,
}

/// Execute a command, performing its side effects.
pub fn execute_command(command: Command) -> Result<()> {
    match command {
        Command::WriteFile { path, content } => {
            std::fs::write(&path, content)?;
            Ok(())
        }
        Command::ShowMessage { message } => {
            eprintln!("{message}");
            Ok(())
        }
        Command::Batch(commands) => {
            for cmd in commands {
                execute_command(cmd)?;
            }
            Ok(())
        }
        Command::None => Ok(()),
    }
}
