use crate::ast_diff::changes::{ASTChange, ASTDiff};
use crate::ast_diff::nodes::NodeRef;
use crate::ast_diff::source::{SourceCode, SourceProvider};
use crate::ast_diff::strategies::{ChangeDetectionStrategies, is_literal_node};
use std::collections::HashMap;
use tree_sitter::{Node, Tree};

/// Merkle tree node containing TreeSitter node + strategy hashes
#[derive(Debug, Clone)]
pub struct MerkleASTNode<'source> {
    /// TreeSitter node reference
    pub node: Node<'source>,

    /// Hash values computed by different strategies  
    pub strategy_hashes: HashMap<&'static str, u64>,

    /// Merkle children
    pub children: Vec<MerkleASTNode<'source>>,

    /// Tree depth for debugging
    pub depth: usize,
}

impl<'source> MerkleASTNode<'source> {
    /// Get hash for a specific strategy
    pub fn get_hash(&self, strategy_name: &str) -> Option<u64> {
        self.strategy_hashes.get(strategy_name).copied()
    }

    /// Check if two nodes differ according to a strategy
    pub fn differs_by_strategy(&self, other: &Self, strategy_name: &str) -> bool {
        match (self.get_hash(strategy_name), other.get_hash(strategy_name)) {
            (Some(old), Some(new)) => old != new,
            _ => false, // Strategy not computed for one of the nodes
        }
    }

    /// Get all nodes in subtree with their hashes (for move detection)
    pub fn collect_all_nodes(&self) -> HashMap<u64, &Self> {
        let mut nodes = HashMap::new();
        self.collect_nodes_recursive(&mut nodes);
        nodes
    }

    fn collect_nodes_recursive<'a>(&'a self, nodes: &mut HashMap<u64, &'a Self>) {
        // Use unified structural hash as the key for move detection
        if let Some(unified_hash) = self.get_hash("unified_structural") {
            nodes.insert(unified_hash, self);
        }

        for child in &self.children {
            child.collect_nodes_recursive(nodes);
        }
    }
}

/// Build Merkle tree from TreeSitter AST using content-aware strategies
pub fn build_merkle_tree<'source>(
    tree: &'source Tree,
    strategies: &ChangeDetectionStrategies,
    source: &dyn SourceProvider,
) -> MerkleASTNode<'source> {
    fn build_recursive<'source>(
        node: &Node<'source>,
        strategies: &ChangeDetectionStrategies,
        source: &dyn SourceProvider,
        depth: usize,
    ) -> MerkleASTNode<'source> {
        // Recursively build children first (bottom-up)
        let children: Vec<_> = node
            .children(&mut node.walk())
            .map(|child| build_recursive(&child, strategies, source, depth + 1))
            .collect();

        // Compute strategy hashes by using appropriate children hashes for each strategy
        let mut strategy_hashes = HashMap::new();

        for strategy in strategies.strategies() {
            // Use the same strategy's hashes from children
            let children_hashes: Vec<u64> = children
                .iter()
                .filter_map(|child| child.get_hash(strategy.name()))
                .collect();

            let hash = strategy.compute_hash(node, &children_hashes, source);
            strategy_hashes.insert(strategy.name(), hash);
        }

        MerkleASTNode {
            node: *node,
            strategy_hashes,
            children,
            depth,
        }
    }

    build_recursive(&tree.root_node(), strategies, source, 0)
}

/// Detect reorder changes by comparing child ordering strategies
pub fn detect_reorder<'result>(
    old: &MerkleASTNode<'result>,
    new: &MerkleASTNode<'result>,
) -> Option<ASTChange<'result>> {
    // Same child set, different order = reorder
    if old.get_hash("child_set") == new.get_hash("child_set")
        && old.differs_by_strategy(new, "child_order")
    {
        Some(ASTChange::Reorder {
            parent: NodeRef::new(old.node),
        })
    } else {
        None
    }
}

/// Detect content changes using the unified structural strategy
/// Content changes are detected when literal nodes have different content but same structure
pub fn detect_content_change<'result>(
    old: &MerkleASTNode<'result>,
    new: &MerkleASTNode<'result>,
) -> Option<ASTChange<'result>> {
    // For literal nodes, content changes are detected through unified_structural hash differences
    // at the node level (the strategy includes both structure and content for literals)
    if is_literal_node(old.node.kind()) && is_literal_node(new.node.kind()) {
        if old.node.kind() == new.node.kind() && old.differs_by_strategy(new, "unified_structural")
        {
            Some(ASTChange::ContentChange {
                old: NodeRef::new(old.node),
                new: NodeRef::new(new.node),
            })
        } else {
            None
        }
    } else {
        None
    }
}

/// Core strategy-based Merkle tree diffing
pub fn diff_merkle_trees<'old, 'new, 'result>(
    old_tree: &MerkleASTNode<'old>,
    new_tree: &MerkleASTNode<'new>,
    strategies: &ChangeDetectionStrategies,
) -> Vec<ASTChange<'result>>
where
    'old: 'result,
    'new: 'result,
{
    let old_unified = old_tree.get_hash("unified_structural").unwrap_or(0);
    let new_unified = new_tree.get_hash("unified_structural").unwrap_or(0);

    // Base case: if unified hashes are identical, no changes in subtree
    if old_unified == new_unified {
        return vec![];
    }

    let mut changes = Vec::new();

    // Check for specific change types
    if let Some(reorder) = detect_reorder(old_tree, new_tree) {
        changes.push(reorder);
    } else if let Some(content_change) = detect_content_change(old_tree, new_tree) {
        changes.push(content_change);
    } else if old_tree.node.kind() != new_tree.node.kind() {
        // Different node kinds = kind change
        changes.push(ASTChange::KindChange {
            old: NodeRef::new(old_tree.node),
            new: NodeRef::new(new_tree.node),
        });
    } else {
        // Structural change - recurse into children
        changes.extend(diff_children_merkle(old_tree, new_tree, strategies));
    }

    changes
}

