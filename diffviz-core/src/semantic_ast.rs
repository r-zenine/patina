//! Semantic AST for Meaningful Code Review Boundaries  
//!
//! This module provides a higher-level abstraction over TreeSitter ASTs by grouping
//! low-level tokens into meaningful semantic constructs (functions, classes, modules).
//! This solves the fragmentation problem where traditional AST diffs create many tiny,
//! meaningless boundaries for large-scale refactors.
//!
//! ## Design Principles
//! 1. **Universal Categories**: 5 semantic unit types that work across programming languages
//! 2. **Type Safety**: Structured metadata per unit type with flexible HashMap extension
//! 3. **Review-Focused**: Each semantic unit represents a discrete, reviewable change
//! 4. **Coverage Guarantee**: Every source byte belongs to exactly one semantic node
//!
//! ## Example Usage
//! ```ignore
//! // This will work once we extend LanguageParser trait
//! let parser = RustParser::new();
//! let semantic_tree = parser.build_semantic_tree(&ast, source)?;
//! let pairs = build_semantic_pairs(old_tree, new_tree)?;
//! ```

use crate::common::ProgrammingLanguage;
use std::collections::HashMap;
use tree_sitter::Node;

/// A semantic AST tree containing meaningful code constructs
#[derive(Debug, Clone)]
pub struct SemanticTree<'a> {
    /// Root semantic node (typically represents the entire file)
    pub root: SemanticNode<'a>,

    /// Programming language this tree represents
    pub language: ProgrammingLanguage,

    /// Mapping from semantic nodes back to source ranges
    pub source_ranges: Vec<SourceRange>,
}

/// Represents a metadata node (attribute, decorator, annotation) and its position
/// relative to the main semantic node
#[derive(Debug, Clone)]
pub struct MetadataNode<'a> {
    /// The TreeSitter node representing the metadata
    pub node: Node<'a>,
    /// Position relative to the main node
    pub position: MetadataPosition,
}

/// Position of a metadata node relative to the main semantic node
#[derive(Debug, Clone, PartialEq)]
pub enum MetadataPosition {
    /// Node appears before the main node as a sibling
    /// Negative values indicate distance (-1 = immediately before, -2 = two nodes before)
    PrecedingSibling(i32),
    /// Node appears after the main node as a sibling
    /// Positive values indicate distance (1 = immediately after, 2 = two nodes after)
    FollowingSibling(i32),
}

/// A semantic unit representing a meaningful code construct
#[derive(Debug, Clone)]
pub struct SemanticNode<'a> {
    /// Reference to the original TreeSitter AST node
    pub tree_sitter_node: Node<'a>,

    /// Metadata nodes (attributes, decorators, annotations) associated with this semantic unit
    pub metadata_nodes: Vec<MetadataNode<'a>>,

    /// Child semantic units nested within this construct
    pub children: Vec<SemanticNode<'a>>,

    /// Reference to the identifier/name node (e.g., function name, class name)
    pub name_node: Option<Node<'a>>,

    /// The type of semantic unit with associated metadata
    pub unit_type: SemanticUnitType<'a>,
}

