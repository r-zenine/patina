//! Line-based renderable diff structure optimized for display
//!
//! This module provides `RenderableDiff`, a line-oriented representation of semantic
//! diffs that bridges between tree-based semantic analysis and character/line-based
//! display systems.

use crate::common::ProgrammingLanguage;
use crate::{
    ast_diff::{BACKGROUND, ESSENTIAL, LineRange, NOISE, RelevanceScore},
    common::SemanticNodeKind,
    reviewable_diff::{NodeChangeStatus, ReviewableDiff},
};
use std::collections::HashMap;

mod line_utils;
mod myers_diff;
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
    Moved,
    Reordered,
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
    FieldAssignment,    // user.name =
    MethodCall,         // obj.method(
    Import,             // use diffviz::
    StructDeclaration,  // struct MyStruct {
    EnumDeclaration,    // enum MyEnum {
}

/// Helper function to create line-by-line diff for Modified changes using Myers algorithm
fn create_line_by_line_diff_for_modified<'source>(
    reviewable: &'source ReviewableDiff,
    old_node: &crate::ast_diff::OwnedNodeData,
    new_node: &crate::ast_diff::OwnedNodeData,
) -> Result<Vec<RenderableLine<'source>>, crate::ast_diff::SourceError> {
    use crate::renderable_diff::myers_diff::{DiffOp, myers_diff_semantic};
    use crate::renderable_diff::semantic_anchors::extract_semantic_anchor;

    // Extract old and new source text using byte ranges
    let old_text = reviewable.old_source.node_text(old_node)?;
    let new_text = reviewable.new_source.node_text(new_node)?;

    // Split into lines and prepare for Myers diff
    let old_lines: Vec<&str> = old_text.lines().collect();
    let new_lines: Vec<&str> = new_text.lines().collect();

    // Extract semantic anchors for each line
    let old_lines_with_anchors: Vec<(&str, Option<SemanticAnchor>)> = old_lines
        .iter()
        .map(|&line| (line, extract_semantic_anchor(line, reviewable, 0)))
        .collect();

    let new_lines_with_anchors: Vec<(&str, Option<SemanticAnchor>)> = new_lines
        .iter()
        .map(|&line| (line, extract_semantic_anchor(line, reviewable, 0)))
        .collect();

    // Use semantic Myers diff algorithm
    let diff_result = myers_diff_semantic(&old_lines_with_anchors, &new_lines_with_anchors);

    // Convert Myers diff operations to RenderableLines
    let mut result_lines = Vec::new();
    let mut line_number = 1;

    for op in &diff_result.ops {
        match op {
            DiffOp::Keep { line } => {
                let annotation = LineAnnotation {
                    start_col: 0,
                    end_col: line.len(),
                    relevance: ESSENTIAL,
                    change_type: None, // No change
                    semantic_kind: reviewable.boundary.semantic_kind.clone(),
                    node_depth: 0,
                };

                // Find the original line in the sources to maintain proper lifetime
                let content = find_original_line_content(line, &old_lines, &new_lines);

                result_lines.push(RenderableLine {
                    line_number,
                    content,
                    byte_range: (0, line.len()),
                    annotations: vec![annotation],
                    semantic_anchor: extract_semantic_anchor(content, reviewable, 0),
                });
                line_number += 1;
            }
            DiffOp::Delete { line } => {
                let annotation = LineAnnotation {
                    start_col: 0,
                    end_col: line.len(),
                    relevance: ESSENTIAL,
                    change_type: Some(ChangeType::Deleted),
                    semantic_kind: reviewable.boundary.semantic_kind.clone(),
                    node_depth: 0,
                };

                let content = find_original_line_content(line, &old_lines, &[]);

                result_lines.push(RenderableLine {
                    line_number,
                    content,
                    byte_range: (0, line.len()),
                    annotations: vec![annotation],
                    semantic_anchor: extract_semantic_anchor(content, reviewable, 0),
                });
                line_number += 1;
            }
            DiffOp::Add { line } => {
                let annotation = LineAnnotation {
                    start_col: 0,
                    end_col: line.len(),
                    relevance: ESSENTIAL,
                    change_type: Some(ChangeType::Added),
                    semantic_kind: reviewable.boundary.semantic_kind.clone(),
                    node_depth: 0,
                };

                let content = find_original_line_content(line, &[], &new_lines);

                result_lines.push(RenderableLine {
                    line_number,
                    content,
                    byte_range: (0, line.len()),
                    annotations: vec![annotation],
                    semantic_anchor: extract_semantic_anchor(content, reviewable, 0),
                });
                line_number += 1;
            }
        }
    }

    Ok(result_lines)
}

