//! Pure AST-based tree diffing algorithm
//!
//! This module provides a simple and focused approach to AST diffing:
//! - Input: Two TreeSitter AST trees (old, new)  
//! - Output: List of subtree reference pairs highlighting differences
//!
//! The algorithm identifies three types of changes:
//! 1. Structural changes - nodes added/removed from tree structure
//! 2. Content changes - same structure but different text/values  
//! 3. Positional changes - nodes moved within the tree

// Module declarations
pub mod changes;
pub mod error;
pub mod merkle;
pub mod nodes;
pub mod source;
pub mod strategies;

#[cfg(test)]
mod tests;

// Re-exports to preserve external import paths
pub use changes::{
    ASTChange, ASTDiff, BACKGROUND, ChangeWithContext, ContextNode, ESSENTIAL, IMPORTANT, NOISE,
    RelevanceScore,
};
pub use error::SourceError;
pub use merkle::{
    MerkleASTNode, build_merkle_tree, detect_content_change, detect_reorder, diff_ast_trees,
    diff_ast_trees_with_strategies, diff_children_merkle, diff_merkle_trees,
};
pub use nodes::{NodeLike, NodeRef, OwnedNodeData};
pub use source::{FullSourceProvider, LineRange, SourceCode, SourceProvider};
pub use strategies::{
    ASTChangeType, ChangeDetectionStrategies, ChangeDetectionStrategy, ChildOrderStrategy,
    ChildSetStrategy, UnifiedStructuralStrategy, is_literal_node,
};
