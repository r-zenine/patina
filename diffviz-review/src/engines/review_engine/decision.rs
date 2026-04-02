use super::ReviewEngine;
use crate::entities::instruction::InstructionStatus;
use crate::entities::{CascadeResult, Instruction};
use crate::errors::Result;

impl ReviewEngine {
    /// Get all chunks (ReviewableDiffIds) for a specific decision
    fn get_chunks_for_decision(
        &self,
        decision_number: u32,
    ) -> Vec<crate::entities::reviewable_diff_id::ReviewableDiffId> {
        self.state
            .decisions
            .decision_index
            .iter()
            .filter(|(_, decision_nums)| decision_nums.contains(&decision_number))
            .map(|(diff_id, _)| diff_id.clone())
            .collect()
    }

    fn cascade_decision(
        &mut self,
        decision_number: u32,
        reviewer: Option<String>,
    ) -> Result<CascadeResult> {
        if let Some(ref r) = reviewer {
            self.state.approve_decision(decision_number, r.clone());
        } else {
            self.state.unapprove_decision(decision_number);
        }

        let chunks = self.get_chunks_for_decision(decision_number);

        for chunk_id in &chunks {
            if let Some(ref r) = reviewer {
                self.state.approve(chunk_id.clone(), r.clone());
            } else {
                self.state.unapprove(chunk_id);
            }
            self.invalidate_cache(chunk_id);
        }

        Ok(if chunks.is_empty() {
            CascadeResult::NoChunksAffected { decision_number }
        } else if reviewer.is_some() {
            CascadeResult::DecisionApproved {
                decision_number,
                chunks_affected: chunks.len(),
            }
        } else {
            CascadeResult::DecisionUnapproved {
                decision_number,
                chunks_affected: chunks.len(),
            }
        })
    }

    /// Approve an entire decision, cascading to all affected chunks
    ///
    /// Returns a CascadeResult describing what was affected by this operation.
    pub fn approve_decision(
        &mut self,
        decision_number: u32,
        reviewer: String,
    ) -> Result<CascadeResult> {
        self.cascade_decision(decision_number, Some(reviewer))
    }

    /// Reject/unapprove an entire decision, cascading to all affected chunks
    ///
    /// Returns a CascadeResult describing what was affected by this operation.
    pub fn reject_decision(&mut self, decision_number: u32) -> Result<CascadeResult> {
        self.cascade_decision(decision_number, None)
    }

    /// Check if a decision is approved
    pub fn is_decision_approved(&self, decision_number: u32) -> bool {
        self.state.is_decision_approved(decision_number)
    }

    /// Get approval progress for a decision: (approved_chunks, total_chunks)
    pub fn decision_approval_progress(&self, decision_number: u32) -> (usize, usize) {
        self.state.decision_approval_progress(decision_number)
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
        reviewable_id: &crate::entities::reviewable_diff_id::ReviewableDiffId,
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
            .get_instructions(&decision_number)
            .map(|instructions| instructions.iter().collect())
    }
}

#[cfg(test)]
#[path = "decision_tests.rs"]
mod tests;
