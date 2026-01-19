//! Instruction system for code review workflow
//!
//! This module contains the instruction entities used in the ReviewableDiff-based
//! review system, allowing reviewers to provide actionable guidance for code changes.

use crate::entities::reviewable_diff_id::ReviewableDiffId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Status of an instruction indicating its validity after code changes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InstructionStatus {
    /// File hash matches, instruction points to correct code
    Active,
    /// File hash changed, instruction may no longer apply
    Stale,
    /// User marked instruction as handled/completed
    Addressed,
}

impl Default for InstructionStatus {
    fn default() -> Self {
        Self::Active
    }
}

/// An instruction for changes on a reviewable diff
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Instruction {
    pub id: String,
    pub reviewable_id: ReviewableDiffId,
    pub author: String,
    pub timestamp: String,
    pub content: String,
    #[serde(default)]
    pub status: InstructionStatus,
    #[serde(default)]
    pub file_content_hash: String,
    #[serde(default)]
    pub content_snapshot: Option<String>,
}

/// Collection of instructions organized by ReviewableDiffId
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReviewInstructions {
    pub instructions: HashMap<ReviewableDiffId, Vec<Instruction>>,
}

impl ReviewInstructions {
    pub fn new() -> Self {
        Self {
            instructions: HashMap::new(),
        }
    }

    pub fn add_instruction(&mut self, instruction: Instruction) {
        let reviewable_id = instruction.reviewable_id.clone();
        self.instructions
            .entry(reviewable_id)
            .or_default()
            .push(instruction);
    }

    pub fn get_instructions(&self, reviewable_id: &ReviewableDiffId) -> Option<&Vec<Instruction>> {
        self.instructions.get(reviewable_id)
    }

    pub fn has_instructions(&self, reviewable_id: &ReviewableDiffId) -> bool {
        self.instructions
            .get(reviewable_id)
            .is_some_and(|instructions| !instructions.is_empty())
    }

    pub fn total_instructions(&self) -> usize {
        self.instructions
            .values()
            .map(|instructions| instructions.len())
            .sum()
    }

    pub fn get_all_instructions(&self) -> Vec<&Instruction> {
        self.instructions
            .values()
            .flat_map(|instructions| instructions.iter())
            .collect()
    }

    /// Get the first instruction for a ReviewableDiffId (convenience method)
    pub fn get_instructions_for_reviewable(
        &self,
        reviewable_id: &ReviewableDiffId,
    ) -> Option<&Instruction> {
        self.instructions.get(reviewable_id)?.first()
    }

    /// Remove all instructions for a specific ReviewableDiffId
    pub fn remove_instructions(
        &mut self,
        reviewable_id: &ReviewableDiffId,
    ) -> Option<Vec<Instruction>> {
        self.instructions.remove(reviewable_id)
    }

    /// Get all instructions filtered by status
    pub fn get_instructions_by_status(&self, status: &InstructionStatus) -> Vec<&Instruction> {
        self.instructions
            .values()
            .flat_map(|instructions| instructions.iter())
            .filter(|instruction| &instruction.status == status)
            .collect()
    }

    /// Remove a specific instruction by ID
    pub fn remove_instruction_by_id(&mut self, instruction_id: &str) -> Option<Instruction> {
        for instructions in self.instructions.values_mut() {
            if let Some(pos) = instructions.iter().position(|i| i.id == instruction_id) {
                return Some(instructions.remove(pos));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::git_ref::DiffQuery;
    use crate::entities::reviewable_diff_id::LineRange;

    fn create_test_reviewable_id() -> ReviewableDiffId {
        ReviewableDiffId::new(
            DiffQuery::head_to_unstaged(),
            "test.rs".to_string(),
            LineRange {
                start_line: 1,
                end_line: 10,
                start_column: 0,
                end_column: 0,
            },
        )
    }

    // ===== InstructionStatus Tests =====

    #[test]
    fn test_instruction_status_serialize_active() {
        let status = InstructionStatus::Active;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"active\"");
    }

    #[test]
    fn test_instruction_status_serialize_stale() {
        let status = InstructionStatus::Stale;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"stale\"");
    }

    #[test]
    fn test_instruction_status_serialize_addressed() {
        let status = InstructionStatus::Addressed;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"addressed\"");
    }

    #[test]
    fn test_instruction_status_deserialize_active() {
        let json = "\"active\"";
        let status: InstructionStatus = serde_json::from_str(json).unwrap();
        assert_eq!(status, InstructionStatus::Active);
    }

    #[test]
    fn test_instruction_status_deserialize_stale() {
        let json = "\"stale\"";
        let status: InstructionStatus = serde_json::from_str(json).unwrap();
        assert_eq!(status, InstructionStatus::Stale);
    }

