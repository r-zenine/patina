//! Command system for side effects
//!
//! In ELM architecture, side effects are modeled as Commands that are executed
//! after state updates. This module defines the Command type and execution logic
//! to isolate I/O operations (file writes, notifications) from update logic.

use crate::Result;

/// Command represents a side effect to be executed
///
/// Commands describe what side effects should happen, but the actual execution
/// is deferred to the command executor. This allows update logic to be pure and
/// testable while still supporting necessary I/O operations.
#[derive(Debug, Clone)]
pub enum Command {
    /// Write content to a file
    WriteFile { path: String, content: String },

    /// Show a message to the user
    ShowMessage { message: String },

    /// Batch multiple commands together
    Batch(Vec<Command>),

    /// No operation - do nothing
    None,
}

/// Execute a command, performing its side effects
///
/// This function takes a Command and executes it. Most commands involve I/O
/// operations that can fail, so this returns a Result. Errors are propagated
/// to the caller for proper error handling.
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