/// Universal semantic unit types that work across programming languages
#[derive(Debug, Clone, PartialEq)]
pub enum SemanticUnitType<'a> {
    /// Data structures: struct, class, interface, enum, tuple
    DataStructure {
        /// Whether this structure has generic/template parameters
        is_generic: bool,

        /// Number of fields/members (None if not easily countable)
        field_count: Option<usize>,

        /// Base classes, traits, interfaces this inherits from/implements
        inheritance: Vec<String>,

        /// Visibility modifier (pub, private, protected, etc.)
        visibility: String,

        /// Reference to the structure's declaration signature
        signature_node: Option<Node<'a>>,

        /// Language-specific metadata (decorators, attributes, etc.)
        metadata: HashMap<String, String>,
    },

    /// Callable constructs: function, method, constructor, lambda
    Callable {
        /// Whether this callable has generic/template parameters
        is_generic: bool,

        /// Number of parameters (excluding self/this)
        parameter_count: usize,

        /// Return type if specified/inferrable
        return_type: Option<String>,

        /// Whether this is an async/awaitable function
        is_async: bool,

        /// Visibility modifier (pub, private, protected, etc.)
        visibility: String,

        /// Whether this is a method (has self/this) vs standalone function
        is_method: bool,

        /// Reference to the function signature (parameters, return type)
        signature_node: Option<Node<'a>>,

        /// Reference to the function body/implementation
        body_node: Option<Node<'a>>,

        /// Language-specific metadata (decorators, extern, unsafe, etc.)
        metadata: HashMap<String, String>,
    },

    /// Variables and constants: global vars, static vars, file-level constants
    Variable {
        /// Whether this is a compile-time constant (const, final, readonly)
        is_const: bool,

        /// Whether this is a static/class variable vs instance variable
        is_static: bool,

        /// Type annotation if present
        type_annotation: Option<String>,

        /// Visibility modifier (pub, private, etc.)
        visibility: String,

        /// Language-specific metadata (mutable, volatile, etc.)
        metadata: HashMap<String, String>,
    },

    /// Import statements: import, use, include, require
    Import {
        /// Type of import (wildcard, specific items, default, etc.)
        import_type: ImportType,

        /// Source module/package being imported from  
        source_module: String,

        /// Specific items being imported (empty for wildcard imports)
        imported_items: Vec<String>,

        /// Language-specific metadata (aliases, conditional imports, etc.)
        metadata: HashMap<String, String>,
    },

    /// Module organization: namespace, module, package, file organization
    Module {
        /// Type of module construct
        module_type: ModuleType,

        /// Whether this module is publicly accessible
        is_public: bool,

        /// Language-specific metadata (version, attributes, etc.)
        metadata: HashMap<String, String>,
    },

    /// Unknown node type - ensures complete AST coverage
    /// Used for AST nodes that don't fit into standard semantic categories
    Unknown {
        /// The tree-sitter node kind (for debugging and analysis)
        node_kind: String,

        /// Language-specific metadata (original text, context, etc.)
        metadata: HashMap<String, String>,
    },
}

/// Types of import statements
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum ImportType {
    /// Wildcard import: import * from foo, use foo::*
    Wildcard,

    /// Specific items: import { a, b } from foo, use foo::{a, b}
    Specific,

    /// Default import: import foo from bar
    Default,

    /// Namespace import: import foo as bar, use foo as bar
    Namespace,
}

/// Types of module constructs
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum ModuleType {
    /// Language namespace: namespace foo, package foo
    Namespace,

    /// Module declaration: mod foo, module foo  
    Module,

    /// File-level organization (implicit module from file structure)
    File,
}

/// Source range mapping for semantic nodes
#[derive(Debug, Clone, PartialEq)]
pub struct SourceRange {
    /// Starting byte offset in source
    pub start_byte: usize,

    /// Ending byte offset in source
    pub end_byte: usize,

    /// Starting line number (1-indexed)
    pub start_line: usize,

    /// Ending line number (1-indexed)
    pub end_line: usize,
}

/// Semantic similarity between two units with multiple concurrent change types
#[derive(Debug, Clone, PartialEq)]
pub struct SemanticSimilarity {
    /// Whether there are name/identifier changes
    pub name_changed: bool,

    /// Whether there are signature changes (parameters, return type, modifiers)
    pub signature_changed: bool,

    /// Whether there are body/implementation changes
    pub body_changed: bool,

    /// Whether there are structural changes (visibility, inheritance, etc.)
    pub structural_changed: bool,

    /// Overall confidence that these units represent the same semantic concept (0.0 to 1.0)
    pub match_confidence: f32,

    /// Specific confidence metrics for different change aspects
    pub confidence_metrics: SimilarityMetrics,
}

/// Detailed confidence metrics for similarity assessment
#[derive(Debug, Clone, PartialEq)]
pub struct SimilarityMetrics {
    /// How similar the names are (0.0 to 1.0)
    pub name_similarity: f32,

    /// How similar the signatures are (0.0 to 1.0)
    pub signature_similarity: f32,

    /// How similar the body structure is (0.0 to 1.0)
    pub body_similarity: f32,

    /// How similar the overall structure is (0.0 to 1.0)
    pub structural_similarity: f32,
}

impl SemanticSimilarity {
    /// Create a new similarity with no changes (identical units)
    pub fn identical() -> Self {
        Self {
            name_changed: false,
            signature_changed: false,
            body_changed: false,
            structural_changed: false,
            match_confidence: 1.0,
            confidence_metrics: SimilarityMetrics::perfect(),
        }
    }

