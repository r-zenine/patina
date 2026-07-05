//! Review engine with ReviewableDiff-based architecture
//!
//! This module provides the core business logic for managing reviews
//! using the new ReviewableDiff system with RenderableDiff caching.

use crate::entities::Instruction;
use crate::entities::ReviewApprovals;
use crate::entities::instruction::InstructionStatus;
use crate::entities::reviewable_diff_id::ReviewableDiffId;
use crate::errors::Result;
use crate::state::{ReviewState, ReviewableDiff};
use diffviz_core::renderable_diff::RenderableDiff;
use std::collections::HashMap;

pub mod decision;
pub mod export_import;

pub use export_import::{
    ExportMetadata, ExportedInstruction, ExportedInstructions, ExportedLineRange, ImportSummary,
};

/// Core review engine with ReviewableDiff-based state management
pub struct ReviewEngine {
    state: ReviewState,
    // Cache for RenderableDiffs to speed up TUI interactions
    // Note: RenderableDiff would be imported from diffviz-core in actual implementation
    renderable_cache: HashMap<ReviewableDiffId, String>, // Simplified - would be RenderableDiff
}

impl ReviewEngine {
    /// Create a new review engine with ReviewableDiffs
    pub fn new(reviewable_diffs: Vec<ReviewableDiff>, author: String) -> Self {
        Self {
            state: ReviewState::new(reviewable_diffs, author),
            renderable_cache: HashMap::new(),
        }
    }

    /// Approve a specific ReviewableDiff
    pub fn approve(&mut self, reviewable_id: ReviewableDiffId, reviewer: String) -> Result<()> {
        self.state.approve(reviewable_id.clone(), reviewer.clone());
        self.invalidate_cache(&reviewable_id);

        // Reverse cascade: if all chunks for any decision are now approved, auto-approve the decision
        for decision_num in self.decisions_for_reviewable(&reviewable_id) {
            let (approved, total) = self.state.decision_approval_progress(decision_num);
            if total > 0 && approved == total && !self.state.is_decision_approved(decision_num) {
                self.state.approve_decision(decision_num, reviewer.clone());
            }
        }

        Ok(())
    }

    /// Reject/unapprove a specific ReviewableDiff
    pub fn reject(&mut self, reviewable_id: ReviewableDiffId) -> Result<()> {
        self.state.unapprove(&reviewable_id);
        self.invalidate_cache(&reviewable_id);

        // Reverse cascade: if a decision was approved but now not all chunks are approved, unapprove it
        for decision_num in self.decisions_for_reviewable(&reviewable_id) {
            if self.state.is_decision_approved(decision_num) {
                let (approved, total) = self.state.decision_approval_progress(decision_num);
                if total > 0 && approved < total {
                    self.state.unapprove_decision(decision_num);
                }
            }
        }

        Ok(())
    }

    /// Add an instruction to a specific ReviewableDiff
    pub fn add_instruction(
        &mut self,
        reviewable_id: ReviewableDiffId,
        content: String,
        author: String,
    ) -> Result<()> {
        let instruction = Instruction {
            id: uuid::Uuid::new_v4().to_string(),
            author,
            timestamp: chrono::Utc::now()
                .format("%Y-%m-%d %H:%M:%S UTC")
                .to_string(),
            content,
            status: InstructionStatus::Active,
        };

        self.state
            .add_instruction(reviewable_id.clone(), instruction);
        self.invalidate_cache(&reviewable_id);

        Ok(())
    }

    /// Overwrite the existing instruction for a ReviewableDiff with fully
    /// edited content, rather than appending a new line to it.
    pub fn edit_instruction(
        &mut self,
        reviewable_id: ReviewableDiffId,
        content: String,
        author: String,
    ) -> Result<()> {
        let instruction = Instruction {
            id: uuid::Uuid::new_v4().to_string(),
            author,
            timestamp: chrono::Utc::now()
                .format("%Y-%m-%d %H:%M:%S UTC")
                .to_string(),
            content,
            status: InstructionStatus::Active,
        };

        self.state
            .replace_instruction(reviewable_id.clone(), instruction);
        self.invalidate_cache(&reviewable_id);

        Ok(())
    }

