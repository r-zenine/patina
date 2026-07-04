//! Schema template generation for decision-log artifacts
//!
//! This module provides the SchemaTemplate trait for YAML template generation.
//! Templates are checked-in files (see `decision_log.template.yaml`); the
//! `template_structure_matches_decision_log` test guards against drift between
//! the template and the entity structs.

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
        include_str!("decision_log.template.yaml").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::decision::{CodeImpact, Decision, DecisionLineRange};
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

    /// A fully-populated sample covering every field, including optional ones.
    /// Serializing it yields the exact key structure the template must mirror.
    fn sample_decision_log() -> DecisionLog {
        DecisionLog {
            commit: "abc123".to_string(),
            decisions: vec![Decision {
                number: 1,
                title: "title".to_string(),
                rationale: Some("rationale".to_string()),
                code_impacts: vec![CodeImpact {
                    reasoning: "reasoning".to_string(),
                    file: "file".to_string(),
                    line_ranges: vec![DecisionLineRange { start: 1, end: 2 }],
                }],
            }],
        }
    }

    /// Recursively compare the key structure of two YAML values.
    /// Mappings must have identical key sets; sequences are compared by their
    /// first element (templates show one example item per list).
    fn assert_same_structure(template: &Value, reference: &Value, path: &str) {
        match (template, reference) {
            (Value::Mapping(t), Value::Mapping(r)) => {
                let t_keys: Vec<&Value> = t.keys().collect();
                let r_keys: Vec<&Value> = r.keys().collect();
                assert_eq!(
                    t_keys, r_keys,
                    "Template keys diverge from DecisionLog struct at '{path}'"
                );
                for (key, t_val) in t {
                    let key_str = key.as_str().unwrap_or("?");
                    assert_same_structure(t_val, &r[key], &format!("{path}.{key_str}"));
                }
            }
            (Value::Sequence(t), Value::Sequence(r)) => {
                assert!(
                    !t.is_empty(),
                    "Template sequence at '{path}' must show one example item"
                );
                assert_same_structure(&t[0], &r[0], &format!("{path}[0]"));
            }
            (t, r) => {
                assert!(
                    !matches!(r, Value::Mapping(_) | Value::Sequence(_)),
                    "Template has scalar at '{path}' where struct expects {r:?}"
                );
                assert!(
                    !matches!(t, Value::Mapping(_) | Value::Sequence(_)),
                    "Template has {t:?} at '{path}' where struct expects a scalar"
                );
            }
        }
    }

    #[test]
    fn template_structure_matches_decision_log() {
        let template: Value =
            serde_yaml::from_str(&DecisionLog::yaml_template()).expect("template must be YAML");
        let reference = serde_yaml::to_value(sample_decision_log()).expect("sample must serialize");
        assert_same_structure(&template, &reference, "$");
    }
}
