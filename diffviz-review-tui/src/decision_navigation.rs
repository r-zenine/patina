//! Tree-based navigation state for decision-first review hierarchy
//!
//! Models the navigation as an actual tree structure where each node has identity
//! and expansion state. Navigation operations work on the tree itself, avoiding
//! synchronization issues with flat indices into dynamically rebuilt structures.

use diffviz_review::{engines::ReviewEngine, ReviewableDiffId};

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

    /// Whether this decision is expanded to show its files
    pub expanded: bool,

    /// Files contained in this decision
    pub files: Vec<FileNode>,
}

/// A file node within a decision
#[derive(Debug, Clone)]
pub struct FileNode {
    /// File path
    pub path: String,

    /// Whether this file is expanded to show its chunks
    pub expanded: bool,

    /// Chunks (diffs) within this file
    pub chunks: Vec<ChunkNode>,
}

/// A chunk (ReviewableDiff) within a file
#[derive(Debug, Clone)]
pub struct ChunkNode {
    /// The ReviewableDiff identifier
    pub chunk_id: ReviewableDiffId,
}

/// Path to a node in the tree (which decision/file/chunk is selected)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreePath {
    /// Index into decisions array
    pub decision_index: usize,

    /// If Some, index into files array of selected decision
    pub file_index: Option<usize>,

    /// If Some, index into chunks array of selected file
    pub chunk_index: Option<usize>,
}

impl TreePath {
    /// Create a path pointing to a decision
    pub fn decision(index: usize) -> Self {
        Self {
            decision_index: index,
            file_index: None,
            chunk_index: None,
        }
    }

    /// Create a path pointing to a file
    pub fn file(decision_index: usize, file_index: usize) -> Self {
        Self {
            decision_index,
            file_index: Some(file_index),
            chunk_index: None,
        }
    }

    /// Create a path pointing to a chunk
    pub fn chunk(decision_index: usize, file_index: usize, chunk_index: usize) -> Self {
        Self {
            decision_index,
            file_index: Some(file_index),
            chunk_index: Some(chunk_index),
        }
    }

