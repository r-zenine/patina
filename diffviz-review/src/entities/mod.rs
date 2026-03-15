// Individual review entity modules
pub mod approval;
pub mod cascade_result;
pub mod decision;
pub mod instruction;

// Core identifier module
pub mod reviewable_diff_id;

// Git reference types
pub mod git_ref;

// Re-exports for backward compatibility
pub use approval::{Approval, ReviewApprovals};
pub use cascade_result::CascadeResult;
pub use decision::{
    CodeImpact, Decision, DecisionApproval, DecisionApprovals, DecisionLineRange,
    DecisionReviewableDiff, ReviewDecisions,
};
pub use instruction::{Instruction, ReviewInstructions};