    /// Approve all ReviewableDiffs in a specific file
    pub fn approve_all_in_file(&mut self, file_path: &str, reviewer: String) -> Result<()> {
        let reviewable_ids: Vec<ReviewableDiffId> = self
            .state
            .reviewable_diffs
            .values()
            .filter(|diff| diff.file_path == file_path)
            .map(|diff| diff.id.clone())
            .collect();

        for reviewable_id in &reviewable_ids {
            self.invalidate_cache(reviewable_id);
        }

        self.state.approve_all_in_file(file_path, reviewer);

        Ok(())
    }

    fn invalidate_cache(&mut self, id: &ReviewableDiffId) {
        self.renderable_cache.remove(id);
    }

    fn decisions_for_reviewable(&self, id: &ReviewableDiffId) -> Vec<u32> {
        self.state
            .decisions
            .decision_index
            .get(id)
            .cloned()
            .unwrap_or_default()
    }

    fn compute_renderable(&self, id: &ReviewableDiffId) -> Option<String> {
        self.state.get_reviewable_diff(id).map(|diff| {
            format_renderable_diff_for_display(
                &RenderableDiff::try_from(&diff.core_diff)
                    .expect("ReviewableDiff should produce valid RenderableDiff"),
            )
        })
    }

    /// Get a RenderableDiff for a ReviewableDiff (with caching)
    pub fn get_renderable_diff(&mut self, reviewable_id: &ReviewableDiffId) -> Option<String> {
        if let Some(cached) = self.renderable_cache.get(reviewable_id) {
            return Some(cached.clone());
        }
        if let Some(renderable) = self.compute_renderable(reviewable_id) {
            self.renderable_cache
                .insert(reviewable_id.clone(), renderable.clone());
            Some(renderable)
        } else {
            None
        }
    }

    /// Get a RenderableDiff object for direct widget usage
    pub fn get_renderable_diff_object(
        &self,
        reviewable_id: &ReviewableDiffId,
    ) -> Option<RenderableDiff<'_>> {
        self.state
            .get_reviewable_diff(reviewable_id)
            .map(|reviewable_diff| {
                RenderableDiff::try_from(&reviewable_diff.core_diff)
                    .expect("ReviewableDiff should produce valid RenderableDiff")
            })
    }

    /// Get all ReviewableDiffs grouped by file
    pub fn get_reviewable_diffs_by_file(&self) -> HashMap<String, Vec<&ReviewableDiff>> {
        self.state.get_reviewable_diffs_by_file()
    }

    /// Get all ReviewableDiff IDs ordered by file and line range
    pub fn get_ordered_reviewable_ids(&self) -> Vec<&ReviewableDiffId> {
        self.state.get_ordered_reviewable_ids()
    }

    /// Get review progress statistics
    pub fn get_review_progress(&self) -> ReviewProgress {
        let (approved, total, percentage) = self.state.approval_progress();
        ReviewProgress {
            total_reviewable_diffs: total,
            approved_reviewable_diffs: approved,
            approval_percentage: percentage,
            total_instructions: self.state.instructions.total_instructions(),
        }
    }

    /// Get reference to the centralized state
    pub fn state(&self) -> &ReviewState {
        &self.state
    }

    /// Get the current author
    pub fn author(&self) -> &str {
        self.state.author()
    }

    /// Load persisted approvals into the engine, replacing current approval state
    pub fn load_approvals(&mut self, approvals: ReviewApprovals) {
        self.state.approvals = approvals;
    }

    /// Load persisted decision approvals into the engine, replacing current decision approval state
    pub fn load_decision_approvals(
        &mut self,
        decision_approvals: crate::entities::DecisionApprovals,
    ) {
        self.state.decision_approvals = decision_approvals;
    }

    /// Get all unique file paths in this review
    pub fn get_file_paths(&self) -> Vec<String> {
        self.state.get_file_paths()
    }

    /// Clear the RenderableDiff cache
    pub fn clear_cache(&mut self) {
        self.renderable_cache.clear();
    }

    /// Get a specific ReviewableDiff by ID
    pub fn get_reviewable_diff(&self, id: &ReviewableDiffId) -> Option<&ReviewableDiff> {
        self.state.get_reviewable_diff(id)
    }
}

/// Review progress information
#[derive(Debug, Clone)]
pub struct ReviewProgress {
    pub total_reviewable_diffs: usize,
    pub approved_reviewable_diffs: usize,
    pub approval_percentage: f32,
    pub total_instructions: usize,
}