/// Find the original line content with proper lifetime from the source slices
fn find_original_line_content<'source>(
    target_line: &str,
    old_lines: &[&'source str],
    new_lines: &[&'source str],
) -> &'source str {
    // Try to find the line in new lines first
    for &line in new_lines {
        if line == target_line {
            return line;
        }
    }

    // Then try old lines
    for &line in old_lines {
        if line == target_line {
            return line;
        }
    }

    // Fallback: return the first available line or a static empty string
    if let Some(&first_line) = new_lines.first().or_else(|| old_lines.first()) {
        first_line
    } else {
        ""
    }
}

/// Idiomatic conversion from ReviewableDiff to RenderableDiff
impl<'source> From<&'source ReviewableDiff> for RenderableDiff<'source> {
    fn from(reviewable: &'source ReviewableDiff) -> Self {
        // Use Myers diff for Modified changes, single source for others
        let lines = match &reviewable.boundary.change_status {
            NodeChangeStatus::Modified {
                old_node, new_node, ..
            } => {
                // Use Myers diff to show proper before/after lines
                match create_line_by_line_diff_for_modified(reviewable, old_node, new_node) {
                    Ok(myers_lines) => myers_lines,
                    Err(_) => {
                        // Fallback to single source approach if Myers diff fails
                        create_single_source_lines(reviewable)
                    }
                }
            }
            _ => {
                // For non-Modified changes, use single source approach
                create_single_source_lines(reviewable)
            }
        };

        // Create simplified metadata
        let boundary_name = extract_boundary_name(reviewable);
        let essential_line_count = lines
            .iter()
            .filter(|line| line.max_relevance() == ESSENTIAL)
            .count();

        // Calculate overall line range from boundary node
        let boundary_node = get_display_node(&reviewable.boundary.change_status)
            .expect("ReviewableDiff should have a valid display node");
        let overall_line_range = reviewable.new_source.as_ref().line_range(boundary_node);

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

        RenderableDiff {
            lines,
            metadata,
            language: reviewable.language,
        }
    }
}

/// Support collecting ReviewableDiffs into RenderableDiffs
impl<'source> FromIterator<&'source ReviewableDiff> for Vec<RenderableDiff<'source>> {
    fn from_iter<I: IntoIterator<Item = &'source ReviewableDiff>>(iter: I) -> Self {
        iter.into_iter().map(RenderableDiff::from).collect()
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
            Some(ChangeType::Moved) => (">", "\x1b[33m"),   // Yellow
            Some(ChangeType::Reordered) => ("↕", "\x1b[33m"), // Yellow
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

/// Get the display node from a NodeChangeStatus (legacy function)
fn get_display_node(change_status: &NodeChangeStatus) -> Option<&crate::ast_diff::OwnedNodeData> {
    match change_status {
        NodeChangeStatus::Unchanged { node, .. } => Some(node),
        NodeChangeStatus::Added { node, .. } => Some(node),
        NodeChangeStatus::Deleted { node, .. } => Some(node),
        NodeChangeStatus::Modified { new_node, .. } => Some(new_node),
        NodeChangeStatus::Moved { new_node, .. } => Some(new_node),
        NodeChangeStatus::Reordered { new_node, .. } => Some(new_node),
    }
}

/// Get priority order for change types (lower = higher priority)
fn change_type_priority(change_type: &ChangeType) -> u8 {
    match change_type {
        ChangeType::Added => 0,
        ChangeType::Deleted => 1,
        ChangeType::Modified => 2,
        ChangeType::Moved => 3,
        ChangeType::Reordered => 4,
    }
}
