/// The type of AST change between two versions of a node
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ASTChangeType {
    /// Structural changes: nodes added/removed
    Structural,
    /// Content changes: same structure, different text
    Content,
    /// Rename changes: kind of node changed
    Rename,
    /// Reorder changes: children reordered
    Reorder,
}

/// Numerical relevance score for context expansion
/// Lower values indicate higher relevance to the change
pub type RelevanceScore = u8;

/// Relevance scoring constants
pub const ESSENTIAL: RelevanceScore = 0; // Contains or is the actual change
pub const IMPORTANT: RelevanceScore = 1; // Direct semantic container of change
pub const BACKGROUND: RelevanceScore = 2; // Sibling context (collapsible in UI)
pub const NOISE: RelevanceScore = 3; // Unrelated context (hideable in UI)
