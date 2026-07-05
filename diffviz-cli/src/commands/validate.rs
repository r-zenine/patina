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

impl ValidateCommand {
    pub fn run(&self) -> Result<()> {
        match self.artifact.as_str() {
            "decision-log" => {
                let path = Path::new(&self.file);
                if !path.exists() {
                    return Err(anyhow::anyhow!("File not found: {}", self.file));
                }

                let content = fs::read_to_string(path)
                    .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", self.file, e))?;

                match DecisionLog::parse(&content) {
                    Ok(log) => {
                        let mut has_errors = false;

                        let violations = log.reasoning_convention_violations();
                        if !violations.is_empty() {
                            has_errors = true;
                            eprintln!(
                                "✗ {} has {} code impact(s) violating the reasoning convention:\n",
                                self.file,
                                violations.len()
                            );
                            for v in &violations {
                                eprintln!(
                                    "  decision #{} ({}) → {}",
                                    v.decision_number, v.decision_title, v.file
                                );
                                eprintln!("    reasoning: {}\n", v.reasoning);
                            }
                            eprintln!(
                                "Every code impact reasoning must start with a critical-tier prefix:"
                            );
                            eprintln!("  [Behavioral - <kind>] ...  or  [Structural - <kind>] ...");
                            eprintln!(
                                "and state the risk or contract change a reviewer must verify."
                            );
                            eprintln!(
                                "If a change is mechanical ripple (import updates, call-site renames,"
                            );
                            eprintln!(
                                "moved code), omit it from code_impacts entirely. See the dev-contribute"
                            );
                            eprintln!("guide, section \"Identifying Code Impacts\".\n");
                        }

                        let repo_start = path.parent().unwrap_or(Path::new("."));
                        match gitkit::GitRepository::discover(repo_start) {
                            Ok(repo) => {
                                if let Some(v) = log.commit_violation(&repo) {
                                    has_errors = true;
                                    eprintln!(
                                        "✗ {} has an unverifiable commit: \"{}\"",
                                        self.file, v.commit
                                    );
                                    eprintln!("    {}\n", v.reason);
                                    eprintln!(
                                        "Implementation logs (any decision with code_impacts) must record the"
                                    );
                                    eprintln!(
                                        "real git hash of the commit containing the code changes — get it via"
                                    );
                                    eprintln!("  git rev-parse HEAD\n");
                                }
                            }
                            Err(e) => {
                                eprintln!(
                                    "⚠ could not open a git repository near {} to verify the commit field: {e}",
                                    self.file
                                );
                            }
                        }

                        if has_errors {
                            return Err(anyhow::anyhow!(
                                "{} failed decision-log validation",
                                self.file
                            ));
                        }

                        println!("✓ {} is valid", self.file);
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("✗ {} is invalid", self.file);
                        eprintln!("\nValidation error: {e}");
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

impl CommandExecutor for ValidateCommand {
    fn execute(&self, _env: Environment) -> Result<()> {
        self.run()
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