fn format_renderable_diff_for_display(renderable_diff: &RenderableDiff) -> String {
    let mut output = String::new();

    output.push_str(&format!(
        "=== {} ===\n",
        renderable_diff.metadata.boundary_name
    ));
    output.push_str(&format!("Language: {:?}\n", renderable_diff.language));
    output.push_str(&format!(
        "Lines: {} | Essential: {}\n\n",
        renderable_diff.lines.len(),
        renderable_diff.metadata.essential_line_count
    ));

    for line in &renderable_diff.lines {
        let change_indicator = if line.annotations.iter().any(|a| a.change_type.is_some()) {
            match line.annotations.iter().find(|a| a.change_type.is_some()) {
                Some(annotation) => match annotation.change_type {
                    Some(diffviz_core::renderable_diff::ChangeType::Added) => "+ ",
                    Some(diffviz_core::renderable_diff::ChangeType::Deleted) => "- ",
                    Some(diffviz_core::renderable_diff::ChangeType::Modified) => "~ ",
                    _ => "  ",
                },
                None => "  ",
            }
        } else {
            "  "
        };

        output.push_str(&format!(
            "{:4} {}{}\n",
            line.line_number, change_indicator, line.content
        ));
    }

    output
}

#[cfg(test)]
pub mod test_helpers {
    use crate::entities::git_ref::DiffQuery;
    use crate::entities::reviewable_diff_id::{LineRange, ReviewableDiffId};
    use crate::state::ReviewableDiff;

    pub fn test_range(start: usize, end: usize) -> LineRange {
        LineRange {
            start_line: start,
            end_line: end,
            start_column: 0,
            end_column: 0,
        }
    }

    pub fn test_id(file: &str, start: usize, end: usize) -> ReviewableDiffId {
        ReviewableDiffId::new(
            DiffQuery::head_to_unstaged(),
            file.to_string(),
            test_range(start, end),
        )
    }

    pub fn create_test_reviewable_diff(file_path: &str, start_line: usize) -> ReviewableDiff {
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
            test_range(start_line, start_line + 10),
        );

        let placeholder_content = format!("test content for {file_path}");
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
                        qualified_name: None,
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
}

#[cfg(test)]
mod tests {
    use super::test_helpers::*;
    use super::*;

    #[test]
    fn test_review_engine_creation() {
        let diffs = vec![
            create_test_reviewable_diff("test1.rs", 1),
            create_test_reviewable_diff("test2.rs", 1),
        ];
        let engine = ReviewEngine::new(diffs, "test_author".to_string());

        assert_eq!(engine.state.total_reviewable_diffs(), 2);
        assert_eq!(engine.author(), "test_author");
    }

    #[test]
    fn test_approve_reviewable_diff() {
        let diff = create_test_reviewable_diff("test.rs", 1);
        let reviewable_id = diff.id.clone();
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        let result = engine.approve(reviewable_id.clone(), "reviewer".to_string());
        assert!(result.is_ok());
        assert!(engine.state.is_approved(&reviewable_id));
    }

    #[test]
    fn test_renderable_diff_caching() {
        let diff = create_test_reviewable_diff("test.rs", 1);
        let reviewable_id = diff.id.clone();
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        // First call should generate and cache
        let renderable1 = engine.get_renderable_diff(&reviewable_id);
        assert!(renderable1.is_some());

        // Second call should use cache
        let renderable2 = engine.get_renderable_diff(&reviewable_id);
        assert_eq!(renderable1, renderable2);

        assert_eq!(engine.renderable_cache.len(), 1);
    }

    #[test]
    fn test_cache_invalidation() {
        let diff = create_test_reviewable_diff("test.rs", 1);
        let reviewable_id = diff.id.clone();
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        engine.get_renderable_diff(&reviewable_id);
        assert_eq!(engine.renderable_cache.len(), 1);

        // Approval should invalidate cache
        engine
            .approve(reviewable_id.clone(), "reviewer".to_string())
            .unwrap();
        assert_eq!(engine.renderable_cache.len(), 0);
    }

