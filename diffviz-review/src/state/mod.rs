//! Review state management with ReviewableDiff-based architecture
//!
//! This module provides centralized state management for review sessions,
//! organized around ReviewableDiffs as the primary unit of review.

use crate::entities::reviewable_diff_id::ReviewableDiffId;
use crate::entities::{
    DecisionApprovals, DecisionInstructions, Instruction, ReviewApprovals, ReviewDecisions,
    ReviewInstructions,
};
use std::collections::{BTreeMap, HashMap};

/// Centralized application state containing all business data
/// Organized around ReviewableDiffs as the primary unit of review.
#[derive(Debug, Clone)]
pub struct ReviewState {
    /// Map of ReviewableDiffs indexed by their unique identifier
    /// Ordered by file path and line range for consistent iteration
    pub reviewable_diffs: BTreeMap<ReviewableDiffId, ReviewableDiff>,

    pub approvals: ReviewApprovals,
    pub instructions: ReviewInstructions,
    pub decisions: ReviewDecisions,
    pub decision_approvals: DecisionApprovals,
    pub decision_instructions: DecisionInstructions,

    pub author: String,
}

// Import ReviewableDiff from diffviz-core now that it's lifetime-independent
use diffviz_core::reviewable_diff::ReviewableDiff as CoreReviewableDiff;

/// Wrapper for core ReviewableDiff with review-layer specific metadata
#[derive(Debug, Clone)]
pub struct ReviewableDiff {
    pub id: ReviewableDiffId,
    pub core_diff: CoreReviewableDiff,
    pub file_path: String,
    /// Original code-impact line ranges from the decision log that collapsed to this semantic unit.
    /// Multiple ranges are present when two or more cited ranges expand to the same function/struct.
    pub cited_ranges: Vec<(usize, usize)>,
}

impl ReviewableDiff {
    /// Create a new ReviewableDiff from core diff and metadata
    pub fn new(id: ReviewableDiffId, core_diff: CoreReviewableDiff, file_path: String) -> Self {
        Self {
            id,
            core_diff,
            file_path,
            cited_ranges: Vec::new(),
        }
    }

    /// Record a cited range, skipping duplicates.
    pub fn add_cited_range(&mut self, start: usize, end: usize) {
        if !self
            .cited_ranges
            .iter()
            .any(|&(s, e)| s == start && e == end)
        {
            self.cited_ranges.push((start, end));
        }
    }

    /// Get the language of this diff
    pub fn language(&self) -> &diffviz_core::common::ProgrammingLanguage {
        &self.core_diff.language
    }

    /// Get the total number of changes
    pub fn total_changes(&self) -> usize {
        self.core_diff.metadata.total_changes
    }

    /// Get a display name for the boundary (e.g., function name, struct name)
    pub fn boundary_name(&self) -> String {
        // Extract name from the boundary node's type and semantic kind
        format!(
            "{} ({:?})",
            self.core_diff.boundary.node_type, self.core_diff.boundary.semantic_kind
        )
    }
}

impl ReviewState {
    /// Create a new review state with ReviewableDiffs
    pub fn new(reviewable_diffs: Vec<ReviewableDiff>, author: String) -> Self {
        let mut diffs_map = BTreeMap::new();
        for diff in reviewable_diffs {
            diffs_map.insert(diff.id.clone(), diff);
        }

        Self {
            reviewable_diffs: diffs_map,
            approvals: ReviewApprovals::new(),
            instructions: ReviewInstructions::new(),
            decisions: ReviewDecisions::new(),
            decision_approvals: DecisionApprovals::new(),
            decision_instructions: DecisionInstructions::new(),
            author,
        }
    }

    /// Create review state with existing review data (for loading sessions)
    pub fn with_review_data(
        reviewable_diffs: Vec<ReviewableDiff>,
        author: String,
        approvals: ReviewApprovals,
        instructions: ReviewInstructions,
        decisions: ReviewDecisions,
        decision_approvals: DecisionApprovals,
        decision_instructions: DecisionInstructions,
    ) -> Self {
        let mut diffs_map = BTreeMap::new();
        for diff in reviewable_diffs {
            diffs_map.insert(diff.id.clone(), diff);
        }

        Self {
            reviewable_diffs: diffs_map,
            approvals,
            instructions,
            decisions,
            decision_approvals,
            decision_instructions,
            author,
        }
    }

    // === Query Methods (for UI to read state) ===

    /// Check if a reviewable diff is approved
    pub fn is_approved(&self, reviewable_id: &ReviewableDiffId) -> bool {
        self.approvals.is_approved(reviewable_id)
    }

