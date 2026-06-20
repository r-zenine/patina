//! Decision-based diff creation from code impact ranges
//!
//! This module provides a new pipeline for creating ReviewableDiffs directly from
//! decision-specified code ranges, replacing the semantic pairing approach.
//!
//! ## Strategy
//! Given a decision's CodeImpact (file path + line range), this module:
//! 1. Parses both old and new file versions with tree-sitter
//! 2. Builds semantic trees for both
//! 3. Finds the semantic unit covering the target range in the new file
//! 4. Expands the range to cover complete semantic unit boundaries
//! 5. Looks up the same-named unit in the old file's semantic tree
//! 6. Classifies the change as Addition/Deletion/Modification
//! 7. Produces a ReviewableDiff with proper DiffNode tree and context

use crate::ast_diff::{
    ASTChangeType, BACKGROUND, ESSENTIAL, IMPORTANT, OwnedNodeData, SourceProvider,
};
use crate::common::{LanguageParser, ProgrammingLanguage};
use crate::reviewable_diff::{DiffMetadata, DiffNode, NodeChangeStatus, ReviewableDiff};
use crate::semantic_ast::{SemanticNode, SemanticTree, SemanticUnitType};
use std::collections::HashMap;
use std::time::Instant;
use thiserror::Error;
use tree_sitter::Node;

/// Errors that can occur during decision-based diff creation
#[derive(Debug, Error)]
pub enum DecisionDiffError {
    #[error("Failed to parse source code: {0}")]
    ParseError(String),

    #[error("Failed to build semantic tree: {0}")]
    SemanticError(String),

    #[error("Target range {start_line}-{end_line} is invalid")]
    InvalidRange { start_line: usize, end_line: usize },

    #[error(
        "Line range {start_line}-{end_line} exceeds source bounds (file has {actual_lines} lines)"
    )]
    LineRangeOutOfBounds {
        start_line: usize,
        end_line: usize,
        actual_lines: usize,
    },

    #[error("No semantic unit found at target range {start_line}-{end_line}")]
    NoUnitAtRange { start_line: usize, end_line: usize },

    #[error("No semantic units contained within target range {start_line}-{end_line}")]
    NoUnitsInRange { start_line: usize, end_line: usize },
}

/// Classification of what changed for a semantic unit
#[derive(Debug, Clone, PartialEq)]
pub enum ChangeClassification {
    /// Unit exists only in new file
    Addition,
    /// Unit exists only in old file
    Deletion,
    /// Unit exists in both files (may have content changes)
    Modification,
}

/// Collect all non-Module semantic units whose byte range is fully contained within
/// [start_byte, end_byte]. Stops recursing into a node once the node itself is collected
/// (children are implicitly included in the collected unit).
fn find_contained_units_recursive<'a>(
    node: &'a SemanticNode<'a>,
    start_byte: usize,
    end_byte: usize,
    result: &mut Vec<&'a SemanticNode<'a>>,
) {
    let node_range = node.tree_sitter_node.byte_range();

    if !matches!(node.unit_type, SemanticUnitType::Module { .. })
        && node_range.start >= start_byte
        && node_range.end <= end_byte
    {
        result.push(node);
        return;
    }

    for child in &node.children {
        find_contained_units_recursive(child, start_byte, end_byte, result);
    }
}

/// Collect all non-Module semantic units that overlap with (touch) the range [start_byte, end_byte].
/// This is used as a fallback when no units are strictly contained within the range.
/// Useful for single-line ranges like "line 6 to 6" which may contain incomplete units.
fn find_units_touching_range_recursive<'a>(
    node: &'a SemanticNode<'a>,
    start_byte: usize,
    end_byte: usize,
    result: &mut Vec<&'a SemanticNode<'a>>,
) {
    let node_range = node.tree_sitter_node.byte_range();

    // Check if this node overlaps with the target range
    let overlaps = node_range.start < end_byte && node_range.end > start_byte;

    if !matches!(node.unit_type, SemanticUnitType::Module { .. }) && overlaps {
        result.push(node);
        return;
    }

    // Only recurse if we haven't found a matching non-Module unit
    if matches!(node.unit_type, SemanticUnitType::Module { .. }) || !overlaps {
        for child in &node.children {
            find_units_touching_range_recursive(child, start_byte, end_byte, result);
        }
    }
}

