//! Line-based renderable diff structure optimized for display
//!
//! This module provides `RenderableDiff`, a line-oriented representation of semantic
//! diffs that bridges between tree-based semantic analysis and character/line-based
//! display systems.

use crate::ast_diff::SourceError;
use crate::common::ProgrammingLanguage;
use crate::{
    ast_diff::{BACKGROUND, ESSENTIAL, LineRange, NOISE, RelevanceScore},
    common::SemanticNodeKind,
    reviewable_diff::{NodeChangeStatus, ReviewableDiff},
};
use std::collections::HashMap;

/// Error type for RenderableDiff creation
#[derive(Debug, thiserror::Error)]
pub enum RenderableDiffError {
    #[error("Failed to extract source for rendering: {0}")]
    SourceError(#[from] SourceError),
}

mod line_diff;
mod line_utils;
mod name_extractors;
mod semantic_anchors;

use line_utils::create_single_source_lines;
use name_extractors::extract_boundary_name;

/// Line-based renderable diff - optimized for display
#[derive(Debug, Clone)]
pub struct RenderableDiff<'source> {
    pub lines: Vec<RenderableLine<'source>>,
    pub metadata: RenderableMetadata,
    pub language: ProgrammingLanguage,
}

/// A single line with all rendering metadata
#[derive(Debug, Clone)]
pub struct RenderableLine<'source> {
    pub line_number: usize,
    pub content: &'source str,
    pub byte_range: (usize, usize),
    pub annotations: Vec<LineAnnotation>,
    pub semantic_anchor: Option<SemanticAnchor>,
}

/// Annotation for a byte range within a line
#[derive(Debug, Clone)]
pub struct LineAnnotation {
    pub start_col: usize,
    pub end_col: usize,
    pub relevance: RelevanceScore,
    pub change_type: Option<ChangeType>,
    pub semantic_kind: SemanticNodeKind,
    pub node_depth: usize,
}

/// Renderable metadata (simplified from DiffMetadata)
#[derive(Debug, Clone)]
pub struct RenderableMetadata {
    pub total_changes: usize,
    pub change_summary: HashMap<crate::ast_diff::ASTChangeType, usize>,
    pub essential_line_count: usize,
    pub boundary_name: String,
    /// Overall line range covered by this renderable diff
    pub overall_line_range: LineRange,
    /// Specific line numbers that contain changes
    pub changed_line_numbers: Vec<usize>,
}

/// Change type for rendering
#[derive(Debug, Clone, PartialEq)]
pub enum ChangeType {
    Added,
    Deleted,
    Modified,
}

/// Semantic anchor identifying what a line represents
#[derive(Debug, Clone, PartialEq)]
pub struct SemanticAnchor {
    pub anchor_type: SemanticAnchorType,
    pub identifier: String,
}

/// Type of semantic anchor
#[derive(Debug, Clone, PartialEq)]
pub enum SemanticAnchorType {
    FunctionSignature,  // fn calculate_total(
    VariableAssignment, // let config =
    Import,             // use diffviz::
    StructDeclaration,  // struct MyStruct {
    EnumDeclaration,    // enum MyEnum {
}

/// Node annotation from DiffNode tree for byte range mapping
#[derive(Debug, Clone)]
struct ByteRangeAnnotation {
    byte_range: (usize, usize),
    relevance: RelevanceScore,
}

/// Build a map of byte ranges to relevance scores from DiffNode tree
/// NOTE: Does NOT include the root node itself, only its children and their descendants
/// TODO: upgrade such that the function signature is not folded
fn build_byte_range_annotations(
    node: &crate::reviewable_diff::DiffNode,
) -> Vec<ByteRangeAnnotation> {
    let mut annotations = Vec::new();

    fn collect_recursive(
        node: &crate::reviewable_diff::DiffNode,
        annotations: &mut Vec<ByteRangeAnnotation>,
    ) {
        // Add annotation for this node
        let node_ref = node.change_status.display_node();
        annotations.push(ByteRangeAnnotation {
            byte_range: (node_ref.start_byte, node_ref.end_byte),
            relevance: node.relevance,
        });

        // Recurse into children
        for child in &node.children {
            collect_recursive(child, annotations);
        }
    }

    // Only collect from children, not the root node itself
    // This prevents the boundary node's ESSENTIAL relevance from overriding all children
    for child in &node.children {
        collect_recursive(child, &mut annotations);
    }

    annotations
}

