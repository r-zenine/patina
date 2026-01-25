//! Unified ReviewableDiff structure for meaningful code review
//!
//! This module provides a self-contained data structure that consolidates
//! all scattered diff information (AST changes, context trees, relevance scores,
//! source content) into a clean, well-structured container for diff rendering.

use crate::ast_diff::{
    ASTChange, ASTChangeType, ChangeWithContext, ContextNode, NodeLike, NodeRef, OwnedNodeData,
    RelevanceScore, SourceProvider,
};
use crate::common::ProgrammingLanguage;
use crate::common::SemanticNodeKind;
use core::fmt;
use std::collections::HashMap;
use std::time::Instant;

/// Self-contained reviewable diff - pure data container
pub struct ReviewableDiff {
    pub language: ProgrammingLanguage,
    pub boundary: DiffNode,
    pub old_source: Box<dyn SourceProvider>,
    pub new_source: Box<dyn SourceProvider>,
    pub metadata: DiffMetadata,
}

/// Hierarchical diff node preserving AST structure
#[derive(Clone)]
pub struct DiffNode {
    pub node_type: String,
    pub semantic_kind: SemanticNodeKind,
    pub change_status: NodeChangeStatus,
    pub relevance: RelevanceScore,
    pub children: Vec<DiffNode>,
}

impl fmt::Display for DiffNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {} -> {:?}\n",
            self.relevance, self.node_type, self.change_status
        )?;

        for child in &self.children {
            if child.node_type == "block" {
                write!(f, "\t -- {}\n", child)?;
            } else {
                write!(f, "\t  {}\n", child)?;
            }
        }

        Ok(())
    }
}

/// Symmetric enum for all change types with consistent old/new access
#[derive(Debug, Clone)]
pub enum NodeChangeStatus {
    Unchanged {
        node: OwnedNodeData,
    },
    Added {
        node: OwnedNodeData,
    },
    Deleted {
        node: OwnedNodeData,
    },
    Modified {
        old_node: OwnedNodeData,
        new_node: OwnedNodeData,
        change_type: ASTChangeType,
    },
    Moved {
        old_node: OwnedNodeData,
        new_node: OwnedNodeData,
    },
    Reordered {
        old_node: OwnedNodeData,
        new_node: OwnedNodeData,
    },
}

/// Metadata about the diff for UI feedback and statistics
#[derive(Clone)]
pub struct DiffMetadata {
    pub total_changes: usize,
    pub change_summary: HashMap<ASTChangeType, usize>,
    pub essential_node_count: usize,
    pub analysis_duration_ms: u64,
}

impl Clone for ReviewableDiff {
    fn clone(&self) -> Self {
        Self {
            language: self.language,
            boundary: self.boundary.clone(),
            old_source: self.old_source.clone_box(),
            new_source: self.new_source.clone_box(),
            metadata: self.metadata.clone(),
        }
    }
}

impl ReviewableDiff {
    /// Convert from ChangeWithContext to ReviewableDiff
    pub fn from_change_with_context<'source>(
        change_with_context: ChangeWithContext<'source>,
        language: ProgrammingLanguage,
        old_source: &'source dyn SourceProvider,
        new_source: &'source dyn SourceProvider,
        parser: &dyn crate::LanguageParser,
        start_time: Instant,
    ) -> Self {
        let boundary_node = convert_context_node_to_diff_node(
            &change_with_context.context_tree,
            &change_with_context.original_changes,
            parser,
        );

        let essential_count = count_essential_nodes(&change_with_context.context_tree);

        // Build change summary from all original changes
        let mut change_summary = HashMap::new();
        for change in &change_with_context.original_changes {
            let change_type = get_ast_change_type(change);
            *change_summary.entry(change_type).or_insert(0) += 1;
        }

        let metadata = DiffMetadata {
            total_changes: change_with_context.original_changes.len(),
            change_summary,
            essential_node_count: essential_count,
            analysis_duration_ms: start_time.elapsed().as_millis() as u64,
        };

        ReviewableDiff {
            language,
            boundary: boundary_node,
            old_source: old_source.clone_box(),
            new_source: new_source.clone_box(),
            metadata,
        }
    }
}

/// Convert a ContextNode to a DiffNode, mapping change information
fn convert_context_node_to_diff_node<'source>(
    context_node: &ContextNode<'source>,
    original_changes: &[ASTChange<'source>],
    parser: &dyn crate::common::LanguageParser,
) -> DiffNode {
    let node_type = context_node.node.node.kind().to_string();
    let semantic_kind = parser.classify_node_kind(&node_type);

    // Determine change status based on all original changes and node position
    let change_status = determine_node_change_status(context_node, original_changes);

    // Override relevance to ESSENTIAL if this node has actual changes
    let relevance = if has_changes(&change_status) {
        crate::ast_diff::ESSENTIAL
    } else {
        context_node.relevance
    };

    // Recursively convert children
    let children = context_node
        .children
        .iter()
        .map(|child| convert_context_node_to_diff_node(child, original_changes, parser))
        .collect();

    DiffNode {
        node_type,
        semantic_kind,
        change_status,
        relevance,
        children,
    }
}