/// Helper: Find smallest unit containing a byte range (recursive)
fn find_unit_recursive<'a>(
    node: &'a SemanticNode<'a>,
    start_byte: usize,
    end_byte: usize,
) -> Option<(&'a SemanticNode<'a>, usize, usize)> {
    let node_range = node.tree_sitter_node.byte_range();

    // Check if this node contains the target range
    if node_range.start > start_byte || node_range.end < end_byte {
        return None;
    }

    // Try to find a smaller unit in the children
    for child in &node.children {
        if let Some(result) = find_unit_recursive(child, start_byte, end_byte) {
            return Some(result);
        }
    }

    // This node is the smallest that contains the range
    // Expand to include complete unit boundaries
    let expanded_start = node_range.start;
    let expanded_end = node_range.end;

    Some((node, expanded_start, expanded_end))
}

/// Helper: Count total lines in source
fn count_lines(source: &str) -> usize {
    if source.is_empty() {
        return 0;
    }
    // Count newlines and add 1 for the last line (which may not end with \n)
    source.bytes().filter(|&b| b == b'\n').count() + 1
}

/// Helper: Clamp line range to source bounds
/// Returns the adjusted (start_line, end_line) clamped to actual file bounds
fn clamp_line_range(source: &str, start_line: usize, end_line: usize) -> (usize, usize) {
    let actual_lines = count_lines(source);
    let clamped_end = std::cmp::min(end_line, actual_lines);
    (start_line, clamped_end)
}

/// Helper: Convert line number to byte offset in source
fn line_to_byte_offset(_root: Node, source: &[u8], line: usize) -> Option<usize> {
    // Line numbers are 1-based in most editors, convert to 0-based
    let target_line = line.saturating_sub(1);

    let mut current_line = 0;
    let mut byte_offset = 0;

    for (byte_idx, ch) in source.iter().enumerate() {
        if current_line == target_line {
            return Some(byte_idx);
        }
        if *ch == b'\n' {
            current_line += 1;
        }
        byte_offset = byte_idx + 1;
    }

    // Handle last line
    if current_line == target_line {
        Some(byte_offset)
    } else {
        None
    }
}

/// 1.2: Find semantic unit by name and type in a tree
///
/// Performs O(n) linear scan of all units, matching by name text and unit type.
/// Returns the first matching unit, or None if not found.
fn find_semantic_unit_by_name<'a>(
    tree: &'a SemanticTree<'a>,
    source: &str,
    target_name: &str,
    target_type: &SemanticUnitType,
) -> Option<&'a SemanticNode<'a>> {
    for unit in tree.all_units() {
        // Check if unit type matches (same variant)
        let types_match =
            std::mem::discriminant(&unit.unit_type) == std::mem::discriminant(target_type);

        if !types_match {
            continue;
        }

        // Check if name matches (handle nameless units like source_file)
        let unit_name = get_unit_name(unit, source.as_bytes());
        let names_match = match (unit_name.as_deref(), target_name) {
            // Both nameless (e.g., source_file units)
            (None, "") => true,
            // Both have matching names
            (Some(name), target) if name == target => true,
            // Name mismatch
            _ => false,
        };

        if names_match {
            return Some(unit);
        }
    }

    None
}

/// Helper: Extract the name from a semantic unit
fn get_unit_name(unit: &SemanticNode, source: &[u8]) -> Option<String> {
    if let Some(name_node) = unit.name_node {
        name_node.utf8_text(source).ok().map(|s| s.to_string())
    } else {
        None
    }
}

