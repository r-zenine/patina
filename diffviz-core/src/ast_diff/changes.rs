use crate::ast_diff::nodes::NodeRef;
use crate::ast_diff::strategies::ASTChangeType;

/// Numerical relevance score for context expansion
/// Lower values indicate higher relevance to the change
pub type RelevanceScore = u8;

/// Relevance scoring constants
pub const ESSENTIAL: RelevanceScore = 0; // Contains or is the actual change
pub const IMPORTANT: RelevanceScore = 1; // Direct semantic container of change  
pub const BACKGROUND: RelevanceScore = 2; // Sibling context (collapsible in UI)
pub const NOISE: RelevanceScore = 3; // Unrelated context (hideable in UI)

/// Represents a specific type of change between two AST nodes
#[derive(Debug, Clone, PartialEq)]
pub enum ASTChange<'result> {
    /// Node was added (only exists in new tree)
    Addition(NodeRef<'result>),
    /// Node was deleted (only exists in old tree)
    Deletion(NodeRef<'result>),
    /// Same node kind but different text content (atomic nodes only)
    ContentChange {
        old: NodeRef<'result>,
        new: NodeRef<'result>,
    },
    /// Same node kind but different internal structure
    StructuralChange {
        old: NodeRef<'result>,
        new: NodeRef<'result>,
    },
    /// Node kind completely changed (function → struct, etc.)
    KindChange {
        old: NodeRef<'result>,
        new: NodeRef<'result>,
    },
    /// Children were reordered but not added/removed
    Reorder { parent: NodeRef<'result> },
}

impl<'result> ASTChange<'result> {
    /// Get the type category of this change
    pub fn change_type(&self) -> ASTChangeType {
        match self {
            ASTChange::Addition(_) | ASTChange::Deletion(_) => ASTChangeType::Structural,
            ASTChange::ContentChange { .. } => ASTChangeType::Content,
            ASTChange::StructuralChange { .. } => ASTChangeType::Structural,
            ASTChange::KindChange { .. } => ASTChangeType::Rename,
            ASTChange::Reorder { .. } => ASTChangeType::Reorder,
        }
    }

    /// Get the primary node involved in this change (for context boundary detection)
    pub fn primary_node(&self) -> &NodeRef<'result> {
        match self {
            ASTChange::Addition(node) => node,
            ASTChange::Deletion(node) => node,
            ASTChange::ContentChange { new, .. } => new, // Use new version for context
            ASTChange::StructuralChange { new, .. } => new, // Use new version for context
            ASTChange::KindChange { new, .. } => new,    // Use new version for context
            ASTChange::Reorder { parent } => parent,
        }
    }
}

/// Context expansion result - enriches AST changes with semantic context
#[derive(Debug, Clone)]
pub struct ChangeWithContext<'source> {
    /// The original AST changes that triggered context expansion (multiple changes can share a boundary)
    pub original_changes: Vec<ASTChange<'source>>,
    /// The boundary node that defines the context scope
    pub context_boundary: NodeRef<'source>,
    /// The context tree with relevance scores (merged for multiple changes)
    pub context_tree: ContextNode<'source>,
}

/// A node in the context tree with relevance classification
#[derive(Debug, Clone)]
pub struct ContextNode<'source> {
    /// The TreeSitter node
    pub node: NodeRef<'source>,
    /// Relevance score (0 = most relevant, higher = less relevant)
    pub relevance: RelevanceScore,
    /// Child nodes in the context tree
    pub children: Vec<ContextNode<'source>>,
}

impl<'source> ContextNode<'source> {
    /// Create a new context node
    pub fn new(node: NodeRef<'source>, relevance: RelevanceScore) -> Self {
        Self {
            node,
            relevance,
            children: Vec::new(),
        }
    }

    /// Add a child to this context node
    pub fn add_child(&mut self, child: ContextNode<'source>) {
        self.children.push(child);
    }

    /// Get the minimum relevance score in this subtree
    pub fn min_relevance(&self) -> RelevanceScore {
        let child_min = self
            .children
            .iter()
            .map(|child| child.min_relevance())
            .min()
            .unwrap_or(NOISE);

        self.relevance.min(child_min)
    }
}

/// Result of comparing two AST trees
#[derive(Debug, Clone)]
pub struct ASTDiff<'result> {
    /// All changes detected between the trees
    pub changes: Vec<ASTChange<'result>>,
}

impl<'result> Default for ASTDiff<'result> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'result> ASTDiff<'result> {
    /// Create an empty diff result
    pub fn new() -> Self {
        Self {
            changes: Vec::new(),
        }
    }

    /// Check if there are any differences
    pub fn has_changes(&self) -> bool {
        !self.changes.is_empty()
    }

    /// Total number of changes
    pub fn total_changes(&self) -> usize {
        self.changes.len()
    }

    /// Get all additions
    pub fn additions(&self) -> impl Iterator<Item = &NodeRef<'result>> {
        self.changes.iter().filter_map(|change| match change {
            ASTChange::Addition(node) => Some(node),
            _ => None,
        })
    }

    /// Get all deletions
    pub fn deletions(&self) -> impl Iterator<Item = &NodeRef<'result>> {
        self.changes.iter().filter_map(|change| match change {
            ASTChange::Deletion(node) => Some(node),
            _ => None,
        })
    }

    /// Get all content changes
    pub fn content_changes(&self) -> impl Iterator<Item = (&NodeRef<'result>, &NodeRef<'result>)> {
        self.changes.iter().filter_map(|change| match change {
            ASTChange::ContentChange { old, new } => Some((old, new)),
            _ => None,
        })
    }

    /// Get all structural changes
    pub fn structural_changes(
        &self,
    ) -> impl Iterator<Item = (&NodeRef<'result>, &NodeRef<'result>)> {
        self.changes.iter().filter_map(|change| match change {
            ASTChange::StructuralChange { old, new } => Some((old, new)),
            _ => None,
        })
    }

    /// Get all kind changes
    pub fn kind_changes(&self) -> impl Iterator<Item = (&NodeRef<'result>, &NodeRef<'result>)> {
        self.changes.iter().filter_map(|change| match change {
            ASTChange::KindChange { old, new } => Some((old, new)),
            _ => None,
        })
    }

    /// Get all reorders
    pub fn reorders(&self) -> impl Iterator<Item = &NodeRef<'result>> {
        self.changes.iter().filter_map(|change| match change {
            ASTChange::Reorder { parent } => Some(parent),
            _ => None,
        })
    }
}
