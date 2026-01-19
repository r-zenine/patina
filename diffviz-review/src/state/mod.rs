//! Review state management with ReviewableDiff-based architecture
//!
//! This module provides centralized state management for review sessions,
//! now organized around ReviewableDiffs as the primary unit of review.

use crate::entities::reviewable_diff_id::{LineRange, ReviewableDiffId};
use crate::entities::{Instruction, ReviewApprovals, ReviewInstructions};
// Simplified structures for review workflow
#[derive(Debug, Clone, Default)]
pub struct ReviewJourney;

impl ReviewJourney {
    pub fn new() -> Self {
        Self
    }

    pub fn something() {}
}

#[derive(Debug, Clone)]
pub struct SessionMetadata {
    pub session_id: String,
    pub created_at: String,
    pub author: String,
}
use std::collections::{BTreeMap, HashMap};

/// Information about an overlap conflict between instructions
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlapConflict {
    /// The ReviewableDiffId of the existing instruction that conflicts
    pub conflicting_id: ReviewableDiffId,
    /// Type of overlap detected
    pub overlap_type: OverlapType,
}

/// Type of range overlap
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OverlapType {
    /// Ranges are exactly the same
    Exact,
    /// Ranges partially overlap
    Partial,
    /// One range is completely contained within another
    Nested,
}

/// Centralized application state containing all business data
/// Now organized around ReviewableDiffs rather than legacy chunks
#[derive(Debug, Clone)]
pub struct ReviewState {
    /// Map of ReviewableDiffs indexed by their unique identifier
    /// Ordered by file path and line range for consistent iteration
    pub reviewable_diffs: BTreeMap<ReviewableDiffId, ReviewableDiff>,

    /// Review progress and decisions
    pub approvals: ReviewApprovals,
    pub instructions: ReviewInstructions,
    pub journey: ReviewJourney,

    /// Session metadata
    pub author: String,
    pub session_metadata: Option<SessionMetadata>,
}

// Import ReviewableDiff from diffviz-core now that it's lifetime-independent
use diffviz_core::reviewable_diff::ReviewableDiff as CoreReviewableDiff;

/// Wrapper for core ReviewableDiff with review-layer specific metadata
#[derive(Debug, Clone)]
pub struct ReviewableDiff {
    pub id: ReviewableDiffId,
    pub core_diff: CoreReviewableDiff,
    pub file_path: String, // Added for review layer convenience
}

impl ReviewableDiff {
    /// Create a new ReviewableDiff from core diff and metadata
    pub fn new(id: ReviewableDiffId, core_diff: CoreReviewableDiff, file_path: String) -> Self {
        Self {
            id,
            core_diff,
            file_path,
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
            journey: ReviewJourney::new(),
            author,
            session_metadata: None,
        }
    }

