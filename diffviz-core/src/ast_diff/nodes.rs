use core::fmt;

use tree_sitter::Node;

/// Trait for node-like objects that provide essential AST node information
/// Allows working with both TreeSitter nodes and owned node data
pub trait NodeLike {
    /// Get the byte offset where this node starts in the source code
    fn start_byte(&self) -> usize;

    /// Get the byte offset where this node ends in the source code  
    fn end_byte(&self) -> usize;

    /// Get the kind of AST node (e.g., "function_item", "struct_item")
    fn kind(&self) -> &str;

    /// Check if this is a TreeSitter node (for position info)
    fn as_tree_sitter_node(&self) -> Option<&Node>;
}

/// Owned node data extracted from TreeSitter Node - fully self-contained
#[derive(Clone, PartialEq)]
pub struct OwnedNodeData {
    pub start_byte: usize,
    pub end_byte: usize,
    pub kind: String,
}

impl fmt::Debug for OwnedNodeData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "NodeData[{}]({},{})",
            self.kind, self.start_byte, self.end_byte
        )
    }
}

impl OwnedNodeData {
    /// Extract owned data from a TreeSitter Node
    pub fn from_tree_sitter_node(node: &Node) -> Self {
        Self {
            start_byte: node.start_byte(),
            end_byte: node.end_byte(),
            kind: node.kind().to_string(),
        }
    }
}

impl NodeLike for OwnedNodeData {
    fn start_byte(&self) -> usize {
        self.start_byte
    }

    fn end_byte(&self) -> usize {
        self.end_byte
    }

    fn kind(&self) -> &str {
        &self.kind
    }

    fn as_tree_sitter_node(&self) -> Option<&Node> {
        None // OwnedNodeData doesn't have a TreeSitter node
    }
}

impl NodeLike for Node<'_> {
    fn start_byte(&self) -> usize {
        self.start_byte()
    }

    fn end_byte(&self) -> usize {
        self.end_byte()
    }

    fn kind(&self) -> &str {
        self.kind()
    }

    fn as_tree_sitter_node(&self) -> Option<&Node> {
        Some(self)
    }
}