/// Build context for ReviewableDiff construction
struct DiffBuildContext<'a> {
    new_unit: Option<&'a SemanticNode<'a>>,
    old_node_data: Option<OwnedNodeData>,
    classification: ChangeClassification,
    parser: &'a dyn LanguageParser,
    start_time: Instant,
}

/// 1.4: Build ReviewableDiff from a semantic unit using owned data
fn build_reviewable_diff_from_unit_with_data(
    context: DiffBuildContext,
    language: ProgrammingLanguage,
    old_source: Box<dyn SourceProvider>,
    new_source: Box<dyn SourceProvider>,
) -> ReviewableDiff {
    let (boundary_node, metadata) = match context.classification {
        ChangeClassification::Addition => {
            let unit = context.new_unit.expect("Addition must have new_unit");
            create_addition_diff(unit, context.parser, context.start_time)
        }
        ChangeClassification::Deletion => {
            // For deletion, we would need the old_unit, but we're not supporting this path
            // in the current implementation. This would require holding a reference to the old tree.
            // For now, create from new_unit marked as deleted
            let unit = context.new_unit.expect("Deletion must have new_unit");
            create_deletion_diff(unit, context.parser, context.start_time)
        }
        ChangeClassification::Modification => {
            let new = context.new_unit.expect("Modification must have new_unit");
            let old = context
                .old_node_data
                .expect("Modification must have old_node_data");
            create_modification_diff_with_data(old, new, context.parser, context.start_time)
        }
    };

    ReviewableDiff {
        language,
        boundary: boundary_node,
        old_source,
        new_source,
        metadata,
    }
}

/// Helper: Create diff for added unit
fn create_addition_diff(
    unit: &SemanticNode,
    parser: &dyn LanguageParser,
    start_time: Instant,
) -> (DiffNode, DiffMetadata) {
    let semantic_kind = unit_type_to_semantic_kind(&unit.unit_type);

    let boundary = DiffNode {
        node_type: get_unit_type_name(&unit.unit_type).to_string(),
        semantic_kind,
        change_status: NodeChangeStatus::Added {
            node: OwnedNodeData::with_identifier(
                &unit.tree_sitter_node,
                unit.identifier.clone(),
            ),
        },
        relevance: calculate_relevance(&unit.unit_type),
        children: build_child_nodes_with_context(&unit.tree_sitter_node, parser),
    };

    let mut change_summary = HashMap::new();
    change_summary.insert(ASTChangeType::Structural, 1);

    let metadata = DiffMetadata {
        total_changes: 1,
        change_summary,
        essential_node_count: count_essential_nodes(&boundary),
        analysis_duration_ms: start_time.elapsed().as_millis() as u64,
    };

    (boundary, metadata)
}

/// Helper: Create diff for deleted unit
fn create_deletion_diff(
    unit: &SemanticNode,
    parser: &dyn LanguageParser,
    start_time: Instant,
) -> (DiffNode, DiffMetadata) {
    let semantic_kind = unit_type_to_semantic_kind(&unit.unit_type);

    let boundary = DiffNode {
        node_type: get_unit_type_name(&unit.unit_type).to_string(),
        semantic_kind,
        change_status: NodeChangeStatus::Deleted {
            node: OwnedNodeData::with_identifier(
                &unit.tree_sitter_node,
                unit.identifier.clone(),
            ),
        },
        relevance: calculate_relevance(&unit.unit_type),
        children: build_child_nodes_with_context(&unit.tree_sitter_node, parser),
    };

    let mut change_summary = HashMap::new();
    change_summary.insert(ASTChangeType::Structural, 1);

    let metadata = DiffMetadata {
        total_changes: 1,
        change_summary,
        essential_node_count: count_essential_nodes(&boundary),
        analysis_duration_ms: start_time.elapsed().as_millis() as u64,
    };

    (boundary, metadata)
}