    #[test]
    fn test_instruction_status_deserialize_addressed() {
        let json = "\"addressed\"";
        let status: InstructionStatus = serde_json::from_str(json).unwrap();
        assert_eq!(status, InstructionStatus::Addressed);
    }

    #[test]
    fn test_instruction_status_default_is_active() {
        let status = InstructionStatus::default();
        assert_eq!(status, InstructionStatus::Active);
    }

    // ===== Instruction Tests =====

    #[test]
    fn test_instruction_creation() {
        let reviewable_id = create_test_reviewable_id();
        let instruction = Instruction {
            id: "instruction_1".to_string(),
            reviewable_id: reviewable_id.clone(),
            content: "Please add error handling".to_string(),
            author: "reviewer".to_string(),
            timestamp: "2023-01-01T00:00:00Z".to_string(),
            status: InstructionStatus::Active,
            file_content_hash: "test_hash".to_string(),
            content_snapshot: None,
        };

        assert_eq!(instruction.id, "instruction_1");
        assert_eq!(instruction.reviewable_id, reviewable_id);
        assert_eq!(instruction.content, "Please add error handling");
        assert_eq!(instruction.author, "reviewer");
    }

    #[test]
    fn test_review_instructions_operations() {
        let mut instructions = ReviewInstructions::new();
        let reviewable_id = create_test_reviewable_id();

        // Initially no instructions
        assert!(!instructions.has_instructions(&reviewable_id));
        assert_eq!(instructions.total_instructions(), 0);

        // Add an instruction
        let instruction = Instruction {
            id: "instruction_1".to_string(),
            reviewable_id: reviewable_id.clone(),
            content: "Please add error handling".to_string(),
            author: "reviewer".to_string(),
            timestamp: "2023-01-01T00:00:00Z".to_string(),
            status: InstructionStatus::Active,
            file_content_hash: "test_hash".to_string(),
            content_snapshot: None,
        };
        instructions.add_instruction(instruction);

        // Verify instruction added
        assert!(instructions.has_instructions(&reviewable_id));
        assert_eq!(instructions.total_instructions(), 1);
        assert_eq!(
            instructions.get_instructions(&reviewable_id).unwrap().len(),
            1
        );
    }

    // ===== Enhanced Instruction Tests =====

    #[test]
    fn test_instruction_with_all_new_fields() {
        let reviewable_id = create_test_reviewable_id();
        let instruction = Instruction {
            id: "instruction_1".to_string(),
            reviewable_id: reviewable_id.clone(),
            content: "Extract to function".to_string(),
            author: "reviewer".to_string(),
            timestamp: "2023-01-01T00:00:00Z".to_string(),
            status: InstructionStatus::Active,
            file_content_hash: "abc123".to_string(),
            content_snapshot: Some("let x = 10;\nlet y = 20;".to_string()),
        };

        assert_eq!(instruction.status, InstructionStatus::Active);
        assert_eq!(instruction.file_content_hash, "abc123");
        assert_eq!(
            instruction.content_snapshot,
            Some("let x = 10;\nlet y = 20;".to_string())
        );
    }

    #[test]
    fn test_instruction_serialize_with_new_fields() {
        let reviewable_id = create_test_reviewable_id();
        let instruction = Instruction {
            id: "instruction_1".to_string(),
            reviewable_id,
            content: "Fix bug".to_string(),
            author: "reviewer".to_string(),
            timestamp: "2023-01-01T00:00:00Z".to_string(),
            status: InstructionStatus::Stale,
            file_content_hash: "def456".to_string(),
            content_snapshot: Some("code snippet".to_string()),
        };

        let json = serde_json::to_string(&instruction).unwrap();
        assert!(json.contains("\"status\":\"stale\""));
        assert!(json.contains("\"file_content_hash\":\"def456\""));
        assert!(json.contains("\"content_snapshot\":\"code snippet\""));
    }

