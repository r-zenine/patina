//! Convert semantic pairs to ReviewableDiffs
//!
//! This module bridges the semantic AST analysis with the ReviewableDiff
//! rendering pipeline, converting semantic pairs into reviewable structures.
//!
//! Phase 6: Integrated with Phase 1 context expansion to produce rich
//! DiffNode trees with varied relevance scores for folding support.

use crate::ast_diff::{
    ASTChangeType, BACKGROUND, ESSENTIAL, IMPORTANT, NOISE, OwnedNodeData, RelevanceScore,
    SourceProvider,
};
use crate::common::{LanguageParser, ProgrammingLanguage, SemanticNodeKind};
use crate::renderable_diff::RenderableDiff;
use crate::reviewable_diff::{DiffMetadata, DiffNode, NodeChangeStatus, ReviewableDiff};
use crate::semantic_ast::{SemanticNode, SemanticPair, SemanticSimilarity, SemanticUnitType};
use std::collections::HashMap;
use std::time::Instant;
use tree_sitter::Node;

/// Convert semantic pairs to ReviewableDiffs
///
/// Phase 6: Now accepts a parser for context expansion with varied relevance scores.
/// The parser is used to classify node kinds and assign relevance (ESSENTIAL, IMPORTANT,
/// BACKGROUND, NOISE) to enable folding in the TUI.
pub fn semantic_pairs_to_reviewable_diffs<'source>(
    pairs: &[SemanticPair<'source>],
    language: ProgrammingLanguage,
    old_source: &'source dyn SourceProvider,
    new_source: &'source dyn SourceProvider,
    parser: &dyn LanguageParser,
) -> Vec<ReviewableDiff> {
    let start_time = Instant::now();

    let reviewable_diffs: Vec<ReviewableDiff> = pairs
        .iter()
        .filter(|pair| should_create_diff_for_pair(pair))
        .map(|pair| {
            create_reviewable_diff_from_pair(
                pair, language, old_source, new_source, start_time, parser,
            )
        })
        .collect();

    reviewable_diffs
        .into_iter()
        .filter(has_visible_content)
        .collect()
}

/// Determine if a semantic pair should generate a ReviewableDiff
fn should_create_diff_for_pair(pair: &SemanticPair) -> bool {
    // Check if this is a root-level module that spans the entire file
    let is_full_file_module = match pair {
        SemanticPair::Matched { old_unit, .. } => is_root_file_module(old_unit),
        SemanticPair::Addition { unit } => is_root_file_module(unit),
        SemanticPair::Deletion { unit } => is_root_file_module(unit),
    };

    // Skip full-file module pairs - they're just containers
    if is_full_file_module {
        return false;
    }

    match pair {
        SemanticPair::Matched { similarity, .. } => {
            // Only create diffs for actual changes
            similarity.has_changes()
        }
        SemanticPair::Addition { .. } | SemanticPair::Deletion { .. } => true,
    }
}

/// Check if a semantic unit represents a root-level module that spans the entire file
fn is_root_file_module(unit: &SemanticNode) -> bool {
    match &unit.unit_type {
        SemanticUnitType::Module { module_type, .. } => {
            // Root file modules start at byte 0 and typically span most/all of the file
            let range = unit.tree_sitter_node.byte_range();
            matches!(module_type, crate::semantic_ast::ModuleType::File) && range.start == 0
        }
        _ => false,
    }
}

/// Create a ReviewableDiff from a single semantic pair
fn create_reviewable_diff_from_pair<'source>(
    pair: &SemanticPair<'source>,
    language: ProgrammingLanguage,
    old_source: &'source dyn SourceProvider,
    new_source: &'source dyn SourceProvider,
    start_time: Instant,
    parser: &dyn LanguageParser,
) -> ReviewableDiff {
    let (boundary_node, metadata) = match pair {
        SemanticPair::Matched {
            old_unit,
            new_unit,
            similarity,
        } => create_matched_diff(old_unit, new_unit, similarity, start_time, parser),
        SemanticPair::Addition { unit } => create_addition_diff(unit, start_time, parser),
        SemanticPair::Deletion { unit } => create_deletion_diff(unit, start_time, parser),
    };

    ReviewableDiff {
        language,
        boundary: boundary_node,
        old_source: old_source.clone_box(),
        new_source: new_source.clone_box(),
        metadata,
    }
}