/// Helper: Create diff for modified unit using owned node data
fn create_modification_diff_with_data(
    old_node: OwnedNodeData,
    new_unit: &SemanticNode,
    parser: &dyn LanguageParser,
    start_time: Instant,
) -> (DiffNode, DiffMetadata) {
    let semantic_kind = unit_type_to_semantic_kind(&new_unit.unit_type);
    let change_type = ASTChangeType::Content; // Default to content change

    let boundary = DiffNode {
        node_type: get_unit_type_name(&new_unit.unit_type).to_string(),
        semantic_kind,
        change_status: NodeChangeStatus::Modified {
            old_node,
            new_node: OwnedNodeData::with_identifier(
                &new_unit.tree_sitter_node,
                new_unit.identifier.clone(),
            ),
            change_type,
        },
        relevance: calculate_relevance(&new_unit.unit_type),
        children: build_child_nodes_with_context(&new_unit.tree_sitter_node, parser),
    };

    let mut change_summary = HashMap::new();
    change_summary.insert(ASTChangeType::Content, 1);

    let metadata = DiffMetadata {
        total_changes: 1,
        change_summary,
        essential_node_count: count_essential_nodes(&boundary),
        analysis_duration_ms: start_time.elapsed().as_millis() as u64,
    };

    (boundary, metadata)
}

/// Build child DiffNodes with context expansion and relevance classification
fn build_child_nodes_with_context(node: &Node, parser: &dyn LanguageParser) -> Vec<DiffNode> {
    build_child_nodes_recursive(node, parser, 0)
}

/// Recursively build child DiffNodes with relevance classification
fn build_child_nodes_recursive(
    node: &Node,
    parser: &dyn LanguageParser,
    depth: usize,
) -> Vec<DiffNode> {
    const MAX_DEPTH: usize = 10;

    if depth > MAX_DEPTH {
        return Vec::new();
    }

    let mut children = Vec::new();
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        let semantic_kind = parser.classify_node_kind(child.kind());
        let relevance = parser.classify_leaf_relevance(&semantic_kind);

        let diff_node = DiffNode {
            node_type: child.kind().to_string(),
            semantic_kind,
            change_status: NodeChangeStatus::Unchanged {
                node: OwnedNodeData::from_tree_sitter_node(&child),
            },
            relevance,
            children: build_child_nodes_recursive(&child, parser, depth + 1),
        };

        children.push(diff_node);
    }

    children
}

/// Convert semantic unit type to semantic node kind
fn unit_type_to_semantic_kind(unit_type: &SemanticUnitType) -> crate::common::SemanticNodeKind {
    use crate::common::SemanticNodeKind;

    match unit_type {
        SemanticUnitType::DataStructure { .. } => SemanticNodeKind::Struct,
        SemanticUnitType::Callable { .. } => SemanticNodeKind::Function,
        SemanticUnitType::Variable { .. } => SemanticNodeKind::Variable,
        SemanticUnitType::Import { .. } => SemanticNodeKind::Import,
        SemanticUnitType::Module { .. } => SemanticNodeKind::Module,
        SemanticUnitType::Unknown { node_kind, .. } => match node_kind.as_str() {
            kind if kind.contains("function") => SemanticNodeKind::Function,
            kind if kind.contains("struct") => SemanticNodeKind::Struct,
            kind if kind.contains("class") => SemanticNodeKind::Class,
            kind if kind.contains("enum") => SemanticNodeKind::Enum,
            kind if kind.contains("import") || kind.contains("use") => SemanticNodeKind::Import,
            kind if kind.contains("module") => SemanticNodeKind::Module,
            _ => SemanticNodeKind::Other(node_kind.clone()),
        },
    }
}

