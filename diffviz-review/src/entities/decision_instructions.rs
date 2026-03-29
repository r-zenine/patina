//! Decision instructions system for decision-level review guidance
//!
//! This module contains the instruction entities for decision-level annotations,
//! allowing reviewers to provide actionable guidance on architectural decisions.

use crate::entities::instruction::{Instruction, InstructionStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Collection of instructions organized by decision number
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DecisionInstructions {
    pub instructions: HashMap<u32, Vec<Instruction>>,
}

impl DecisionInstructions {
    /// Create a new empty DecisionInstructions collection
    pub fn new() -> Self {
        Self {
            instructions: HashMap::new(),
        }
    }

    /// Add an instruction to a specific decision
    pub fn add_instruction(&mut self, decision_number: u32, instruction: Instruction) {
        self.instructions
            .entry(decision_number)
            .or_default()
            .push(instruction);
    }

    /// Get instructions for a specific decision
    pub fn get_instructions(&self, decision_number: u32) -> Option<&Vec<Instruction>> {
        self.instructions.get(&decision_number)
    }

    /// Check if a decision has any instructions
    pub fn has_instructions(&self, decision_number: u32) -> bool {
        self.instructions
            .get(&decision_number)
            .is_some_and(|instructions| !instructions.is_empty())
    }

    /// Get total count of all instructions across all decisions
    pub fn total_instructions(&self) -> usize {
        self.instructions
            .values()
            .map(|instructions| instructions.len())
            .sum()
    }

    /// Get all instructions across all decisions as flat list
    pub fn get_all_instructions(&self) -> Vec<&Instruction> {
        self.instructions
            .values()
            .flat_map(|instructions| instructions.iter())
            .collect()
    }

    /// Get first instruction for a specific decision (convenience)
    pub fn get_instructions_for_decision(&self, decision_number: u32) -> Option<&Instruction> {
        self.instructions.get(&decision_number)?.first()
    }

    /// Remove all instructions for a specific decision
    pub fn remove_instructions(&mut self, decision_number: u32) -> Option<Vec<Instruction>> {
        self.instructions.remove(&decision_number)
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

    fn make_instruction(id: &str, content: &str) -> Instruction {
        Instruction {
            id: id.to_string(),
            content: content.to_string(),
            author: "reviewer".to_string(),
            timestamp: "2023-01-01T00:00:00Z".to_string(),
            status: InstructionStatus::Active,
        }
    }

    // ===== Basic Operations Tests =====

    #[test]
    fn test_new_creates_empty_collection() {
        let instructions = DecisionInstructions::new();
        assert_eq!(instructions.total_instructions(), 0);
        assert_eq!(instructions.get_all_instructions().len(), 0);
    }

    #[test]
    fn test_add_instruction_to_decision() {
        let mut instructions = DecisionInstructions::new();
        let instruction = make_instruction("inst_1", "Review this decision");

        instructions.add_instruction(1, instruction);

        assert!(instructions.has_instructions(1));
        assert_eq!(instructions.total_instructions(), 1);
    }

    #[test]
    fn test_has_instructions_returns_true_when_present() {
        let mut instructions = DecisionInstructions::new();
        instructions.add_instruction(1, make_instruction("inst_1", "Test"));

        assert!(instructions.has_instructions(1));
    }

    #[test]
    fn test_has_instructions_returns_false_when_absent() {
        let instructions = DecisionInstructions::new();
        assert!(!instructions.has_instructions(1));
    }

    #[test]
    fn test_get_instructions_for_missing_decision_returns_none() {
        let instructions = DecisionInstructions::new();
        assert!(instructions.get_instructions(1).is_none());
    }

    #[test]
    fn test_get_instructions_for_present_decision_returns_vec() {
        let mut instructions = DecisionInstructions::new();
        instructions.add_instruction(1, make_instruction("inst_1", "Test"));

        let result = instructions.get_instructions(1);
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_total_instructions_counts_all_decisions() {
        let mut instructions = DecisionInstructions::new();
        instructions.add_instruction(1, make_instruction("inst_1", "Decision 1"));
        instructions.add_instruction(1, make_instruction("inst_2", "Decision 1 again"));
        instructions.add_instruction(2, make_instruction("inst_3", "Decision 2"));

        assert_eq!(instructions.total_instructions(), 3);
    }

    #[test]
    fn test_get_all_instructions_returns_flat_list() {
        let mut instructions = DecisionInstructions::new();
        instructions.add_instruction(1, make_instruction("inst_1", "Decision 1"));
        instructions.add_instruction(2, make_instruction("inst_2", "Decision 2"));
        instructions.add_instruction(3, make_instruction("inst_3", "Decision 3"));

        let all = instructions.get_all_instructions();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_get_instructions_for_decision_returns_first() {
        let mut instructions = DecisionInstructions::new();
        let inst1 = make_instruction("inst_1", "First");
        let inst2 = make_instruction("inst_2", "Second");

        instructions.add_instruction(1, inst1.clone());
        instructions.add_instruction(1, inst2);

        let first = instructions.get_instructions_for_decision(1);
        assert!(first.is_some());
        assert_eq!(first.unwrap().id, "inst_1");
    }

    #[test]
    fn test_get_instructions_for_decision_missing_returns_none() {
        let instructions = DecisionInstructions::new();
        assert!(instructions.get_instructions_for_decision(1).is_none());
    }

    // ===== Removal Tests =====

    #[test]
    fn test_remove_instructions_removes_all_for_decision() {
        let mut instructions = DecisionInstructions::new();
        instructions.add_instruction(1, make_instruction("inst_1", "Test 1"));
        instructions.add_instruction(1, make_instruction("inst_2", "Test 2"));
        instructions.add_instruction(2, make_instruction("inst_3", "Test 3"));

        let removed = instructions.remove_instructions(1);

        assert!(removed.is_some());
        assert_eq!(removed.unwrap().len(), 2);
        assert_eq!(instructions.total_instructions(), 1);
        assert!(!instructions.has_instructions(1));
    }

    #[test]
    fn test_remove_instructions_missing_decision_returns_none() {
        let mut instructions = DecisionInstructions::new();
        let removed = instructions.remove_instructions(1);
        assert!(removed.is_none());
    }

    #[test]
    fn test_remove_instruction_by_id_success() {
        let mut instructions = DecisionInstructions::new();
        instructions.add_instruction(1, make_instruction("inst_1", "Test"));
        instructions.add_instruction(1, make_instruction("inst_2", "Test 2"));

        let removed = instructions.remove_instruction_by_id("inst_1");

        assert!(removed.is_some());
        assert_eq!(removed.unwrap().id, "inst_1");
        assert_eq!(instructions.total_instructions(), 1);
    }

    #[test]
    fn test_remove_instruction_by_id_not_found() {
        let mut instructions = DecisionInstructions::new();
        instructions.add_instruction(1, make_instruction("inst_1", "Test"));

        let removed = instructions.remove_instruction_by_id("nonexistent");

        assert!(removed.is_none());
        assert_eq!(instructions.total_instructions(), 1);
    }

    #[test]
    fn test_remove_instruction_by_id_across_decisions() {
        let mut instructions = DecisionInstructions::new();
        instructions.add_instruction(1, make_instruction("inst_1", "Decision 1"));
        instructions.add_instruction(2, make_instruction("inst_2", "Decision 2"));
        instructions.add_instruction(3, make_instruction("inst_3", "Decision 3"));

        let removed = instructions.remove_instruction_by_id("inst_2");

        assert!(removed.is_some());
        assert_eq!(instructions.total_instructions(), 2);
        assert!(!instructions.has_instructions(2));
    }

    // ===== Multiple Instructions Tests =====

    #[test]
    fn test_add_multiple_instructions_to_same_decision() {
        let mut instructions = DecisionInstructions::new();

        instructions.add_instruction(1, make_instruction("inst_1", "First"));
        instructions.add_instruction(1, make_instruction("inst_2", "Second"));
        instructions.add_instruction(1, make_instruction("inst_3", "Third"));

        assert_eq!(instructions.total_instructions(), 3);
        assert_eq!(instructions.get_instructions(1).unwrap().len(), 3);
    }

    #[test]
    fn test_add_instructions_to_different_decisions() {
        let mut instructions = DecisionInstructions::new();

        instructions.add_instruction(1, make_instruction("inst_1", "D1"));
        instructions.add_instruction(2, make_instruction("inst_2", "D2"));
        instructions.add_instruction(3, make_instruction("inst_3", "D3"));
        instructions.add_instruction(10, make_instruction("inst_4", "D10"));

        assert_eq!(instructions.total_instructions(), 4);
        assert_eq!(instructions.get_instructions(1).unwrap().len(), 1);
        assert_eq!(instructions.get_instructions(10).unwrap().len(), 1);
    }

    // ===== Status Filtering Tests =====

    #[test]
    fn test_get_instructions_by_status_active() {
        let mut instructions = DecisionInstructions::new();

        instructions.add_instruction(1, make_instruction("inst_1", "Active 1"));
        instructions.add_instruction(2, make_instruction("inst_2", "Active 2"));

        let mut addressed = make_instruction("inst_3", "Addressed");
        addressed.status = InstructionStatus::Addressed;
        instructions.add_instruction(3, addressed);

        let active = instructions.get_instructions_by_status(&InstructionStatus::Active);
        assert_eq!(active.len(), 2);
    }

    #[test]
    fn test_get_instructions_by_status_addressed() {
        let mut instructions = DecisionInstructions::new();

        let mut addressed1 = make_instruction("inst_1", "Addressed 1");
        addressed1.status = InstructionStatus::Addressed;
        instructions.add_instruction(1, addressed1);

        let mut addressed2 = make_instruction("inst_2", "Addressed 2");
        addressed2.status = InstructionStatus::Addressed;
        instructions.add_instruction(2, addressed2);

        instructions.add_instruction(3, make_instruction("inst_3", "Active"));

        let addressed = instructions.get_instructions_by_status(&InstructionStatus::Addressed);
        assert_eq!(addressed.len(), 2);
    }

    // ===== Serialization Tests =====

    #[test]
    fn test_serialization_round_trip() {
        let mut instructions = DecisionInstructions::new();
        instructions.add_instruction(1, make_instruction("inst_1", "Test 1"));
        instructions.add_instruction(2, make_instruction("inst_2", "Test 2"));

        let json = serde_json::to_string(&instructions).unwrap();
        let deserialized: DecisionInstructions = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.total_instructions(), 2);
        assert_eq!(deserialized.get_instructions(1).unwrap()[0].id, "inst_1");
        assert_eq!(deserialized.get_instructions(2).unwrap()[0].id, "inst_2");
    }

    #[test]
    fn test_deserialization_preserves_structure() {
        let json = r#"{
            "instructions": {
                "1": [
                    {
                        "id": "inst_1",
                        "content": "Test instruction",
                        "author": "reviewer",
                        "timestamp": "2023-01-01T00:00:00Z",
                        "status": "active"
                    }
                ],
                "2": [
                    {
                        "id": "inst_2",
                        "content": "Another instruction",
                        "author": "reviewer",
                        "timestamp": "2023-01-01T00:00:00Z",
                        "status": "addressed"
                    }
                ]
            }
        }"#;

        let instructions: DecisionInstructions = serde_json::from_str(json).unwrap();

        assert_eq!(instructions.total_instructions(), 2);
        assert!(instructions.has_instructions(1));
        assert!(instructions.has_instructions(2));
        assert_eq!(
            instructions.get_instructions(1).unwrap()[0].status,
            InstructionStatus::Active
        );
        assert_eq!(
            instructions.get_instructions(2).unwrap()[0].status,
            InstructionStatus::Addressed
        );
    }
}