/// Determine the change status for a node based on its relationship to the original changes
fn determine_node_change_status<'source>(
    context_node: &ContextNode<'source>,
    original_changes: &[ASTChange<'source>],
) -> NodeChangeStatus {
    let node = context_node.node;

    // Check if this node is affected by any of the original changes
    // Priority: Modified > Added > Deleted > Moved > Reordered > Unchanged
    for original_change in original_changes {
        match determine_single_change_status(node, original_change) {
            Some(status) => return status,
            None => continue,
        }
    }

    // If no changes affect this node, it's unchanged
    NodeChangeStatus::Unchanged {
        node: OwnedNodeData::from_node_ref(&node),
    }
}

/// Determine change status for a single change, returns None if node is not affected
fn determine_single_change_status<'source>(
    node: NodeRef<'source>,
    original_change: &ASTChange<'source>,
) -> Option<NodeChangeStatus> {
    // Check if this node is the actual change node
    match original_change {
        ASTChange::Addition(change_node) => {
            if nodes_are_same(node, *change_node) {
                Some(NodeChangeStatus::Added {
                    node: OwnedNodeData::from_node_ref(&node),
                })
            } else {
                None
            }
        }
        ASTChange::Deletion(change_node) => {
            if nodes_are_same(node, *change_node) {
                Some(NodeChangeStatus::Deleted {
                    node: OwnedNodeData::from_node_ref(&node),
                })
            } else {
                None
            }
        }
        ASTChange::ContentChange { old, new } => {
            if nodes_are_same(node, *new) {
                Some(NodeChangeStatus::Modified {
                    old_node: OwnedNodeData::from_node_ref(old),
                    new_node: OwnedNodeData::from_node_ref(new),
                    change_type: ASTChangeType::Content,
                })
            } else {
                None
            }
        }
        ASTChange::StructuralChange { old, new } => {
            if nodes_are_same(node, *new) {
                Some(NodeChangeStatus::Modified {
                    old_node: OwnedNodeData::from_node_ref(old),
                    new_node: OwnedNodeData::from_node_ref(new),
                    change_type: ASTChangeType::Structural,
                })
            } else {
                None
            }
        }
        ASTChange::KindChange { old, new } => {
            if nodes_are_same(node, *new) {
                Some(NodeChangeStatus::Modified {
                    old_node: OwnedNodeData::from_node_ref(old),
                    new_node: OwnedNodeData::from_node_ref(new),
                    change_type: ASTChangeType::Rename,
                })
            } else {
                None
            }
        }
        ASTChange::Reorder { parent, .. } => {
            if nodes_are_same(node, *parent) {
                // For reordering, we use the parent as both old and new
                Some(NodeChangeStatus::Reordered {
                    old_node: OwnedNodeData::from_node_ref(parent),
                    new_node: OwnedNodeData::from_node_ref(parent),
                })
            } else {
                None
            }
        }
    }
}

/// Check if two NodeRef instances refer to the same node (by byte position)
fn nodes_are_same<'source>(node1: NodeRef<'source>, node2: NodeRef<'source>) -> bool {
    node1.start_byte() == node2.start_byte() && node1.end_byte() == node2.end_byte()
}

/// Check if a NodeChangeStatus represents an actual change
fn has_changes(change_status: &NodeChangeStatus) -> bool {
    !matches!(change_status, NodeChangeStatus::Unchanged { .. })
}

/// Count nodes with ESSENTIAL relevance in a context tree
fn count_essential_nodes<'source>(context_node: &ContextNode<'source>) -> usize {
    let mut count = 0;
    if context_node.relevance == crate::ast_diff::ESSENTIAL {
        count += 1;
    }
    for child in &context_node.children {
        count += count_essential_nodes(child);
    }
    count
}

/// Get the ASTChangeType from an ASTChange
fn get_ast_change_type<'source>(change: &ASTChange<'source>) -> ASTChangeType {
    match change {
        ASTChange::Addition(_) | ASTChange::Deletion(_) => ASTChangeType::Structural,
        ASTChange::ContentChange { .. } => ASTChangeType::Content,
        ASTChange::StructuralChange { .. } => ASTChangeType::Structural,
        ASTChange::KindChange { .. } => ASTChangeType::Rename,
        ASTChange::Reorder { .. } => ASTChangeType::Reorder,
    }
}

/// Build a rich context tree from an AST change
fn build_context_tree_from_change<'source>(
    change: &ASTChange<'source>,
    parser: &dyn crate::common::LanguageParser,
) -> ChangeWithContext<'source> {
    let primary_node = change.primary_node();

    // Step 1: Find context boundary by walking up the tree
    let context_boundary = find_context_boundary(primary_node, change.change_type(), parser);

    // Step 2: Build rich context tree recursively from boundary
    let context_tree = build_context_tree_recursive(&context_boundary, primary_node, parser, 0);

    ChangeWithContext {
        original_changes: vec![change.clone()],
        context_boundary,
        context_tree,
    }
}

