//! Utilities for line splitting, annotation mapping, and diff creation

use super::semantic_anchors::extract_semantic_anchor;
use super::{ChangeType, LineAnnotation, RenderableLine};
use crate::{
    ast_diff::{NodeLike, RelevanceScore, SourceProvider},
    common::SemanticNodeKind,
    reviewable_diff::{DiffNode, NodeChangeStatus, ReviewableDiff},
};

/// Helper struct for tracking line information during conversion
#[derive(Debug)]
pub struct LineInfo<'source> {
    pub number: usize,
    pub text: &'source str,
    pub byte_range: (usize, usize),
}

/// Node annotation collected from tree traversal
#[derive(Debug, Clone)]
pub struct NodeAnnotation {
    pub byte_range: (usize, usize),
    pub relevance: RelevanceScore,
    pub change_type: Option<ChangeType>,
    pub semantic_kind: SemanticNodeKind,
    pub depth: usize,
}

/// Create lines using single source (for non-Modified boundaries)
pub fn create_single_source_lines<'source>(
    reviewable: &'source ReviewableDiff,
) -> Result<Vec<RenderableLine<'source>>, crate::ast_diff::SourceError> {
    let source = extract_boundary_source(reviewable)?;

    // Split into lines with byte positions
    let line_infos = split_into_lines_with_positions(source);

    // Collect all node annotations from tree and adjust byte ranges to boundary-relative
    let boundary_start_byte = get_display_node(&reviewable.boundary.change_status)
        .expect("ReviewableDiff should have a valid display node")
        .start_byte();
    let mut node_annotations = collect_all_annotations(&reviewable.boundary);

    // Adjust annotation byte ranges to be relative to boundary source
    for annotation in &mut node_annotations {
        annotation.byte_range.0 = annotation.byte_range.0.saturating_sub(boundary_start_byte);
        annotation.byte_range.1 = annotation.byte_range.1.saturating_sub(boundary_start_byte);
    }

    // Map annotations to lines
    Ok(line_infos
        .into_iter()
        .map(|line_info| {
            let annotations = map_annotations_to_line(&line_info, &node_annotations);

            RenderableLine {
                line_number: line_info.number,
                content: line_info.text,
                byte_range: line_info.byte_range,
                annotations,
                semantic_anchor: extract_semantic_anchor(
                    line_info.text,
                    reviewable,
                    boundary_start_byte + line_info.byte_range.0,
                ),
            }
        })
        .collect())
}

/// Extract boundary source text from ReviewableDiff
fn extract_boundary_source(
    reviewable: &ReviewableDiff,
) -> Result<&str, crate::ast_diff::SourceError> {
    let (display_node, source_provider) = get_display_node_with_source(
        &reviewable.boundary.change_status,
        reviewable.old_source.as_ref(),
        reviewable.new_source.as_ref(),
    )
    .expect("ReviewableDiff should have a valid display node");

    source_provider.node_text(display_node)
}

/// Get the display node with the correct source provider for text extraction
fn get_display_node_with_source<'a>(
    change_status: &'a NodeChangeStatus,
    old_source: &'a dyn SourceProvider,
    new_source: &'a dyn SourceProvider,
) -> Option<(&'a crate::ast_diff::OwnedNodeData, &'a dyn SourceProvider)> {
    match change_status {
        NodeChangeStatus::Unchanged { node, .. } => Some((node, new_source)), // Use new_source for consistency
        NodeChangeStatus::Added { node, .. } => Some((node, new_source)), // Added nodes are in new source
        NodeChangeStatus::Deleted { node, .. } => Some((node, old_source)), // Deleted nodes are in old source
        NodeChangeStatus::Modified { new_node, .. } => Some((new_node, new_source)), // Use new version
    }
}

/// Get the display node from a NodeChangeStatus (legacy function)
fn get_display_node(change_status: &NodeChangeStatus) -> Option<&crate::ast_diff::OwnedNodeData> {
    match change_status {
        NodeChangeStatus::Unchanged { node, .. } => Some(node),
        NodeChangeStatus::Added { node, .. } => Some(node),
        NodeChangeStatus::Deleted { node, .. } => Some(node),
        NodeChangeStatus::Modified { new_node, .. } => Some(new_node),
    }
}

/// Split source into lines preserving byte positions
fn split_into_lines_with_positions(source: &str) -> Vec<LineInfo<'_>> {
    let mut lines = Vec::new();
    let mut byte_offset = 0;

    for (line_num, line_text) in source.lines().enumerate() {
        lines.push(LineInfo {
            number: line_num + 1,
            text: line_text,
            byte_range: (byte_offset, byte_offset + line_text.len()),
        });
        byte_offset += line_text.len() + 1; // +1 for newline
    }

    lines
}

