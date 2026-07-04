// Individual review entity modules
pub mod approval;
pub mod cascade_result;
pub mod decision;
pub mod decision_instructions;
pub mod instruction;

// Core identifier module
pub mod reviewable_diff_id;

// Git reference types
pub mod git_ref;

// Re-exports
pub use approval::{Approval, ApprovalMap, ApprovalRecord, DecisionApprovals, ReviewApprovals};
pub use cascade_result::CascadeResult;
pub use decision::{
    CodeImpact, Decision, DecisionApproval, DecisionLineRange, DecisionLog, DecisionReviewableDiff,
    ReasoningConventionViolation, ReviewDecisions,
};
pub use instruction::{
    DecisionInstructions, Instruction, InstructionMap, InstructionStatus, ReviewInstructions,
};
