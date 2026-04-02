//! Decision system for decision-based code review workflow
//!
//! This module contains the decision entities used in the decision-based review system,
//! allowing reviewers to understand code changes organized by the architectural decisions
//! that produced them.

use crate::entities::reviewable_diff_id::ReviewableDiffId;
use crate::errors::Result;
use crate::state::ReviewState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Import the derive macro (must be at module level for #[derive(...)] to work)
extern crate diffviz_schema_macro;
use diffviz_schema_macro::SchemaTemplate;

/// An inclusive range of line numbers affected by a code impact
///
/// Represents a contiguous block of lines within a file that are affected by a decision.
/// Both start and end are inclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, SchemaTemplate)]
pub struct DecisionLineRange {
    /// First line in the affected range (inclusive)
    pub start: usize,
    /// Last line in the affected range (inclusive)
    pub end: usize,
}

/// How a single decision affects a specific file
///
/// Describes which lines in a file are affected by a decision and explains the connection.
/// The line_ranges specify the exact code affected, while reasoning explains why this
/// decision impacts these particular lines.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, SchemaTemplate)]
pub struct CodeImpact {
    ////// Explanation of why this decision impacts these specific lines
    #[schema(
        example = "Middleware validates JWT tokens and injects user context",
        comment = "Why this file is affected by this decision"
    )]
    pub reasoning: String,

    /// Path to the source file relative to repository root
    #[schema(
        example = "src/auth/middleware.rs",
        comment = "Path to the source file relative to repository root"
    )]
    pub file: String,

    /// Line ranges within this file affected by the decision
    pub line_ranges: Vec<DecisionLineRange>,
}

/// An architectural decision and its effects on the codebase
///
/// A decision captures a specific architectural or design choice made during development
/// and maps that choice to the exact locations in code where it was implemented.
/// Decisions provide semantic context for code review by organizing changes around the
/// decisions that drove them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, SchemaTemplate)]
pub struct Decision {
    /// Sequential identifier for this decision (starting from 1)
    #[schema(comment = "Sequential identifier for this decision (starting from 1)")]
    pub number: u32,

    /// One-sentence summary of the decision
    #[schema(
        example = "Add authentication middleware",
        comment = "One-sentence summary of the architectural decision"
    )]
    pub title: String,

    /// Optional explanation of why this decision was chosen, including constraints or trade-offs considered
    #[schema(
        example = "Middleware must validate tokens for security requirements",
        comment = "Why this choice was made — constraints, priorities, trade-offs"
    )]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,

    /// All files and line ranges affected by this decision
    pub code_impacts: Vec<CodeImpact>,
}

/// A collection of architectural decisions made during development
///
/// A decision log documents the set of architectural decisions that produced a set of code changes.
/// Each decision in the log maps to specific lines of code that were affected by that decision.
/// The decision log serves as a semantic index for code review, enabling reviewers to understand
/// *why* code was changed, not just *what* changed.
///
/// The log can be used in two phases:
/// - **Strategy phase**: Decisions planned but not yet implemented (commit is optional)
/// - **Implementation phase**: Decisions realized in code (commit is required for tracking)
#[derive(Debug, Clone, Serialize, Deserialize, SchemaTemplate)]
pub struct DecisionLog {
    #[schema(
        comment = "Git hash of the commit containing the code changes described by these decisions"
    )]
    pub commit: String,

    /// The ordered list of architectural decisions made in this contribution
    pub decisions: Vec<Decision>,
}

impl DecisionLog {
    /// Parse decisions from YAML content. The caller is responsible for reading the source.
    /// Returns `Err` if the content cannot be deserialized.
    pub fn parse(content: &str) -> Result<DecisionLog> {
        Ok(serde_yaml::from_str(content)?)
    }
}

/// A value type pairing a ReviewableDiffId with the decision it belongs to.
/// Produced by ReviewEngine::get_decision_reviewable_diffs() and consumed by the navigation tree.
#[derive(Debug, Clone)]
pub struct DecisionReviewableDiff {
    pub chunk_id: ReviewableDiffId,
    pub decision_number: u32,
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