    /// Create similarity for body-only changes
    pub fn body_change() -> Self {
        Self {
            name_changed: false,
            signature_changed: false,
            body_changed: true,
            structural_changed: false,
            match_confidence: 1.0,
            confidence_metrics: SimilarityMetrics {
                name_similarity: 1.0,
                signature_similarity: 1.0,
                body_similarity: 0.8,
                structural_similarity: 1.0,
            },
        }
    }

    /// Create similarity for name refactoring
    pub fn name_refactor(confidence: f32) -> Self {
        Self {
            name_changed: true,
            signature_changed: false,
            body_changed: false,
            structural_changed: false,
            match_confidence: confidence,
            confidence_metrics: SimilarityMetrics {
                name_similarity: 0.0,
                signature_similarity: 1.0,
                body_similarity: 1.0,
                structural_similarity: 1.0,
            },
        }
    }

    /// Create similarity for signature changes
    pub fn signature_change(name_identical: bool, signature_similarity: f32) -> Self {
        Self {
            name_changed: !name_identical,
            signature_changed: true,
            body_changed: false,
            structural_changed: false,
            match_confidence: signature_similarity,
            confidence_metrics: SimilarityMetrics {
                name_similarity: if name_identical { 1.0 } else { 0.0 },
                signature_similarity,
                body_similarity: 1.0,
                structural_similarity: 1.0,
            },
        }
    }

    /// Create similarity for structural refactoring
    pub fn structural_refactor(confidence: f32) -> Self {
        Self {
            name_changed: false,
            signature_changed: false,
            body_changed: false,
            structural_changed: true,
            match_confidence: confidence,
            confidence_metrics: SimilarityMetrics {
                name_similarity: 1.0,
                signature_similarity: 1.0,
                body_similarity: 1.0,
                structural_similarity: confidence,
            },
        }
    }

    /// Create similarity for unrelated units
    pub fn unrelated() -> Self {
        Self {
            name_changed: true,
            signature_changed: true,
            body_changed: true,
            structural_changed: true,
            match_confidence: 0.0,
            confidence_metrics: SimilarityMetrics::none(),
        }
    }

    /// Whether this similarity represents identical units
    pub fn is_identical(&self) -> bool {
        !self.name_changed
            && !self.signature_changed
            && !self.body_changed
            && !self.structural_changed
    }

    /// Whether any changes exist
    pub fn has_changes(&self) -> bool {
        self.name_changed || self.signature_changed || self.body_changed || self.structural_changed
    }

    /// Get primary change type for backwards compatibility
    pub fn primary_change_type(&self) -> &'static str {
        if self.is_identical() {
            "Identical"
        } else if self.structural_changed {
            "Structural"
        } else if self.signature_changed {
            "Signature"
        } else if self.name_changed {
            "NameRefactor"
        } else if self.body_changed {
            "BodyChange"
        } else {
            "Unknown"
        }
    }
}

impl SimilarityMetrics {
    /// Perfect similarity metrics (1.0 for all)
    pub fn perfect() -> Self {
        Self {
            name_similarity: 1.0,
            signature_similarity: 1.0,
            body_similarity: 1.0,
            structural_similarity: 1.0,
        }
    }

    /// No similarity metrics (0.0 for all)
    pub fn none() -> Self {
        Self {
            name_similarity: 0.0,
            signature_similarity: 0.0,
            body_similarity: 0.0,
            structural_similarity: 0.0,
        }
    }
}

impl<'a> SemanticTree<'a> {
    /// Create a new semantic tree
    pub fn new(root: SemanticNode<'a>, language: ProgrammingLanguage) -> Self {
        Self {
            root,
            language,
            source_ranges: Vec::new(),
        }
    }

    /// Get all semantic units in the tree (flattened) - NO FILTERING for exhaustive coverage
    /// Returns every semantic node in the tree to ensure complete AST coverage
    pub fn all_units(&'a self) -> Vec<&'a SemanticNode<'a>> {
        let mut units = Vec::new();

        // Include ALL nodes, even the root, for complete coverage
        Self::collect_all_units(&self.root, &mut units);

        units
    }

    /// Collect all units recursively without any filtering
    fn collect_all_units(node: &'a SemanticNode<'a>, units: &mut Vec<&'a SemanticNode<'a>>) {
        units.push(node);

        // Recurse into all children
        for child in &node.children {
            Self::collect_all_units(child, units);
        }
    }

