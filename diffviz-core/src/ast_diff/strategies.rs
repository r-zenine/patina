use crate::ast_diff::source::SourceProvider;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use tree_sitter::Node;

/// Types of changes that can be detected by different strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ASTChangeType {
    Structural,
    Content,
    Rename,
    Reorder,
}

/// Strategy for detecting specific types of changes in AST nodes
/// All strategies work with TreeSitter Node information and restricted source access
pub trait ChangeDetectionStrategy: Send + Sync + std::fmt::Debug {
    /// Unique identifier for this strategy
    fn name(&self) -> &'static str;

    /// Compute hash using TreeSitter Node information, children hashes, and restricted source access
    ///
    /// # Parameters
    /// - `node`: The TreeSitter node to compute hash for
    /// - `children_hashes`: Hash values from child nodes using this same strategy
    /// - `source`: Restricted source code access for extracting node text content
    ///
    /// # Architectural Note
    /// The `source` parameter provides minimal, AST-node-bound access to prevent string-based
    /// analysis while enabling content-aware change detection.
    fn compute_hash(
        &self,
        node: &Node,
        children_hashes: &[u64],
        source: &dyn SourceProvider,
    ) -> u64;

    /// What type of change this strategy detects
    fn detects_change_type(&self) -> ASTChangeType;

    /// Clone this strategy (for cloning the strategy collection)
    fn clone_strategy(&self) -> Box<dyn ChangeDetectionStrategy>;
}

/// Unified strategy that detects both structural and content changes intelligently
/// Uses content-aware hashing: structure + content for literals, structure only for containers
#[derive(Debug, Clone)]
pub struct UnifiedStructuralStrategy;

impl ChangeDetectionStrategy for UnifiedStructuralStrategy {
    fn name(&self) -> &'static str {
        "unified_structural"
    }

    fn compute_hash(
        &self,
        node: &Node,
        children_hashes: &[u64],
        source: &dyn SourceProvider,
    ) -> u64 {
        let mut hasher = DefaultHasher::new();

        // Always hash the node structure
        node.kind().hash(&mut hasher);
        node.child_count().hash(&mut hasher);

        // For literal/atomic nodes, include content in the hash
        if is_literal_node(node.kind()) {
            if let Ok(text) = source.node_text(node) {
                // Hash the actual text content for content changes
                text.hash(&mut hasher);
            } else {
                // Fallback to byte range if text extraction fails
                let byte_length = node.end_byte() - node.start_byte();
                byte_length.hash(&mut hasher);
            }
        }

        // Always include children hashes for structural awareness
        for &child_hash in children_hashes {
            child_hash.hash(&mut hasher);
        }

        hasher.finish()
    }

    fn detects_change_type(&self) -> ASTChangeType {
        ASTChangeType::Structural
    }

    fn clone_strategy(&self) -> Box<dyn ChangeDetectionStrategy> {
        Box::new(self.clone())
    }
}

/// Check if a node kind represents a literal that should include content in hash
/// This is a carefully curated list of truly atomic content nodes
pub fn is_literal_node(kind: &str) -> bool {
    matches!(
        kind,
        "string_literal"
            | "integer_literal"
            | "float_literal"
            | "boolean_literal"
            | "identifier"
            | "char_literal"
            | "raw_string_literal"
            | "string"
            | "number"
    )
}

/// Strategy that detects reorders by comparing child order
#[derive(Debug, Clone)]
pub struct ChildOrderStrategy;

impl ChangeDetectionStrategy for ChildOrderStrategy {
    fn name(&self) -> &'static str {
        "child_order"
    }

    fn compute_hash(
        &self,
        node: &Node,
        children_hashes: &[u64],
        _source: &dyn SourceProvider,
    ) -> u64 {
        let mut hasher = DefaultHasher::new();

        // Hash depends on exact order of children
        node.kind().hash(&mut hasher);
        for &child_hash in children_hashes {
            child_hash.hash(&mut hasher);
        }

        hasher.finish()
    }

    fn detects_change_type(&self) -> ASTChangeType {
        ASTChangeType::Reorder
    }

    fn clone_strategy(&self) -> Box<dyn ChangeDetectionStrategy> {
        Box::new(self.clone())
    }
}

/// Strategy that detects child set changes (order-independent)
#[derive(Debug, Clone)]
pub struct ChildSetStrategy;

impl ChangeDetectionStrategy for ChildSetStrategy {
    fn name(&self) -> &'static str {
        "child_set"
    }

    fn compute_hash(
        &self,
        node: &Node,
        children_hashes: &[u64],
        _source: &dyn SourceProvider,
    ) -> u64 {
        let mut hasher = DefaultHasher::new();

        // Hash is order-independent (sorted children hashes)
        node.kind().hash(&mut hasher);

        let mut sorted_hashes = children_hashes.to_vec();
        sorted_hashes.sort_unstable();

        for &child_hash in &sorted_hashes {
            child_hash.hash(&mut hasher);
        }

        hasher.finish()
    }

    fn detects_change_type(&self) -> ASTChangeType {
        ASTChangeType::Structural
    }

    fn clone_strategy(&self) -> Box<dyn ChangeDetectionStrategy> {
        Box::new(self.clone())
    }
}

/// Collection of change detection strategies
#[derive(Debug)]
pub struct ChangeDetectionStrategies {
    strategies: Vec<Box<dyn ChangeDetectionStrategy>>,
}

impl Clone for ChangeDetectionStrategies {
    fn clone(&self) -> Self {
        Self {
            strategies: self
                .strategies
                .iter()
                .map(|strategy| strategy.clone_strategy())
                .collect(),
        }
    }
}

impl ChangeDetectionStrategies {
    pub fn new() -> Self {
        Self {
            strategies: Vec::new(),
        }
    }

    pub fn with_strategy(mut self, strategy: Box<dyn ChangeDetectionStrategy>) -> Self {
        self.strategies.push(strategy);
        self
    }

    /// Default set of strategies for comprehensive change detection
    pub fn default_strategies() -> Self {
        Self::new()
            .with_strategy(Box::new(UnifiedStructuralStrategy))
            .with_strategy(Box::new(ChildOrderStrategy))
            .with_strategy(Box::new(ChildSetStrategy))
    }

    /// Compute all strategy hashes for a node
    pub fn compute_all_hashes(
        &self,
        node: &Node,
        children_hashes: &[u64],
        source: &dyn SourceProvider,
    ) -> HashMap<&'static str, u64> {
        self.strategies
            .iter()
            .map(|strategy| {
                (
                    strategy.name(),
                    strategy.compute_hash(node, children_hashes, source),
                )
            })
            .collect()
    }

    /// Get strategies by change type
    pub fn strategies_for_type(
        &self,
        change_type: ASTChangeType,
    ) -> Vec<&dyn ChangeDetectionStrategy> {
        self.strategies
            .iter()
            .filter(|strategy| strategy.detects_change_type() == change_type)
            .map(|strategy| strategy.as_ref())
            .collect()
    }

    /// Access to internal strategies for Merkle tree operations
    pub(crate) fn strategies(&self) -> &[Box<dyn ChangeDetectionStrategy>] {
        &self.strategies
    }
}

impl Default for ChangeDetectionStrategies {
    fn default() -> Self {
        Self::default_strategies()
    }
}
