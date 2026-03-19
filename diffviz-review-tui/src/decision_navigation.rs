//! Tree-based navigation state for decision-first review hierarchy
//!
//! Models the navigation as an actual tree structure where each node has identity
//! and expansion state. Navigation operations work on the tree itself, avoiding
//! synchronization issues with flat indices into dynamically rebuilt structures.

use diffviz_review::{ReviewableDiffId, engines::ReviewEngine, entities::DecisionReviewableDiff};

/// Navigation tree that models the decision hierarchy
#[derive(Debug, Clone)]
pub struct DecisionNavigationTree {
    /// The decision nodes that form the tree
    pub nodes: Vec<DecisionNode>,

    /// Currently selected position in the tree
    pub selected_path: TreePath,
}

/// A decision node in the tree
#[derive(Debug, Clone)]
pub struct DecisionNode {
    /// Decision number
    pub decision_number: u32,

    /// Whether this decision is expanded to show its chunks
    pub expanded: bool,

    /// Chunks contained directly in this decision
    pub chunks: Vec<ChunkNode>,
}

/// A chunk (ReviewableDiff) within a decision
#[derive(Debug, Clone)]
pub struct ChunkNode {
    /// The decision-reviewable-diff pairing for this chunk
    pub chunk: DecisionReviewableDiff,

    /// Display name for this chunk (e.g., "file.rs" or "file.rs#[10-20]")
    pub display_name: String,
}

/// Path to a node in the tree (which decision/chunk is selected)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreePath {
    /// Index into decisions array
    pub decision_index: usize,

    /// If Some, index into chunks array of selected decision
    pub chunk_index: Option<usize>,
}

impl TreePath {
    /// Create a path pointing to a decision
    pub fn decision(index: usize) -> Self {
        Self {
            decision_index: index,
            chunk_index: None,
        }
    }

    /// Create a path pointing to a chunk
    pub fn chunk(decision_index: usize, chunk_index: usize) -> Self {
        Self {
            decision_index,
            chunk_index: Some(chunk_index),
        }
    }

    /// Get the depth of this path (0=decision, 1=chunk)
    pub fn depth(&self) -> usize {
        if self.chunk_index.is_some() { 1 } else { 0 }
    }
}

/// Flattened view of the tree for linear navigation
#[derive(Debug, Clone)]
pub struct FlattenedNode {
    /// Path to this node in the tree
    pub path: TreePath,

    /// What kind of node this is
    pub kind: FlattenedNodeKind,
}

#[derive(Debug, Clone)]
pub enum FlattenedNodeKind {
    Decision {
        number: u32,
        expanded: bool,
    },
    Chunk {
        decision_num: u32,
        chunk_id: ReviewableDiffId,
        display_name: String,
    },
}

