//! Decision system for decision-based code review workflow
//!
//! This module contains the decision entities used in the decision-based review system,
//! allowing reviewers to understand code changes organized by the architectural decisions
//! that produced them.

use crate::entities::reviewable_diff_id::ReviewableDiffId;
use crate::state::ReviewState;
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
                    change_type: ChangeType::Modification,
                    confidence: Confidence::Medium,
                    reasoning: "Code change not mapped to any architectural decision".to_string(),
                });
            }

            let unmapped_decision = Decision {
                number: 0,
                title: "Unmapped Changes".to_string(),
                summary: "Code changes that are not mapped to any architectural decision"
                    .to_string(),
                decision_log_line: None,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::git_ref::DiffQuery;
    use crate::entities::reviewable_diff_id::LineRange;
    use crate::state::ReviewableDiff;
    use diffviz_core::ast_diff::{OwnedNodeData, SourceCode};
    use diffviz_core::common::{ProgrammingLanguage, SemanticNodeKind};
    use diffviz_core::reviewable_diff::{
        DiffMetadata, DiffNode, NodeChangeStatus, ReviewableDiff as CoreReviewableDiff,
    };
    use std::collections::HashMap;

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

    fn create_test_reviewable_diff(
        file_path: &str,
        start_line: usize,
        end_line: usize,
    ) -> ReviewableDiff {
        let reviewable_id = ReviewableDiffId::new(
            DiffQuery::head_to_unstaged(),
            file_path.to_string(),
            LineRange {
                start_line,
                end_line,
                start_column: 0,
                end_column: 0,
            },
        );

        let placeholder_content = format!("content for {file_path}");
        let old_source = Box::new(SourceCode::new(placeholder_content.clone()));
        let new_source = Box::new(SourceCode::new(placeholder_content.clone()));

        let core_diff = CoreReviewableDiff {
            language: ProgrammingLanguage::Rust,
            boundary: DiffNode {
                node_type: "test".to_string(),
                semantic_kind: SemanticNodeKind::Other("test".to_string()),
                change_status: NodeChangeStatus::Unchanged {
                    node: OwnedNodeData {
                        start_byte: 0,
                        end_byte: placeholder_content.len(),
                        kind: "test".to_string(),
                    },
                },
                relevance: 0,
                children: vec![],
            },
            old_source,
            new_source,
            metadata: DiffMetadata {
                total_changes: 1,
                change_summary: HashMap::new(),
                essential_node_count: 1,
                analysis_duration_ms: 0,
            },
        };

        ReviewableDiff {
            id: reviewable_id,
            core_diff,
            file_path: file_path.to_string(),
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
    fn test_build_index_exact_overlap() {
        let mut decisions = ReviewDecisions::new();
        decisions.add_decision(Decision {
            number: 1,
            title: "Decision 1".to_string(),
            summary: "Summary".to_string(),
            decision_log_line: None,
            code_impacts: vec![CodeImpact {
                file: "src/auth.rs".to_string(),
                line_ranges: vec![DecisionLineRange { start: 10, end: 20 }],
                change_type: ChangeType::Modification,
                confidence: Confidence::High,
                reasoning: "Auth changes".to_string(),
            }],
        });

        let diff = create_test_reviewable_diff("src/auth.rs", 10, 20);
        let review_state = ReviewState::new(vec![diff.clone()], "author".to_string());

        decisions.build_index_from_review_state(&review_state);

        let decision_list = decisions.get_decisions_for_diff(&diff.id);
        assert_eq!(decision_list.len(), 1);
        assert_eq!(decision_list[0].number, 1);
    }

    #[test]
    fn test_build_index_partial_overlap() {
        let mut decisions = ReviewDecisions::new();
        decisions.add_decision(Decision {
            number: 1,
            title: "Decision 1".to_string(),
            summary: "Summary".to_string(),
            decision_log_line: None,
            code_impacts: vec![CodeImpact {
                file: "src/auth.rs".to_string(),
                line_ranges: vec![DecisionLineRange { start: 10, end: 30 }],
                change_type: ChangeType::Modification,
                confidence: Confidence::High,
                reasoning: "Auth changes".to_string(),
            }],
        });

        // Diff that partially overlaps: 15-25 overlaps with decision range 10-30
        let diff = create_test_reviewable_diff("src/auth.rs", 15, 25);
        let review_state = ReviewState::new(vec![diff.clone()], "author".to_string());

        decisions.build_index_from_review_state(&review_state);

        let decision_list = decisions.get_decisions_for_diff(&diff.id);
        assert_eq!(decision_list.len(), 1);
        assert_eq!(decision_list[0].number, 1);
    }

    #[test]
    fn test_build_index_no_overlap() {
        let mut decisions = ReviewDecisions::new();
        decisions.add_decision(Decision {
            number: 1,
            title: "Decision 1".to_string(),
            summary: "Summary".to_string(),
            decision_log_line: None,
            code_impacts: vec![CodeImpact {
                file: "src/auth.rs".to_string(),
                line_ranges: vec![DecisionLineRange { start: 10, end: 20 }],
                change_type: ChangeType::Modification,
                confidence: Confidence::High,
                reasoning: "Auth changes".to_string(),
            }],
        });

        // Diff that doesn't overlap: 30-40 doesn't overlap with decision range 10-20
        let diff = create_test_reviewable_diff("src/auth.rs", 30, 40);
        let review_state = ReviewState::new(vec![diff.clone()], "author".to_string());

        decisions.build_index_from_review_state(&review_state);

        let decision_list = decisions.get_decisions_for_diff(&diff.id);
        assert_eq!(decision_list.len(), 0);
    }

    #[test]
    fn test_build_index_different_file_no_match() {
        let mut decisions = ReviewDecisions::new();
        decisions.add_decision(Decision {
            number: 1,
            title: "Decision 1".to_string(),
            summary: "Summary".to_string(),
            decision_log_line: None,
            code_impacts: vec![CodeImpact {
                file: "src/auth.rs".to_string(),
                line_ranges: vec![DecisionLineRange { start: 10, end: 20 }],
                change_type: ChangeType::Modification,
                confidence: Confidence::High,
                reasoning: "Auth changes".to_string(),
            }],
        });

        // Diff in different file should not match
        let diff = create_test_reviewable_diff("src/other.rs", 10, 20);
        let review_state = ReviewState::new(vec![diff.clone()], "author".to_string());

        decisions.build_index_from_review_state(&review_state);

        let decision_list = decisions.get_decisions_for_diff(&diff.id);
        assert_eq!(decision_list.len(), 0);
    }

    #[test]
    fn test_build_index_multiple_decisions() {
        let mut decisions = ReviewDecisions::new();
        decisions.add_decision(Decision {
            number: 1,
            title: "Decision 1".to_string(),
            summary: "Summary".to_string(),
            decision_log_line: None,
            code_impacts: vec![CodeImpact {
                file: "src/auth.rs".to_string(),
                line_ranges: vec![DecisionLineRange { start: 10, end: 30 }],
                change_type: ChangeType::Modification,
                confidence: Confidence::High,
                reasoning: "Auth changes".to_string(),
            }],
        });

        decisions.add_decision(Decision {
            number: 2,
            title: "Decision 2".to_string(),
            summary: "Summary".to_string(),
            decision_log_line: None,
            code_impacts: vec![CodeImpact {
                file: "src/auth.rs".to_string(),
                line_ranges: vec![DecisionLineRange { start: 15, end: 25 }],
                change_type: ChangeType::Modification,
                confidence: Confidence::High,
                reasoning: "More auth changes".to_string(),
            }],
        });

        // Diff in range that overlaps both decisions
        let diff = create_test_reviewable_diff("src/auth.rs", 18, 22);
        let review_state = ReviewState::new(vec![diff.clone()], "author".to_string());

        decisions.build_index_from_review_state(&review_state);

        let decision_list = decisions.get_decisions_for_diff(&diff.id);
        assert_eq!(decision_list.len(), 2);
        assert_eq!(decision_list[0].number, 1);
        assert_eq!(decision_list[1].number, 2);
    }

    #[test]
    fn test_build_index_nested_range() {
        let mut decisions = ReviewDecisions::new();
        decisions.add_decision(Decision {
            number: 1,
            title: "Decision 1".to_string(),
            summary: "Summary".to_string(),
            decision_log_line: None,
            code_impacts: vec![CodeImpact {
                file: "src/auth.rs".to_string(),
                line_ranges: vec![DecisionLineRange { start: 10, end: 50 }],
                change_type: ChangeType::Modification,
                confidence: Confidence::High,
                reasoning: "Large refactor".to_string(),
            }],
        });

        // Diff nested inside decision range
        let diff = create_test_reviewable_diff("src/auth.rs", 20, 30);
        let review_state = ReviewState::new(vec![diff.clone()], "author".to_string());

        decisions.build_index_from_review_state(&review_state);

        let decision_list = decisions.get_decisions_for_diff(&diff.id);
        assert_eq!(decision_list.len(), 1);
        assert_eq!(decision_list[0].number, 1);
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

    #[test]
    fn test_create_unmapped_decision_with_unmapped_diffs() {
        let mut decisions = ReviewDecisions::new();
        decisions.add_decision(Decision {
            number: 1,
            title: "Decision 1".to_string(),
            summary: "Summary".to_string(),
            decision_log_line: None,
            code_impacts: vec![CodeImpact {
                file: "src/auth.rs".to_string(),
                line_ranges: vec![DecisionLineRange { start: 10, end: 20 }],
                change_type: ChangeType::Modification,
                confidence: Confidence::High,
                reasoning: "Auth changes".to_string(),
            }],
        });

        // Create two diffs: one mapped, one unmapped
        let mapped_diff = create_test_reviewable_diff("src/auth.rs", 10, 20);
        let unmapped_diff = create_test_reviewable_diff("src/other.rs", 5, 15);

        let review_state = ReviewState::new(
            vec![mapped_diff.clone(), unmapped_diff.clone()],
            "author".to_string(),
        );

        // Build index first to map Decision 1
        decisions.build_index_from_review_state(&review_state);

        // Create Decision 0 for unmapped
        decisions.create_unmapped_decision(&review_state);

        // Verify Decision 0 exists
        let decision_0 = decisions.get_decision(0);
        assert!(decision_0.is_some());
        assert_eq!(decision_0.unwrap().title, "Unmapped Changes");

        // Verify unmapped diff is mapped to Decision 0
        let decisions_for_unmapped = decisions.get_decisions_for_diff(&unmapped_diff.id);
        assert_eq!(decisions_for_unmapped.len(), 1);
        assert_eq!(decisions_for_unmapped[0].number, 0);

        // Verify mapped diff is still mapped to Decision 1 (not Decision 0)
        let decisions_for_mapped = decisions.get_decisions_for_diff(&mapped_diff.id);
        assert_eq!(decisions_for_mapped.len(), 1);
        assert_eq!(decisions_for_mapped[0].number, 1);
    }

    #[test]
    fn test_create_unmapped_decision_with_no_unmapped_diffs() {
        let mut decisions = ReviewDecisions::new();
        decisions.add_decision(Decision {
            number: 1,
            title: "Decision 1".to_string(),
            summary: "Summary".to_string(),
            decision_log_line: None,
            code_impacts: vec![CodeImpact {
                file: "src/auth.rs".to_string(),
                line_ranges: vec![DecisionLineRange { start: 10, end: 20 }],
                change_type: ChangeType::Modification,
                confidence: Confidence::High,
                reasoning: "Auth changes".to_string(),
            }],
        });

        // Create only mapped diffs
        let diff1 = create_test_reviewable_diff("src/auth.rs", 10, 20);
        let review_state = ReviewState::new(vec![diff1], "author".to_string());

        decisions.build_index_from_review_state(&review_state);
        decisions.create_unmapped_decision(&review_state);

        // Verify Decision 0 was NOT created
        let decision_0 = decisions.get_decision(0);
        assert!(decision_0.is_none());

        // Verify we still only have Decision 1
        let all = decisions.all_decisions();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].number, 1);
    }

    #[test]
    fn test_create_unmapped_decision_with_all_unmapped() {
        let mut decisions = ReviewDecisions::new();

        // Create only unmapped diffs
        let diff1 = create_test_reviewable_diff("src/auth.rs", 10, 20);
        let diff2 = create_test_reviewable_diff("src/other.rs", 5, 15);
        let review_state =
            ReviewState::new(vec![diff1.clone(), diff2.clone()], "author".to_string());

        decisions.build_index_from_review_state(&review_state);
        decisions.create_unmapped_decision(&review_state);

        // Verify Decision 0 exists
        let decision_0 = decisions.get_decision(0);
        assert!(decision_0.is_some());

        // Verify both diffs are mapped to Decision 0
        let decisions_for_diff1 = decisions.get_decisions_for_diff(&diff1.id);
        assert_eq!(decisions_for_diff1.len(), 1);
        assert_eq!(decisions_for_diff1[0].number, 0);

        let decisions_for_diff2 = decisions.get_decisions_for_diff(&diff2.id);
        assert_eq!(decisions_for_diff2.len(), 1);
        assert_eq!(decisions_for_diff2[0].number, 0);

        // Verify Decision 0 has correct code impacts
        let code_impacts = &decision_0.unwrap().code_impacts;
        assert_eq!(code_impacts.len(), 2);

        // Verify line ranges are captured correctly
        let auth_impact = code_impacts.iter().find(|c| c.file == "src/auth.rs");
        assert!(auth_impact.is_some());
        assert_eq!(auth_impact.unwrap().line_ranges[0].start, 10);
        assert_eq!(auth_impact.unwrap().line_ranges[0].end, 20);

        let other_impact = code_impacts.iter().find(|c| c.file == "src/other.rs");
        assert!(other_impact.is_some());
        assert_eq!(other_impact.unwrap().line_ranges[0].start, 5);
        assert_eq!(other_impact.unwrap().line_ranges[0].end, 15);
    }

    #[test]
    fn test_create_unmapped_decision_preserves_existing_decisions() {
        let mut decisions = ReviewDecisions::new();

        // Add Decision 1 and 2
        decisions.add_decision(Decision {
            number: 1,
            title: "Decision 1".to_string(),
            summary: "Summary 1".to_string(),
            decision_log_line: None,
            code_impacts: vec![CodeImpact {
                file: "src/file1.rs".to_string(),
                line_ranges: vec![DecisionLineRange { start: 1, end: 10 }],
                change_type: ChangeType::Addition,
                confidence: Confidence::High,
                reasoning: "Change 1".to_string(),
            }],
        });

        decisions.add_decision(Decision {
            number: 2,
            title: "Decision 2".to_string(),
            summary: "Summary 2".to_string(),
            decision_log_line: None,
            code_impacts: vec![CodeImpact {
                file: "src/file2.rs".to_string(),
                line_ranges: vec![DecisionLineRange { start: 20, end: 30 }],
                change_type: ChangeType::Modification,
                confidence: Confidence::High,
                reasoning: "Change 2".to_string(),
            }],
        });

        let diff1 = create_test_reviewable_diff("src/file1.rs", 1, 10);
        let diff2 = create_test_reviewable_diff("src/file2.rs", 20, 30);
        let unmapped_diff = create_test_reviewable_diff("src/unmapped.rs", 50, 60);

        let review_state = ReviewState::new(
            vec![diff1.clone(), diff2.clone(), unmapped_diff.clone()],
            "author".to_string(),
        );

        decisions.build_index_from_review_state(&review_state);
        decisions.create_unmapped_decision(&review_state);

        // Verify all decisions still exist
        let all = decisions.all_decisions();
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].number, 0);
        assert_eq!(all[1].number, 1);
        assert_eq!(all[2].number, 2);

        // Verify correct mappings
        assert_eq!(decisions.get_decisions_for_diff(&diff1.id).len(), 1);
        assert_eq!(decisions.get_decisions_for_diff(&diff1.id)[0].number, 1);

        assert_eq!(decisions.get_decisions_for_diff(&diff2.id).len(), 1);
        assert_eq!(decisions.get_decisions_for_diff(&diff2.id)[0].number, 2);

        assert_eq!(decisions.get_decisions_for_diff(&unmapped_diff.id).len(), 1);
        assert_eq!(
            decisions.get_decisions_for_diff(&unmapped_diff.id)[0].number,
            0
        );
    }
}