    /// Get filtered semantic units for UI display (maintains existing filtering behavior)
    /// Excludes root nodes and applies size/type filtering to prevent oversized ReviewableDiffs
    pub fn filtered_units(&'a self) -> Vec<&'a SemanticNode<'a>> {
        let mut units = Vec::new();

        // Start with root's children, excluding the root itself to avoid full-file diffs
        for child in &self.root.children {
            self.collect_filtered_units(child, &mut units, false);
        }

        units
    }

    /// Recursively collect semantic units with filtering rules
    fn collect_filtered_units(
        &self,
        node: &'a SemanticNode<'a>,
        units: &mut Vec<&'a SemanticNode<'a>>,
        _is_root_call: bool,
    ) {
        // Apply filtering rules to prevent problematic ReviewableDiffs
        if self.should_include_unit(node) {
            units.push(node);
        }

        // Continue recursion through children
        for child in &node.children {
            self.collect_filtered_units(child, units, false);
        }
    }

    /// Determine if a semantic unit should be included based on filtering rules
    fn should_include_unit(&self, node: &'a SemanticNode<'a>) -> bool {
        // Rule 1: Size filtering with type-specific thresholds
        let node_size = self.estimate_node_size(node);

        match &node.unit_type {
            // Imports can be small but meaningful - allow 1+ lines, max 10
            SemanticUnitType::Import { .. } => (1..=10).contains(&node_size),

            // Exclude module-level nodes that represent entire files
            SemanticUnitType::Module { module_type, .. } => {
                !matches!(module_type, ModuleType::File)
            }

            // Other semantic constructs use original size thresholds
            SemanticUnitType::Callable { .. }
            | SemanticUnitType::DataStructure { .. }
            | SemanticUnitType::Variable { .. } => (3..=50).contains(&node_size),

            // Unknown nodes are filtered like other constructs for now
            // TODO: Consider if we want different filtering for unknown nodes
            SemanticUnitType::Unknown { .. } => {
                (1..=50).contains(&node_size) // Slightly more lenient
            }
        }
    }

    /// Estimate the size of a semantic unit in lines
    fn estimate_node_size(&self, node: &'a SemanticNode<'a>) -> usize {
        let start_line = node.tree_sitter_node.start_position().row;
        let end_line = node.tree_sitter_node.end_position().row;

        // Add 1 because tree-sitter uses 0-based line numbers
        end_line.saturating_sub(start_line) + 1
    }

    /// Find all units of a specific type
    pub fn find_units_by_type(&'a self, unit_type: &str) -> Vec<&'a SemanticNode<'a>> {
        self.all_units()
            .into_iter()
            .filter(|unit| unit.unit_type_name() == unit_type)
            .collect()
    }
}

impl<'a> SemanticNode<'a> {
    /// Create a new semantic node
    pub fn new(
        tree_sitter_node: Node<'a>,
        name_node: Option<Node<'a>>,
        unit_type: SemanticUnitType<'a>,
        metadata_nodes: Vec<MetadataNode<'a>>,
    ) -> Self {
        Self {
            tree_sitter_node,
            metadata_nodes,
            children: Vec::new(),
            name_node,
            unit_type,
        }
    }

    /// Get the name of this semantic unit from source
    pub fn name(&self, source: &str) -> Option<String> {
        self.name_node?
            .utf8_text(source.as_bytes())
            .ok()
            .map(|s| s.to_string())
    }

