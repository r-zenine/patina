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
    /// Instruction points to correct code
    Active,
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
    pub author: String,
    pub timestamp: String,
    pub content: String,
    #[serde(default)]
    pub status: InstructionStatus,
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

    pub fn add_instruction(&mut self, reviewable_id: ReviewableDiffId, instruction: Instruction) {
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

    fn make_instruction(id: &str, content: &str) -> Instruction {
        Instruction {
            id: id.to_string(),
            content: content.to_string(),
            author: "reviewer".to_string(),
            timestamp: "2023-01-01T00:00:00Z".to_string(),
            status: InstructionStatus::Active,
        }
    }

    // ===== InstructionStatus Tests =====

    #[test]
    fn test_instruction_status_serialize_active() {
        let status = InstructionStatus::Active;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"active\"");
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
        let instruction = Instruction {
            id: "instruction_1".to_string(),
            content: "Please add error handling".to_string(),
            author: "reviewer".to_string(),
            timestamp: "2023-01-01T00:00:00Z".to_string(),
            status: InstructionStatus::Active,
        };

        assert_eq!(instruction.id, "instruction_1");
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
        instructions.add_instruction(
            reviewable_id.clone(),
            make_instruction("instruction_1", "Please add error handling"),
        );

        // Verify instruction added
        assert!(instructions.has_instructions(&reviewable_id));
        assert_eq!(instructions.total_instructions(), 1);
        assert_eq!(
            instructions.get_instructions(&reviewable_id).unwrap().len(),
            1
        );
    }

    #[test]
    fn test_instruction_deserialize() {
        let json = r#"{
            "id": "instruction_1",
            "content": "Test",
            "author": "reviewer",
            "timestamp": "2023-01-01T00:00:00Z",
            "status": "active"
        }"#;

        let instruction: Instruction = serde_json::from_str(json).unwrap();
        assert_eq!(instruction.status, InstructionStatus::Active);
        assert_eq!(instruction.content, "Test");
    }

    // ===== ReviewInstructions Enhanced Methods Tests =====

    #[test]
    fn test_get_instructions_by_status_active() {
        let mut instructions = ReviewInstructions::new();
        let reviewable_id_1 = create_test_reviewable_id();

        let mut reviewable_id_2 = create_test_reviewable_id();
        reviewable_id_2.line_range.start_line = 20;

        instructions.add_instruction(
            reviewable_id_1.clone(),
            make_instruction("instruction_1", "Active instruction"),
        );

        let mut addressed = make_instruction("instruction_2", "Addressed instruction");
        addressed.status = InstructionStatus::Addressed;
        instructions.add_instruction(reviewable_id_2, addressed);

        let active_instructions =
            instructions.get_instructions_by_status(&InstructionStatus::Active);
        assert_eq!(active_instructions.len(), 1);
        assert_eq!(active_instructions[0].id, "instruction_1");
    }

    #[test]
    fn test_remove_instruction_by_id_success() {
        let mut instructions = ReviewInstructions::new();
        let reviewable_id = create_test_reviewable_id();

        instructions.add_instruction(
            reviewable_id.clone(),
            make_instruction("instruction_1", "Test"),
        );

        assert_eq!(instructions.total_instructions(), 1);

        let removed = instructions.remove_instruction_by_id("instruction_1");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().id, "instruction_1");
        assert_eq!(instructions.total_instructions(), 0);
    }

    #[test]
    fn test_remove_instruction_by_id_not_found() {
        let mut instructions = ReviewInstructions::new();
        let reviewable_id = create_test_reviewable_id();

        instructions.add_instruction(reviewable_id, make_instruction("instruction_1", "Test"));

        let removed = instructions.remove_instruction_by_id("nonexistent");
        assert!(removed.is_none());
        assert_eq!(instructions.total_instructions(), 1);
    }
}