/// Helper function to create line-by-line diff for Modified changes using the
/// `similar`-backed line diff engine (see [`line_diff`]).
fn create_line_by_line_diff_for_modified<'source>(
    reviewable: &'source ReviewableDiff,
    old_node: &crate::ast_diff::OwnedNodeData,
    new_node: &crate::ast_diff::OwnedNodeData,
) -> Result<Vec<RenderableLine<'source>>, crate::ast_diff::SourceError> {
    use crate::renderable_diff::line_diff::{DiffOp, align_by_anchors, diff_lines};
    use crate::renderable_diff::line_utils::line_byte_spans;
    use crate::renderable_diff::semantic_anchors::extract_semantic_anchor;

    // Extract old and new source text using byte ranges
    let old_text = reviewable.old_source.node_text(old_node)?;
    let new_text = reviewable.new_source.node_text(new_node)?;

    let old_lines: Vec<&str> = old_text.lines().collect();
    let new_lines: Vec<&str> = new_text.lines().collect();

    // Content-only byte spans (terminator-width-accurate, fixes CRLF drift)
    // relative to each extracted text, one per line, indexed identically to
    // old_lines/new_lines.
    let old_spans = line_byte_spans(old_text);
    let new_spans = line_byte_spans(new_text);

    let old_boundary_start = old_node.start_byte;
    let boundary_start = new_node.start_byte;

    let old_anchors: Vec<Option<SemanticAnchor>> = old_lines
        .iter()
        .zip(&old_spans)
        .map(|(&line, span)| extract_semantic_anchor(line, reviewable, old_boundary_start + span.0))
        .collect();
    let new_anchors: Vec<Option<SemanticAnchor>> = new_lines
        .iter()
        .zip(&new_spans)
        .map(|(&line, span)| extract_semantic_anchor(line, reviewable, boundary_start + span.0))
        .collect();

    let ops = align_by_anchors(
        diff_lines(&old_lines, &new_lines),
        &old_anchors,
        &new_anchors,
    );

    // Build byte range annotations from DiffNode tree (all annotations with byte positions)
    let byte_annotations = build_byte_range_annotations(&reviewable.boundary);

    let mut result_lines = Vec::with_capacity(ops.len());
    let mut line_number = 1;

    for op in &ops {
        match *op {
            DiffOp::Keep { new_idx, .. } => {
                let content = new_lines[new_idx];
                let span = new_spans[new_idx];
                let line_byte_range = (boundary_start + span.0, boundary_start + span.1);

                // Determine relevance using precedence rule:
                // - If ANY overlapping annotation is ESSENTIAL, use ESSENTIAL
                // - Otherwise, use minimum (most important) relevance
                let relevance =
                    determine_line_relevance_with_precedence(line_byte_range, &byte_annotations);

                let annotation = LineAnnotation {
                    start_col: 0,
                    end_col: content.len(),
                    relevance,
                    change_type: None, // No change
                    semantic_kind: reviewable.boundary.semantic_kind.clone(),
                    node_depth: 0,
                };

                result_lines.push(RenderableLine {
                    line_number,
                    content,
                    byte_range: (0, content.len()),
                    annotations: vec![annotation],
                    semantic_anchor: new_anchors[new_idx].clone(),
                });
                line_number += 1;
            }
            DiffOp::Delete { old_idx } => {
                let content = old_lines[old_idx];

                let annotation = LineAnnotation {
                    start_col: 0,
                    end_col: content.len(),
                    relevance: ESSENTIAL,
                    change_type: Some(ChangeType::Deleted),
                    semantic_kind: reviewable.boundary.semantic_kind.clone(),
                    node_depth: 0,
                };

                result_lines.push(RenderableLine {
                    line_number,
                    content,
                    byte_range: (0, content.len()),
                    annotations: vec![annotation],
                    semantic_anchor: old_anchors[old_idx].clone(),
                });
                line_number += 1;
            }
            DiffOp::Add { new_idx } => {
                let content = new_lines[new_idx];

                let annotation = LineAnnotation {
                    start_col: 0,
                    end_col: content.len(),
                    relevance: ESSENTIAL,
                    change_type: Some(ChangeType::Added),
                    semantic_kind: reviewable.boundary.semantic_kind.clone(),
                    node_depth: 0,
                };

                result_lines.push(RenderableLine {
                    line_number,
                    content,
                    byte_range: (0, content.len()),
                    annotations: vec![annotation],
                    semantic_anchor: new_anchors[new_idx].clone(),
                });
                line_number += 1;
            }
            DiffOp::Modify { old_idx, new_idx } => {
                // Modify operation: display as two lines - old (deleted) then new (added)
                // This shows the semantic pairing while maintaining traditional diff view
                let old_content = old_lines[old_idx];
                let old_annotation = LineAnnotation {
                    start_col: 0,
                    end_col: old_content.len(),
                    relevance: ESSENTIAL,
                    change_type: Some(ChangeType::Deleted),
                    semantic_kind: reviewable.boundary.semantic_kind.clone(),
                    node_depth: 0,
                };

                result_lines.push(RenderableLine {
                    line_number,
                    content: old_content,
                    byte_range: (0, old_content.len()),
                    annotations: vec![old_annotation],
                    semantic_anchor: old_anchors[old_idx].clone(),
                });
                line_number += 1;

                let new_content = new_lines[new_idx];
                let new_annotation = LineAnnotation {
                    start_col: 0,
                    end_col: new_content.len(),
                    relevance: ESSENTIAL,
                    change_type: Some(ChangeType::Added),
                    semantic_kind: reviewable.boundary.semantic_kind.clone(),
                    node_depth: 0,
                };

                result_lines.push(RenderableLine {
                    line_number,
                    content: new_content,
                    byte_range: (0, new_content.len()),
                    annotations: vec![new_annotation],
                    semantic_anchor: new_anchors[new_idx].clone(),
                });
                line_number += 1;
            }
        }
    }

    Ok(result_lines)
}