/// Compare children using Merkle tree approach
pub fn diff_children_merkle<'old, 'new, 'result>(
    old_parent: &MerkleASTNode<'old>,
    new_parent: &MerkleASTNode<'new>,
    strategies: &ChangeDetectionStrategies,
) -> Vec<ASTChange<'result>>
where
    'old: 'result,
    'new: 'result,
{
    let mut changes = Vec::new();
    let max_children = old_parent.children.len().max(new_parent.children.len());

    for i in 0..max_children {
        match (old_parent.children.get(i), new_parent.children.get(i)) {
            (Some(old_child), Some(new_child)) => {
                changes.extend(diff_merkle_trees(old_child, new_child, strategies));
            }
            (Some(old_child), None) => {
                changes.push(ASTChange::Deletion(NodeRef::new(old_child.node)));
            }
            (None, Some(new_child)) => {
                changes.push(ASTChange::Addition(NodeRef::new(new_child.node)));
            }
            (None, None) => unreachable!(),
        }
    }

    changes
}

/// Strategy-based AST diffing with Merkle trees and content-aware hashing
pub fn diff_ast_trees_with_strategies<'old, 'new, 'result>(
    old_tree: &'old Tree,
    new_tree: &'new Tree,
    old_source: &str,
    new_source: &str,
    strategies: ChangeDetectionStrategies,
) -> ASTDiff<'result>
where
    'old: 'result,
    'new: 'result,
{
    // Build Merkle trees using TreeSitter Node data and restricted source access
    let old_source_code = SourceCode::new(old_source);
    let new_source_code = SourceCode::new(new_source);
    let old_merkle = build_merkle_tree(old_tree, &strategies, &old_source_code);
    let new_merkle = build_merkle_tree(new_tree, &strategies, &new_source_code);

    // Diff using pure hash comparison
    let changes = diff_merkle_trees(&old_merkle, &new_merkle, &strategies);

    ASTDiff { changes }
}

/// Core AST diffing algorithm (legacy - will be replaced)
///
/// Compares two TreeSitter AST trees and identifies differences at the subtree level.
/// Returns references to differing subtrees without complex semantic analysis.
pub fn diff_ast_trees<'old, 'new, 'result>(
    old_tree: &'old Tree,
    new_tree: &'new Tree,
) -> ASTDiff<'result>
where
    'old: 'result,
    'new: 'result,
{
    let mut diff = ASTDiff::new();

    // Get root nodes for comparison
    let old_root = old_tree.root_node();
    let new_root = new_tree.root_node();

    // Compare the trees starting from root
    compare_nodes(&old_root, &new_root, &mut diff);

    diff
}

/// Recursively compare two AST nodes and their children
fn compare_nodes<'old, 'new, 'result>(
    old_node: &Node<'old>,
    new_node: &Node<'new>,
    diff: &mut ASTDiff<'result>,
) where
    'old: 'result,
    'new: 'result,
{
    // Check 1: Different node kinds = KindChange
    if old_node.kind() != new_node.kind() {
        diff.changes.push(ASTChange::KindChange {
            old: NodeRef::new(*old_node),
            new: NodeRef::new(*new_node),
        });
        return;
    }

    // Check 2: For atomic nodes, compare content
    let is_atomic_node = matches!(
        old_node.kind(),
        "string_literal" | "integer_literal" | "float_literal" | "boolean_literal" | "identifier"
    );

    if is_atomic_node {
        // For atomic nodes, compare byte ranges to detect content changes
        let old_range = (old_node.start_byte(), old_node.end_byte());
        let new_range = (new_node.start_byte(), new_node.end_byte());

        if old_range != new_range {
            // Same kind, different content = ContentChange
            diff.changes.push(ASTChange::ContentChange {
                old: NodeRef::new(*old_node),
                new: NodeRef::new(*new_node),
            });
            return;
        }
    }

    // For non-atomic nodes, we need to recurse into children to find specific changes
    // We'll only mark as StructuralChange if no more granular changes are found

    // Note: We'll let the child comparison logic handle the detection
    // If children differ, it will detect Additions/Deletions
    // If children are similar but content differs, it will detect more specific changes

    // Compare children - this is where structural changes are detected
    compare_children(old_node, new_node, diff);
}

/// Compare children of two nodes to detect structural changes
fn compare_children<'old, 'new, 'result>(
    old_node: &Node<'old>,
    new_node: &Node<'new>,
    diff: &mut ASTDiff<'result>,
) where
    'old: 'result,
    'new: 'result,
{
    let old_children: Vec<Node> = old_node.children(&mut old_node.walk()).collect();
    let new_children: Vec<Node> = new_node.children(&mut new_node.walk()).collect();

    // Simple approach: compare children by position
    // TODO: This could be improved with more sophisticated matching
    let max_children = old_children.len().max(new_children.len());

    for i in 0..max_children {
        match (old_children.get(i), new_children.get(i)) {
            (Some(old_child), Some(new_child)) => {
                // Both children exist, compare them recursively
                compare_nodes(old_child, new_child, diff);
            }
            (Some(old_child), None) => {
                // Child was deleted
                diff.changes
                    .push(ASTChange::Deletion(NodeRef::new(*old_child)));
            }
            (None, Some(new_child)) => {
                // Child was added
                diff.changes
                    .push(ASTChange::Addition(NodeRef::new(*new_child)));
            }
            (None, None) => unreachable!(),
        }
    }
}