    /// Get the depth of this path (0=decision, 1=file, 2=chunk)
    pub fn depth(&self) -> usize {
        if self.chunk_index.is_some() {
            2
        } else if self.file_index.is_some() {
            1
        } else {
            0
        }
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
    File {
        decision_num: u32,
        path: String,
        expanded: bool,
    },
    Chunk {
        decision_num: u32,
        file_path: String,
        chunk_id: ReviewableDiffId,
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
            // Collect unique files for this decision
            let mut file_paths = Vec::new();
            for impact in &decision.code_impacts {
                if !file_paths.contains(&impact.file) {
                    file_paths.push(impact.file.clone());
                }
            }
            file_paths.sort();

            // Build file nodes for each file
            let mut files = Vec::new();
            for file_path in file_paths {
                // Get chunks for this file
                let chunks =
                    get_chunks_for_file_in_decision(review_engine, decision.number, &file_path);

                let chunk_nodes = chunks
                    .into_iter()
                    .map(|chunk_id| ChunkNode { chunk_id })
                    .collect();

                files.push(FileNode {
                    path: file_path,
                    expanded: false,
                    chunks: chunk_nodes,
                });
            }

            nodes.push(DecisionNode {
                decision_number: decision.number,
                expanded: false,
                files,
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

            // If decision is expanded, add its files
            if decision_node.expanded {
                for (file_idx, file_node) in decision_node.files.iter().enumerate() {
                    result.push(FlattenedNode {
                        path: TreePath::file(decision_idx, file_idx),
                        kind: FlattenedNodeKind::File {
                            decision_num: decision_node.decision_number,
                            path: file_node.path.clone(),
                            expanded: file_node.expanded,
                        },
                    });

                    // If file is expanded, add its chunks
                    if file_node.expanded {
                        for (chunk_idx, chunk_node) in file_node.chunks.iter().enumerate() {
                            result.push(FlattenedNode {
                                path: TreePath::chunk(decision_idx, file_idx, chunk_idx),
                                kind: FlattenedNodeKind::Chunk {
                                    decision_num: decision_node.decision_number,
                                    file_path: file_node.path.clone(),
                                    chunk_id: chunk_node.chunk_id.clone(),
                                },
                            });
                        }
                    }
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

    /// Toggle expansion of the currently selected node
    pub fn toggle_expansion(&mut self) {
        let path = &self.selected_path;

        if let Some(decision) = self.nodes.get_mut(path.decision_index) {
            if let Some(file_idx) = path.file_index {
                // Toggle file expansion
                if let Some(file) = decision.files.get_mut(file_idx) {
                    file.expanded = !file.expanded;
                }
            } else {
                // Toggle decision expansion
                decision.expanded = !decision.expanded;
            }
        }
    }

    /// Expand the currently selected node
    pub fn expand_current(&mut self) {
        let path = &self.selected_path;

        if let Some(decision) = self.nodes.get_mut(path.decision_index) {
            if let Some(file_idx) = path.file_index {
                if let Some(file) = decision.files.get_mut(file_idx) {
                    file.expanded = true;
                }
            } else {
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

    /// Get the currently selected file path
    pub fn selected_file_path(&self) -> Option<String> {
        let path = &self.selected_path;
        self.nodes
            .get(path.decision_index)
            .and_then(|d| path.file_index.and_then(|f_idx| d.files.get(f_idx)))
            .map(|f| f.path.clone())
    }

    /// Get the currently selected chunk ID
    pub fn selected_chunk_id(&self) -> Option<ReviewableDiffId> {
        let path = &self.selected_path;
        self.nodes
            .get(path.decision_index)
            .and_then(|d| {
                path.file_index
                    .and_then(|f_idx| d.files.get(f_idx))
                    .and_then(|f| path.chunk_index.and_then(|c_idx| f.chunks.get(c_idx)))
            })
            .map(|c| c.chunk_id.clone())
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
            for file in &mut node.files {
                file.expanded = false;
            }
        }
        self.selected_path = TreePath::decision(0);
    }
}

impl Default for DecisionNavigationTree {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to get chunks for a specific file within a decision
fn get_chunks_for_file_in_decision(
    review_engine: &ReviewEngine,
    decision_number: u32,
    file_path: &str,
) -> Vec<ReviewableDiffId> {
    if let Some(decision) = review_engine.get_decision(decision_number) {
        let mut chunk_ids: Vec<ReviewableDiffId> = Vec::new();
        for code_impact in &decision.code_impacts {
            if code_impact.file == file_path {
                // Find ReviewableDiffs that match this code impact's line ranges
                // Only include diffs created by this decision (ID file_path contains #d{decision_number}:)
                let decision_marker = format!("#d{decision_number}:");
                let all_diffs = review_engine.get_ordered_reviewable_ids();
                for id_ref in &all_diffs {
                    let id = (*id_ref).clone();
                    // Filter by decision number to avoid showing chunks from other decisions
                    if !id.file_path.contains(&decision_marker) {
                        continue;
                    }
                    if let Some(diff) = review_engine.get_reviewable_diff(&id) {
                        if diff.file_path == file_path {
                            // Check if diff overlaps with any line range in this code impact
                            for line_range in &code_impact.line_ranges {
                                if diff.id.line_range.end_line >= line_range.start
                                    && diff.id.line_range.start_line <= line_range.end
                                {
                                    if !chunk_ids.contains(&id) {
                                        chunk_ids.push(id.clone());
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
        chunk_ids
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_path_depth() {
        assert_eq!(TreePath::decision(0).depth(), 0);
        assert_eq!(TreePath::file(0, 0).depth(), 1);
        assert_eq!(TreePath::chunk(0, 0, 0).depth(), 2);
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