/// Determine relevance for a line: the minimum (most important) relevance among
/// annotations overlapping the line. ESSENTIAL == 0 is the domain minimum, so
/// "ESSENTIAL wins over any other relevance" falls out of `min()` for free.
///
/// Lines overlapping no annotation default to ESSENTIAL (decision D011,
/// plan-core-hardening) — a deliberate choice that prevents such lines from ever
/// folding in the TUI; do not change this default without a TUI-driven decision.
fn determine_line_relevance_with_precedence(
    line_byte_range: (usize, usize),
    annotations: &[ByteRangeAnnotation],
) -> RelevanceScore {
    annotations
        .iter()
        .filter(|ann| line_utils::ranges_overlap(ann.byte_range, line_byte_range))
        .map(|ann| ann.relevance)
        .min()
        .unwrap_or(ESSENTIAL)
}

/// Fallible conversion from ReviewableDiff to RenderableDiff
impl<'source> TryFrom<&'source ReviewableDiff> for RenderableDiff<'source> {
    type Error = RenderableDiffError;

    fn try_from(reviewable: &'source ReviewableDiff) -> Result<Self, Self::Error> {
        // Use Myers diff for Modified changes, single source for others
        let lines = match &reviewable.boundary.change_status {
            NodeChangeStatus::Modified {
                old_node, new_node, ..
            } => create_line_by_line_diff_for_modified(reviewable, old_node, new_node)?,
            _ => create_single_source_lines(reviewable)?,
        };

        // Create simplified metadata
        let boundary_name = extract_boundary_name(reviewable);
        let essential_line_count = lines
            .iter()
            .filter(|line| line.max_relevance() == ESSENTIAL)
            .count();

        // Calculate overall line range from boundary node
        let (boundary_node, boundary_source) =
            reviewable.boundary.change_status.display_node_with_source(
                reviewable.old_source.as_ref(),
                reviewable.new_source.as_ref(),
            );
        let overall_line_range = boundary_source.line_range(boundary_node);

        // Collect line numbers that have changes
        let changed_line_numbers: Vec<usize> = lines
            .iter()
            .filter(|line| line.has_changes())
            .map(|line| line.line_number)
            .collect();

        let metadata = RenderableMetadata {
            total_changes: reviewable.metadata.total_changes,
            change_summary: reviewable.metadata.change_summary.clone(),
            essential_line_count,
            boundary_name,
            overall_line_range,
            changed_line_numbers,
        };

        Ok(RenderableDiff {
            lines,
            metadata,
            language: reviewable.language,
        })
    }
}

