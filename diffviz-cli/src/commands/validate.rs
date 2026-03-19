//! Validate command for checking decision-log YAML files
//!
//! Validates that a YAML file conforms to the decision-log schema.
//! Uses DecisionLog::parse() for validation, reusing the same logic
//! that the main CLI uses when processing decision-log files.

use crate::commands::CommandExecutor;
use crate::environment::Environment;
use anyhow::Result;
use diffviz_review::DecisionLog;
use std::fs;
use std::path::Path;

/// Handler for the `diffviz validate` command
pub struct ValidateCommand {
    artifact: String,
    file: String,
}

impl ValidateCommand {
    pub fn new(artifact: String, file: String) -> Self {
        Self { artifact, file }
    }
}

impl CommandExecutor for ValidateCommand {
    fn execute(&self, _env: Environment) -> Result<()> {
        match self.artifact.as_str() {
            "decision-log" => {
                let path = Path::new(&self.file);
                if !path.exists() {
                    return Err(anyhow::anyhow!("File not found: {}", self.file));
                }

                let content = fs::read_to_string(path)
                    .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", self.file, e))?;

                match DecisionLog::parse(&content) {
                    Ok(_) => {
                        println!("✓ {} is valid", self.file);
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("✗ {} is invalid", self.file);
                        eprintln!("\nValidation error: {}", e);
                        eprintln!("\nExpected schema:");
                        eprintln!("  Run: diffviz templates decision-log");
                        eprintln!("  to see the current schema.\n");
                        Err(e.into())
                    }
                }
            }
            _ => Err(anyhow::anyhow!(
                "Unknown artifact type: '{}'. Supported: decision-log",
                self.artifact
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_command_decision_log() {
        let cmd = ValidateCommand::new("decision-log".to_string(), "test.yaml".to_string());
        assert_eq!(cmd.artifact, "decision-log");
        assert_eq!(cmd.file, "test.yaml");
    }

    #[test]
    fn validate_command_unknown_artifact() {
        let cmd = ValidateCommand::new("unknown".to_string(), "test.yaml".to_string());
        assert_eq!(cmd.artifact, "unknown");
    }
}
