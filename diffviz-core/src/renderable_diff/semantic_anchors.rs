//! Utilities for extracting semantic anchors from lines
//!
//! This module provides functionality to identify semantic anchors in code lines,
//! such as function names, struct/enum names, and variable bindings. These anchors
//! are used by the Myers diff algorithm to treat semantically related lines as having
//! edit distance 0, resulting in better diff grouping.
//!
//! Anchors are derived entirely from the `DiffNode` tree built during parsing — no
//! string/regex scanning of source text. A node is an anchor candidate for a line if
//! its own declaration starts on that line (`start_byte` falls within the line's byte
//! range); the deepest such candidate wins.

use super::{SemanticAnchor, SemanticAnchorType};
use crate::{
    common::SemanticNodeKind,
    reviewable_diff::{DiffNode, NodeChangeStatus, ReviewableDiff},
};

/// Extract semantic anchor for the line spanning `[line_byte_start, line_byte_start + line.len())`.
pub fn extract_semantic_anchor(
    line: &str,
    reviewable: &ReviewableDiff,
    line_byte_start: usize,
) -> Option<SemanticAnchor> {
    let line_byte_end = line_byte_start + line.len();
    find_anchor_in_range(&reviewable.boundary, line_byte_start, line_byte_end)
}

/// Walk the DiffNode tree looking for the deepest node whose declaration starts within
/// `[line_start, line_end)`. Children are checked before the node itself, so a more
/// specific (nested) anchor always takes precedence over an enclosing one.
fn find_anchor_in_range(
    node: &DiffNode,
    line_start: usize,
    line_end: usize,
) -> Option<SemanticAnchor> {
    for child in &node.children {
        if let Some(anchor) = find_anchor_in_range(child, line_start, line_end) {
            return Some(anchor);
        }
    }

    let start_byte = node_start_byte(&node.change_status);
    if start_byte < line_start || start_byte >= line_end {
        return None;
    }

    let anchor_type = kind_to_anchor_type(&node.semantic_kind)?;
    let identifier = identifier_for(&node.change_status)?;
    Some(SemanticAnchor {
        anchor_type,
        identifier,
    })
}

fn node_start_byte(status: &NodeChangeStatus) -> usize {
    match status {
        NodeChangeStatus::Unchanged { node }
        | NodeChangeStatus::Added { node }
        | NodeChangeStatus::Deleted { node } => node.start_byte,
        NodeChangeStatus::Modified { new_node, .. } => new_node.start_byte,
    }
}

fn identifier_for(status: &NodeChangeStatus) -> Option<String> {
    match status {
        NodeChangeStatus::Unchanged { node }
        | NodeChangeStatus::Added { node }
        | NodeChangeStatus::Deleted { node } => node.identifier.clone(),
        NodeChangeStatus::Modified { new_node, .. } => new_node.identifier.clone(),
    }
}

fn kind_to_anchor_type(kind: &SemanticNodeKind) -> Option<SemanticAnchorType> {
    match kind {
        SemanticNodeKind::Function => Some(SemanticAnchorType::FunctionSignature),
        SemanticNodeKind::Struct => Some(SemanticAnchorType::StructDeclaration),
        SemanticNodeKind::Enum => Some(SemanticAnchorType::EnumDeclaration),
        SemanticNodeKind::Variable => Some(SemanticAnchorType::VariableAssignment),
        SemanticNodeKind::Import => Some(SemanticAnchorType::Import),
        _ => None,
    }
}
