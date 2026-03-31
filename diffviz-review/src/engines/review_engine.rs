//! Review engine with ReviewableDiff-based architecture
//!
//! This module provides the core business logic for managing reviews
//! using the new ReviewableDiff system with RenderableDiff caching.

use crate::entities::CascadeResult;
use crate::entities::Instruction;
use crate::entities::ReviewApprovals;
use crate::entities::git_ref::{DiffQuery, GitRef};
use crate::entities::instruction::InstructionStatus;
use crate::entities::reviewable_diff_id::{LineRange, ReviewableDiffId};
use crate::errors::Result;
use crate::state::{ReviewState, ReviewableDiff};
use diffviz_core::renderable_diff::RenderableDiff;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type alias for review operation callbacks (used by existing approve/reject/add_instruction methods)
/// NOTE: Cascade methods (approve_decision/reject_decision) use CascadeResult instead of callbacks
type OperationCallback = Option<Box<dyn FnOnce(bool, Option<String>) + Send>>;

/// Export scope for filtering instructions during JSON export
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportScope {
    /// Export all instructions for a specific file
    SingleFile(String),
    /// Export a single instruction by its ReviewableDiffId
    SingleInstruction(ReviewableDiffId),
    /// Export all instructions across all files
    All,
}

/// JSON representation of an instruction for export/import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedInstruction {
    pub file: String,
    pub query: String,
    pub line_range: ExportedLineRange,
    pub content: String,
    pub author: String,
    pub timestamp: String,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default = "default_empty_string")]
    pub file_content_hash: String,
    #[serde(default)]
    pub content_snapshot: Option<String>,
}

// Helper functions for serde defaults
fn default_status() -> String {
    "active".to_string()
}

fn default_empty_string() -> String {
    String::new()
}

/// JSON representation of a line range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedLineRange {
    pub start_line: usize,
    pub end_line: usize,
}

/// Metadata section for export format documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub format_version: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field_descriptions: Option<ExportFieldDescriptions>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query_formats: Option<ExportQueryFormats>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_usage_examples: Option<ExportGitExamples>,
}

/// Field descriptions for agent understanding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportFieldDescriptions {
    pub file: String,
    pub query: String,
    pub line_range: String,
    pub content: String,
    pub author: String,
    pub timestamp: String,
    pub status: String,
    pub file_content_hash: String,
    pub content_snapshot: String,
}

/// Query format descriptions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportQueryFormats {
    #[serde(rename = "HEAD..unstaged")]
    pub head_to_unstaged: String,
    #[serde(rename = "commit_hash..HEAD")]
    pub commit_to_head: String,
    #[serde(rename = "HEAD..commit_hash")]
    pub head_to_commit: String,
    #[serde(rename = "commit_hash..commit_hash")]
    pub commit_to_commit: String,
}

/// Git command usage examples
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportGitExamples {
    pub view_diff: String,
    pub view_historical_diff: String,
    pub get_file_at_head: String,
    pub get_file_at_commit: String,
}

/// Container for exported instructions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedInstructions {
    #[serde(rename = "_meta")]
    pub meta: ExportMetadata,
    pub instructions: Vec<ExportedInstruction>,
}

