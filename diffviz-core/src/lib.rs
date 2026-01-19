// Re-export common types for external access
pub use common::*;

// Declare modules
pub mod common;
pub mod parsers;

pub mod ast_diff;
pub mod renderable_diff;
pub mod reviewable_diff;
pub mod reviewable_diff_from_semantic;
pub mod semantic_ast;
pub mod semantic_unit_partitioner;
// Re-export key types for external use
pub use ast_diff::{
    ASTChange, ASTChangeType, ASTDiff, ChangeDetectionStrategies, ChangeDetectionStrategy,
    ChildOrderStrategy, ChildSetStrategy, LineRange, MerkleASTNode, NodeLike, NodeRef,
    OwnedNodeData, SourceCode, SourceProvider, UnifiedStructuralStrategy, diff_ast_trees,
    diff_ast_trees_with_strategies,
};

pub use reviewable_diff::{
    DiffMetadata, DiffNode, NodeChangeStatus, ReviewableDiff, expand_changes_to_reviewable_diffs,
};

pub use renderable_diff::{RenderableDiff, RenderableLine, RenderableMetadata};

pub use semantic_unit_partitioner::{
    PartitioningConfig, PartitioningError, SemanticUnit, SemanticUnitExtractor, SemanticUnitType,
    UnitPair, partition_ast_trees,
};

pub use semantic_ast::{
    CoverageStats, ImportType, ModuleType, SemanticError, SemanticNode, SemanticPair,
    SemanticSimilarity, SemanticTree, SemanticUnitType as SemanticASTUnitType, SourceRange,
    build_semantic_pairs, build_semantic_pairs_with_coverage,
};
