//! Decision system for decision-based code review workflow
//!
//! This module contains the decision entities used in the decision-based review system,
//! allowing reviewers to understand code changes organized by the architectural decisions
//! that produced them.

use crate::entities::reviewable_diff_id::ReviewableDiffId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of code change produced by a decision
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeType {
    Addition,
    Modification,
    Deletion,
}

/// Confidence level in the decision-to-code mapping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    High,
    Medium,
    Low,
}

/// A range of lines in source code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionLineRange {
    pub start: usize,
    pub end: usize,
}

/// Code impact of a single decision on a file
/// Maps a decision to specific function-level code ranges
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeImpact {
    pub file: String,
    pub line_ranges: Vec<DecisionLineRange>,
    pub change_type: ChangeType,
    pub confidence: Confidence,
    pub reasoning: String,
}

/// An architectural decision and its code impacts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decision {
    pub number: u32,
    pub title: String,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_log_line: Option<usize>,
    pub code_impacts: Vec<CodeImpact>,
}

/// Collection of decisions organized for quick lookup and indexing
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReviewDecisions {
    /// All decisions indexed by decision number
    pub decisions: HashMap<u32, Decision>,

    /// Index mapping ReviewableDiffId -> set of decision numbers that affect this code
    /// This allows UI to show "decision #1 affects this code block"
    #[serde(skip)]
    pub decision_index: HashMap<ReviewableDiffId, Vec<u32>>,
}

impl ReviewDecisions {
    pub fn new() -> Self {
        Self {
            decisions: HashMap::new(),
            decision_index: HashMap::new(),
        }
    }

    /// Add a decision and update the decision index
    pub fn add_decision(&mut self, decision: Decision) {
        let decision_number = decision.number;

        // For each code impact, add this decision to the index
        // This creates ReviewableDiffIds for each line range and maps decision to them
        for impact in &decision.code_impacts {
            // For each line range in this impact, create a ReviewableDiffId
            // We create a synthetic ReviewableDiffId based on file and line range
            for line_range in &impact.line_ranges {
                let reviewable_id = self.create_synthetic_reviewable_id(
                    &impact.file,
                    line_range.start,
                    line_range.end,
                );

                self.decision_index
                    .entry(reviewable_id)
                    .or_default()
                    .push(decision_number);
            }
        }

        self.decisions.insert(decision_number, decision);
    }

    /// Get a decision by number
    pub fn get_decision(&self, number: u32) -> Option<&Decision> {
        self.decisions.get(&number)
    }

    /// Get all decisions that affect a specific ReviewableDiffId
    pub fn get_decisions_for_diff(&self, reviewable_id: &ReviewableDiffId) -> Vec<&Decision> {
        self.decision_index
            .get(reviewable_id)
            .map(|decision_numbers| {
                decision_numbers
                    .iter()
                    .filter_map(|num| self.decisions.get(num))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all decisions in order by number
    pub fn all_decisions(&self) -> Vec<&Decision> {
        let mut decisions: Vec<_> = self.decisions.values().collect();
        decisions.sort_by_key(|d| d.number);
        decisions
    }

    /// Create a synthetic ReviewableDiffId from file path and line range
    /// This is used internally to index decisions by the code ranges they affect
    fn create_synthetic_reviewable_id(
        &self,
        file: &str,
        start: usize,
        end: usize,
    ) -> ReviewableDiffId {
        // Use the line range as the diff range
        use crate::entities::git_ref::DiffQuery;
        use crate::entities::reviewable_diff_id::LineRange;

        ReviewableDiffId::new(
            DiffQuery::head_to_unstaged(),
            file.to_string(),
            LineRange {
                start_line: start,
                end_line: end,
                start_column: 0,
                end_column: 0,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_decision() -> Decision {
        Decision {
            number: 1,
            title: "Refactor authentication module".to_string(),
            summary: "Extract auth logic into separate module for clarity".to_string(),
            decision_log_line: Some(15),
            code_impacts: vec![CodeImpact {
                file: "src/auth.rs".to_string(),
                line_ranges: vec![DecisionLineRange { start: 10, end: 50 }],
                change_type: ChangeType::Addition,
                confidence: Confidence::High,
                reasoning: "New authentication module implementation".to_string(),
            }],
        }
    }

    #[test]
    fn test_review_decisions_add_and_get() {
        let mut decisions = ReviewDecisions::new();
        let decision = create_test_decision();

        decisions.add_decision(decision.clone());

        assert_eq!(decisions.get_decision(1), Some(&decision));
    }

    #[test]
    fn test_review_decisions_all_decisions() {
        let mut decisions = ReviewDecisions::new();

        decisions.add_decision(Decision {
            number: 2,
            title: "Second decision".to_string(),
            summary: "Summary 2".to_string(),
            decision_log_line: None,
            code_impacts: vec![],
        });

        decisions.add_decision(Decision {
            number: 1,
            title: "First decision".to_string(),
            summary: "Summary 1".to_string(),
            decision_log_line: None,
            code_impacts: vec![],
        });

        let all = decisions.all_decisions();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].number, 1);
        assert_eq!(all[1].number, 2);
    }

    #[test]
    fn test_decision_serialization() {
        let decision = create_test_decision();
        let json = serde_json::to_string(&decision).unwrap();
        let deserialized: Decision = serde_json::from_str(&json).unwrap();
        assert_eq!(decision, deserialized);
    }

    #[test]
    fn test_confidence_serialization() {
        let confidence = Confidence::High;
        let json = serde_json::to_string(&confidence).unwrap();
        assert_eq!(json, "\"high\"");

        let deserialized: Confidence = serde_json::from_str(&json).unwrap();
        assert_eq!(confidence, deserialized);
    }

    #[test]
    fn test_change_type_serialization() {
        let change_type = ChangeType::Addition;
        let json = serde_json::to_string(&change_type).unwrap();
        assert_eq!(json, "\"addition\"");

        let deserialized: ChangeType = serde_json::from_str(&json).unwrap();
        assert_eq!(change_type, deserialized);
    }
}