    #[test]
    fn test_approve_all_in_file() {
        let diffs = vec![
            create_test_reviewable_diff("test1.rs", 1),
            create_test_reviewable_diff("test1.rs", 20),
            create_test_reviewable_diff("test2.rs", 1),
        ];
        let mut engine = ReviewEngine::new(diffs, "test_author".to_string());

        let result = engine.approve_all_in_file("test1.rs", "reviewer".to_string());
        assert!(result.is_ok());

        // Check that only test1.rs diffs are approved
        let by_file = engine.get_reviewable_diffs_by_file();
        let test1_diffs = by_file.get("test1.rs").unwrap();
        let test2_diffs = by_file.get("test2.rs").unwrap();

        for diff in test1_diffs {
            assert!(engine.state.is_approved(&diff.id));
        }
        for diff in test2_diffs {
            assert!(!engine.state.is_approved(&diff.id));
        }
    }

    #[test]
    fn test_review_progress() {
        let diffs = vec![
            create_test_reviewable_diff("test1.rs", 1),
            create_test_reviewable_diff("test2.rs", 1),
        ];
        let mut engine = ReviewEngine::new(diffs.clone(), "test_author".to_string());

        let progress = engine.get_review_progress();
        assert_eq!(progress.total_reviewable_diffs, 2);
        assert_eq!(progress.approved_reviewable_diffs, 0);
        assert_eq!(progress.approval_percentage, 0.0);

        // Approve one diff
        engine
            .approve(diffs[0].id.clone(), "reviewer".to_string())
            .unwrap();

        let progress = engine.get_review_progress();
        assert_eq!(progress.approved_reviewable_diffs, 1);
        assert_eq!(progress.approval_percentage, 50.0);
    }

    #[test]
    fn test_get_renderable_diff_object() {
        let diff = create_test_reviewable_diff("test.rs", 1);
        let reviewable_id = diff.id.clone();
        let engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        // Test the get_renderable_diff_object method
        let renderable_diff = engine.get_renderable_diff_object(&reviewable_id);
        assert!(renderable_diff.is_some());

        let renderable = renderable_diff.unwrap();

        // Should have the expected language and lines
        assert_eq!(
            renderable.language,
            diffviz_core::common::ProgrammingLanguage::Rust
        );
        assert!(!renderable.lines.is_empty());

        // Should have expected metadata
        assert_eq!(
            renderable.metadata.boundary_name,
            "test content for test.rs"
        );
    }

    // Tests for overlap detection in ReviewEngine (Phase 1)
    #[test]
    fn test_add_instruction_without_overlap() {
        let diff = create_test_reviewable_diff("test.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        let reviewable_id = test_id("test.rs", 10, 12);

        let result = engine.add_instruction(
            reviewable_id.clone(),
            "Extract this to a separate function".to_string(),
            "reviewer".to_string(),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_add_instruction_non_overlapping_ranges() {
        let diff = create_test_reviewable_diff("test.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        engine
            .add_instruction(
                test_id("test.rs", 10, 12),
                "First instruction".to_string(),
                "reviewer".to_string(),
            )
            .unwrap();

        let result = engine.add_instruction(
            test_id("test.rs", 20, 22),
            "Second instruction".to_string(),
            "reviewer".to_string(),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_add_two_instructions_to_exact_same_range_folds_into_one_note() {
        let diff = create_test_reviewable_diff("test.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        let id = test_id("test.rs", 10, 20);

        engine
            .add_instruction(
                id.clone(),
                "First instruction".to_string(),
                "reviewer".to_string(),
            )
            .unwrap();
        engine
            .add_instruction(
                id.clone(),
                "Second instruction".to_string(),
                "reviewer".to_string(),
            )
            .unwrap();

        // Single-note model: editing means appending to the existing note.
        assert_eq!(engine.state().instructions.total_instructions(), 1);
        let notes = engine.state().get_instructions(&id).unwrap();
        assert_eq!(notes[0].content, "First instruction\nSecond instruction");
    }

    #[test]
    fn test_add_instruction_adjacent_ranges_remain_separate() {
        let diff = create_test_reviewable_diff("test.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        engine
            .add_instruction(
                test_id("test.rs", 10, 12),
                "First instruction".to_string(),
                "reviewer".to_string(),
            )
            .unwrap();

        let result = engine.add_instruction(
            test_id("test.rs", 13, 15),
            "Second instruction".to_string(),
            "reviewer".to_string(),
        );

        assert!(result.is_ok());
    }
}