    /// Add a decision to the collection
    /// Note: Call build_index_from_review_state() after adding all decisions to populate the decision_index
    pub fn add_decision(&mut self, decision: Decision) {
        self.decisions.insert(decision.number, decision);
    }

    /// Build the decision index by detecting overlaps between code impacts and review state diffs
    /// This maps each ReviewableDiffId from the review state to the decisions that affect it
    pub fn build_index_from_review_state(&mut self, review_state: &ReviewState) {
        self.decision_index.clear();

        // Get decisions in order by number to ensure consistent ordering
        let mut decision_numbers: Vec<u32> = self.decisions.keys().copied().collect();
        decision_numbers.sort();

        // For each decision (in order) and its code impacts
        for decision_number in decision_numbers {
            let decision = &self.decisions[&decision_number];
            for impact in &decision.code_impacts {
                // For each ReviewableDiff in the review state
                for reviewable_diff in review_state.reviewable_diffs.values() {
                    // Check if this diff is in the same file
                    if reviewable_diff.file_path == impact.file {
                        // Check if the diff's line range overlaps with any of this impact's ranges
                        for code_impact_range in &impact.line_ranges {
                            if Self::ranges_overlap(
                                reviewable_diff.id.line_range().start_line,
                                reviewable_diff.id.line_range().end_line,
                                code_impact_range.start,
                                code_impact_range.end,
                            ) {
                                // Found an overlap - map this diff to this decision
                                self.decision_index
                                    .entry(reviewable_diff.id.clone())
                                    .or_default()
                                    .push(decision_number);
                                // Break to avoid adding same decision multiple times for same diff
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    /// Check if two line ranges overlap
    /// Range 1: [start1, end1], Range 2: [start2, end2] (both inclusive)
    /// Ranges overlap if: start1 <= end2 && start2 <= end1
    fn ranges_overlap(start1: usize, end1: usize, start2: usize, end2: usize) -> bool {
        start1 <= end2 && start2 <= end1
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

    /// Create a synthetic Decision 0 for unmapped diffs
    /// Ensures all ReviewableDiffs are accessible through decision-based navigation
    /// Only creates Decision 0 if there are unmapped diffs
    pub fn create_unmapped_decision(&mut self, review_state: &ReviewState) {
        // Find all diffs that are not mapped to any decision
        let unmapped_diffs: Vec<_> = review_state
            .reviewable_diffs
            .values()
            .filter(|diff| !self.decision_index.contains_key(&diff.id))
            .collect();

        // Only create Decision 0 if there are unmapped diffs
        if !unmapped_diffs.is_empty() {
            let mut code_impacts = Vec::new();

            // Create a CodeImpact for each unmapped diff
            for diff in &unmapped_diffs {
                code_impacts.push(CodeImpact {
                    file: diff.file_path.clone(),
                    line_ranges: vec![DecisionLineRange {
                        start: diff.id.line_range().start_line,
                        end: diff.id.line_range().end_line,
                    }],
                    reasoning: "Code change not mapped to any architectural decision".to_string(),
                });
            }

            let unmapped_decision = Decision {
                number: 0,
                title: "Unmapped Changes".to_string(),
                rationale: Some(
                    "Code changes that are not mapped to any architectural decision".to_string(),
                ),
                code_impacts,
            };

            self.add_decision(unmapped_decision);

            // Add all unmapped diffs to the index for Decision 0
            for diff in unmapped_diffs {
                self.decision_index
                    .entry(diff.id.clone())
                    .or_default()
                    .push(0);
            }
        }
    }
}

// DecisionApproval and DecisionApprovals are now defined in approval.rs
// as ApprovalRecord and ApprovalMap<u32> respectively.
pub use crate::entities::approval::{ApprovalRecord as DecisionApproval, DecisionApprovals};

#[cfg(test)]
#[path = "decision_tests.rs"]
mod tests;
