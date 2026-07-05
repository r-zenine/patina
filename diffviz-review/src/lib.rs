pub mod adapters;
pub mod engines;
pub mod entities;
pub mod errors;
pub mod persistence;
pub mod providers;
pub mod review_engine_builder;
pub mod state;
pub mod summary;
pub mod templates;

// Re-export key types for external use
pub use engines::{ReviewEngine, ReviewProgress};
pub use entities::git_ref::{DiffQuery, GitRef};
pub use entities::reviewable_diff_id::{LineRange, ReviewableDiffId};
pub use entities::{
    ApprovalMap, ApprovalRecord, CodeImpact, CommitViolation, Decision, DecisionApproval,
    DecisionApprovals, DecisionInstructions, DecisionLineRange, DecisionLog,
    DecisionReviewableDiff, Instruction, InstructionMap, ReasoningConventionViolation,
    ReviewApprovals, ReviewDecisions, ReviewInstructions,
};
pub use persistence::{
    PersistenceError, load_review_state, save_review_state, summarize_review_state,
};
pub use providers::{DiffProvider, FileStats, FileStatus};
pub use review_engine_builder::ReviewEngineBuilder;
pub use state::{ReviewState, ReviewableDiff};
pub use summary::ReviewSummary;
pub use templates::SchemaTemplate;
