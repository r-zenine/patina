//! Instruction system for code review workflow
//!
//! This module contains the instruction entities used in the review system,
//! allowing reviewers to provide actionable guidance for code changes.

use crate::entities::reviewable_diff_id::ReviewableDiffId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;

/// Status of an instruction indicating its validity after code changes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum InstructionStatus {
    /// Instruction points to correct code
    #[default]
    Active,
    /// User marked instruction as handled/completed
    Addressed,
}

/// An instruction for changes on a reviewable diff
///
/// Each key holds a single instruction — the one shared note for that chunk
/// or decision. "Editing" means appending to it (see [`Instruction::append`]),
/// never creating a second instruction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Instruction {
    pub id: String,
    pub author: String,
    pub timestamp: String,
    pub content: String,
    #[serde(default)]
    pub status: InstructionStatus,
}

impl Instruction {
    /// Folds a new contribution into this instruction (the single-note model).
    ///
    /// The new content lands on its own line, a distinct contributor is added
    /// to the comma-separated author list, the timestamp moves to the latest
    /// contribution, and the note reactivates — fresh guidance means it is no
    /// longer addressed.
    pub fn append(&mut self, content: &str, author: &str, timestamp: String) {
        self.content.push('\n');
        self.content.push_str(content);
        if !self.author.split(", ").any(|a| a == author) {
            self.author.push_str(", ");
            self.author.push_str(author);
        }
        self.timestamp = timestamp;
        self.status = InstructionStatus::Active;
    }
}

/// Generic collection of instructions keyed by any hashable type.
///
/// Used for both chunk-level instructions (`K = ReviewableDiffId`) and
/// decision-level instructions (`K = u32`).
///
/// Enforces the single-note invariant on the write path: adding an
/// instruction to a key that already has one folds the new content into the
/// existing note via [`Instruction::append`]. The `Vec` storage remains for
/// serialization compatibility with previously persisted states; new writes
/// never grow it past one entry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(bound(
    serialize = "K: Serialize + Hash + Eq + Clone",
    deserialize = "K: for<'de2> Deserialize<'de2> + Hash + Eq + Clone"
))]
pub struct InstructionMap<K: Hash + Eq + Clone> {
    pub instructions: HashMap<K, Vec<Instruction>>,
}

impl<K: Hash + Eq + Clone> InstructionMap<K> {
    pub fn new() -> Self {
        Self {
            instructions: HashMap::new(),
        }
    }

    pub fn add_instruction(&mut self, key: K, instruction: Instruction) {
        let notes = self.instructions.entry(key).or_default();
        match notes.first_mut() {
            Some(existing) => existing.append(
                &instruction.content,
                &instruction.author,
                instruction.timestamp,
            ),
            None => notes.push(instruction),
        }
    }

    pub fn get_instructions(&self, key: &K) -> Option<&Vec<Instruction>> {
        self.instructions.get(key)
    }

    pub fn has_instructions(&self, key: &K) -> bool {
        self.instructions.get(key).is_some_and(|v| !v.is_empty())
    }

    pub fn total_instructions(&self) -> usize {
        self.instructions.values().map(|v| v.len()).sum()
    }

    pub fn get_all_instructions(&self) -> Vec<&Instruction> {
        self.instructions.values().flat_map(|v| v.iter()).collect()
    }

    pub fn get_first_instruction(&self, key: &K) -> Option<&Instruction> {
        self.instructions.get(key)?.first()
    }

    pub fn remove_instructions(&mut self, key: &K) -> Option<Vec<Instruction>> {
        self.instructions.remove(key)
    }

    pub fn get_instructions_by_status(&self, status: &InstructionStatus) -> Vec<&Instruction> {
        self.instructions
            .values()
            .flat_map(|v| v.iter())
            .filter(|i| &i.status == status)
            .collect()
    }

    pub fn remove_instruction_by_id(&mut self, instruction_id: &str) -> Option<Instruction> {
        for instructions in self.instructions.values_mut() {
            if let Some(pos) = instructions.iter().position(|i| i.id == instruction_id) {
                return Some(instructions.remove(pos));
            }
        }
        None
    }
}

/// Instructions keyed by ReviewableDiffId (chunk-level annotations)
pub type ReviewInstructions = InstructionMap<ReviewableDiffId>;

impl ReviewInstructions {
    /// Get the first instruction for a ReviewableDiffId (compatibility method)
    pub fn get_instructions_for_reviewable(
        &self,
        reviewable_id: &ReviewableDiffId,
    ) -> Option<&Instruction> {
        self.get_first_instruction(reviewable_id)
    }
}

/// Instructions keyed by decision number (decision-level annotations)
pub type DecisionInstructions = InstructionMap<u32>;

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
    fn test_adding_to_existing_key_folds_into_single_note() {
        let mut instructions = ReviewInstructions::new();
        let reviewable_id = create_test_reviewable_id();

        instructions.add_instruction(
            reviewable_id.clone(),
            make_instruction("instruction_1", "Add error handling"),
        );
        instructions.add_instruction(
            reviewable_id.clone(),
            make_instruction("instruction_2", "Also cover the timeout path"),
        );

        let notes = instructions.get_instructions(&reviewable_id).unwrap();
        assert_eq!(notes.len(), 1, "a key never holds more than one note");
        assert_eq!(notes[0].id, "instruction_1", "the original note persists");
        assert_eq!(
            notes[0].content,
            "Add error handling\nAlso cover the timeout path"
        );
        assert_eq!(instructions.total_instructions(), 1);
    }

    #[test]
    fn test_append_merges_distinct_authors_and_reactivates() {
        let mut note = make_instruction("instruction_1", "Add error handling");
        note.status = InstructionStatus::Addressed;

        note.append("Also cover the timeout path", "bob", "t2".to_string());

        assert_eq!(note.author, "reviewer, bob");
        assert_eq!(note.timestamp, "t2");
        assert_eq!(
            note.status,
            InstructionStatus::Active,
            "fresh guidance reactivates an addressed note"
        );

        // Same contributor again — no duplicate attribution.
        note.append("One more thing", "bob", "t3".to_string());
        assert_eq!(note.author, "reviewer, bob");
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
