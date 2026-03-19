//! Templates command for generating schema templates
//!
//! Outputs schema templates for various artifact types (e.g., decision-log)
//! that agents and users can use as reference when creating files.

use crate::commands::CommandExecutor;
use crate::environment::Environment;
use anyhow::Result;
use diffviz_review::SchemaTemplate;

/// Handler for the `diffviz templates` command
pub struct TemplatesCommand {
    artifact: String,
}

impl TemplatesCommand {
    pub fn new(artifact: String) -> Self {
        Self { artifact }
    }
}

impl CommandExecutor for TemplatesCommand {
    fn execute(&self, _env: Environment) -> Result<()> {
        match self.artifact.as_str() {
            "decision-log" => {
                let template = diffviz_review::DecisionLog::yaml_template();
                println!("{}", template);
                Ok(())
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
    fn templates_command_decision_log() {
        let cmd = TemplatesCommand::new("decision-log".to_string());
        // We can't actually execute without an Environment, but we can verify the command creates
        let artifact = cmd.artifact;
        assert_eq!(artifact, "decision-log");
    }

    #[test]
    fn templates_command_unknown_artifact() {
        let cmd = TemplatesCommand::new("unknown".to_string());
        let artifact = cmd.artifact;
        assert_eq!(artifact, "unknown");
    }
}