/// Create a diff node for matched semantic units with changes
fn create_matched_diff<'source>(
    old_unit: &SemanticNode<'source>,
    new_unit: &SemanticNode<'source>,
    similarity: &SemanticSimilarity,
    start_time: Instant,
    parser: &dyn LanguageParser,
) -> (DiffNode, DiffMetadata) {
    let change_type = similarity_to_change_type(similarity);
    let semantic_kind = unit_type_to_semantic_kind(&old_unit.unit_type);

    // Create the boundary node with children from context expansion (Phase 6)
    let boundary = DiffNode {
        node_type: get_unit_type_name(&old_unit.unit_type).to_string(),
        semantic_kind,
        change_status: NodeChangeStatus::Modified {
            old_node: OwnedNodeData::from_tree_sitter_node(&old_unit.tree_sitter_node),
            new_node: OwnedNodeData::from_tree_sitter_node(&new_unit.tree_sitter_node),
            change_type,
        },
        relevance: calculate_relevance(&old_unit.unit_type),
        children: build_child_nodes_with_context(&new_unit.tree_sitter_node, parser),
    };

    let mut change_summary = HashMap::new();
    let mut total_changes = 0;

    // Count all applicable change types
    if similarity.signature_changed || similarity.structural_changed {
        change_summary.insert(ASTChangeType::Structural, 1);
        total_changes += 1;
    }
    if similarity.name_changed {
        change_summary.insert(ASTChangeType::Rename, 1);
        total_changes += 1;
    }
    if similarity.body_changed {
        change_summary.insert(ASTChangeType::Content, 1);
        total_changes += 1;
    }

    // Ensure we have at least one change
    if total_changes == 0 {
        change_summary.insert(change_type, 1);
        total_changes = 1;
    }

    let metadata = DiffMetadata {
        total_changes,
        change_summary,
        essential_node_count: count_essential_nodes(&boundary),
        analysis_duration_ms: start_time.elapsed().as_millis() as u64,
    };

    (boundary, metadata)
}