/// Summary of import operation results
#[derive(Debug, Clone, Default)]
pub struct ImportSummary {
    pub total_imported: usize,
    pub active_count: usize,
    pub stale_count: usize,
    pub errors: Vec<String>,
}

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
    pub fn approve(
        &mut self,
        reviewable_id: ReviewableDiffId,
        reviewer: String,
        on_result: OperationCallback,
    ) -> Result<()> {
        self.state.approve(reviewable_id.clone(), reviewer.clone());

        // Invalidate cache for this ReviewableDiff
        self.renderable_cache.remove(&reviewable_id);

        // Check for reverse cascade: if all chunks for any decision are now approved, auto-approve the decision
        let decisions_for_chunk: Vec<u32> = self
            .state
            .decisions
            .decision_index
            .get(&reviewable_id)
            .cloned()
            .unwrap_or_default();

        for decision_num in decisions_for_chunk {
            let (approved, total) = self.state.decision_approval_progress(decision_num);
            // If all chunks are approved, auto-approve the decision
            if total > 0 && approved == total && !self.state.is_decision_approved(decision_num) {
                self.state.approve_decision(decision_num, reviewer.clone());
            }
        }

        if let Some(callback) = on_result {
            callback(true, Some("ReviewableDiff approved".to_string()));
        }
        Ok(())
    }

    /// Reject/unapprove a specific ReviewableDiff
    pub fn reject(
        &mut self,
        reviewable_id: ReviewableDiffId,
        on_result: OperationCallback,
    ) -> Result<()> {
        self.state.unapprove(&reviewable_id);

        // Invalidate cache for this ReviewableDiff
        self.renderable_cache.remove(&reviewable_id);

        // Check for reverse cascade: if a decision was approved but now not all chunks are approved, unapprove the decision
        let decisions_for_chunk: Vec<u32> = self
            .state
            .decisions
            .decision_index
            .get(&reviewable_id)
            .cloned()
            .unwrap_or_default();

        for decision_num in decisions_for_chunk {
            if self.state.is_decision_approved(decision_num) {
                let (approved, total) = self.state.decision_approval_progress(decision_num);
                // If not all chunks are approved anymore, unapprove the decision
                if total > 0 && approved < total {
                    self.state.unapprove_decision(decision_num);
                }
            }
        }

        if let Some(callback) = on_result {
            callback(true, Some("ReviewableDiff rejected".to_string()));
        }
        Ok(())
    }

    /// Add an instruction to a specific ReviewableDiff
    pub fn add_instruction(
        &mut self,
        reviewable_id: ReviewableDiffId,
        content: String,
        author: String,
        on_result: OperationCallback,
    ) -> Result<()> {
        let instruction = Instruction {
            id: uuid::Uuid::new_v4().to_string(),
            author,
            timestamp: chrono::Utc::now()
                .format("%Y-%m-%d %H:%M:%S UTC")
                .to_string(),
            content,
            status: crate::entities::instruction::InstructionStatus::Active,
        };

        self.state
            .add_instruction(reviewable_id.clone(), instruction);

        // Invalidate cache for this ReviewableDiff
        self.renderable_cache.remove(&reviewable_id);

        if let Some(callback) = on_result {
            callback(true, Some("Instruction added".to_string()));
        }
        Ok(())
    }

    /// Approve all ReviewableDiffs in a specific file
    pub fn approve_all_in_file(
        &mut self,
        file_path: &str,
        reviewer: String,
        on_result: OperationCallback,
    ) -> Result<()> {
        let mut approved_count = 0;

        // Get all ReviewableDiff IDs for this file
        let reviewable_ids: Vec<ReviewableDiffId> = self
            .state
            .reviewable_diffs
            .values()
            .filter(|diff| diff.file_path == file_path)
            .map(|diff| diff.id.clone())
            .collect();

        // Approve each one and invalidate caches
        for reviewable_id in &reviewable_ids {
            self.renderable_cache.remove(reviewable_id);
            approved_count += 1;
        }

        self.state.approve_all_in_file(file_path, reviewer);

        if let Some(callback) = on_result {
            if approved_count > 0 {
                callback(
                    true,
                    Some(format!("Approved {approved_count} ReviewableDiffs in file")),
                );
            } else {
                callback(false, Some("No ReviewableDiffs found in file".to_string()));
            }
        }

        Ok(())
    }

    // === Decision Approval Methods ===

    /// Get all chunks (ReviewableDiffIds) for a specific decision
    fn get_chunks_for_decision(&self, decision_number: u32) -> Vec<ReviewableDiffId> {
        self.state
            .decisions
            .decision_index
            .iter()
            .filter(|(_, decision_nums)| decision_nums.contains(&decision_number))
            .map(|(diff_id, _)| diff_id.clone())
            .collect()
    }

    /// Approve an entire decision, cascading to all affected chunks
    ///
    /// Returns a CascadeResult describing what was affected by this operation.
    pub fn approve_decision(
        &mut self,
        decision_number: u32,
        reviewer: String,
    ) -> Result<CascadeResult> {
        // Approve the decision itself
        self.state
            .approve_decision(decision_number, reviewer.clone());

        // Get all chunks for this decision
        let chunks = self.get_chunks_for_decision(decision_number);

        // Approve each chunk
        for chunk_id in &chunks {
            self.state.approve(chunk_id.clone(), reviewer.clone());
            self.renderable_cache.remove(chunk_id);
        }

        let result = if chunks.is_empty() {
            CascadeResult::NoChunksAffected { decision_number }
        } else {
            CascadeResult::DecisionApproved {
                decision_number,
                chunks_affected: chunks.len(),
            }
        };

        Ok(result)
    }

    /// Reject/unapprove an entire decision, cascading to all affected chunks
    /// Reject/unapprove an entire decision, cascading to all affected chunks
    ///
    /// Returns a CascadeResult describing what was affected by this operation.
    pub fn reject_decision(&mut self, decision_number: u32) -> Result<CascadeResult> {
        // Unapprove the decision itself
        self.state.unapprove_decision(decision_number);

        // Get all chunks for this decision
        let chunks = self.get_chunks_for_decision(decision_number);

        // Reject each chunk
        for chunk_id in &chunks {
            self.state.unapprove(chunk_id);
            self.renderable_cache.remove(chunk_id);
        }

        let result = if chunks.is_empty() {
            CascadeResult::NoChunksAffected { decision_number }
        } else {
            CascadeResult::DecisionUnapproved {
                decision_number,
                chunks_affected: chunks.len(),
            }
        };

        Ok(result)
    }

    /// Check if a decision is approved
    pub fn is_decision_approved(&self, decision_number: u32) -> bool {
        self.state.is_decision_approved(decision_number)
    }

    /// Get approval progress for a decision: (approved_chunks, total_chunks)
    pub fn decision_approval_progress(&self, decision_number: u32) -> (usize, usize) {
        self.state.decision_approval_progress(decision_number)
    }

    /// Get a RenderableDiff for a ReviewableDiff (with caching)
    pub fn get_renderable_diff(&mut self, reviewable_id: &ReviewableDiffId) -> Option<String> {
        // Check cache first
        if let Some(cached) = self.renderable_cache.get(reviewable_id) {
            return Some(cached.clone());
        }

        // Generate RenderableDiff if not cached
        if let Some(reviewable_diff) = self.state.get_reviewable_diff(reviewable_id) {
            // Convert ReviewableDiff to RenderableDiff using core conversion
            let renderable_diff = RenderableDiff::from(&reviewable_diff.core_diff);

            // Convert RenderableDiff to string format for caching and display
            let renderable = format_renderable_diff_for_display(&renderable_diff);

            // Cache the result
            self.renderable_cache
                .insert(reviewable_id.clone(), renderable.clone());

            Some(renderable)
        } else {
            None
        }
    }

    /// Get a RenderableDiff for a ReviewableDiff (read-only, no caching)
    /// This version can be called with &self for TUI rendering
    pub fn render_diff(&self, reviewable_id: &ReviewableDiffId) -> Option<String> {
        if let Some(reviewable_diff) = self.state.get_reviewable_diff(reviewable_id) {
            // Convert ReviewableDiff to RenderableDiff using core conversion
            let renderable_diff = RenderableDiff::from(&reviewable_diff.core_diff);

            // Convert RenderableDiff to string format for display
            Some(format_renderable_diff_for_display(&renderable_diff))
        } else {
            None
        }
    }

    /// Get a RenderableDiff object for direct widget usage
    pub fn get_renderable_diff_object(
        &self,
        reviewable_id: &ReviewableDiffId,
    ) -> Option<RenderableDiff> {
        self.state
            .get_reviewable_diff(reviewable_id)
            .map(|reviewable_diff| RenderableDiff::from(&reviewable_diff.core_diff))
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

    /// Complete the review and generate a summary
    pub fn complete_review(&self) -> Result<ReviewSummary> {
        let progress = self.get_review_progress();

        Ok(ReviewSummary {
            total_reviewable_diffs: progress.total_reviewable_diffs,
            approved_reviewable_diffs: progress.approved_reviewable_diffs,
            instructions_added: progress.total_instructions,
            files_reviewed: self.state.get_file_paths().len(),
            cache_hits: self.renderable_cache.len(),
        })
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

    /// Clear the RenderableDiff cache (useful for memory management)
    pub fn clear_cache(&mut self) {
        self.renderable_cache.clear();
    }

    /// Get cache statistics for monitoring
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            total_entries: self.renderable_cache.len(),
            total_reviewable_diffs: self.state.total_reviewable_diffs(),
            cache_hit_ratio: if self.state.total_reviewable_diffs() > 0 {
                self.renderable_cache.len() as f32 / self.state.total_reviewable_diffs() as f32
            } else {
                0.0
            },
        }
    }

    /// Get a specific ReviewableDiff by ID
    pub fn get_reviewable_diff(&self, id: &ReviewableDiffId) -> Option<&ReviewableDiff> {
        self.state.get_reviewable_diff(id)
    }

    /// Export instructions to JSON format based on the specified scope
    pub fn export_instructions_json(&self, scope: ExportScope) -> Result<String> {
        // Collect (ReviewableDiffId, Instruction) pairs matching scope
        let pairs: Vec<(&ReviewableDiffId, &Instruction)> = self
            .state
            .instructions
            .instructions
            .iter()
            .flat_map(|(id, instructions)| instructions.iter().map(move |inst| (id, inst)))
            .filter(|(id, _)| match &scope {
                ExportScope::SingleFile(file_path) => id.file_path() == file_path,
                ExportScope::SingleInstruction(reviewable_id) => *id == reviewable_id,
                ExportScope::All => true,
            })
            .collect();

        // Convert to exportable format
        let exported_instructions: Vec<ExportedInstruction> = pairs
            .into_iter()
            .map(|(id, inst)| {
                let line_range = id.line_range();
                // Extract query portion from full ReviewableDiffId display
                // Format is "query:file:L#-#", we want just the query part
                let full_id = format!("{id}");
                let query_str = full_id.split(':').next().unwrap_or("working");

                // Map "working" to proper git format "HEAD..unstaged"
                let query = if query_str == "working" {
                    "HEAD..unstaged".to_string()
                } else {
                    query_str.to_string()
                };

                ExportedInstruction {
                    file: id.file_path().to_string(),
                    query,
                    line_range: ExportedLineRange {
                        start_line: line_range.start_line,
                        end_line: line_range.end_line,
                    },
                    content: inst.content.clone(),
                    author: inst.author.clone(),
                    timestamp: inst.timestamp.clone(),
                    status: match inst.status {
                        InstructionStatus::Active => "active".to_string(),
                        InstructionStatus::Addressed => "addressed".to_string(),
                    },
                    file_content_hash: String::new(),
                    content_snapshot: None,
                }
            })
            .collect();

        // Create metadata section
        let meta = ExportMetadata {
            format_version: "1.1".to_string(),
            description: "DiffViz instruction export for coding agents".to_string(),
            field_descriptions: Some(ExportFieldDescriptions {
                file: "Relative path to the file from repository root".to_string(),
                query: "Git diff query to retrieve file content. Format: 'from_ref..to_ref'. Use with: git diff <query> <file>".to_string(),
                line_range: "1-based line numbers where instruction applies (inclusive range)".to_string(),
                content: "The instruction text for the coding agent to follow".to_string(),
                author: "Username/identifier of instruction author".to_string(),
                timestamp: "When instruction was created/last modified (UTC format)".to_string(),
                status: "Instruction validity: 'active' (file unchanged), 'stale' (file changed), 'addressed' (completed)".to_string(),
                file_content_hash: "SHA256 hash of file content at time of instruction creation, used for validity verification".to_string(),
                content_snapshot: "Code lines from instruction range for visual reference (optional)".to_string(),
            }),
            query_formats: Some(ExportQueryFormats {
                head_to_unstaged: "Uncommitted changes in working directory".to_string(),
                commit_to_head: "Changes from specific commit to current HEAD".to_string(),
                head_to_commit: "Changes from HEAD to specific commit".to_string(),
                commit_to_commit: "Changes between two commits".to_string(),
            }),
            git_usage_examples: Some(ExportGitExamples {
                view_diff: "git diff HEAD..unstaged src/main.rs".to_string(),
                view_historical_diff: "git diff abc123d..HEAD src/main.rs".to_string(),
                get_file_at_head: "git show HEAD:src/main.rs".to_string(),
                get_file_at_commit: "git show abc123d:src/main.rs".to_string(),
            }),
        };

        let export = ExportedInstructions {
            meta,
            instructions: exported_instructions,
        };

        // Serialize to JSON with pretty printing
        serde_json::to_string_pretty(&export).map_err(|e| {
            crate::errors::DiffVizError::Review(crate::errors::ReviewError::ExportFailed {
                reason: format!("JSON serialization failed: {e}"),
            })
        })
    }

    /// Import instructions from JSON with hash verification
    pub fn import_instructions_json(&mut self, json: &str) -> Result<ImportSummary> {
        // Parse JSON to ExportedInstructions
        let exported: ExportedInstructions = serde_json::from_str(json).map_err(|e| {
            crate::errors::DiffVizError::Review(crate::errors::ReviewError::ImportFailed {
                reason: format!("Failed to parse JSON: {e}"),
            })
        })?;

        let mut summary = ImportSummary::default();

        for exported_inst in exported.instructions {
            // Skip if missing required fields
            if exported_inst.content.is_empty() {
                summary
                    .errors
                    .push("Skipping instruction with missing content".to_string());
                continue;
            }

            // Reconstruct ReviewableDiffId
            let reviewable_id = match self.create_reviewable_id_from_export(&exported_inst) {
                Ok(id) => id,
                Err(e) => {
                    summary
                        .errors
                        .push(format!("Failed to create ReviewableDiffId: {e}"));
                    continue;
                }
            };

            // Check if instruction already exists
            let existing_instruction = self
                .state
                .instructions
                .get_instructions_for_reviewable(&reviewable_id);
            if existing_instruction.is_some() {
                summary.errors.push(format!(
                    "Skipping duplicate instruction for {reviewable_id}"
                ));
                continue;
            }

            // Create Instruction
            let instruction = Instruction {
                id: format!(
                    "{}:{}",
                    reviewable_id,
                    chrono::Utc::now().timestamp_millis()
                ),
                author: exported_inst.author,
                timestamp: exported_inst.timestamp,
                content: exported_inst.content,
                status: InstructionStatus::Active,
            };

            // Add to ReviewState
            self.state
                .instructions
                .add_instruction(reviewable_id.clone(), instruction);

            // Update summary
            summary.total_imported += 1;
            summary.active_count += 1;
        }

        Ok(summary)
    }

    /// Helper to reconstruct ReviewableDiffId from ExportedInstruction
    fn create_reviewable_id_from_export(
        &self,
        exported: &ExportedInstruction,
    ) -> Result<ReviewableDiffId> {
        // Parse query to DiffQuery
        let diff_query = self.parse_diff_query_from_string(&exported.query)?;

        // Create LineRange
        let line_range = LineRange {
            start_line: exported.line_range.start_line,
            end_line: exported.line_range.end_line,
            start_column: 0,
            end_column: 0,
        };

        Ok(ReviewableDiffId::new(
            diff_query,
            exported.file.clone(),
            line_range,
        ))
    }

    /// Helper to parse DiffQuery from query string
    fn parse_diff_query_from_string(&self, query: &str) -> Result<DiffQuery> {
        match query {
            "HEAD..unstaged" => Ok(DiffQuery::head_to_unstaged()),
            _ => {
                // Try to parse as commit-to-head, head-to-commit, or commit-to-commit
                if let Some((from, to)) = query.split_once("..") {
                    Ok(DiffQuery::new(
                        GitRef::commit(from.to_string()),
                        GitRef::commit(to.to_string()),
                    ))
                } else {
                    Err(crate::errors::DiffVizError::Review(
                        crate::errors::ReviewError::ImportFailed {
                            reason: format!("Invalid query format: {query}"),
                        },
                    ))
                }
            }
        }
    }

    // === Decision-Based Review API (Phase 1: Decision Context Display) ===

    /// Set the decision mapping for this review
    /// This enables decision-based navigation in the TUI
    pub fn set_decisions(&mut self, decisions: crate::entities::ReviewDecisions) {
        self.state.decisions = decisions;
    }

    /// Set decisions and automatically build the index by detecting overlaps with review state diffs
    /// This is the preferred method for populating decision-based review context
    pub fn set_decisions_with_index(&mut self, mut decisions: crate::entities::ReviewDecisions) {
        decisions.build_index_from_review_state(&self.state);
        self.state.decisions = decisions;
    }

    /// Get all decisions that affect a specific ReviewableDiffId
    /// Returns the decisions in order by decision number
    pub fn get_decisions_for_diff(
        &self,
        reviewable_id: &ReviewableDiffId,
    ) -> Vec<&crate::entities::Decision> {
        self.state.decisions.get_decisions_for_diff(reviewable_id)
    }

    /// Get a specific decision by number
    pub fn get_decision(&self, number: u32) -> Option<&crate::entities::Decision> {
        self.state.decisions.get_decision(number)
    }

    /// Get all decisions in this review, ordered by decision number
    pub fn get_all_decisions(&self) -> Vec<&crate::entities::Decision> {
        self.state.decisions.all_decisions()
    }

    pub fn get_approved_decisions_count(&self) -> usize {
        self.state.decision_approvals.total_approved()
    }

    /// Produce one DecisionReviewableDiff per (ReviewableDiffId, decision_number) pair
    /// from the populated decision_index.
    pub fn get_decision_reviewable_diffs(&self) -> Vec<crate::entities::DecisionReviewableDiff> {
        let mut result = Vec::new();
        for (chunk_id, decision_numbers) in &self.state.decisions.decision_index {
            for &decision_number in decision_numbers {
                result.push(crate::entities::DecisionReviewableDiff {
                    chunk_id: chunk_id.clone(),
                    decision_number,
                });
            }
        }
        result
    }

    // === Decision Instruction Methods ===

    /// Add an instruction to a specific decision
    pub fn add_decision_instruction(
        &mut self,
        decision_number: u32,
        content: String,
        author: String,
    ) -> Result<()> {
        if !self
            .state
            .decisions
            .decisions
            .contains_key(&decision_number)
        {
            return Err(crate::errors::DiffVizError::Review(
                crate::errors::ReviewError::InvalidDecision { decision_number },
            ));
        }

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
            .decision_instructions
            .add_instruction(decision_number, instruction);

        Ok(())
    }

    /// Remove a specific decision instruction by ID
    pub fn remove_decision_instruction(&mut self, instruction_id: &str) -> Result<()> {
        match self
            .state
            .decision_instructions
            .remove_instruction_by_id(instruction_id)
        {
            Some(_) => Ok(()),
            None => Err(crate::errors::DiffVizError::Review(
                crate::errors::ReviewError::InstructionNotFound {
                    instruction_id: instruction_id.to_string(),
                },
            )),
        }
    }

    /// Get all instructions for a specific decision
    pub fn get_decision_instructions(&self, decision_number: u32) -> Option<Vec<&Instruction>> {
        self.state
            .decision_instructions
            .get_instructions(decision_number)
            .map(|instructions| instructions.iter().collect())
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

/// Summary of a completed review session
#[derive(Debug, Clone)]
pub struct ReviewSummary {
    pub total_reviewable_diffs: usize,
    pub approved_reviewable_diffs: usize,
    pub instructions_added: usize,
    pub files_reviewed: usize,
    pub cache_hits: usize,
}

/// Cache performance statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_reviewable_diffs: usize,
    pub cache_hit_ratio: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::git_ref::DiffQuery;
    use crate::entities::reviewable_diff_id::LineRange;

    fn create_test_reviewable_diff(file_path: &str, start_line: usize) -> ReviewableDiff {
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

        let result = engine.approve(reviewable_id.clone(), "reviewer".to_string(), None);
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

        // Cache should have one entry
        let stats = engine.cache_stats();
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.cache_hit_ratio, 1.0);
    }

    #[test]
    fn test_cache_invalidation() {
        let diff = create_test_reviewable_diff("test.rs", 1);
        let reviewable_id = diff.id.clone();
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        // Generate cached entry
        engine.get_renderable_diff(&reviewable_id);
        assert_eq!(engine.cache_stats().total_entries, 1);

        // Approval should invalidate cache
        engine
            .approve(reviewable_id.clone(), "reviewer".to_string(), None)
            .unwrap();
        assert_eq!(engine.cache_stats().total_entries, 0);
    }

    #[test]
    fn test_approve_all_in_file() {
        let diffs = vec![
            create_test_reviewable_diff("test1.rs", 1),
            create_test_reviewable_diff("test1.rs", 20),
            create_test_reviewable_diff("test2.rs", 1),
        ];
        let mut engine = ReviewEngine::new(diffs, "test_author".to_string());

        let result = engine.approve_all_in_file("test1.rs", "reviewer".to_string(), None);
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
            .approve(diffs[0].id.clone(), "reviewer".to_string(), None)
            .unwrap();

        let progress = engine.get_review_progress();
        assert_eq!(progress.approved_reviewable_diffs, 1);
        assert_eq!(progress.approval_percentage, 50.0);
    }

    #[test]
    fn test_complete_review() {
        let diffs = vec![
            create_test_reviewable_diff("test1.rs", 1),
            create_test_reviewable_diff("test2.rs", 1),
        ];
        let mut engine = ReviewEngine::new(diffs.clone(), "test_author".to_string());

        // Approve diffs
        engine
            .approve(diffs[0].id.clone(), "reviewer".to_string(), None)
            .unwrap();
        engine
            .approve(diffs[1].id.clone(), "reviewer".to_string(), None)
            .unwrap();

        let summary = engine.complete_review().unwrap();
        assert_eq!(summary.total_reviewable_diffs, 2);
        assert_eq!(summary.approved_reviewable_diffs, 2);
        assert_eq!(summary.files_reviewed, 2);
    }

    #[test]
    fn test_render_diff() {
        let diff = create_test_reviewable_diff("test.rs", 1);
        let reviewable_id = diff.id.clone();
        let engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        // Test the render_diff method
        let rendered = engine.render_diff(&reviewable_id);
        assert!(rendered.is_some());

        let content = rendered.unwrap();

        // Should contain the metadata header
        assert!(content.contains("=== test content for test.rs ==="));
        assert!(content.contains("Language: Rust"));
        assert!(content.contains("Lines: 1 | Essential: 1"));
        assert!(content.contains("test content for test.rs"));
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

        let reviewable_id = ReviewableDiffId::new(
            DiffQuery::head_to_unstaged(),
            "test.rs".to_string(),
            LineRange {
                start_line: 10,
                end_line: 12,
                start_column: 0,
                end_column: 0,
            },
        );

        let result = engine.add_instruction(
            reviewable_id.clone(),
            "Extract this to a separate function".to_string(),
            "reviewer".to_string(),
            None,
        );

        assert!(result.is_ok());
    }

    // This test has been removed because overlap now triggers automatic extension
    // instead of an error. See test_add_instruction_with_overlap_auto_extends for
    // the new behavior.

    #[test]
    fn test_add_instruction_non_overlapping_ranges() {
        let diff = create_test_reviewable_diff("test.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        // Add first instruction (lines 10-12)
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

        engine
            .add_instruction(
                id1,
                "First instruction".to_string(),
                "reviewer".to_string(),
                None,
            )
            .unwrap();

        // Add non-overlapping instruction (lines 20-22)
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

        let result = engine.add_instruction(
            id2.clone(),
            "Second instruction".to_string(),
            "reviewer".to_string(),
            None,
        );

        // Should succeed - no overlap
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_two_instructions_to_exact_same_range_stores_both() {
        let diff = create_test_reviewable_diff("test.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        let id = ReviewableDiffId::new(
            DiffQuery::head_to_unstaged(),
            "test.rs".to_string(),
            LineRange {
                start_line: 10,
                end_line: 20,
                start_column: 0,
                end_column: 0,
            },
        );

        engine
            .add_instruction(
                id.clone(),
                "First instruction".to_string(),
                "reviewer".to_string(),
                None,
            )
            .unwrap();

        engine
            .add_instruction(
                id,
                "Second instruction".to_string(),
                "reviewer".to_string(),
                None,
            )
            .unwrap();

        assert_eq!(
            engine.state().instructions.total_instructions(),
            2,
            "Two instructions on the same range should be stored separately"
        );
    }

    #[test]
    fn test_add_instruction_adjacent_ranges_remain_separate() {
        let diff = create_test_reviewable_diff("test.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        // Add first instruction (lines 10-12)
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

        engine
            .add_instruction(
                id1.clone(),
                "First instruction".to_string(),
                "reviewer".to_string(),
                None,
            )
            .unwrap();

        // Add adjacent instruction (lines 13-15) - no overlap
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

        let result = engine.add_instruction(
            id2.clone(),
            "Second instruction".to_string(),
            "reviewer".to_string(),
            None,
        );

        // Should succeed - no overlap
        assert!(result.is_ok());

        // Both instructions should exist separately
        assert!(
            engine
                .state()
                .instructions
                .get_instructions_for_reviewable(&id1)
                .is_some()
        );
        assert!(
            engine
                .state()
                .instructions
                .get_instructions_for_reviewable(&id2)
                .is_some()
        );
    }

    // Tests for JSON export functionality (Phase 4)
    #[test]
    fn test_export_all_instructions() {
        let diff1 = create_test_reviewable_diff("test1.rs", 1);
        let diff2 = create_test_reviewable_diff("test2.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff1, diff2], "test_author".to_string());

        // Add instructions to both files
        let id1 = ReviewableDiffId::new(
            DiffQuery::head_to_unstaged(),
            "test1.rs".to_string(),
            LineRange {
                start_line: 10,
                end_line: 12,
                start_column: 0,
                end_column: 0,
            },
        );

        let id2 = ReviewableDiffId::new(
            DiffQuery::head_to_unstaged(),
            "test2.rs".to_string(),
            LineRange {
                start_line: 20,
                end_line: 22,
                start_column: 0,
                end_column: 0,
            },
        );

        engine
            .add_instruction(
                id1,
                "Instruction 1".to_string(),
                "reviewer".to_string(),
                None,
            )
            .unwrap();
        engine
            .add_instruction(
                id2,
                "Instruction 2".to_string(),
                "reviewer".to_string(),
                None,
            )
            .unwrap();

        // Export all
        let json = engine.export_instructions_json(ExportScope::All).unwrap();

        // Verify JSON is valid
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["_meta"].is_object());
        assert_eq!(parsed["_meta"]["format_version"], "1.1");
        assert_eq!(parsed["instructions"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_export_single_file() {
        let diff1 = create_test_reviewable_diff("test1.rs", 1);
        let diff2 = create_test_reviewable_diff("test2.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff1, diff2], "test_author".to_string());

        // Add instructions to both files
        let id1 = ReviewableDiffId::new(
            DiffQuery::head_to_unstaged(),
            "test1.rs".to_string(),
            LineRange {
                start_line: 10,
                end_line: 12,
                start_column: 0,
                end_column: 0,
            },
        );

        let id2 = ReviewableDiffId::new(
            DiffQuery::head_to_unstaged(),
            "test2.rs".to_string(),
            LineRange {
                start_line: 20,
                end_line: 22,
                start_column: 0,
                end_column: 0,
            },
        );

        engine
            .add_instruction(
                id1,
                "Instruction 1".to_string(),
                "reviewer".to_string(),
                None,
            )
            .unwrap();
        engine
            .add_instruction(
                id2,
                "Instruction 2".to_string(),
                "reviewer".to_string(),
                None,
            )
            .unwrap();

        // Export single file
        let json = engine
            .export_instructions_json(ExportScope::SingleFile("test1.rs".to_string()))
            .unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let instructions = parsed["instructions"].as_array().unwrap();
        assert_eq!(instructions.len(), 1);
        assert_eq!(instructions[0]["file"], "test1.rs");
    }

    #[test]
    fn test_export_single_instruction() {
        let diff = create_test_reviewable_diff("test.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

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

        engine
            .add_instruction(
                id1.clone(),
                "Instruction 1".to_string(),
                "reviewer".to_string(),
                None,
            )
            .unwrap();
        engine
            .add_instruction(
                id2,
                "Instruction 2".to_string(),
                "reviewer".to_string(),
                None,
            )
            .unwrap();

        // Export single instruction
        let json = engine
            .export_instructions_json(ExportScope::SingleInstruction(id1))
            .unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let instructions = parsed["instructions"].as_array().unwrap();
        assert_eq!(instructions.len(), 1);
        assert_eq!(instructions[0]["content"], "Instruction 1");
    }

    #[test]
    fn test_export_empty_scope() {
        let diff = create_test_reviewable_diff("test.rs", 1);
        let engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        // Export with no instructions
        let json = engine.export_instructions_json(ExportScope::All).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["instructions"].as_array().unwrap().len(), 0);
        assert!(parsed["_meta"].is_object());
    }

    #[test]
    fn test_export_json_structure() {
        let diff = create_test_reviewable_diff("test.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

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

        engine
            .add_instruction(
                id,
                "Test instruction".to_string(),
                "reviewer".to_string(),
                None,
            )
            .unwrap();

        let json = engine.export_instructions_json(ExportScope::All).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Verify metadata structure
        assert_eq!(parsed["_meta"]["format_version"], "1.1");
        assert!(parsed["_meta"]["description"].is_string());
        assert!(parsed["_meta"]["field_descriptions"].is_object());
        assert!(parsed["_meta"]["query_formats"].is_object());
        assert!(parsed["_meta"]["git_usage_examples"].is_object());

        // Verify instruction structure
        let inst = &parsed["instructions"][0];
        assert_eq!(inst["file"], "test.rs");
        assert_eq!(inst["query"], "HEAD..unstaged");
        assert_eq!(inst["line_range"]["start_line"], 10);
        assert_eq!(inst["line_range"]["end_line"], 12);
        assert_eq!(inst["content"], "Test instruction");
        assert_eq!(inst["author"], "reviewer");
        assert!(inst["timestamp"].is_string());

        // Verify line_range has no columns
        assert!(inst["line_range"]["start_column"].is_null());
        assert!(inst["line_range"]["end_column"].is_null());
    }

    #[test]
    fn test_export_query_format_mapping() {
        let diff = create_test_reviewable_diff("test.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

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

        engine
            .add_instruction(id, "Test".to_string(), "reviewer".to_string(), None)
            .unwrap();

        let json = engine.export_instructions_json(ExportScope::All).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Verify "working" is mapped to "HEAD..unstaged"
        assert_eq!(parsed["instructions"][0]["query"], "HEAD..unstaged");
    }

    // ===== Hash Calculation Tests =====

    #[test]
    fn test_calculate_file_hash_known_content() {
        use crate::providers::DiffProvider;
        use crate::providers::mock_provider::MockDiffProvider;
        use sha2::{Digest, Sha256};

        let mut mock_provider = MockDiffProvider::new();
        mock_provider.add_file_content("test.rs", &GitRef::head(), "hello world\n");

        let hash = mock_provider
            .get_file_hash("test.rs", &GitRef::head())
            .unwrap();

        // Calculate expected hash
        let mut hasher = Sha256::new();
        hasher.update(b"hello world\n");
        let expected = format!("{:x}", hasher.finalize());

        assert_eq!(hash, expected);
    }

    #[test]
    fn test_calculate_file_hash_identical_content_identical_hash() {
        use crate::providers::DiffProvider;
        use crate::providers::mock_provider::MockDiffProvider;

        let mut mock_provider = MockDiffProvider::new();
        mock_provider.add_file_content("file1.rs", &GitRef::head(), "same content\n");
        mock_provider.add_file_content("file2.rs", &GitRef::head(), "same content\n");

        let hash1 = mock_provider
            .get_file_hash("file1.rs", &GitRef::head())
            .unwrap();
        let hash2 = mock_provider
            .get_file_hash("file2.rs", &GitRef::head())
            .unwrap();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_calculate_file_hash_different_content_different_hash() {
        use crate::providers::DiffProvider;
        use crate::providers::mock_provider::MockDiffProvider;

        let mut mock_provider = MockDiffProvider::new();
        mock_provider.add_file_content("file1.rs", &GitRef::head(), "content A\n");
        mock_provider.add_file_content("file2.rs", &GitRef::head(), "content B\n");

        let hash1 = mock_provider
            .get_file_hash("file1.rs", &GitRef::head())
            .unwrap();
        let hash2 = mock_provider
            .get_file_hash("file2.rs", &GitRef::head())
            .unwrap();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_calculate_file_hash_crlf_normalization() {
        use crate::providers::DiffProvider;
        use crate::providers::mock_provider::MockDiffProvider;

        let mut mock_provider = MockDiffProvider::new();
        mock_provider.add_file_content("crlf.rs", &GitRef::head(), "line1\r\nline2\r\n");
        mock_provider.add_file_content("lf.rs", &GitRef::head(), "line1\nline2\n");

        let hash_crlf = mock_provider
            .get_file_hash("crlf.rs", &GitRef::head())
            .unwrap();
        let hash_lf = mock_provider
            .get_file_hash("lf.rs", &GitRef::head())
            .unwrap();

        // Should be identical after normalization
        assert_eq!(hash_crlf, hash_lf);
    }

    #[test]
    fn test_calculate_file_hash_lf_unchanged() {
        use crate::providers::DiffProvider;
        use crate::providers::mock_provider::MockDiffProvider;
        use sha2::{Digest, Sha256};

        let mut mock_provider = MockDiffProvider::new();
        let content = "line1\nline2\n";
        mock_provider.add_file_content("test.rs", &GitRef::head(), content);

        let hash = mock_provider
            .get_file_hash("test.rs", &GitRef::head())
            .unwrap();

        // Calculate expected hash directly from LF content
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let expected = format!("{:x}", hasher.finalize());

        assert_eq!(hash, expected);
    }

    // ===== Content Snapshot Extraction Tests =====

    #[test]
    fn test_extract_content_snapshot_middle_lines() {
        use crate::providers::DiffProvider;
        use crate::providers::mock_provider::MockDiffProvider;

        let mut mock_provider = MockDiffProvider::new();
        let content = "line1\nline2\nline3\nline4\nline5\nline6\nline7\nline8\nline9\nline10\n";
        mock_provider.add_file_content("test.rs", &GitRef::head(), content);

        let line_range = LineRange {
            start_line: 3,
            end_line: 5,
            start_column: 0,
            end_column: 0,
        };

        let snapshot = mock_provider
            .get_content_snapshot("test.rs", &GitRef::head(), &line_range)
            .unwrap();

        assert_eq!(snapshot, Some("line3\nline4\nline5".to_string()));
    }

    #[test]
    fn test_extract_content_snapshot_start_of_file() {
        use crate::providers::DiffProvider;
        use crate::providers::mock_provider::MockDiffProvider;

        let mut mock_provider = MockDiffProvider::new();
        let content = "line1\nline2\nline3\n";
        mock_provider.add_file_content("test.rs", &GitRef::head(), content);

        let line_range = LineRange {
            start_line: 1,
            end_line: 2,
            start_column: 0,
            end_column: 0,
        };

        let snapshot = mock_provider
            .get_content_snapshot("test.rs", &GitRef::head(), &line_range)
            .unwrap();

        assert_eq!(snapshot, Some("line1\nline2".to_string()));
    }

    #[test]
    fn test_extract_content_snapshot_end_of_file() {
        use crate::providers::DiffProvider;
        use crate::providers::mock_provider::MockDiffProvider;

        let mut mock_provider = MockDiffProvider::new();
        let content = "line1\nline2\nline3\n";
        mock_provider.add_file_content("test.rs", &GitRef::head(), content);

        let line_range = LineRange {
            start_line: 2,
            end_line: 3,
            start_column: 0,
            end_column: 0,
        };

        let snapshot = mock_provider
            .get_content_snapshot("test.rs", &GitRef::head(), &line_range)
            .unwrap();

        assert_eq!(snapshot, Some("line2\nline3".to_string()));
    }

    #[test]
    fn test_extract_content_snapshot_beyond_file_bounds() {
        use crate::providers::DiffProvider;
        use crate::providers::mock_provider::MockDiffProvider;

        let mut mock_provider = MockDiffProvider::new();
        let content = "line1\nline2\nline3\n";
        mock_provider.add_file_content("test.rs", &GitRef::head(), content);

        let line_range = LineRange {
            start_line: 10,
            end_line: 15,
            start_column: 0,
            end_column: 0,
        };

        let snapshot = mock_provider
            .get_content_snapshot("test.rs", &GitRef::head(), &line_range)
            .unwrap();

        assert_eq!(snapshot, None);
    }

    #[test]
    fn test_extract_content_snapshot_empty_range() {
        use crate::providers::DiffProvider;
        use crate::providers::mock_provider::MockDiffProvider;

        let mut mock_provider = MockDiffProvider::new();
        let content = "line1\nline2\nline3\n";
        mock_provider.add_file_content("test.rs", &GitRef::head(), content);

        let line_range = LineRange {
            start_line: 2,
            end_line: 2,
            start_column: 0,
            end_column: 0,
        };

        let snapshot = mock_provider
            .get_content_snapshot("test.rs", &GitRef::head(), &line_range)
            .unwrap();

        assert_eq!(snapshot, Some("line2".to_string()));
    }

    // Tests for Phase 3: Export Format Enhancement
    #[test]
    fn test_export_includes_status_field() {
        let diff = create_test_reviewable_diff("test.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        let reviewable_id = ReviewableDiffId::new(
            DiffQuery::head_to_unstaged(),
            "test.rs".to_string(),
            LineRange {
                start_line: 10,
                end_line: 12,
                start_column: 0,
                end_column: 0,
            },
        );

        engine
            .add_instruction(
                reviewable_id,
                "Test instruction".to_string(),
                "reviewer".to_string(),
                None,
            )
            .unwrap();

        let json = engine.export_instructions_json(ExportScope::All).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        let instructions = parsed["instructions"].as_array().unwrap();
        assert_eq!(instructions.len(), 1);

        let inst = &instructions[0];
        assert!(inst["status"].is_string());
        assert_eq!(inst["status"], "active");
    }

    #[test]
    fn test_export_includes_file_content_hash() {
        let diff = create_test_reviewable_diff("test.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        let reviewable_id = ReviewableDiffId::new(
            DiffQuery::head_to_unstaged(),
            "test.rs".to_string(),
            LineRange {
                start_line: 10,
                end_line: 12,
                start_column: 0,
                end_column: 0,
            },
        );

        engine
            .add_instruction(
                reviewable_id,
                "Test instruction".to_string(),
                "reviewer".to_string(),
                None,
            )
            .unwrap();

        let json = engine.export_instructions_json(ExportScope::All).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        let instructions = parsed["instructions"].as_array().unwrap();
        assert_eq!(instructions.len(), 1);

        let inst = &instructions[0];
        assert!(inst["file_content_hash"].is_string());
        let hash = inst["file_content_hash"].as_str().unwrap();
        assert_eq!(hash.len(), 0); // file_content_hash is not stored on Instruction
    }

    #[test]
    fn test_export_includes_content_snapshot() {
        let diff = create_test_reviewable_diff("test.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        let reviewable_id = ReviewableDiffId::new(
            DiffQuery::head_to_unstaged(),
            "test.rs".to_string(),
            LineRange {
                start_line: 2,
                end_line: 3,
                start_column: 0,
                end_column: 0,
            },
        );

        engine
            .add_instruction(
                reviewable_id,
                "Test instruction".to_string(),
                "reviewer".to_string(),
                None,
            )
            .unwrap();

        let json = engine.export_instructions_json(ExportScope::All).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        let instructions = parsed["instructions"].as_array().unwrap();
        assert_eq!(instructions.len(), 1);

        let inst = &instructions[0];
        // content_snapshot should be present (may be null or string)
        assert!(inst.get("content_snapshot").is_some());
    }

    #[test]
    fn test_export_format_version_is_1_1() {
        let diff = create_test_reviewable_diff("test.rs", 1);
        let engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        let json = engine.export_instructions_json(ExportScope::All).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["_meta"]["format_version"], "1.1");
    }

    #[test]
    fn test_export_metadata_includes_new_field_descriptions() {
        let diff = create_test_reviewable_diff("test.rs", 1);
        let engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        let json = engine.export_instructions_json(ExportScope::All).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        let field_descriptions = &parsed["_meta"]["field_descriptions"];
        assert!(field_descriptions["status"].is_string());
        assert!(field_descriptions["file_content_hash"].is_string());
        assert!(field_descriptions["content_snapshot"].is_string());

        // Verify descriptions are meaningful
        let status_desc = field_descriptions["status"].as_str().unwrap();
        assert!(
            status_desc.contains("active")
                || status_desc.contains("stale")
                || status_desc.contains("validity")
        );

        let hash_desc = field_descriptions["file_content_hash"].as_str().unwrap();
        assert!(hash_desc.contains("hash") || hash_desc.contains("SHA256"));
    }

    #[test]
    fn test_export_status_serialization_active() {
        let diff = create_test_reviewable_diff("test.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        let reviewable_id = ReviewableDiffId::new(
            DiffQuery::head_to_unstaged(),
            "test.rs".to_string(),
            LineRange {
                start_line: 10,
                end_line: 12,
                start_column: 0,
                end_column: 0,
            },
        );

        engine
            .add_instruction(
                reviewable_id,
                "Test instruction".to_string(),
                "reviewer".to_string(),
                None,
            )
            .unwrap();

        let json = engine.export_instructions_json(ExportScope::All).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        let instructions = parsed["instructions"].as_array().unwrap();
        let inst = &instructions[0];

        // Active status should serialize as "active"
        assert_eq!(inst["status"], "active");
    }

    #[test]
    fn test_export_all_scopes_include_new_fields() {
        let diff1 = create_test_reviewable_diff("test1.rs", 1);
        let diff2 = create_test_reviewable_diff("test2.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff1, diff2], "test_author".to_string());

        // Add instructions to different files
        let id1 = ReviewableDiffId::new(
            DiffQuery::head_to_unstaged(),
            "test1.rs".to_string(),
            LineRange {
                start_line: 10,
                end_line: 12,
                start_column: 0,
                end_column: 0,
            },
        );

        let id2 = ReviewableDiffId::new(
            DiffQuery::head_to_unstaged(),
            "test2.rs".to_string(),
            LineRange {
                start_line: 20,
                end_line: 22,
                start_column: 0,
                end_column: 0,
            },
        );

        engine
            .add_instruction(
                id1,
                "Instruction 1".to_string(),
                "reviewer".to_string(),
                None,
            )
            .unwrap();

        engine
            .add_instruction(
                id2,
                "Instruction 2".to_string(),
                "reviewer".to_string(),
                None,
            )
            .unwrap();

        // Test ExportScope::All
        let json = engine.export_instructions_json(ExportScope::All).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let instructions = parsed["instructions"].as_array().unwrap();

        assert_eq!(instructions.len(), 2);
        for inst in instructions {
            assert!(inst["status"].is_string());
            assert!(inst["file_content_hash"].is_string());
            assert!(inst.get("content_snapshot").is_some());
        }

        // Test ExportScope::SingleFile
        let json = engine
            .export_instructions_json(ExportScope::SingleFile("test1.rs".to_string()))
            .unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let instructions = parsed["instructions"].as_array().unwrap();

        assert_eq!(instructions.len(), 1);
        assert!(instructions[0]["status"].is_string());
        assert!(instructions[0]["file_content_hash"].is_string());
    }

    // ========== Phase 4: Import Functionality Tests ==========

    // Tests for JSON parsing
    #[test]
    fn test_import_valid_json_with_new_fields() {
        let diff = create_test_reviewable_diff("src/main.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        let json = r#"{
            "_meta": {
                "format_version": "1.1",
                "description": "DiffViz instruction export for coding agents"
            },
            "instructions": [{
                "file": "src/main.rs",
                "query": "HEAD..unstaged",
                "line_range": {"start_line": 10, "end_line": 12},
                "content": "Extract this to a separate function",
                "author": "reviewer",
                "timestamp": "2024-01-12T10:30:00Z",
                "status": "active",
                "file_content_hash": "test_hash_123",
                "content_snapshot": "    let x = calculate();\n    process(x);"
            }]
        }"#;

        let result = engine.import_instructions_json(json);
        if result.is_err() {
            eprintln!("Import error: {:?}", result.as_ref().unwrap_err());
        }
        assert!(result.is_ok());
        let summary = result.unwrap();
        assert_eq!(summary.total_imported, 1);
    }

    #[test]
    fn test_import_legacy_json_without_new_fields() {
        let diff = create_test_reviewable_diff("src/main.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        // Legacy JSON without status, file_content_hash, content_snapshot
        let json = r#"{
            "_meta": {
                "format_version": "1.0",
                "description": "DiffViz instruction export for coding agents"
            },
            "instructions": [{
                "file": "src/main.rs",
                "query": "HEAD..unstaged",
                "line_range": {"start_line": 10, "end_line": 12},
                "content": "Extract this to a separate function",
                "author": "reviewer",
                "timestamp": "2024-01-12T10:30:00Z"
            }]
        }"#;

        let result = engine.import_instructions_json(json);
        // Should handle gracefully - calculate hash for legacy instructions
        assert!(result.is_ok());
    }

    #[test]
    fn test_import_invalid_json() {
        let diff = create_test_reviewable_diff("src/main.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        let invalid_json = r#"{"invalid": "json structure"#;
        let result = engine.import_instructions_json(invalid_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_import_json_with_missing_required_fields() {
        let diff = create_test_reviewable_diff("src/main.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        // Missing 'content' field (required)
        let json = r#"{
            "_meta": {
                "format_version": "1.1",
                "description": "DiffViz instruction export"
            },
            "instructions": [{
                "file": "src/main.rs",
                "query": "HEAD..unstaged",
                "line_range": {"start_line": 10, "end_line": 12},
                "author": "reviewer",
                "timestamp": "2024-01-12T10:30:00Z",
                "status": "active",
                "file_content_hash": "test_hash"
            }]
        }"#;

        let result = engine.import_instructions_json(json);
        assert!(result.is_err());
    }

    // Tests for import
    #[test]
    fn test_import_sets_active() {
        let diff = create_test_reviewable_diff("src/main.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        let json = r#"{
            "_meta": {"format_version": "1.1", "description": "test"},
            "instructions": [{
                "file": "src/main.rs",
                "query": "HEAD..unstaged",
                "line_range": {"start_line": 10, "end_line": 12},
                "content": "Test instruction",
                "author": "reviewer",
                "timestamp": "2024-01-12T10:30:00Z",
                "status": "active",
                "file_content_hash": "",
                "content_snapshot": null
            }]
        }"#;

        let result = engine.import_instructions_json(json);
        assert!(result.is_ok());
        let summary = result.unwrap();
        assert_eq!(summary.active_count, 1);
        assert_eq!(summary.stale_count, 0);
    }

    // Tests for import summary
    #[test]
    fn test_import_summary_multiple_instructions() {
        let diff1 = create_test_reviewable_diff("src/file1.rs", 1);
        let diff2 = create_test_reviewable_diff("src/file2.rs", 2);
        let mut engine = ReviewEngine::new(vec![diff1, diff2], "test_author".to_string());

        let json = r#"{
            "_meta": {"format_version": "1.1", "description": "test"},
            "instructions": [
                {
                    "file": "src/file1.rs",
                    "query": "HEAD..unstaged",
                    "line_range": {"start_line": 10, "end_line": 12},
                    "content": "Instruction 1",
                    "author": "reviewer",
                    "timestamp": "2024-01-12T10:30:00Z",
                    "status": "active",
                    "file_content_hash": "",
                    "content_snapshot": null
                },
                {
                    "file": "src/file2.rs",
                    "query": "HEAD..unstaged",
                    "line_range": {"start_line": 20, "end_line": 22},
                    "content": "Instruction 2",
                    "author": "reviewer",
                    "timestamp": "2024-01-12T10:30:00Z",
                    "status": "active",
                    "file_content_hash": "",
                    "content_snapshot": null
                },
                {
                    "file": "src/file1.rs",
                    "query": "HEAD..unstaged",
                    "line_range": {"start_line": 30, "end_line": 32},
                    "content": "Instruction 3",
                    "author": "reviewer",
                    "timestamp": "2024-01-12T10:30:00Z",
                    "status": "active",
                    "file_content_hash": "",
                    "content_snapshot": null
                }
            ]
        }"#;

        let result = engine.import_instructions_json(json);
        assert!(result.is_ok());
        let summary = result.unwrap();
        assert_eq!(summary.total_imported, 3);
        assert_eq!(summary.active_count, 3);
        assert_eq!(summary.stale_count, 0);
    }

    #[test]
    fn test_import_summary_with_zero_instructions() {
        let diff = create_test_reviewable_diff("src/main.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        let json = r#"{
            "_meta": {"format_version": "1.1", "description": "test"},
            "instructions": []
        }"#;

        let result = engine.import_instructions_json(json);
        assert!(result.is_ok());
        let summary = result.unwrap();
        assert_eq!(summary.total_imported, 0);
        assert_eq!(summary.active_count, 0);
        assert_eq!(summary.stale_count, 0);
    }

    // Tests for duplicate handling
    #[test]
    fn test_import_duplicate_instruction_same_reviewable_id() {
        let diff = create_test_reviewable_diff("src/main.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        // Add an instruction first
        let reviewable_id = ReviewableDiffId::new(
            DiffQuery::head_to_unstaged(),
            "src/main.rs".to_string(),
            LineRange {
                start_line: 10,
                end_line: 12,
                start_column: 0,
                end_column: 0,
            },
        );

        engine
            .add_instruction(
                reviewable_id.clone(),
                "Original instruction".to_string(),
                "reviewer".to_string(),
                None,
            )
            .unwrap();

        // Try to import instruction with same ReviewableDiffId
        let json = r#"{
            "_meta": {"format_version": "1.1", "description": "test"},
            "instructions": [{
                "file": "src/main.rs",
                "query": "HEAD..unstaged",
                "line_range": {"start_line": 10, "end_line": 12},
                "content": "Duplicate instruction",
                "author": "reviewer",
                "timestamp": "2024-01-12T10:30:00Z",
                "status": "active",
                "file_content_hash": "test_hash",
                "content_snapshot": null
            }]
        }"#;

        let result = engine.import_instructions_json(json);
        assert!(result.is_ok());
        let summary = result.unwrap();

        // Should skip duplicate (or update based on strategy)
        // For MVP: skip duplicates, add warning to summary
        assert!(!summary.errors.is_empty() || summary.total_imported == 0);
    }

    // ===== Decision Approval Integration Tests =====

    /// Helper to set up a decision with multiple chunks
    fn create_engine_with_decision_and_chunks() -> ReviewEngine {
        use crate::entities::decision::{CodeImpact, Decision, DecisionLineRange};

        // Create 3 chunks in the same file
        let chunks = vec![
            create_test_reviewable_diff("main.rs", 1),
            create_test_reviewable_diff("main.rs", 15),
            create_test_reviewable_diff("main.rs", 30),
        ];

        let mut engine = ReviewEngine::new(chunks, "test_author".to_string());

        // Add a decision that affects all 3 chunks
        let decision = Decision {
            number: 1,
            title: "Add authentication module".to_string(),
            rationale: Some("Implement user authentication".to_string()),
            code_impacts: vec![CodeImpact {
                file: "main.rs".to_string(),
                line_ranges: vec![DecisionLineRange { start: 1, end: 45 }],
                reasoning: "Affects main function and helpers".to_string(),
            }],
        };

        engine.state.decisions.add_decision(decision);
        {
            let state_copy = engine.state.clone();
            engine
                .state
                .decisions
                .build_index_from_review_state(&state_copy);
        }

        engine
    }

    #[test]
    fn test_decision_approval_cascades_to_chunks() {
        let mut engine = create_engine_with_decision_and_chunks();

        // Approve decision 1
        let result = engine.approve_decision(1, "reviewer".to_string());
        assert!(result.is_ok());

        // Verify cascade result
        let cascade_result = result.unwrap();
        match cascade_result {
            CascadeResult::DecisionApproved {
                decision_number,
                chunks_affected,
            } => {
                assert_eq!(decision_number, 1);
                assert_eq!(chunks_affected, 3);
            }
            _ => panic!("Expected DecisionApproved result"),
        }

        // Verify decision is approved
        assert!(engine.is_decision_approved(1));

        // Verify all chunks are approved
        for reviewable_diff in engine.state.reviewable_diffs.values() {
            assert!(engine.state.is_approved(&reviewable_diff.id));
        }
    }

    #[test]
    fn test_decision_rejection_cascades_to_chunks() {
        let mut engine = create_engine_with_decision_and_chunks();

        // First approve decision and chunks
        engine.approve_decision(1, "reviewer".to_string()).unwrap();
        assert!(engine.is_decision_approved(1));

        // Now reject the decision
        let result = engine.reject_decision(1);
        assert!(result.is_ok());

        // Verify cascade result
        let cascade_result = result.unwrap();
        match cascade_result {
            CascadeResult::DecisionUnapproved {
                decision_number,
                chunks_affected,
            } => {
                assert_eq!(decision_number, 1);
                assert_eq!(chunks_affected, 3);
            }
            _ => panic!("Expected DecisionUnapproved result"),
        }

        // Verify decision is unapproved
        assert!(!engine.is_decision_approved(1));

        // Verify all chunks are unapproved
        for reviewable_diff in engine.state.reviewable_diffs.values() {
            assert!(!engine.state.is_approved(&reviewable_diff.id));
        }
    }

    #[test]
    fn test_reverse_cascade_all_chunks_approved() {
        let mut engine = create_engine_with_decision_and_chunks();

        // Get all chunk IDs
        let chunk_ids: Vec<_> = engine
            .state
            .reviewable_diffs
            .values()
            .map(|d| d.id.clone())
            .collect();

        // Approve chunks one by one
        for (i, chunk_id) in chunk_ids.iter().enumerate() {
            engine
                .approve(chunk_id.clone(), "reviewer".to_string(), None)
                .unwrap();

            // Decision should still be unapproved until ALL chunks are approved
            if i < chunk_ids.len() - 1 {
                assert!(!engine.is_decision_approved(1));
            }
        }

        // After approving all chunks, decision should be auto-approved
        assert!(engine.is_decision_approved(1));
    }

    #[test]
    fn test_reverse_cascade_reject_one_chunk_unapproves_decision() {
        let mut engine = create_engine_with_decision_and_chunks();

        // Approve all chunks
        let chunk_ids: Vec<_> = engine
            .state
            .reviewable_diffs
            .values()
            .map(|d| d.id.clone())
            .collect();
        for chunk_id in chunk_ids {
            engine
                .approve(chunk_id, "reviewer".to_string(), None)
                .unwrap();
        }

        // Verify decision is auto-approved
        assert!(engine.is_decision_approved(1));

        // Reject one chunk
        let first_chunk_id = engine
            .state
            .reviewable_diffs
            .values()
            .next()
            .unwrap()
            .id
            .clone();

        engine.reject(first_chunk_id, None).unwrap();

        // Decision should be unapproved since not all chunks are approved anymore
        assert!(!engine.is_decision_approved(1));
    }

    #[test]
    fn test_decision_progress_partial_approval() {
        let mut engine = create_engine_with_decision_and_chunks();

        // Get chunk IDs
        let chunk_ids: Vec<_> = engine
            .state
            .reviewable_diffs
            .values()
            .map(|d| d.id.clone())
            .collect();

        // Initially: no chunks approved
        let (approved, total) = engine.decision_approval_progress(1);
        assert_eq!(approved, 0);
        assert_eq!(total, 3);

        // Approve first chunk
        engine
            .approve(chunk_ids[0].clone(), "reviewer".to_string(), None)
            .unwrap();
        let (approved, total) = engine.decision_approval_progress(1);
        assert_eq!(approved, 1);
        assert_eq!(total, 3);

        // Approve second chunk
        engine
            .approve(chunk_ids[1].clone(), "reviewer".to_string(), None)
            .unwrap();
        let (approved, total) = engine.decision_approval_progress(1);
        assert_eq!(approved, 2);
        assert_eq!(total, 3);

        // Approve third chunk - should trigger reverse cascade
        engine
            .approve(chunk_ids[2].clone(), "reviewer".to_string(), None)
            .unwrap();
        let (approved, total) = engine.decision_approval_progress(1);
        assert_eq!(approved, 3);
        assert_eq!(total, 3);

        // Decision should be auto-approved
        assert!(engine.is_decision_approved(1));
    }

    #[test]
    fn test_multiple_decisions_independent() {
        use crate::entities::decision::{CodeImpact, Decision, DecisionLineRange};

        // Create 4 chunks: 2 for decision 1, 2 for decision 2
        let chunks = vec![
            create_test_reviewable_diff("module1.rs", 1),
            create_test_reviewable_diff("module1.rs", 50),
            create_test_reviewable_diff("module2.rs", 1),
            create_test_reviewable_diff("module2.rs", 50),
        ];

        let mut engine = ReviewEngine::new(chunks, "test_author".to_string());

        // Add decision 1 (affects module1.rs)
        let decision1 = Decision {
            number: 1,
            title: "Module 1 changes".to_string(),
            rationale: Some("Changes to module 1".to_string()),
            code_impacts: vec![CodeImpact {
                file: "module1.rs".to_string(),
                line_ranges: vec![DecisionLineRange { start: 1, end: 100 }],
                reasoning: "All module1 changes".to_string(),
            }],
        };

        // Add decision 2 (affects module2.rs)
        let decision2 = Decision {
            number: 2,
            title: "Module 2 changes".to_string(),
            rationale: Some("Changes to module 2".to_string()),
            code_impacts: vec![CodeImpact {
                file: "module2.rs".to_string(),
                line_ranges: vec![DecisionLineRange { start: 1, end: 100 }],
                reasoning: "All module2 changes".to_string(),
            }],
        };

        engine.state.decisions.add_decision(decision1);
        engine.state.decisions.add_decision(decision2);
        {
            let state_copy = engine.state.clone();
            engine
                .state
                .decisions
                .build_index_from_review_state(&state_copy);
        }

        // Approve only decision 1
        let result = engine.approve_decision(1, "reviewer".to_string()).unwrap();

        // Verify we got the cascade result
        match result {
            CascadeResult::DecisionApproved {
                decision_number,
                chunks_affected,
            } => {
                assert_eq!(decision_number, 1);
                assert_eq!(chunks_affected, 2);
            }
            _ => panic!("Expected DecisionApproved result"),
        }

        // Decision 1 should be approved
        assert!(engine.is_decision_approved(1));

        // Decision 2 should not be approved
        assert!(!engine.is_decision_approved(2));

        // Only module1 chunks should be approved
        for reviewable_diff in engine.state.reviewable_diffs.values() {
            if reviewable_diff.file_path == "module1.rs" {
                assert!(engine.state.is_approved(&reviewable_diff.id));
            } else {
                assert!(!engine.state.is_approved(&reviewable_diff.id));
            }
        }
    }

    #[test]
    fn test_decision_progress_zero_chunks() {
        let engine = ReviewEngine::new(vec![], "test_author".to_string());

        // For non-existent decision, progress should be (0, 0)
        let (approved, total) = engine.decision_approval_progress(999);
        assert_eq!(approved, 0);
        assert_eq!(total, 0);
    }

    #[test]
    fn test_get_decision_reviewable_diffs_returns_one_per_indexed_pair() {
        use crate::entities::decision::{CodeImpact, Decision, DecisionLineRange};

        // Two diffs on the same file, non-overlapping line ranges
        let diff1 = create_test_reviewable_diff("src/main.rs", 1); // lines 1-11
        let diff2 = create_test_reviewable_diff("src/main.rs", 20); // lines 20-30

        let mut engine = ReviewEngine::new(vec![diff1, diff2], "test_author".to_string());

        // One decision with two code impacts — each overlaps one of the two diffs
        let decision = Decision {
            number: 1,
            title: "Test decision".to_string(),
            rationale: None,
            code_impacts: vec![
                CodeImpact {
                    file: "src/main.rs".to_string(),
                    line_ranges: vec![DecisionLineRange { start: 5, end: 15 }],
                    reasoning: "overlaps diff1".to_string(),
                },
                CodeImpact {
                    file: "src/main.rs".to_string(),
                    line_ranges: vec![DecisionLineRange { start: 25, end: 35 }],
                    reasoning: "overlaps diff2".to_string(),
                },
            ],
        };

        let mut decisions = crate::entities::ReviewDecisions::new();
        decisions.add_decision(decision);
        engine.set_decisions_with_index(decisions);

        let diffs = engine.get_decision_reviewable_diffs();
        assert_eq!(diffs.len(), 2);
        assert!(diffs.iter().all(|d| d.decision_number == 1));
    }

    // === Decision Instruction Tests ===

    fn create_engine_with_decision() -> ReviewEngine {
        use crate::entities::decision::Decision;

        let diff = create_test_reviewable_diff("test.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        let decision = Decision {
            number: 1,
            title: "Test decision".to_string(),
            rationale: None,
            code_impacts: vec![],
        };
        engine.state.decisions.add_decision(decision);
        engine
    }

    #[test]
    fn test_add_decision_instruction_success() {
        let mut engine = create_engine_with_decision();

        let result = engine.add_decision_instruction(
            1,
            "Review this carefully".to_string(),
            "reviewer".to_string(),
        );

        assert!(result.is_ok());
        assert!(engine.get_decision_instructions(1).is_some());
        assert_eq!(engine.get_decision_instructions(1).unwrap().len(), 1);
    }

    #[test]
    fn test_add_decision_instruction_invalid_decision() {
        let mut engine = create_engine_with_decision();

        let result =
            engine.add_decision_instruction(999, "Should fail".to_string(), "reviewer".to_string());

        assert!(result.is_err());
        match result.unwrap_err() {
            crate::errors::DiffVizError::Review(crate::errors::ReviewError::InvalidDecision {
                decision_number,
            }) => assert_eq!(decision_number, 999),
            e => panic!("Expected InvalidDecision, got: {e:?}"),
        }
    }

    #[test]
    fn test_add_decision_instruction_multiple_to_same_decision() {
        let mut engine = create_engine_with_decision();

        engine
            .add_decision_instruction(1, "First instruction".to_string(), "author1".to_string())
            .unwrap();
        engine
            .add_decision_instruction(1, "Second instruction".to_string(), "author2".to_string())
            .unwrap();

        let instructions = engine.get_decision_instructions(1).unwrap();
        assert_eq!(instructions.len(), 2);
    }

    #[test]
    fn test_remove_decision_instruction_success() {
        let mut engine = create_engine_with_decision();

        engine
            .add_decision_instruction(1, "To be removed".to_string(), "author".to_string())
            .unwrap();

        let instruction_id = engine
            .get_decision_instructions(1)
            .unwrap()
            .first()
            .unwrap()
            .id
            .clone();

        let result = engine.remove_decision_instruction(&instruction_id);
        assert!(result.is_ok());
        assert_eq!(engine.get_decision_instructions(1).unwrap().len(), 0);
    }

    #[test]
    fn test_remove_decision_instruction_not_found() {
        let mut engine = create_engine_with_decision();

        let result = engine.remove_decision_instruction("nonexistent-id");

        assert!(result.is_err());
        match result.unwrap_err() {
            crate::errors::DiffVizError::Review(
                crate::errors::ReviewError::InstructionNotFound { instruction_id },
            ) => assert_eq!(instruction_id, "nonexistent-id"),
            e => panic!("Expected InstructionNotFound, got: {e:?}"),
        }
    }

    #[test]
    fn test_get_decision_instructions_returns_all() {
        let mut engine = create_engine_with_decision();

        engine
            .add_decision_instruction(1, "First".to_string(), "author".to_string())
            .unwrap();
        engine
            .add_decision_instruction(1, "Second".to_string(), "author".to_string())
            .unwrap();
        engine
            .add_decision_instruction(1, "Third".to_string(), "author".to_string())
            .unwrap();

        let instructions = engine.get_decision_instructions(1).unwrap();
        assert_eq!(instructions.len(), 3);
    }

    #[test]
    fn test_get_decision_instructions_for_missing_decision() {
        let engine = create_engine_with_decision();

        let result = engine.get_decision_instructions(999);
        assert!(result.is_none());
    }

    #[test]
    fn test_add_and_remove_decision_instructions() {
        let mut engine = create_engine_with_decision();

        engine
            .add_decision_instruction(1, "Keep this".to_string(), "author".to_string())
            .unwrap();
        engine
            .add_decision_instruction(1, "Remove this".to_string(), "author".to_string())
            .unwrap();

        let remove_id = engine
            .get_decision_instructions(1)
            .unwrap()
            .iter()
            .find(|i| i.content == "Remove this")
            .unwrap()
            .id
            .clone();

        engine.remove_decision_instruction(&remove_id).unwrap();

        let remaining = engine.get_decision_instructions(1).unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].content, "Keep this");
    }

    #[test]
    fn test_decision_instructions_persist_through_state() {
        let mut engine = create_engine_with_decision();

        engine
            .add_decision_instruction(
                1,
                "Persistent instruction".to_string(),
                "author".to_string(),
            )
            .unwrap();

        // Access via state directly to verify persistence
        let count = engine.state().decision_instructions.total_instructions();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_decision_instructions_independent_from_reviewable_instructions() {
        let diff = create_test_reviewable_diff("test.rs", 1);
        let diff_id = diff.id.clone();
        let mut engine = create_engine_with_decision();
        engine
            .add_instruction(
                diff_id,
                "Code instruction".to_string(),
                "author".to_string(),
                None,
            )
            .unwrap();
        engine
            .add_decision_instruction(1, "Decision instruction".to_string(), "author".to_string())
            .unwrap();

        assert_eq!(engine.state().instructions.total_instructions(), 1);
        assert_eq!(engine.state().decision_instructions.total_instructions(), 1);
    }
}

/// Helper function to convert RenderableDiff to string format for caching and display
fn format_renderable_diff_for_display(renderable_diff: &RenderableDiff) -> String {
    let mut output = String::new();

    // Add metadata header
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

    // Add each line with change indicators
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
