// Re-export common types for external access
pub use common::*;

// Declare modules
pub mod common;
pub mod parsers;

pub mod ast_diff;
pub mod decision_based_diff;
pub mod renderable_diff;
pub mod reviewable_diff;
pub mod semantic_ast;
// Re-export key types for external use
pub use ast_diff::{ASTChangeType, LineRange, NodeLike, OwnedNodeData, SourceCode, SourceProvider};

pub use reviewable_diff::{DiffMetadata, DiffNode, NodeChangeStatus, ReviewableDiff};

pub use renderable_diff::{
    RenderableDiff, RenderableDiffError, RenderableLine, RenderableMetadata,
};

pub use semantic_ast::{
    ImportType, ModuleType, SemanticError, SemanticNode, SemanticTree,
    SemanticUnitType as SemanticASTUnitType, SourceRange,
};

pub use decision_based_diff::{
    ChangeClassification, DecisionDiffError, create_reviewable_diff_from_range,
};