    /// Create review state with existing review data (for loading sessions)
    pub fn with_review_data(
        reviewable_diffs: Vec<ReviewableDiff>,
        author: String,
        journey: ReviewJourney,
        approvals: ReviewApprovals,
        instructions: ReviewInstructions,
    ) -> Self {
        let mut diffs_map = BTreeMap::new();
        for diff in reviewable_diffs {
            diffs_map.insert(diff.id.clone(), diff);
        }

        Self {
            reviewable_diffs: diffs_map,
            approvals,
            instructions,
            journey,
            author,
            session_metadata: None,
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
    pub fn add_instruction(&mut self, instruction: Instruction) -> &mut Self {
        self.instructions.add_instruction(instruction);
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

    /// Set session metadata
    pub fn with_session_metadata(&mut self, metadata: SessionMetadata) -> &mut Self {
        self.session_metadata = Some(metadata);
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

    /// Check if a new instruction would overlap with existing instructions
    /// Returns Some(OverlapConflict) if overlap detected, None otherwise
    pub fn check_instruction_overlap(
        &self,
        reviewable_id: &ReviewableDiffId,
    ) -> Option<OverlapConflict> {
        // Get all existing instructions
        let all_instructions = self.instructions.get_all_instructions();

        // Check each existing instruction for overlap
        for existing_inst in all_instructions {
            // Skip if same file and query
            if existing_inst.reviewable_id.query != reviewable_id.query
                || existing_inst.reviewable_id.file_path != reviewable_id.file_path
            {
                continue;
            }

            let existing_range = &existing_inst.reviewable_id.line_range;
            let new_range = &reviewable_id.line_range;

            // Check for overlap
            if let Some(overlap_type) = Self::detect_range_overlap(existing_range, new_range) {
                return Some(OverlapConflict {
                    conflicting_id: existing_inst.reviewable_id.clone(),
                    overlap_type,
                });
            }
        }

        None
    }

    /// Detect if two line ranges overlap and determine the type
    fn detect_range_overlap(range1: &LineRange, range2: &LineRange) -> Option<OverlapType> {
        // Check if ranges overlap (inclusive ranges)
        let overlaps = range1.start_line <= range2.end_line && range2.start_line <= range1.end_line;

        if !overlaps {
            return None;
        }

        // Determine type of overlap
        if range1.start_line == range2.start_line && range1.end_line == range2.end_line {
            Some(OverlapType::Exact)
        } else if (range1.start_line >= range2.start_line && range1.end_line <= range2.end_line)
            || (range2.start_line >= range1.start_line && range2.end_line <= range1.end_line)
        {
            Some(OverlapType::Nested)
        } else {
            Some(OverlapType::Partial)
        }
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
            file_path: "test.rs".to_string(),
        }
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
            reviewable_id: reviewable_id.clone(),
            content: "Extract this to a separate function".to_string(),
            author: "reviewer".to_string(),
            timestamp: "2024-01-10T10:00:00Z".to_string(),
            status: crate::entities::instruction::InstructionStatus::Active,
            file_content_hash: String::new(),
            content_snapshot: None,
        };

        state.add_instruction(instruction);

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
            reviewable_id: reviewable_id.clone(),
            content: "First instruction".to_string(),
            author: "reviewer".to_string(),
            timestamp: "2024-01-10T10:00:00Z".to_string(),
            status: crate::entities::instruction::InstructionStatus::Active,
            file_content_hash: String::new(),
            content_snapshot: None,
        };

        let instruction2 = Instruction {
            id: "inst_2".to_string(),
            reviewable_id: reviewable_id.clone(),
            content: "Second instruction".to_string(),
            author: "reviewer".to_string(),
            timestamp: "2024-01-10T10:01:00Z".to_string(),
            status: crate::entities::instruction::InstructionStatus::Active,
            file_content_hash: String::new(),
            content_snapshot: None,
        };

        state.add_instruction(instruction1);
        state.add_instruction(instruction2);

        let instructions = state.get_instructions(&reviewable_id).unwrap();
        assert_eq!(instructions.len(), 2);
        assert_eq!(instructions[0].content, "First instruction");
        assert_eq!(instructions[1].content, "Second instruction");
    }

    // Tests for overlap detection (Phase 1)
    #[test]
    fn test_overlap_detection_exact_same_range() {
        let diff = create_test_reviewable_diff();
        let mut state = ReviewState::new(vec![diff], "test_author".to_string());

        let id1 = ReviewableDiffId::new(
            DiffQuery::head_to_unstaged(),
            "test.rs".to_string(),
            LineRange {
                start_line: 10,
                end_line: 12,
                start_column: 0,
                end_column: 0,
            },
        );

        let id2 = ReviewableDiffId::new(
            DiffQuery::head_to_unstaged(),
            "test.rs".to_string(),
            LineRange {
                start_line: 10,
                end_line: 12,
                start_column: 0,
                end_column: 0,
            },
        );

        // Add first instruction
        use crate::entities::Instruction;
        let inst1 = Instruction {
            id: "inst_1".to_string(),
            reviewable_id: id1.clone(),
            content: "First instruction".to_string(),
            author: "reviewer".to_string(),
            timestamp: "2024-01-10T10:00:00Z".to_string(),
            status: crate::entities::instruction::InstructionStatus::Active,
            file_content_hash: String::new(),
            content_snapshot: None,
        };
        state.add_instruction(inst1);

        // Check overlap for second instruction with same range
        let overlap = state.check_instruction_overlap(&id2);
        assert!(overlap.is_some());
    }

    #[test]
    fn test_overlap_detection_partial_overlap() {
        let diff = create_test_reviewable_diff();
        let mut state = ReviewState::new(vec![diff], "test_author".to_string());

        let id1 = ReviewableDiffId::new(
            DiffQuery::head_to_unstaged(),
            "test.rs".to_string(),
            LineRange {
                start_line: 10,
                end_line: 12,
                start_column: 0,
                end_column: 0,
            },
        );

        let id2 = ReviewableDiffId::new(
            DiffQuery::head_to_unstaged(),
            "test.rs".to_string(),
            LineRange {
                start_line: 11,
                end_line: 13,
                start_column: 0,
                end_column: 0,
            },
        );

        // Add first instruction
        use crate::entities::Instruction;
        let inst1 = Instruction {
            id: "inst_1".to_string(),
            reviewable_id: id1.clone(),
            content: "First instruction".to_string(),
            author: "reviewer".to_string(),
            timestamp: "2024-01-10T10:00:00Z".to_string(),
            status: crate::entities::instruction::InstructionStatus::Active,
            file_content_hash: String::new(),
            content_snapshot: None,
        };
        state.add_instruction(inst1);

        // Check overlap - ranges 10-12 and 11-13 share lines 11 and 12
        let overlap = state.check_instruction_overlap(&id2);
        assert!(overlap.is_some());
    }

    #[test]
    fn test_overlap_detection_adjacent_ranges_no_overlap() {
        let diff = create_test_reviewable_diff();
        let mut state = ReviewState::new(vec![diff], "test_author".to_string());

        let id1 = ReviewableDiffId::new(
            DiffQuery::head_to_unstaged(),
            "test.rs".to_string(),
            LineRange {
                start_line: 10,
                end_line: 12,
                start_column: 0,
                end_column: 0,
            },
        );

        let id2 = ReviewableDiffId::new(
            DiffQuery::head_to_unstaged(),
            "test.rs".to_string(),
            LineRange {
                start_line: 13,
                end_line: 15,
                start_column: 0,
                end_column: 0,
            },
        );

        // Add first instruction
        use crate::entities::Instruction;
        let inst1 = Instruction {
            id: "inst_1".to_string(),
            reviewable_id: id1.clone(),
            content: "First instruction".to_string(),
            author: "reviewer".to_string(),
            timestamp: "2024-01-10T10:00:00Z".to_string(),
            status: crate::entities::instruction::InstructionStatus::Active,
            file_content_hash: String::new(),
            content_snapshot: None,
        };
        state.add_instruction(inst1);

        // Check overlap - ranges 10-12 and 13-15 are adjacent but don't overlap
        let overlap = state.check_instruction_overlap(&id2);
        assert!(overlap.is_none());
    }

    #[test]
    fn test_overlap_detection_distant_ranges_no_overlap() {
        let diff = create_test_reviewable_diff();
        let mut state = ReviewState::new(vec![diff], "test_author".to_string());

        let id1 = ReviewableDiffId::new(
            DiffQuery::head_to_unstaged(),
            "test.rs".to_string(),
            LineRange {
                start_line: 10,
                end_line: 12,
                start_column: 0,
                end_column: 0,
            },
        );

        let id2 = ReviewableDiffId::new(
            DiffQuery::head_to_unstaged(),
            "test.rs".to_string(),
            LineRange {
                start_line: 20,
                end_line: 22,
                start_column: 0,
                end_column: 0,
            },
        );

        // Add first instruction
        use crate::entities::Instruction;
        let inst1 = Instruction {
            id: "inst_1".to_string(),
            reviewable_id: id1.clone(),
            content: "First instruction".to_string(),
            author: "reviewer".to_string(),
            timestamp: "2024-01-10T10:00:00Z".to_string(),
            status: crate::entities::instruction::InstructionStatus::Active,
            file_content_hash: String::new(),
            content_snapshot: None,
        };
        state.add_instruction(inst1);

        // Check overlap - ranges 10-12 and 20-22 are distant, no overlap
        let overlap = state.check_instruction_overlap(&id2);
        assert!(overlap.is_none());
    }

    #[test]
    fn test_overlap_detection_nested_range() {
        let diff = create_test_reviewable_diff();
        let mut state = ReviewState::new(vec![diff], "test_author".to_string());

        let id1 = ReviewableDiffId::new(
            DiffQuery::head_to_unstaged(),
            "test.rs".to_string(),
            LineRange {
                start_line: 10,
                end_line: 15,
                start_column: 0,
                end_column: 0,
            },
        );

        let id2 = ReviewableDiffId::new(
            DiffQuery::head_to_unstaged(),
            "test.rs".to_string(),
            LineRange {
                start_line: 11,
                end_line: 12,
                start_column: 0,
                end_column: 0,
            },
        );

        // Add first instruction with range 10-15
        use crate::entities::Instruction;
        let inst1 = Instruction {
            id: "inst_1".to_string(),
            reviewable_id: id1.clone(),
            content: "First instruction".to_string(),
            author: "reviewer".to_string(),
            timestamp: "2024-01-10T10:00:00Z".to_string(),
            status: crate::entities::instruction::InstructionStatus::Active,
            file_content_hash: String::new(),
            content_snapshot: None,
        };
        state.add_instruction(inst1);

        // Check overlap - range 11-12 is contained within 10-15
        let overlap = state.check_instruction_overlap(&id2);
        assert!(overlap.is_some());
    }

    #[test]
    fn test_overlap_detection_empty_state() {
        let diff = create_test_reviewable_diff();
        let state = ReviewState::new(vec![diff], "test_author".to_string());

        let id = ReviewableDiffId::new(
            DiffQuery::head_to_unstaged(),
            "test.rs".to_string(),
            LineRange {
                start_line: 10,
                end_line: 12,
                start_column: 0,
                end_column: 0,
            },
        );

        // No instructions added, so no overlap
        let overlap = state.check_instruction_overlap(&id);
        assert!(overlap.is_none());
    }
}
