use crate::ast_diff::error::SourceError;
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
}

impl SourceCode {
    /// Create a new SourceCode wrapper for the given source text
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
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
    pub fn line_range_from_bytes(&self, start_byte: usize, end_byte: usize) -> LineRange {
        let content_str = &self.content;

        // Find start position
        let start_pos = content_str[..start_byte.min(content_str.len())]
            .lines()
            .count();
        let start_line = if start_byte == 0 { 1 } else { start_pos };
        let start_column = content_str[..start_byte.min(content_str.len())]
            .rfind('\n')
            .map(|pos| start_byte - pos - 1)
            .unwrap_or(start_byte);

        // Find end position
        let end_content = &content_str[..end_byte.min(content_str.len())];
        let end_pos = end_content.lines().count();
        let end_line = if end_byte == 0 { 1 } else { end_pos };
        let end_column = end_content
            .rfind('\n')
            .map(|pos| end_byte - pos - 1)
            .unwrap_or(end_byte);

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