/// Collect all node annotations from the tree
fn collect_all_annotations(node: &DiffNode) -> Vec<NodeAnnotation> {
    let mut annotations = Vec::new();
    collect_recursive(node, &mut annotations, 0);

    annotations
}

/// Recursively collect annotations from the diff tree
fn collect_recursive(node: &DiffNode, annotations: &mut Vec<NodeAnnotation>, depth: usize) {
    // Add annotation for this node if it has a valid node reference
    if let Some(node_ref) = get_display_node(&node.change_status) {
        let change_type = extract_change_type(&node.change_status);

        annotations.push(NodeAnnotation {
            byte_range: (node_ref.start_byte(), node_ref.end_byte()),
            relevance: node.relevance,
            change_type,
            semantic_kind: node.semantic_kind.clone(),
            depth,
        });
    }

    // Recurse into children
    for child in &node.children {
        collect_recursive(child, annotations, depth + 1);
    }
}

/// Extract change type from NodeChangeStatus
fn extract_change_type(change_status: &NodeChangeStatus) -> Option<ChangeType> {
    match change_status {
        NodeChangeStatus::Added { .. } => Some(ChangeType::Added),
        NodeChangeStatus::Deleted { .. } => Some(ChangeType::Deleted),
        NodeChangeStatus::Modified { .. } => Some(ChangeType::Modified),
        NodeChangeStatus::Unchanged { .. } => None,
    }
}

/// Find all annotations that intersect with a line's byte range
fn map_annotations_to_line(
    line_info: &LineInfo,
    node_annotations: &[NodeAnnotation],
) -> Vec<LineAnnotation> {
    node_annotations
        .iter()
        .filter(|ann| ranges_overlap(ann.byte_range, line_info.byte_range))
        .map(|ann| {
            // Calculate column positions within the line
            let start_col = ann.byte_range.0.saturating_sub(line_info.byte_range.0);
            let end_col = (ann.byte_range.1.min(line_info.byte_range.1))
                .saturating_sub(line_info.byte_range.0);

            LineAnnotation {
                start_col,
                end_col: end_col.max(start_col), // Ensure end_col >= start_col
                relevance: ann.relevance,
                change_type: ann.change_type.clone(),
                semantic_kind: ann.semantic_kind.clone(),
                node_depth: ann.depth,
            }
        })
        .collect()
}

/// Check if two byte ranges overlap
fn ranges_overlap(range1: (usize, usize), range2: (usize, usize)) -> bool {
    range1.0 < range2.1 && range2.0 < range1.1
}

/// Extract expanded source text that includes metadata nodes (like attributes)
/// This creates a temporary owned string that combines all the relevant text
#[allow(dead_code)]
fn extract_expanded_source_text<'source>(
    source_provider: &'source dyn crate::ast_diff::SourceProvider,
    main_node: &tree_sitter::Node,
    metadata_nodes: &[crate::semantic_ast::MetadataNode<'source>],
) -> Result<String, crate::ast_diff::SourceError> {
    use crate::semantic_ast::MetadataPosition;

    // If there are no metadata nodes, just return the main node text as owned String
    if metadata_nodes.is_empty() {
        return Ok(source_provider.node_text(main_node)?.to_string());
    }

    let mut result = String::new();

    // Collect preceding metadata nodes (like attributes)
    let mut preceding_nodes: Vec<_> = metadata_nodes
        .iter()
        .filter(|m| matches!(m.position, MetadataPosition::PrecedingSibling(_)))
        .collect();

    // Sort by position (earlier attributes first)
    preceding_nodes.sort_by_key(|m| m.node.start_byte());

    // Add all preceding metadata nodes first
    for meta in preceding_nodes {
        let meta_text = source_provider.node_text(&meta.node)?;
        result.push_str(meta_text);
        result.push('\n'); // Add newline after each attribute
    }

    // Add the main node text
    let main_text = source_provider.node_text(main_node)?;
    result.push_str(main_text);

    // Collect following metadata nodes
    let mut following_nodes: Vec<_> = metadata_nodes
        .iter()
        .filter(|m| matches!(m.position, MetadataPosition::FollowingSibling(_)))
        .collect();

    // Sort by position (earlier following metadata first)
    following_nodes.sort_by_key(|m| m.node.start_byte());

    // Add all following metadata nodes
    for meta in following_nodes {
        result.push('\n'); // Add newline before following metadata
        let meta_text = source_provider.node_text(&meta.node)?;
        result.push_str(meta_text);
    }

    Ok(result)
}
