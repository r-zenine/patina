//! Schema template generation for decision-log artifacts
//!
//! This module provides YAML templates for the decision-log schema.
//! Templates serve as examples for users and agents to understand the expected
//! structure of decision-log files.
//!
//! In Phase 2, this manual implementation will be replaced by a derive macro
//! that auto-generates templates from struct definitions.

use crate::entities::decision::DecisionLog;

/// Trait for exporting schema templates
///
/// Types implementing this trait can generate human-readable YAML templates
/// that show the expected structure and field types for serialization.
pub trait SchemaTemplate {
    /// Generate a YAML template showing the schema structure with placeholders
    fn yaml_template() -> String;
}

impl SchemaTemplate for DecisionLog {
    fn yaml_template() -> String {
        r#"# Decision Log - Schema Template
# Use this file to document architectural decisions made in this contribution.
# See https://github.com/anthropics/patina/tree/main/diffviz-review for detailed explanation.

commit: "git-hash-here"  # Git hash of commit containing these code changes
                         # Required during implementation, optional during strategy phase

decisions:
  # Each decision maps architectural choice to actual code changes
  - number: 1
    title: "[Decision made in one sentence]"
    rationale: "[Why this choice - constraints, priorities, trade-offs]"  # Optional
    code_impacts:
      # One or more files affected by this decision
      - file: "[path/to/file.rs]"
        reasoning: "[Why this file is affected by this decision]"
        line_ranges:
          # One or more line ranges in this file affected
          - start: 10
            end: 50

  - number: 2
    title: "[Next decision]"
    rationale: "[Rationale]"  # Optional
    code_impacts:
      - file: "[another/file.rs]"
        reasoning: "[Why affected]"
        line_ranges:
          - start: 100
            end: 150
"#
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml::Value;

    #[test]
    fn template_parses_as_valid_yaml() {
        let template = DecisionLog::yaml_template();
        let result: Result<Value, _> = serde_yaml::from_str(&template);
        assert!(
            result.is_ok(),
            "Template should parse as valid YAML, but got: {:?}",
            result.err()
        );
    }

    #[test]
    fn template_contains_required_fields() {
        let template = DecisionLog::yaml_template();
        assert!(
            template.contains("commit:"),
            "Template should contain 'commit' field"
        );
        assert!(
            template.contains("decisions:"),
            "Template should contain 'decisions' field"
        );
        assert!(
            template.contains("number:"),
            "Template should contain 'number' field in decision"
        );
        assert!(
            template.contains("title:"),
            "Template should contain 'title' field in decision"
        );
        assert!(
            template.contains("rationale:"),
            "Template should contain 'rationale' field in decision"
        );
        assert!(
            template.contains("code_impacts:"),
            "Template should contain 'code_impacts' field"
        );
        assert!(
            template.contains("file:"),
            "Template should contain 'file' field in code impact"
        );
        assert!(
            template.contains("reasoning:"),
            "Template should contain 'reasoning' field"
        );
        assert!(
            template.contains("line_ranges:"),
            "Template should contain 'line_ranges' field"
        );
        assert!(
            template.contains("start:"),
            "Template should contain 'start' field in line range"
        );
        assert!(
            template.contains("end:"),
            "Template should contain 'end' field in line range"
        );
    }

    #[test]
    fn template_structure_matches_decision_log() {
        let template = DecisionLog::yaml_template();
        // Verify we can parse the template back as a DecisionLog
        // (it has example values so won't fully validate, but should parse structurally)
        let result = serde_yaml::from_str::<serde_yaml::Value>(&template);
        assert!(result.is_ok(), "Template should be parseable as YAML");

        let value = result.unwrap();
        assert!(
            value.get("commit").is_some(),
            "Template should have 'commit' key at root"
        );
        assert!(
            value.get("decisions").is_some(),
            "Template should have 'decisions' key at root"
        );

        let decisions = &value["decisions"];
        assert!(
            decisions.is_sequence(),
            "decisions should be a sequence/array"
        );
    }
}
