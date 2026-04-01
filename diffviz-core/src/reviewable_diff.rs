//! Unified ReviewableDiff structure for meaningful code review
//!
//! This module provides a self-contained data structure that consolidates
//! all scattered diff information (AST changes, context trees, relevance scores,
//! source content) into a clean, well-structured container for diff rendering.

use crate::ast_diff::{ASTChangeType, OwnedNodeData, RelevanceScore, SourceProvider};
use crate::common::ProgrammingLanguage;
use crate::common::SemanticNodeKind;
use core::fmt;
use std::collections::HashMap;

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
        writeln!(
            f,
            "[{}] {} -> {:?}",
            self.relevance, self.node_type, self.change_status
        )?;

        for child in &self.children {
            if child.node_type == "block" {
                writeln!(f, "\t -- {child}")?;
            } else {
                writeln!(f, "\t  {child}")?;
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