    /// Get instructions for a reviewable diff
    pub fn get_instructions(&self, reviewable_id: &ReviewableDiffId) -> Option<&Vec<Instruction>> {
        self.instructions.get_instructions(reviewable_id)
    }

    /// Get overall approval progress
    pub fn approval_progress(&self) -> (usize, usize, f32) {
        let total = self.total_reviewable_diffs();
        let approved = self.approvals.total_approved();
        let percentage = self.approvals.approval_percentage(total);
        (approved, total, percentage)
    }

    /// Get total number of reviewable diffs
    pub fn total_reviewable_diffs(&self) -> usize {
        self.reviewable_diffs.len()
    }

    /// Check if reviewable diff has instructions
    pub fn has_instructions(&self, reviewable_id: &ReviewableDiffId) -> bool {
        self.instructions.has_instructions(reviewable_id)
    }

    /// Get all ReviewableDiff IDs ordered by file and line range
    pub fn get_ordered_reviewable_ids(&self) -> Vec<&ReviewableDiffId> {
        self.reviewable_diffs.keys().collect()
    }

    /// Get ReviewableDiffs grouped by file path
    pub fn get_reviewable_diffs_by_file(&self) -> HashMap<String, Vec<&ReviewableDiff>> {
        let mut by_file = HashMap::new();
        for diff in self.reviewable_diffs.values() {
            by_file
                .entry(diff.file_path.clone())
                .or_insert_with(Vec::new)
                .push(diff);
        }
        by_file
    }

    /// Get a specific ReviewableDiff by ID
    pub fn get_reviewable_diff(&self, reviewable_id: &ReviewableDiffId) -> Option<&ReviewableDiff> {
        self.reviewable_diffs.get(reviewable_id)
    }

    // === Immutable Update Methods (return new state) ===

    /// Approve a reviewable diff (returns new state)
    pub fn approve(&mut self, reviewable_id: ReviewableDiffId, reviewer: String) -> &mut Self {
        let timestamp = chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string();

        self.approvals.approve(reviewable_id, reviewer, timestamp);
        self
    }

    /// Unapprove/reject a reviewable diff (returns new state)
    pub fn unapprove(&mut self, reviewable_id: &ReviewableDiffId) -> &mut Self {
        self.approvals.unapprove(reviewable_id);
        self
    }

    /// Add an instruction (returns new state)
    pub fn add_instruction(
        &mut self,
        reviewable_id: ReviewableDiffId,
        instruction: Instruction,
    ) -> &mut Self {
        self.instructions
            .add_instruction(reviewable_id, instruction);
        self
    }

    /// Overwrite an existing instruction's content in place (returns new state)
    pub fn replace_instruction(
        &mut self,
        reviewable_id: ReviewableDiffId,
        instruction: Instruction,
    ) -> &mut Self {
        self.instructions
            .replace_instruction(reviewable_id, instruction);
        self
    }

    /// Approve all reviewable diffs in a file (returns new state)
    pub fn approve_all_in_file(&mut self, file_path: &str, reviewer: String) -> &mut Self {
        let timestamp = chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string();

        // Find all reviewable diffs for this file and approve them
        let reviewable_ids: Vec<ReviewableDiffId> = self
            .reviewable_diffs
            .values()
            .filter(|diff| diff.file_path == file_path)
            .map(|diff| diff.id.clone())
            .collect();

        for reviewable_id in reviewable_ids {
            self.approvals
                .approve(reviewable_id, reviewer.clone(), timestamp.clone());
        }

        self
    }

    // === Decision Approval Methods ===

    /// Check if a decision is approved
    pub fn is_decision_approved(&self, decision_number: u32) -> bool {
        self.decision_approvals.is_approved(&decision_number)
    }

    /// Get approval progress for a decision: (approved_chunks, total_chunks)
    pub fn decision_approval_progress(&self, decision_number: u32) -> (usize, usize) {
        // Find all chunks for this decision via decision_index
        let chunks_for_decision: Vec<&ReviewableDiffId> = self
            .decisions
            .decision_index
            .iter()
            .filter(|(_, nums)| nums.contains(&decision_number))
            .map(|(diff_id, _)| diff_id)
            .collect();

        let total_chunks = chunks_for_decision.len();
        let approved_chunks = chunks_for_decision
            .iter()
            .filter(|diff_id| self.approvals.is_approved(diff_id))
            .count();

        (approved_chunks, total_chunks)
    }