    /// Get the type name of this semantic unit
    pub fn unit_type_name(&self) -> &'static str {
        match &self.unit_type {
            SemanticUnitType::DataStructure { .. } => "data_structure",
            SemanticUnitType::Callable { .. } => "callable",
            SemanticUnitType::Variable { .. } => "variable",
            SemanticUnitType::Import { .. } => "import",
            SemanticUnitType::Module { .. } => "module",
            SemanticUnitType::Unknown { .. } => "unknown",
        }
    }

    /// Get the source range for this semantic unit including metadata nodes
    pub fn source_range(&self) -> SourceRange {
        // Find the leftmost node (earliest preceding metadata or main node)
        let start_byte = self
            .metadata_nodes
            .iter()
            .filter_map(|m| match m.position {
                MetadataPosition::PrecedingSibling(_) => Some(m.node.start_byte()),
                _ => None,
            })
            .min()
            .unwrap_or(self.tree_sitter_node.start_byte());

        // Find the rightmost node (latest following metadata or main node)
        let end_byte = self
            .metadata_nodes
            .iter()
            .filter_map(|m| match m.position {
                MetadataPosition::FollowingSibling(_) => Some(m.node.end_byte()),
                _ => None,
            })
            .max()
            .unwrap_or(self.tree_sitter_node.end_byte());

        // Find the start position from the leftmost node
        let start_node = self
            .metadata_nodes
            .iter()
            .filter(|m| matches!(m.position, MetadataPosition::PrecedingSibling(_)))
            .min_by_key(|m| m.node.start_byte())
            .map(|m| m.node)
            .unwrap_or(self.tree_sitter_node);

        // Find the end position from the rightmost node
        let end_node = self
            .metadata_nodes
            .iter()
            .filter(|m| matches!(m.position, MetadataPosition::FollowingSibling(_)))
            .max_by_key(|m| m.node.end_byte())
            .map(|m| m.node)
            .unwrap_or(self.tree_sitter_node);

        SourceRange {
            start_byte,
            end_byte,
            start_line: start_node.start_position().row + 1,
            end_line: end_node.end_position().row + 1,
        }
    }

    /// Check if this unit can be meaningfully paired with another unit
    pub fn can_pair_with(
        &self,
        other: &SemanticNode,
        old_source: &dyn crate::ast_diff::SourceProvider,
        new_source: &dyn crate::ast_diff::SourceProvider,
    ) -> bool {
        // Must be same unit type to be pairable
        if self.unit_type_name() != other.unit_type_name() {
            return false;
        }

        // Special handling for Import units - they should be paired based on their full content
        if let (
            SemanticUnitType::Import {
                source_module: old_mod,
                imported_items: old_items,
                ..
            },
            SemanticUnitType::Import {
                source_module: new_mod,
                imported_items: new_items,
                ..
            },
        ) = (&self.unit_type, &other.unit_type)
        {
            return old_mod == new_mod && old_items == new_items;
        }

        // For other units, must have the same name to be pairable
        let old_name = self
            .name_node
            .and_then(|node| old_source.node_text(&node).ok());
        let new_name = other
            .name_node
            .and_then(|node| new_source.node_text(&node).ok());

        match (old_name, new_name) {
            (Some(old), Some(new)) => old == new,
            // If either doesn't have a name, they can't be paired
            _ => false,
        }
    }

    /// Check if this semantic node is structurally identical to another
    /// This performs a deep comparison of the semantic structure without expensive text parsing
    pub fn is_semantically_identical(
        &self,
        other: &SemanticNode,
        old_source: &dyn crate::ast_diff::SourceProvider,
        new_source: &dyn crate::ast_diff::SourceProvider,
    ) -> bool {
        // Must have the same semantic unit type (discriminant comparison)
        if std::mem::discriminant(&self.unit_type) != std::mem::discriminant(&other.unit_type) {
            return false;
        }

        // Must have the same name
        let self_name = self
            .name_node
            .and_then(|node| old_source.node_text(&node).ok());
        let other_name = other
            .name_node
            .and_then(|node| new_source.node_text(&node).ok());

        if self_name != other_name {
            return false;
        }

        // Must have the same number of children
        if self.children.len() != other.children.len() {
            return false;
        }

        // Must have the same number of metadata nodes
        if self.metadata_nodes.len() != other.metadata_nodes.len() {
            return false;
        }

        // Recursively check all children are identical
        for (self_child, other_child) in self.children.iter().zip(other.children.iter()) {
            if !self_child.is_semantically_identical(other_child, old_source, new_source) {
                return false;
            }
        }

        // Check metadata nodes are identical
        for (self_meta, other_meta) in self.metadata_nodes.iter().zip(other.metadata_nodes.iter()) {
            // Compare metadata positions
            if self_meta.position != other_meta.position {
                return false;
            }

            // Compare metadata text content
            let self_meta_text = old_source.node_text(&self_meta.node).ok();
            let other_meta_text = new_source.node_text(&other_meta.node).ok();
            if self_meta_text != other_meta_text {
                return false;
            }
        }

        // For detailed unit type comparison, we need to check the specific fields
        self.unit_types_are_identical(&other.unit_type, old_source, new_source)
    }

    /// Compare the detailed fields of two semantic unit types
    fn unit_types_are_identical(
        &self,
        other_unit_type: &SemanticUnitType,
        old_source: &dyn crate::ast_diff::SourceProvider,
        new_source: &dyn crate::ast_diff::SourceProvider,
    ) -> bool {
        use SemanticUnitType::*;

        match (&self.unit_type, other_unit_type) {
            (
                Callable {
                    is_generic: self_generic,
                    parameter_count: self_params,
                    return_type: self_ret,
                    is_async: self_async,
                    visibility: self_vis,
                    is_method: self_method,
                    signature_node: self_sig,
                    ..
                },
                Callable {
                    is_generic: other_generic,
                    parameter_count: other_params,
                    return_type: other_ret,
                    is_async: other_async,
                    visibility: other_vis,
                    is_method: other_method,
                    signature_node: other_sig,
                    ..
                },
            ) => {
                self_generic == other_generic
                    && self_params == other_params
                    && self_ret == other_ret
                    && self_async == other_async
                    && self_vis == other_vis
                    && self_method == other_method
                    && self.compare_optional_nodes(*self_sig, *other_sig, old_source, new_source)
            }
            (
                DataStructure {
                    is_generic: self_generic,
                    field_count: self_fields,
                    visibility: self_vis,
                    signature_node: self_sig,
                    metadata: self_meta,
                    ..
                },
                DataStructure {
                    is_generic: other_generic,
                    field_count: other_fields,
                    visibility: other_vis,
                    signature_node: other_sig,
                    metadata: other_meta,
                    ..
                },
            ) => {
                self_generic == other_generic
                    && self_fields == other_fields
                    && self_vis == other_vis
                    && self_meta == other_meta
                    && self.compare_optional_nodes(*self_sig, *other_sig, old_source, new_source)
            }
            // For other unit types, we rely on discriminant comparison
            // which we already checked above
            _ => true,
        }
    }

    /// Helper to compare optional TreeSitter nodes
    fn compare_optional_nodes(
        &self,
        self_node: Option<tree_sitter::Node>,
        other_node: Option<tree_sitter::Node>,
        old_source: &dyn crate::ast_diff::SourceProvider,
        new_source: &dyn crate::ast_diff::SourceProvider,
    ) -> bool {
        match (self_node, other_node) {
            (Some(self_n), Some(other_n)) => {
                let self_text = old_source.node_text(&self_n).ok();
                let other_text = new_source.node_text(&other_n).ok();
                self_text == other_text
            }
            (None, None) => true,
            _ => false, // One has node, other doesn't
        }
    }
}

