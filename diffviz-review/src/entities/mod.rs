// Individual review entity modules
pub mod approval;
pub mod instruction;

// Core identifier module
pub mod reviewable_diff_id;

// Git reference types
pub mod git_ref;

// Re-exports for backward compatibility
pub use approval::{Approval, ReviewApprovals};
pub use instruction::{Instruction, ReviewInstructions};
