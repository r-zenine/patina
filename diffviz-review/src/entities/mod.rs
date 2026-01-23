// Individual review entity modules
pub mod approval;
pub mod decision;
pub mod instruction;

// Core identifier module
pub mod reviewable_diff_id;

// Git reference types
pub mod git_ref;

// Re-exports for backward compatibility
pub use approval::{Approval, ReviewApprovals};
pub use decision::{
    ChangeType, CodeImpact, Confidence, Decision, DecisionApproval, DecisionApprovals,
    DecisionLineRange, ReviewDecisions,
};
pub use instruction::{Instruction, ReviewInstructions};
