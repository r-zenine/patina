use crate::ast_diff::error::SourceError;
use crate::ast_diff::line_index::LineIndex;
use crate::ast_diff::nodes::NodeLike;

/// Line range information for a node
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineRange {
    /// Start line number (1-based)
    pub start_line: usize,
    /// End line number (1-based)
    pub end_line: usize,
    /// Start column (0-based)
    pub start_column: usize,
    /// End column (0-based)
    pub end_column: usize,
}

/// Trait for accessing source content via AST nodes
/// Maintains AST-first architecture by requiring node-based access
pub trait SourceProvider: Send + Sync {
    /// Extract the actual text content for any node-like object.
    /// Works with both TreeSitter Node and OwnedNodeData through the NodeLike trait.
    fn node_text<'a>(&'a self, node: &dyn NodeLike) -> Result<&'a str, SourceError>;

    /// Get the line range (line numbers and columns) for any node-like object.
    /// Line numbers are 1-based, columns are 0-based.
    fn line_range(&self, node: &dyn NodeLike) -> LineRange;

    /// Get the complete source code
    fn full_source(&self) -> &str;

    /// Clone this SourceProvider into a Box
    /// This enables cloning through trait objects
    fn clone_box(&self) -> Box<dyn SourceProvider>;
}

/// Restricted source code access that only allows extracting content for specific AST nodes.
/// This prevents string-based analysis while enabling content-aware AST diffing.
///
/// # Architectural Constraint
/// This wrapper intentionally provides a minimal interface to prevent future developers
/// from falling back to string-based operations instead of AST-based analysis.
///
/// # What is NOT provided (intentionally):
/// - String searching/regex operations
/// - Line-based operations
/// - Arbitrary slicing beyond node boundaries  
/// - String length/indexing operations
/// - General string comparison utilities
#[derive(Debug, Clone)]
pub struct SourceCode {
    content: String,
    line_index: LineIndex,
}

impl SourceCode {
    /// Create a new SourceCode wrapper for the given source text
    pub fn new(content: impl Into<String>) -> Self {
        let content = content.into();
        let line_index = LineIndex::new(&content);
        Self {
            content,
            line_index,
        }
    }

    /// Extract the actual text content for a specific TreeSitter node.
    /// This is the ONLY method provided to maintain AST-first architecture.
    ///
    /// # Parameters
    /// - `node`: The TreeSitter node to extract text for
    ///
    /// # Returns
    /// The UTF-8 string content of the node, or an error if the byte range is invalid
    pub fn node_text(&self, node: &impl NodeLike) -> Result<&str, SourceError> {
        let node_start = node.start_byte();
        let node_end = node.end_byte();
        let source_bytes = self.content.as_bytes();
        let source_length = source_bytes.len();

        // Validate node byte range against source length
        if node_end > source_length {
            return Err(SourceError::NodeRangeOutOfBounds {
                node_start,
                node_end,
                source_length,
            });
        }

        // Extract text using byte range (works for both TreeSitter and owned nodes)
        std::str::from_utf8(&source_bytes[node_start..node_end]).map_err(SourceError::from)
    }

    /// Calculate line range from byte positions
    /// This is used for owned node data that doesn't have TreeSitter position information
    ///
    /// Both endpoints go through `LineIndex` (fixes the column-0 off-by-one: a
    /// node starting at column 0 of line N used to report N-1 because
    /// `prefix.lines().count()` ignores a trailing newline).
    pub fn line_range_from_bytes(&self, start_byte: usize, end_byte: usize) -> LineRange {
        let start_byte = start_byte.min(self.content.len());
        let end_byte = end_byte.min(self.content.len());

        let start_line = self.line_index.byte_to_line(start_byte).max(1);
        let end_line = self.line_index.byte_to_line(end_byte).max(1);
        let start_column = start_byte - self.line_index.line_start(start_line).unwrap_or(0);
        let end_column = end_byte - self.line_index.line_start(end_line).unwrap_or(0);

        LineRange {
            start_line,
            end_line,
            start_column,
            end_column,
        }
    }
}

impl SourceProvider for SourceCode {
    fn node_text<'a>(&'a self, node: &dyn NodeLike) -> Result<&'a str, SourceError> {
        let node_start = node.start_byte();
        let node_end = node.end_byte();
        let source_bytes = self.content.as_bytes();
        let source_length = source_bytes.len();

        // Validate node byte range against source length
        if node_end > source_length {
            return Err(SourceError::NodeRangeOutOfBounds {
                node_start,
                node_end,
                source_length,
            });
        }

        // Extract text using byte range
        std::str::from_utf8(&source_bytes[node_start..node_end]).map_err(SourceError::from)
    }

    fn line_range(&self, node: &dyn NodeLike) -> LineRange {
        // Check if this is a TreeSitter node with position info
        if let Some(ts_node) = node.as_tree_sitter_node() {
            let start_pos = ts_node.start_position();
            let end_pos = ts_node.end_position();

            LineRange {
                start_line: start_pos.row + 1, // TreeSitter is 0-based, convert to 1-based
                end_line: end_pos.row + 1,
                start_column: start_pos.column,
                end_column: end_pos.column,
            }
        } else {
            // Fall back to calculating from byte positions for OwnedNodeData
            self.line_range_from_bytes(node.start_byte(), node.end_byte())
        }
    }

    fn full_source(&self) -> &str {
        &self.content
    }

    fn clone_box(&self) -> Box<dyn SourceProvider> {
        Box::new(self.clone())
    }
}