/// Create a diff node for added semantic units
fn create_addition_diff<'source>(
    unit: &SemanticNode<'source>,
    start_time: Instant,
    parser: &dyn LanguageParser,
) -> (DiffNode, DiffMetadata) {
    let semantic_kind = unit_type_to_semantic_kind(&unit.unit_type);

    let boundary = DiffNode {
        node_type: get_unit_type_name(&unit.unit_type).to_string(),
        semantic_kind,
        change_status: NodeChangeStatus::Added {
            node: OwnedNodeData::from_tree_sitter_node(&unit.tree_sitter_node),
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

/// Create a diff node for deleted semantic units
fn create_deletion_diff<'source>(
    unit: &SemanticNode<'source>,
    start_time: Instant,
    parser: &dyn LanguageParser,
) -> (DiffNode, DiffMetadata) {
    let semantic_kind = unit_type_to_semantic_kind(&unit.unit_type);

    let boundary = DiffNode {
        node_type: get_unit_type_name(&unit.unit_type).to_string(),
        semantic_kind,
        change_status: NodeChangeStatus::Deleted {
            node: OwnedNodeData::from_tree_sitter_node(&unit.tree_sitter_node),
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

/// Build child nodes with context expansion (Phase 6 integration with Phase 1)
///
/// Walks the tree-sitter AST children and assigns relevance scores using the
/// parser's classification methods. This enables folding in the TUI by marking
/// nodes as ESSENTIAL, IMPORTANT, BACKGROUND, or NOISE.
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
        // Classify the node kind using the parser
        let semantic_kind = parser.classify_node_kind(child.kind());

        // Assign relevance based on semantic classification (same as Phase 1)
        let relevance = parser.classify_leaf_relevance(&semantic_kind);

        // Build the DiffNode for this child
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
/// Convert semantic similarity to AST change type
fn similarity_to_change_type(similarity: &SemanticSimilarity) -> ASTChangeType {
    // Prioritize change types: Structural > Signature > Rename > Content
    if similarity.structural_changed || similarity.signature_changed {
        ASTChangeType::Structural
    } else if similarity.name_changed {
        ASTChangeType::Rename
    } else if similarity.body_changed {
        ASTChangeType::Content
    } else {
        // Identical case
        ASTChangeType::Content
    }
}

/// Convert semantic unit type to semantic node kind
fn unit_type_to_semantic_kind(unit_type: &SemanticUnitType) -> SemanticNodeKind {
    match unit_type {
        SemanticUnitType::DataStructure { .. } => SemanticNodeKind::Struct,
        SemanticUnitType::Callable { .. } => SemanticNodeKind::Function,
        SemanticUnitType::Variable { .. } => SemanticNodeKind::Variable,
        SemanticUnitType::Import { .. } => SemanticNodeKind::Import,
        SemanticUnitType::Module { .. } => SemanticNodeKind::Module,
        SemanticUnitType::Unknown { node_kind, .. } => {
            // Map common AST node kinds to appropriate semantic kinds
            match node_kind.as_str() {
                kind if kind.contains("function") => SemanticNodeKind::Function,
                kind if kind.contains("struct") => SemanticNodeKind::Struct,
                kind if kind.contains("class") => SemanticNodeKind::Class,
                kind if kind.contains("enum") => SemanticNodeKind::Enum,
                kind if kind.contains("import") || kind.contains("use") => SemanticNodeKind::Import,
                kind if kind.contains("module") => SemanticNodeKind::Module,
                _ => SemanticNodeKind::Other(node_kind.clone()),
            }
        }
    }
}

/// Calculate relevance score for a semantic unit type
fn calculate_relevance(unit_type: &SemanticUnitType) -> RelevanceScore {
    match unit_type {
        SemanticUnitType::DataStructure { .. } | SemanticUnitType::Callable { .. } => ESSENTIAL,
        SemanticUnitType::Variable { .. } | SemanticUnitType::Import { .. } => IMPORTANT,
        SemanticUnitType::Module { .. } => ESSENTIAL,
        SemanticUnitType::Unknown { node_kind, .. } => {
            // Give unknown nodes medium relevance by default
            // Could be made smarter based on node_kind
            if node_kind.contains("error") {
                NOISE // Error nodes are typically not important for review
            } else {
                BACKGROUND // Unknown nodes get background relevance
            }
        }
    }
}

/// Get a readable name for a semantic unit type
fn get_unit_type_name(unit_type: &SemanticUnitType) -> &'static str {
    match unit_type {
        SemanticUnitType::DataStructure { .. } => "DataStructure",
        SemanticUnitType::Callable { .. } => "Function",
        SemanticUnitType::Variable { .. } => "Variable",
        SemanticUnitType::Import { .. } => "Import",
        SemanticUnitType::Module { .. } => "Module",
        SemanticUnitType::Unknown { .. } => "Unknown",
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

/// Check if a ReviewableDiff has any visible content when rendered
/// Uses the same logic as the show command: checks for actual changes, not just foldable lines
fn has_visible_content(reviewable_diff: &ReviewableDiff) -> bool {
    let renderable_diff = RenderableDiff::from(reviewable_diff);

    // First check: are there any non-folded lines?
    let visible_lines = renderable_diff
        .lines
        .iter()
        .filter(|line| !line.should_fold())
        .count();

    if visible_lines == 0 {
        return false;
    }

    // Second check: are there any actual changes? (matching show command logic)
    let changed_lines = renderable_diff
        .lines
        .iter()
        .filter(|line| line.has_changes())
        .count();

    changed_lines > 0
}