    /// Approve a decision (returns new state)
    pub fn approve_decision(&mut self, decision_number: u32, reviewer: String) -> &mut Self {
        let timestamp = chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string();

        self.decision_approvals
            .approve(decision_number, reviewer, timestamp);
        self
    }

    /// Unapprove/reject a decision (returns new state)
    pub fn unapprove_decision(&mut self, decision_number: u32) -> &mut Self {
        self.decision_approvals.unapprove(&decision_number);
        self
    }

    /// Get the author
    pub fn author(&self) -> &str {
        &self.author
    }

    /// Get all unique file paths in this review
    pub fn get_file_paths(&self) -> Vec<String> {
        let files: std::collections::HashSet<String> = self
            .reviewable_diffs
            .values()
            .map(|diff| diff.file_path.clone())
            .collect();
        let mut result: Vec<String> = files.into_iter().collect();
        result.sort();
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::git_ref::DiffQuery;
    use crate::entities::reviewable_diff_id::LineRange;

    fn create_test_reviewable_diff() -> ReviewableDiff {
        use diffviz_core::{
            ast_diff::{OwnedNodeData, SourceCode},
            common::{ProgrammingLanguage, SemanticNodeKind},
            reviewable_diff::{
                DiffMetadata, DiffNode, NodeChangeStatus, ReviewableDiff as CoreReviewableDiff,
            },
        };
        use std::collections::HashMap;

        let reviewable_id = ReviewableDiffId::new(
            DiffQuery::head_to_unstaged(),
            "test.rs".to_string(),
            LineRange {
                start_line: 1,
                end_line: 10,
                start_column: 0,
                end_column: 0,
            },
        );

        let placeholder_content = "test content".to_string();
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
                        identifier: None,
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

        ReviewableDiff::new(reviewable_id, core_diff, "test.rs".to_string())
    }

    #[test]
    fn test_review_state_creation() {
        let diff = create_test_reviewable_diff();
        let state = ReviewState::new(vec![diff], "test_author".to_string());

        assert_eq!(state.author, "test_author");
        assert_eq!(state.total_reviewable_diffs(), 1);
        assert_eq!(state.approval_progress(), (0, 1, 0.0));
    }

    #[test]
    fn test_approve_reviewable_diff() {
        let diff = create_test_reviewable_diff();
        let reviewable_id = diff.id.clone();
        let mut state = ReviewState::new(vec![diff], "test_author".to_string());

        state.approve(reviewable_id.clone(), "reviewer".to_string());

        assert!(state.is_approved(&reviewable_id));
        assert_eq!(state.approval_progress(), (1, 1, 100.0));
    }

    fn create_test_reviewable_diff_for_file(file_path: &str, start_line: usize) -> ReviewableDiff {
        use diffviz_core::{
            ast_diff::{OwnedNodeData, SourceCode},
            common::{ProgrammingLanguage, SemanticNodeKind},
            reviewable_diff::{
                DiffMetadata, DiffNode, NodeChangeStatus, ReviewableDiff as CoreReviewableDiff,
            },
        };
        use std::collections::HashMap;

        let reviewable_id = ReviewableDiffId::new(
            DiffQuery::head_to_unstaged(),
            file_path.to_string(),
            LineRange {
                start_line,
                end_line: start_line + 10,
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
                        identifier: None,
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

        ReviewableDiff::new(reviewable_id, core_diff, file_path.to_string())
    }

    #[test]
    fn test_grouping_by_file() {
        let diff1 = create_test_reviewable_diff_for_file("file1.rs", 1);
        let diff2 = create_test_reviewable_diff_for_file("file1.rs", 20);
        let diff3 = create_test_reviewable_diff_for_file("file2.rs", 1);

        let state = ReviewState::new(vec![diff1, diff2, diff3], "test_author".to_string());
        let by_file = state.get_reviewable_diffs_by_file();

        assert_eq!(by_file.len(), 2);
        assert_eq!(by_file.get("file1.rs").unwrap().len(), 2);
        assert_eq!(by_file.get("file2.rs").unwrap().len(), 1);
    }

    // Tests for instruction-only review system (Phase 1)
    #[test]
    fn test_review_state_can_be_created_with_only_instructions_and_approvals() {
        let diff = create_test_reviewable_diff();
        let state = ReviewState::new(vec![diff], "test_author".to_string());

        // Should compile and work without comments and suggestions
        assert_eq!(state.total_reviewable_diffs(), 1);
        assert_eq!(state.author(), "test_author");
    }

    #[test]
    fn test_adding_instruction_to_review_state() {
        use crate::entities::Instruction;

        let diff = create_test_reviewable_diff();
        let reviewable_id = diff.id.clone();
        let mut state = ReviewState::new(vec![diff], "test_author".to_string());

        let instruction = Instruction {
            id: "inst_1".to_string(),
            content: "Extract this to a separate function".to_string(),
            author: "reviewer".to_string(),
            timestamp: "2024-01-10T10:00:00Z".to_string(),
            status: crate::entities::instruction::InstructionStatus::Active,
        };

        state.add_instruction(reviewable_id.clone(), instruction);

        assert!(state.has_instructions(&reviewable_id));
        assert_eq!(state.get_instructions(&reviewable_id).unwrap().len(), 1);
    }

    #[test]
    fn test_getting_instructions_by_reviewable_diff_id() {
        use crate::entities::Instruction;

        let diff = create_test_reviewable_diff();
        let reviewable_id = diff.id.clone();
        let mut state = ReviewState::new(vec![diff], "test_author".to_string());

        let instruction1 = Instruction {
            id: "inst_1".to_string(),
            content: "First instruction".to_string(),
            author: "reviewer".to_string(),
            timestamp: "2024-01-10T10:00:00Z".to_string(),
            status: crate::entities::instruction::InstructionStatus::Active,
        };

        let instruction2 = Instruction {
            id: "inst_2".to_string(),
            content: "Second instruction".to_string(),
            author: "reviewer".to_string(),
            timestamp: "2024-01-10T10:01:00Z".to_string(),
            status: crate::entities::instruction::InstructionStatus::Active,
        };

        state.add_instruction(reviewable_id.clone(), instruction1);
        state.add_instruction(reviewable_id.clone(), instruction2);

        // Single-note model: the second add folds into the existing note.
        let instructions = state.get_instructions(&reviewable_id).unwrap();
        assert_eq!(instructions.len(), 1);
        assert_eq!(
            instructions[0].content,
            "First instruction\nSecond instruction"
        );
    }

    #[test]
    fn test_decision_instructions_field_initializes_empty() {
        let diff = create_test_reviewable_diff();
        let state = ReviewState::new(vec![diff], "test_author".to_string());

        assert_eq!(state.decision_instructions.total_instructions(), 0);
    }

    #[test]
    fn test_decision_instructions_survives_state_clone() {
        let diff = create_test_reviewable_diff();
        let mut state = ReviewState::new(vec![diff], "test_author".to_string());

        use crate::entities::instruction::{Instruction, InstructionStatus};
        state.decision_instructions.add_instruction(
            1,
            Instruction {
                id: "di_1".to_string(),
                content: "Check this decision".to_string(),
                author: "reviewer".to_string(),
                timestamp: "2024-01-10T10:00:00Z".to_string(),
                status: InstructionStatus::Active,
            },
        );

        let cloned = state.clone();
        assert_eq!(cloned.decision_instructions.total_instructions(), 1);
    }

    #[test]
    fn test_decision_instructions_accessible_through_state() {
        let diff = create_test_reviewable_diff();
        let mut state = ReviewState::new(vec![diff], "test_author".to_string());

        use crate::entities::instruction::{Instruction, InstructionStatus};
        state.decision_instructions.add_instruction(
            42,
            Instruction {
                id: "di_42".to_string(),
                content: "Review decision 42 carefully".to_string(),
                author: "author".to_string(),
                timestamp: "2024-01-10T10:00:00Z".to_string(),
                status: InstructionStatus::Active,
            },
        );

        assert!(state.decision_instructions.has_instructions(&42));
        assert_eq!(
            state
                .decision_instructions
                .get_instructions(&42)
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn test_review_state_with_review_data_includes_decision_instructions() {
        use crate::entities::DecisionInstructions;
        use crate::entities::instruction::{Instruction, InstructionStatus};

        let diff = create_test_reviewable_diff();
        let mut decision_instructions = DecisionInstructions::new();
        decision_instructions.add_instruction(
            1,
            Instruction {
                id: "di_1".to_string(),
                content: "Test decision instruction".to_string(),
                author: "author".to_string(),
                timestamp: "2024-01-10T10:00:00Z".to_string(),
                status: InstructionStatus::Active,
            },
        );

        let state = ReviewState::with_review_data(
            vec![diff],
            "test_author".to_string(),
            ReviewApprovals::new(),
            ReviewInstructions::new(),
            ReviewDecisions::new(),
            DecisionApprovals::new(),
            decision_instructions,
        );

        assert_eq!(state.decision_instructions.total_instructions(), 1);
        assert!(state.decision_instructions.has_instructions(&1));
    }
}