    #[test]
    fn test_instruction_deserialize_with_new_fields() {
        let reviewable_id = create_test_reviewable_id();
        let json = format!(
            r#"{{
                "id": "instruction_1",
                "reviewable_id": {},
                "content": "Test",
                "author": "reviewer",
                "timestamp": "2023-01-01T00:00:00Z",
                "status": "active",
                "file_content_hash": "xyz789",
                "content_snapshot": "snippet"
            }}"#,
            serde_json::to_string(&reviewable_id).unwrap()
        );

        let instruction: Instruction = serde_json::from_str(&json).unwrap();
        assert_eq!(instruction.status, InstructionStatus::Active);
        assert_eq!(instruction.file_content_hash, "xyz789");
        assert_eq!(instruction.content_snapshot, Some("snippet".to_string()));
    }

    #[test]
    fn test_instruction_deserialize_legacy_without_new_fields() {
        let reviewable_id = create_test_reviewable_id();
        let json = format!(
            r#"{{
                "id": "instruction_1",
                "reviewable_id": {},
                "content": "Legacy instruction",
                "author": "reviewer",
                "timestamp": "2023-01-01T00:00:00Z"
            }}"#,
            serde_json::to_string(&reviewable_id).unwrap()
        );

        let instruction: Instruction = serde_json::from_str(&json).unwrap();
        // Should deserialize with default values
        assert_eq!(instruction.content, "Legacy instruction");
    }

    #[test]
    fn test_instruction_optional_content_snapshot_none() {
        let reviewable_id = create_test_reviewable_id();
        let instruction = Instruction {
            id: "instruction_1".to_string(),
            reviewable_id,
            content: "Test".to_string(),
            author: "reviewer".to_string(),
            timestamp: "2023-01-01T00:00:00Z".to_string(),
            status: InstructionStatus::Active,
            file_content_hash: "hash123".to_string(),
            content_snapshot: None,
        };

        assert_eq!(instruction.content_snapshot, None);

        // Serialize and verify None is handled correctly
        let json = serde_json::to_string(&instruction).unwrap();
        let deserialized: Instruction = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.content_snapshot, None);
    }

    // ===== ReviewInstructions Enhanced Methods Tests =====

    #[test]
    fn test_get_instructions_by_status_active() {
        let mut instructions = ReviewInstructions::new();
        let reviewable_id_1 = create_test_reviewable_id();

        let mut reviewable_id_2 = create_test_reviewable_id();
        reviewable_id_2.line_range.start_line = 20;

        // Add active instruction
        instructions.add_instruction(Instruction {
            id: "instruction_1".to_string(),
            reviewable_id: reviewable_id_1.clone(),
            content: "Active instruction".to_string(),
            author: "reviewer".to_string(),
            timestamp: "2023-01-01T00:00:00Z".to_string(),
            status: InstructionStatus::Active,
            file_content_hash: "hash1".to_string(),
            content_snapshot: None,
        });

        // Add stale instruction
        instructions.add_instruction(Instruction {
            id: "instruction_2".to_string(),
            reviewable_id: reviewable_id_2,
            content: "Stale instruction".to_string(),
            author: "reviewer".to_string(),
            timestamp: "2023-01-01T00:00:00Z".to_string(),
            status: InstructionStatus::Stale,
            file_content_hash: "hash2".to_string(),
            content_snapshot: None,
        });

        let active_instructions =
            instructions.get_instructions_by_status(&InstructionStatus::Active);
        assert_eq!(active_instructions.len(), 1);
        assert_eq!(active_instructions[0].id, "instruction_1");
    }

    #[test]
    fn test_get_instructions_by_status_stale() {
        let mut instructions = ReviewInstructions::new();
        let reviewable_id = create_test_reviewable_id();

        // Add stale instruction
        instructions.add_instruction(Instruction {
            id: "instruction_1".to_string(),
            reviewable_id,
            content: "Stale instruction".to_string(),
            author: "reviewer".to_string(),
            timestamp: "2023-01-01T00:00:00Z".to_string(),
            status: InstructionStatus::Stale,
            file_content_hash: "hash1".to_string(),
            content_snapshot: None,
        });

        let stale_instructions = instructions.get_instructions_by_status(&InstructionStatus::Stale);
        assert_eq!(stale_instructions.len(), 1);
        assert_eq!(stale_instructions[0].id, "instruction_1");
    }

    #[test]
    fn test_remove_instruction_by_id_success() {
        let mut instructions = ReviewInstructions::new();
        let reviewable_id = create_test_reviewable_id();

        // Add instruction
        instructions.add_instruction(Instruction {
            id: "instruction_1".to_string(),
            reviewable_id: reviewable_id.clone(),
            content: "Test".to_string(),
            author: "reviewer".to_string(),
            timestamp: "2023-01-01T00:00:00Z".to_string(),
            status: InstructionStatus::Active,
            file_content_hash: "hash1".to_string(),
            content_snapshot: None,
        });

        assert_eq!(instructions.total_instructions(), 1);

        // Remove by ID
        let removed = instructions.remove_instruction_by_id("instruction_1");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().id, "instruction_1");
        assert_eq!(instructions.total_instructions(), 0);
    }

    #[test]
    fn test_remove_instruction_by_id_not_found() {
        let mut instructions = ReviewInstructions::new();
        let reviewable_id = create_test_reviewable_id();

        // Add instruction
        instructions.add_instruction(Instruction {
            id: "instruction_1".to_string(),
            reviewable_id,
            content: "Test".to_string(),
            author: "reviewer".to_string(),
            timestamp: "2023-01-01T00:00:00Z".to_string(),
            status: InstructionStatus::Active,
            file_content_hash: "hash1".to_string(),
            content_snapshot: None,
        });

        // Try to remove non-existent ID
        let removed = instructions.remove_instruction_by_id("nonexistent");
        assert!(removed.is_none());
        assert_eq!(instructions.total_instructions(), 1);
    }
}
