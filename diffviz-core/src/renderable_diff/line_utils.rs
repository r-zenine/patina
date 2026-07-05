//! Utilities for line splitting, annotation mapping, and diff creation

use super::semantic_anchors::extract_semantic_anchor;
use super::{ChangeType, LineAnnotation, RenderableLine};
use crate::{
    ast_diff::{NodeLike, RelevanceScore},
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
    let boundary_start_byte = reviewable
        .boundary
        .change_status
        .display_node()
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
    let (display_node, source_provider) =
        reviewable.boundary.change_status.display_node_with_source(
            reviewable.old_source.as_ref(),
            reviewable.new_source.as_ref(),
        );

    source_provider.node_text(display_node)
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
    let node_ref = node.change_status.display_node();
    let change_type = extract_change_type(&node.change_status);

    annotations.push(NodeAnnotation {
        byte_range: (node_ref.start_byte(), node_ref.end_byte()),
        relevance: node.relevance,
        change_type,
        semantic_kind: node.semantic_kind.clone(),
        depth,
    });

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
pub(super) fn ranges_overlap(range1: (usize, usize), range2: (usize, usize)) -> bool {
    range1.0 < range2.1 && range2.0 < range1.1
}