/// Errors that can occur during semantic AST operations
#[derive(Debug)]
pub enum SemanticError {
    /// Attempting to compare trees from different programming languages
    LanguageMismatch {
        old: ProgrammingLanguage,
        new: ProgrammingLanguage,
    },

    /// Error during semantic tree construction
    TreeBuildError(String),

    /// Language not supported for semantic tree building
    UnsupportedLanguage(String),

    /// Error during similarity calculation
    SimilarityError(String),
}

impl std::fmt::Display for SemanticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SemanticError::LanguageMismatch { old, new } => {
                write!(
                    f,
                    "Cannot compare semantic trees from different languages: {old:?} vs {new:?}"
                )
            }
            SemanticError::TreeBuildError(msg) => {
                write!(f, "Semantic tree build error: {msg}")
            }
            SemanticError::UnsupportedLanguage(msg) => {
                write!(f, "Unsupported language: {msg}")
            }
            SemanticError::SimilarityError(msg) => {
                write!(f, "Similarity calculation error: {msg}")
            }
        }
    }
}

impl std::error::Error for SemanticError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{LanguageParser, ProgrammingLanguage};

    #[test]
    fn test_semantic_unit_type_names() {
        // Create dummy TreeSitter node (normally this would come from actual parsing)
        use crate::parsers::RustParser;
        use tree_sitter::Parser;

        let parser_impl = RustParser::new();
        let mut ts_parser = Parser::new();
        ts_parser.set_language(parser_impl.get_language()).unwrap();

        let code = "fn test() {}";
        let tree = ts_parser.parse(code, None).unwrap();
        let node = tree.root_node();

        let function_unit = SemanticNode::new(
            node,
            None,
            SemanticUnitType::Callable {
                is_generic: false,
                parameter_count: 0,
                return_type: None,
                is_async: false,
                visibility: "pub".to_string(),
                is_method: false,
                signature_node: None,
                body_node: None,
                metadata: HashMap::new(),
            },
            Vec::new(), // metadata_nodes
        );

        assert_eq!(function_unit.unit_type_name(), "callable");
    }

    #[test]
    fn test_semantic_tree_creation() {
        use crate::parsers::RustParser;
        use tree_sitter::Parser;

        let parser_impl = RustParser::new();
        let mut ts_parser = Parser::new();
        ts_parser.set_language(parser_impl.get_language()).unwrap();

        let code = r#"fn test() {
    let x = 1;
    let y = 2;
    x + y
}"#;
        let tree = ts_parser.parse(code, None).unwrap();
        let node = tree.root_node();

        // Create a function node as a child of a module root
        let function_node = SemanticNode::new(
            node,
            None,
            SemanticUnitType::Callable {
                is_generic: false,
                parameter_count: 0,
                return_type: None,
                is_async: false,
                visibility: "public".to_string(),
                is_method: false,
                signature_node: None,
                body_node: None,
                metadata: HashMap::new(),
            },
            Vec::new(), // metadata_nodes
        );

        let mut module_unit = SemanticNode::new(
            node,
            None,
            SemanticUnitType::Module {
                module_type: ModuleType::Module,
                is_public: false,
                metadata: HashMap::new(),
            },
            Vec::new(), // metadata_nodes
        );

        // Add the function as a child of the module
        module_unit.children.push(function_node);

        let semantic_tree = SemanticTree::new(module_unit, ProgrammingLanguage::Rust);
        assert_eq!(semantic_tree.language, ProgrammingLanguage::Rust);
        assert_eq!(semantic_tree.all_units().len(), 2); // Should find the module root + function child
    }

    #[test]
    fn test_all_units_vs_filtered_units() {
        use crate::parsers::RustParser;
        use tree_sitter::Parser;

        let parser_impl = RustParser::new();
        let mut ts_parser = Parser::new();
        ts_parser.set_language(parser_impl.get_language()).unwrap();

        let code = "fn test() { let x = 1; }";
        let tree = ts_parser.parse(code, None).unwrap();
        let semantic_tree = parser_impl.build_semantic_tree(&tree, code).unwrap();

        let all_units = semantic_tree.all_units();
        let filtered_units = semantic_tree.filtered_units();

        // all_units should return more nodes than filtered_units
        assert!(
            all_units.len() >= filtered_units.len(),
            "all_units ({}) should be >= filtered_units ({})",
            all_units.len(),
            filtered_units.len()
        );

        // all_units should include the root node
        assert!(
            all_units
                .iter()
                .any(|node| matches!(node.unit_type, SemanticUnitType::Module { .. }))
        );

        println!("All units: {}", all_units.len());
        println!("Filtered units: {}", filtered_units.len());
    }

    #[test]
    fn test_unknown_node_handling() {
        use crate::parsers::RustParser;
        use tree_sitter::Parser;

        // Create a semantic node with Unknown type
        let parser_impl = RustParser::new();
        let mut ts_parser = Parser::new();
        ts_parser.set_language(parser_impl.get_language()).unwrap();

        // Parse some valid code but we'll create an Unknown node manually
        let tree = ts_parser.parse("fn test() {}", None).unwrap();
        let root_node = tree.root_node();

        let unknown_node = SemanticNode::new(
            root_node,
            None,
            SemanticUnitType::Unknown {
                node_kind: "ERROR".to_string(),
                metadata: HashMap::new(),
            },
            Vec::new(), // metadata_nodes
        );

        // Test that unknown nodes are handled properly
        assert_eq!(unknown_node.unit_type_name(), "unknown");

        // Create semantic tree with unknown nodes
        let semantic_tree = SemanticTree::new(unknown_node, ProgrammingLanguage::Rust);
        let all_units = semantic_tree.all_units();

        // Should include unknown nodes in all_units
        assert!(!all_units.is_empty());
        assert!(
            all_units
                .iter()
                .any(|node| matches!(node.unit_type, SemanticUnitType::Unknown { .. }))
        );
    }
}