// Implementation methods
impl<'source> RenderableLine<'source> {
    /// Get the highest relevance score for this line
    pub fn max_relevance(&self) -> RelevanceScore {
        self.annotations
            .iter()
            .map(|a| a.relevance)
            .min() // Lower score = higher relevance
            .unwrap_or(NOISE)
    }

    /// Check if line has any changes
    pub fn has_changes(&self) -> bool {
        self.annotations.iter().any(|a| a.change_type.is_some())
    }

    /// Get primary change type for line prefix (+, -, ~)
    pub fn primary_change_type(&self) -> Option<&ChangeType> {
        self.annotations
            .iter()
            .filter_map(|a| a.change_type.as_ref())
            .min_by_key(|ct| change_type_priority(ct))
    }

    /// Check if line should be folded/hidden
    pub fn should_fold(&self) -> bool {
        self.max_relevance() >= BACKGROUND && !self.has_changes()
    }

    /// Get display style (prefix and color) for this line
    pub fn get_display_style(&self) -> (&'static str, &'static str) {
        match self.primary_change_type() {
            Some(ChangeType::Added) => ("+", "\x1b[32m"),   // Green
            Some(ChangeType::Deleted) => ("-", "\x1b[31m"), // Red
            Some(ChangeType::Modified) => ("~", "\x1b[33m"), // Yellow
            None => ("  ", "\x1b[37m"),                     // Light Gray
        }
    }
}

/// Implementation methods for RenderableDiff
impl<'source> RenderableDiff<'source> {
    /// Get the overall line range covered by this renderable diff
    pub fn line_range(&self) -> LineRange {
        self.metadata.overall_line_range
    }

    /// Get all line numbers that contain changes
    pub fn changed_line_numbers(&self) -> &[usize] {
        &self.metadata.changed_line_numbers
    }

    /// Check if a specific line number has changes
    pub fn line_has_changes(&self, line_number: usize) -> bool {
        self.metadata.changed_line_numbers.contains(&line_number)
    }

    /// Get the range of lines with changes (min and max)
    /// Returns None if no lines have changes
    pub fn changed_line_range(&self) -> Option<(usize, usize)> {
        if self.metadata.changed_line_numbers.is_empty() {
            None
        } else {
            Some((
                *self.metadata.changed_line_numbers.iter().min().unwrap(),
                *self.metadata.changed_line_numbers.iter().max().unwrap(),
            ))
        }
    }

    /// Get all lines that have changes
    pub fn changed_lines(&self) -> Vec<&RenderableLine<'source>> {
        self.lines
            .iter()
            .filter(|line| line.has_changes())
            .collect()
    }

    /// Count the number of lines with changes
    pub fn changed_line_count(&self) -> usize {
        self.metadata.changed_line_numbers.len()
    }
}

/// Get priority order for change types (lower = higher priority)
fn change_type_priority(change_type: &ChangeType) -> u8 {
    match change_type {
        ChangeType::Added => 0,
        ChangeType::Deleted => 1,
        ChangeType::Modified => 2,
    }
}