/// Get a readable name for a semantic unit type
fn get_unit_type_name(unit_type: &SemanticUnitType) -> &'static str {
    match unit_type {
        SemanticUnitType::DataStructure { .. } => "DataStructure",
        SemanticUnitType::Callable { .. } => "Callable",
        SemanticUnitType::Variable { .. } => "Variable",
        SemanticUnitType::Import { .. } => "Import",
        SemanticUnitType::Module { .. } => "Module",
        SemanticUnitType::Unknown { .. } => "Unknown",
    }
}

/// Calculate relevance score for a semantic unit type
fn calculate_relevance(unit_type: &SemanticUnitType) -> crate::ast_diff::RelevanceScore {
    match unit_type {
        SemanticUnitType::DataStructure { .. } | SemanticUnitType::Callable { .. } => ESSENTIAL,
        SemanticUnitType::Variable { .. } | SemanticUnitType::Import { .. } => IMPORTANT,
        SemanticUnitType::Module { .. } => ESSENTIAL,
        SemanticUnitType::Unknown { node_kind, .. } => {
            if node_kind.contains("error") {
                crate::ast_diff::NOISE
            } else {
                BACKGROUND
            }
        }
    }
}

/// Count essential nodes in a diff tree
fn count_essential_nodes(node: &DiffNode) -> usize {
    let mut count = if node.relevance == ESSENTIAL { 1 } else { 0 };
    for child in &node.children {
        count += count_essential_nodes(child);
    }
    count
}

