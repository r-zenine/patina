// Temporarily disabled: debug module needs significant refactoring
// pub mod debug;
pub mod engines;
pub mod entities;
pub mod errors;
pub mod providers;
pub mod review_engine_builder;
// Temporarily disabled: session needs refactoring
// pub mod session;
pub mod state;

// Re-export key types for external use
pub use engines::{CacheStats, ReviewEngine, ReviewProgress, ReviewSummary};
pub use entities::git_ref::{DiffQuery, GitRef};
pub use entities::reviewable_diff_id::{LineRange, ReviewableDiffId};
pub use entities::{
    Approval, CodeImpact, Decision, DecisionLineRange, DecisionLog, DecisionReviewableDiff,
    Instruction, ReviewApprovals, ReviewDecisions, ReviewInstructions,
};
pub use providers::{DiffProvider, FileStats, FileStatus};
pub use review_engine_builder::ReviewEngineBuilder;
pub use state::{ReviewState, ReviewableDiff};