/// Find the appropriate context boundary by walking up the parent chain
fn find_context_boundary<'source>(
    change_node: &NodeRef<'source>,
    change_type: ASTChangeType,
    parser: &dyn crate::common::LanguageParser,
) -> NodeRef<'source> {
    // Get semantic kind of the change node
    let change_semantic_kind = parser.classify_node_kind(change_node.kind());

    // Get priority-ordered boundary kinds from parser
    let boundary_kinds = parser.get_context_boundaries(&change_type, &change_semantic_kind);

    // Walk up parent chain to find first matching boundary
    let mut current = change_node.node;

    while let Some(parent) = current.parent() {
        let parent_semantic_kind = parser.classify_node_kind(parent.kind());

        // Check if this parent matches any of our boundary kinds
        if boundary_kinds.contains(&parent_semantic_kind) {
            return NodeRef::new(parent);
        }

        current = parent;
    }

    // No matching boundary found - use the primary node itself
    *change_node
}

/// Recursively build context tree with relevance scores
fn build_context_tree_recursive<'source>(
    node: &NodeRef<'source>,
    change_node: &NodeRef<'source>,
    parser: &dyn crate::common::LanguageParser,
    depth: usize,
) -> ContextNode<'source> {
    use crate::ast_diff::ESSENTIAL;

    const MAX_DEPTH: usize = 10;

    // Prevent infinite recursion
    if depth > MAX_DEPTH {
        let semantic_kind = parser.classify_node_kind(node.kind());
        let relevance = parser.classify_leaf_relevance(&semantic_kind);
        return ContextNode::new(*node, relevance);
    }

    // Determine relevance for this node
    let relevance = if is_on_change_path(node, change_node) {
        ESSENTIAL
    } else {
        let semantic_kind = parser.classify_node_kind(node.kind());
        parser.classify_leaf_relevance(&semantic_kind)
    };

    // Create node with initial relevance
    let mut context_node = ContextNode::new(*node, relevance);

    // Recursively build children using TreeSitter cursor pattern
    let mut cursor = node.node.walk();
    for child in node.node.children(&mut cursor) {
        let child_ref = NodeRef::new(child);
        let child_context =
            build_context_tree_recursive(&child_ref, change_node, parser, depth + 1);
        context_node.add_child(child_context);
    }

    context_node
}

/// Check if a node is on the path from root to change node
fn is_on_change_path<'source>(node: &NodeRef<'source>, change_node: &NodeRef<'source>) -> bool {
    // Walk up from change_node to see if we encounter node
    let mut current = change_node.node;

    // Check if we're at the change node itself
    if nodes_equal(node, change_node) {
        return true;
    }

    // Walk up parent chain
    while let Some(parent) = current.parent() {
        let parent_ref = NodeRef::new(parent);
        if nodes_equal(&parent_ref, node) {
            return true;
        }
        current = parent;
    }

    false
}

/// Compare two nodes for equality (by position and kind)
fn nodes_equal<'source>(a: &NodeRef<'source>, b: &NodeRef<'source>) -> bool {
    a.start_byte() == b.start_byte() && a.end_byte() == b.end_byte() && a.kind() == b.kind()
}

/// Convert AST changes to reviewable diffs with context expansion
pub fn expand_changes_to_reviewable_diffs<'source>(
    changes: &[ASTChange<'source>],
    parser: &dyn crate::common::LanguageParser,
    old_source: &'source dyn SourceProvider,
    new_source: &'source dyn SourceProvider,
    language: ProgrammingLanguage,
) -> Vec<ReviewableDiff> {
    let start_time = Instant::now();

    // Create a ReviewableDiff for each change with rich context expansion
    let reviewable_diffs: Vec<_> = changes
        .iter()
        .map(|change| {
            // Build rich context tree using context expansion algorithm
            let change_with_context = build_context_tree_from_change(change, parser);
            ReviewableDiff::from_change_with_context(
                change_with_context,
                language,
                old_source,
                new_source,
                parser,
                start_time,
            )
        })
        .collect();

    reviewable_diffs
}

// Debug implementations for better developer experience

impl std::fmt::Debug for ReviewableDiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReviewableDiff")
            .field("language", &self.language)
            .field("boundary", &self.boundary)
            .field("metadata", &self.metadata)
            .finish_non_exhaustive()
    }
}

impl std::fmt::Debug for DiffNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let relevance_str = match self.relevance {
            0 => "ESSENTIAL",
            1 => "IMPORTANT",
            2 => "BACKGROUND",
            3 => "NOISE",
            _ => "UNKNOWN",
        };

        f.debug_struct("DiffNode")
            .field("node_type", &self.node_type)
            .field("semantic_kind", &self.semantic_kind)
            .field("change_status", &self.change_status)
            .field(
                "relevance",
                &format_args!("{} ({})", relevance_str, self.relevance),
            )
            .field("children_count", &self.children.len())
            .field("children", &self.children)
            .finish()
    }
}

impl std::fmt::Debug for DiffMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DiffMetadata")
            .field("total_changes", &self.total_changes)
            .field("change_summary", &self.change_summary)
            .field("essential_node_count", &self.essential_node_count)
            .field(
                "analysis_duration_ms",
                &format_args!("{}ms", self.analysis_duration_ms),
            )
            .finish()
    }
}