/// Public API - Create ReviewableDiff(s) from a decision's code range.
///
/// Two strategies depending on what tree-sitter finds at the target range:
/// - **Expand**: the range falls inside a single semantic unit → return that one unit.
/// - **Decompose**: expansion would reach the Module root (e.g. range spans an impl
///   header gap) → collect every non-Module unit *contained within* the range and
///   return one ReviewableDiff per unit.
pub fn create_reviewable_diff_from_range(
    _file_path: &str,
    start_line: usize,
    end_line: usize,
    old_source: Option<&dyn SourceProvider>,
    new_source: &dyn SourceProvider,
    language: ProgrammingLanguage,
    parser: &dyn LanguageParser,
) -> Result<Vec<ReviewableDiff>, DecisionDiffError> {
    let start_time = Instant::now();

    if start_line == 0 || end_line == 0 || start_line > end_line {
        return Err(DecisionDiffError::InvalidRange {
            start_line,
            end_line,
        });
    }

    let new_source_str = new_source.full_source();
    let (start_line, end_line) = clamp_line_range(new_source_str, start_line, end_line);

    let new_ast = parser
        .try_parse(new_source_str)
        .map_err(|e| DecisionDiffError::ParseError(format!("Failed to parse new file: {e}")))?;

    let new_tree = parser
        .build_semantic_tree(&new_ast, new_source_str)
        .map_err(|e| {
            DecisionDiffError::SemanticError(format!("Failed to build new semantic tree: {e}"))
        })?;

    let start_byte = line_to_byte_offset(
        new_tree.root.tree_sitter_node,
        new_source_str.as_bytes(),
        start_line,
    )
    .ok_or(DecisionDiffError::NoUnitAtRange {
        start_line,
        end_line,
    })?;
    let end_byte = line_to_byte_offset(
        new_tree.root.tree_sitter_node,
        new_source_str.as_bytes(),
        end_line,
    )
    .ok_or(DecisionDiffError::NoUnitAtRange {
        start_line,
        end_line,
    })?;

    let (new_unit, _, _) = find_unit_recursive(&new_tree.root, start_byte, end_byte).ok_or(
        DecisionDiffError::NoUnitAtRange {
            start_line,
            end_line,
        },
    )?;

    // Decompose path: expansion hit the Module root
    if matches!(new_unit.unit_type, SemanticUnitType::Module { .. }) {
        let mut contained: Vec<&SemanticNode> = Vec::new();
        find_contained_units_recursive(&new_tree.root, start_byte, end_byte, &mut contained);

        // If no units strictly contained within range, try to find units that touch the range
        if contained.is_empty() {
            find_units_touching_range_recursive(
                &new_tree.root,
                start_byte,
                end_byte,
                &mut contained,
            );
        }

        if contained.is_empty() {
            return Err(DecisionDiffError::NoUnitsInRange {
                start_line,
                end_line,
            });
        }

        // Look up old counterparts for all contained units in one pass while old_tree is alive
        let old_nodes: Vec<Option<OwnedNodeData>> = if let Some(old_source_provider) = old_source {
            let old_source_str = old_source_provider.full_source();
            let old_ast = parser.try_parse(old_source_str).map_err(|e| {
                DecisionDiffError::ParseError(format!("Failed to parse old file: {e}"))
            })?;
            let old_tree = parser
                .build_semantic_tree(&old_ast, old_source_str)
                .map_err(|e| {
                    DecisionDiffError::SemanticError(format!(
                        "Failed to build old semantic tree: {e}"
                    ))
                })?;
            contained
                .iter()
                .map(|unit| {
                    find_semantic_unit_by_name(
                        &old_tree,
                        old_source_str,
                        &get_unit_name(unit, new_source_str.as_bytes()).unwrap_or_default(),
                        &unit.unit_type,
                    )
                    .map(|old_unit| {
                        OwnedNodeData::with_identifier(
                            &old_unit.tree_sitter_node,
                            old_unit.identifier.clone(),
                        )
                    })
                })
                .collect()
        } else {
            vec![None; contained.len()]
        };

        let diffs = contained
            .into_iter()
            .zip(old_nodes)
            .map(|(unit, old_node_data)| {
                let classification = if old_node_data.is_some() {
                    ChangeClassification::Modification
                } else {
                    ChangeClassification::Addition
                };
                let context = DiffBuildContext {
                    new_unit: Some(unit),
                    old_node_data,
                    classification,
                    parser,
                    start_time,
                };
                build_reviewable_diff_from_unit_with_data(
                    context,
                    language,
                    old_source
                        .map(|p| p.clone_box())
                        .unwrap_or_else(|| new_source.clone_box()),
                    new_source.clone_box(),
                )
            })
            .collect();

        return Ok(diffs);
    }

    // Expand path: single unit found
    let old_node_data = if let Some(old_source_provider) = old_source {
        let old_source_str = old_source_provider.full_source();
        let old_ast = parser
            .try_parse(old_source_str)
            .map_err(|e| DecisionDiffError::ParseError(format!("Failed to parse old file: {e}")))?;
        let old_tree = parser
            .build_semantic_tree(&old_ast, old_source_str)
            .map_err(|e| {
                DecisionDiffError::SemanticError(format!("Failed to build old semantic tree: {e}"))
            })?;
        find_semantic_unit_by_name(
            &old_tree,
            old_source_str,
            &get_unit_name(new_unit, new_source_str.as_bytes()).unwrap_or_default(),
            &new_unit.unit_type,
        )
        .map(|unit| OwnedNodeData::with_identifier(&unit.tree_sitter_node, unit.identifier.clone()))
    } else {
        None
    };

    let classification = if old_node_data.is_some() {
        ChangeClassification::Modification
    } else {
        ChangeClassification::Addition
    };

    let context = DiffBuildContext {
        new_unit: Some(new_unit),
        old_node_data,
        classification,
        parser,
        start_time,
    };

    Ok(vec![build_reviewable_diff_from_unit_with_data(
        context,
        language,
        old_source
            .map(|p| p.clone_box())
            .unwrap_or_else(|| new_source.clone_box()),
        new_source.clone_box(),
    )])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_classification_enum() {
        // Test that ChangeClassification enum is properly defined
        assert_eq!(format!("{:?}", ChangeClassification::Addition), "Addition");
        assert_eq!(format!("{:?}", ChangeClassification::Deletion), "Deletion");
        assert_eq!(
            format!("{:?}", ChangeClassification::Modification),
            "Modification"
        );
    }
}
