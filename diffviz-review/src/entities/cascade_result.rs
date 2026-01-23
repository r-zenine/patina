/// Results from cascade operations (decision approval/rejection cascading to chunks)
///
/// When approving or rejecting a decision, the operation cascades to all related chunks.
/// This type captures what happened during that cascade operation for UI feedback.
///
/// Result of a decision cascade operation
///
/// Returned by `approve_decision()` and `reject_decision()` to describe what changed.
/// The TUI can use this to provide feedback about the scope of the operation.
#[derive(Debug, Clone)]
pub enum CascadeResult {
    /// Decision was approved and cascaded to all its chunks
    DecisionApproved {
        /// The decision number that was approved
        decision_number: u32,
        /// How many chunks were affected by this cascade
        chunks_affected: usize,
    },

    /// Decision was unapproved/rejected and cascaded to all its chunks
    DecisionUnapproved {
        /// The decision number that was unapproved
        decision_number: u32,
        /// How many chunks were affected by this cascade
        chunks_affected: usize,
    },

    /// Edge case: decision had no chunks to cascade to
    NoChunksAffected {
        /// The decision number
        decision_number: u32,
    },
}

impl CascadeResult {
    /// Get the decision number involved in this cascade
    pub fn decision_number(&self) -> u32 {
        match self {
            CascadeResult::DecisionApproved {
                decision_number, ..
            } => *decision_number,
            CascadeResult::DecisionUnapproved {
                decision_number, ..
            } => *decision_number,
            CascadeResult::NoChunksAffected { decision_number } => *decision_number,
        }
    }

    /// Get a human-readable description of what happened
    pub fn description(&self) -> String {
        match self {
            CascadeResult::DecisionApproved {
                decision_number,
                chunks_affected,
            } => {
                let chunk_word = if *chunks_affected == 1 { "chunk" } else { "chunks" };
                format!("Decision #{decision_number} and {chunks_affected} {chunk_word} approved")
            }
            CascadeResult::DecisionUnapproved {
                decision_number,
                chunks_affected,
            } => {
                let chunk_word = if *chunks_affected == 1 { "chunk" } else { "chunks" };
                format!("Decision #{decision_number} and {chunks_affected} {chunk_word} unapproved")
            }
            CascadeResult::NoChunksAffected { decision_number } => {
                format!("Decision #{decision_number} had no chunks to affect")
            }
        }
    }

    /// Check if this cascade actually affected any chunks
    pub fn affected_any(&self) -> bool {
        !matches!(self, CascadeResult::NoChunksAffected { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approved_result_description() {
        let result = CascadeResult::DecisionApproved {
            decision_number: 1,
            chunks_affected: 3,
        };
        assert_eq!(result.description(), "Decision #1 and 3 chunks approved");
    }

    #[test]
    fn approved_result_single_chunk() {
        let result = CascadeResult::DecisionApproved {
            decision_number: 2,
            chunks_affected: 1,
        };
        assert_eq!(result.description(), "Decision #2 and 1 chunk approved");
    }

    #[test]
    fn unapproved_result_description() {
        let result = CascadeResult::DecisionUnapproved {
            decision_number: 5,
            chunks_affected: 2,
        };
        assert_eq!(result.description(), "Decision #5 and 2 chunks unapproved");
    }

    #[test]
    fn no_chunks_affected_description() {
        let result = CascadeResult::NoChunksAffected {
            decision_number: 10,
        };
        assert!(!result.affected_any());
        assert_eq!(result.decision_number(), 10);
    }
}
