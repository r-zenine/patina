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

    /// Identifier/name text extracted during parsing (populated for the 4 core languages).
    /// Used by `OwnedNodeData` to carry identifier information after the live tree is dropped.
    pub identifier: Option<String>,

    /// Container-qualified name (e.g. `"Type::name"`, `"mod::name"`), populated for
    /// units built with a known container path. Unit identity for old/new pairing is
    /// (qualified_name, unit type) rather than bare name, so changes to a method on
    /// `impl A` never pair against a same-named method on `impl B`.
    pub qualified_name: Option<String>,
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

impl<'a> SemanticTree<'a> {
    /// Create a new semantic tree
    pub fn new(root: SemanticNode<'a>, language: ProgrammingLanguage) -> Self {
        Self { root, language }
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
            identifier: None,
            qualified_name: None,
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
}

/// Errors that can occur during semantic AST operations
#[derive(Debug, thiserror::Error)]
pub enum SemanticError {
    /// Attempting to compare trees from different programming languages
    #[error("Cannot compare semantic trees from different languages: {old:?} vs {new:?}")]
    LanguageMismatch {
        old: ProgrammingLanguage,
        new: ProgrammingLanguage,
    },

    /// Error during semantic tree construction
    #[error("Semantic tree build error: {0}")]
    TreeBuildError(String),

    /// Language not supported for semantic tree building
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),

    /// Error during similarity calculation
    #[error("Similarity calculation error: {0}")]
    SimilarityError(String),
}

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
        ts_parser.set_language(&parser_impl.get_language()).unwrap();

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
        ts_parser.set_language(&parser_impl.get_language()).unwrap();

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
    fn test_unknown_node_handling() {
        use crate::parsers::RustParser;
        use tree_sitter::Parser;

        // Create a semantic node with Unknown type
        let parser_impl = RustParser::new();
        let mut ts_parser = Parser::new();
        ts_parser.set_language(&parser_impl.get_language()).unwrap();

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
