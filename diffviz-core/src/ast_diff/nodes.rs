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

    /// Extract owned data from a NodeRef
    pub fn from_node_ref(node_ref: &NodeRef) -> Self {
        Self::from_tree_sitter_node(&node_ref.node)
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

/// Reference to a node in an AST tree for diff results
/// This is a thin wrapper around TreeSitter's native Node
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodeRef<'tree> {
    /// TreeSitter's native node reference
    pub node: Node<'tree>,
}

impl<'tree> NodeRef<'tree> {
    /// Create a NodeRef from a TreeSitter Node
    pub fn new(node: Node<'tree>) -> Self {
        Self { node }
    }

    /// Get the text content of this node
    pub fn text<'a>(&self, source: &'a str) -> Result<&'a str, std::str::Utf8Error> {
        self.node.utf8_text(source.as_bytes())
    }
}

impl<'tree> NodeLike for NodeRef<'tree> {
    /// Get the byte offset where this node starts in the source code
    fn start_byte(&self) -> usize {
        self.node.start_byte()
    }

    /// Get the byte offset where this node ends in the source code
    fn end_byte(&self) -> usize {
        self.node.end_byte()
    }

    /// Get the kind of AST node (e.g., "function_item", "struct_item")
    fn kind(&self) -> &str {
        self.node.kind()
    }

    fn as_tree_sitter_node(&self) -> Option<&Node> {
        Some(&self.node)
    }
}