impl DecisionNavigationTree {
    /// Create a new empty navigation tree
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            selected_path: TreePath::decision(0),
        }
    }

    /// Build tree from ReviewEngine decisions
    pub fn build_from_review_engine(review_engine: &ReviewEngine) -> Self {
        let decisions = review_engine.get_all_decisions();
        let mut nodes = Vec::new();

        for decision in decisions {
            // Collect all chunks for this decision
            let all_chunks: Vec<DecisionReviewableDiff> = review_engine
                .get_decision_reviewable_diffs()
                .into_iter()
                .filter(|dc| dc.decision_number == decision.number)
                .collect();

            // Group chunks by file_path
            let mut chunks_by_file: std::collections::HashMap<String, Vec<ReviewableDiffId>> =
                std::collections::HashMap::new();
            for dc in &all_chunks {
                chunks_by_file
                    .entry(dc.chunk_id.file_path.clone())
                    .or_default()
                    .push(dc.chunk_id.clone());
            }

            // Create chunk nodes with smart display names
            let mut chunk_nodes = Vec::new();
            for dc in all_chunks {
                let file_path = &dc.chunk_id.file_path;
                let file_chunk_count = chunks_by_file.get(file_path).map(|v| v.len()).unwrap_or(1);
                let display_name =
                    format_chunk_display_name(file_path, &dc.chunk_id, file_chunk_count);

                chunk_nodes.push(ChunkNode {
                    chunk: dc,
                    display_name,
                });
            }

            nodes.push(DecisionNode {
                decision_number: decision.number,
                expanded: false,
                chunks: chunk_nodes,
            });
        }

        Self {
            nodes,
            selected_path: TreePath::decision(0),
        }
    }

    /// Flatten the tree into a linear sequence for rendering and navigation
    pub fn flatten(&self) -> Vec<FlattenedNode> {
        let mut result = Vec::new();

        for (decision_idx, decision_node) in self.nodes.iter().enumerate() {
            // Add decision node
            result.push(FlattenedNode {
                path: TreePath::decision(decision_idx),
                kind: FlattenedNodeKind::Decision {
                    number: decision_node.decision_number,
                    expanded: decision_node.expanded,
                },
            });

            // If decision is expanded, add its chunks directly
            if decision_node.expanded {
                for (chunk_idx, chunk_node) in decision_node.chunks.iter().enumerate() {
                    result.push(FlattenedNode {
                        path: TreePath::chunk(decision_idx, chunk_idx),
                        kind: FlattenedNodeKind::Chunk {
                            decision_num: decision_node.decision_number,
                            chunk_id: chunk_node.chunk.chunk_id.clone(),
                            display_name: chunk_node.display_name.clone(),
                        },
                    });
                }
            }
        }

        result
    }

    /// Navigate to the next item in the flattened tree
    pub fn navigate_next(&mut self) {
        let flattened = self.flatten();
        if flattened.is_empty() {
            return;
        }

        // Find current position
        if let Some(current_idx) = flattened.iter().position(|n| n.path == self.selected_path) {
            // Move to next if not at end
            if current_idx + 1 < flattened.len() {
                self.selected_path = flattened[current_idx + 1].path.clone();
            }
        }
    }

    /// Navigate to the previous item in the flattened tree
    pub fn navigate_prev(&mut self) {
        let flattened = self.flatten();
        if flattened.is_empty() {
            return;
        }

        // Find current position
        if let Some(current_idx) = flattened.iter().position(|n| n.path == self.selected_path) {
            // Move to previous if not at start
            if current_idx > 0 {
                self.selected_path = flattened[current_idx - 1].path.clone();
            }
        }
    }

    /// Toggle expansion of the currently selected node (decisions only)
    pub fn toggle_expansion(&mut self) {
        let path = &self.selected_path;

        // Only decisions can be expanded/collapsed now
        if path.chunk_index.is_none() {
            if let Some(decision) = self.nodes.get_mut(path.decision_index) {
                decision.expanded = !decision.expanded;
            }
        }
    }

    /// Expand the currently selected node (decisions only)
    pub fn expand_current(&mut self) {
        let path = &self.selected_path;

        // Only decisions can be expanded
        if path.chunk_index.is_none() {
            if let Some(decision) = self.nodes.get_mut(path.decision_index) {
                decision.expanded = true;
            }
        }
    }

    /// Get the currently selected decision number
    pub fn selected_decision_number(&self) -> Option<u32> {
        self.nodes
            .get(self.selected_path.decision_index)
            .map(|d| d.decision_number)
    }

    /// Get the currently selected chunk ID
    pub fn selected_chunk_id(&self) -> Option<ReviewableDiffId> {
        let path = &self.selected_path;
        self.nodes
            .get(path.decision_index)
            .and_then(|d| path.chunk_index.and_then(|c_idx| d.chunks.get(c_idx)))
            .map(|c| c.chunk.chunk_id.clone())
    }

    /// Check if a decision is expanded
    pub fn is_decision_expanded(&self, decision_number: u32) -> bool {
        self.nodes
            .iter()
            .find(|d| d.decision_number == decision_number)
            .map(|d| d.expanded)
            .unwrap_or(false)
    }

    /// Reset to initial state
    pub fn reset(&mut self) {
        for node in &mut self.nodes {
            node.expanded = false;
        }
        self.selected_path = TreePath::decision(0);
    }
}

impl Default for DecisionNavigationTree {
    fn default() -> Self {
        Self::new()
    }
}

/// Format chunk display name with smart naming
/// Single-chunk files display as "file.rs", multi-chunk files display as "file.rs#[10-20]"
fn format_chunk_display_name(
    file_path: &str,
    chunk_id: &ReviewableDiffId,
    file_chunk_count: usize,
) -> String {
    let basename = file_path.split('/').next_back().unwrap_or(file_path);

    if file_chunk_count == 1 {
        basename.to_string()
    } else {
        format!(
            "{}#[{}-{}]",
            basename, chunk_id.line_range.start_line, chunk_id.line_range.end_line
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_path_depth() {
        assert_eq!(TreePath::decision(0).depth(), 0);
        assert_eq!(TreePath::chunk(0, 0).depth(), 1);
    }

    #[test]
    fn test_empty_tree_flatten() {
        let tree = DecisionNavigationTree::new();
        let flattened = tree.flatten();
        assert_eq!(flattened.len(), 0);
    }

    #[test]
    fn test_navigate_empty_tree() {
        let mut tree = DecisionNavigationTree::new();
        tree.navigate_next();
        tree.navigate_prev();
        // Should not panic on empty tree
    }
}
