//! Common types and abstractions shared across the diffviz-core modules
//!
//! This module contains the core types, traits, and abstractions that are used
//! by ast_diff.rs, reviewable_diff.rs, and renderable_diff.rs modules.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tree_sitter::Tree;

#[derive(Debug, Error)]
pub enum ASTError {
    #[error("Failed to parse code: {message}")]
    ParseError { message: String },

    #[error("Language not supported: {language}")]
    UnsupportedLanguage { language: String },

    #[error("Tree-sitter parser error: {error}")]
    TreeSitterError { error: String },

    #[error("AST normalization failed: {reason}")]
    NormalizationError { reason: String },

    #[error("Insufficient context: suggest {suggested_expansion_lines} more lines")]
    InsufficientContext { suggested_expansion_lines: usize },

    #[error("Repository operation failed: {message}")]
    RepositoryError { message: String },

    #[error("Invariant violation: {message}")]
    InvariantViolation { message: String },
}

/// Semantic node kinds that abstract over language-specific TreeSitter node types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SemanticNodeKind {
    /// Function, method, or procedure definition
    Function,
    /// Class definition
    Class,
    /// Struct definition
    Struct,
    /// Enum definition
    Enum,
    /// Interface or trait definition
    Interface,
    /// Implementation block (Rust impl, etc.)
    ImplBlock,
    /// Module definition
    Module,
    /// Import/use statement
    Import,
    /// Variable declaration
    Variable,
    /// Signature component (visibility modifiers, parameters, type parameters, function modifiers, etc.)
    /// These are essential parts of function/struct/enum/trait/module signatures
    SignatureComponent,
    /// Statement within a function body
    Statement,
    /// Expression
    Expression,
    /// Type definition
    TypeDefinition,
    /// Comment
    Comment,
    /// The root file/source
    SourceFile,
    /// Unknown or language-specific node type
    Other(String),
}

pub type Result<T> = std::result::Result<T, ASTError>;

/// Programming language classification for AST processing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProgrammingLanguage {
    Rust,
    Python,
    TypeScript,
    JavaScript,
    Go,
    Java,
    C,
    Cpp,
    Unknown,
}

impl std::fmt::Display for ProgrammingLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Rust => "Rust",
            Self::Python => "Python",
            Self::TypeScript => "TypeScript",
            Self::JavaScript => "JavaScript",
            Self::Go => "Go",
            Self::Java => "Java",
            Self::C => "C",
            Self::Cpp => "C++",
            Self::Unknown => "Unknown",
        };
        write!(f, "{name}")
    }
}

impl ProgrammingLanguage {
    /// Detect programming language from file extension
    pub fn from_file_path(path: &str) -> Self {
        if let Some(extension) = path.split('.').next_back() {
            match extension.to_lowercase().as_str() {
                "rs" => Self::Rust,
                "py" => Self::Python,
                "ts" | "tsx" => Self::TypeScript,
                "js" | "jsx" => Self::JavaScript,
                "go" => Self::Go,
                "java" => Self::Java,
                "c" | "h" => Self::C,
                "cpp" | "cxx" | "cc" | "hpp" | "hxx" => Self::Cpp,
                _ => Self::Unknown,
            }
        } else {
            Self::Unknown
        }
    }
}

/// Represents a semantic unit found in the AST
#[derive(Debug, Clone)]
pub struct SemanticUnit {
    pub unit_type: SemanticNodeKind,
    pub identifier: Option<String>,
    pub source_text: String,
    pub start_byte: usize,
    pub end_byte: usize,
}

/// Simplified trait for language-specific AST parsing and analysis
pub trait LanguageParser: Send + Sync {
    /// Parse content into TreeSitter AST
    fn try_parse(&self, content: &str) -> Result<Tree>;

    /// Get the TreeSitter language for this parser
    fn get_language(&self) -> tree_sitter::Language;

    /// Build a semantic AST tree from a TreeSitter AST
    ///
    /// This method transforms low-level TreeSitter nodes into meaningful semantic
    /// constructs suitable for code review. Implementation should:
    /// 1. Identify semantic units (functions, classes, modules, etc.)
    /// 2. Extract metadata for each unit type
    /// 3. Build hierarchical tree ensuring complete source coverage
    /// 4. Handle language-specific constructs appropriately
    fn build_semantic_tree<'a>(
        &self,
        ast: &'a tree_sitter::Tree,
        source: &str,
    ) -> std::result::Result<
        crate::semantic_ast::SemanticTree<'a>,
        crate::semantic_ast::SemanticError,
    >;

    /// Classify TreeSitter node kinds into semantic categories  
    fn classify_node_kind(&self, node_kind: &str) -> SemanticNodeKind;

    /// Get context boundary node kinds for a given change, in priority order
    /// Default implementation provides generic boundaries that work across languages
    fn get_context_boundaries(
        &self,
        change_type: &crate::ast_diff::ASTChangeType,
        _change_node_kind: &SemanticNodeKind,
    ) -> Vec<SemanticNodeKind> {
        use crate::ast_diff::ASTChangeType;

        // Default implementation: generic boundaries that work across languages
        match change_type {
            ASTChangeType::Content => {
                // For content changes, try to find function-level context first
                vec![
                    SemanticNodeKind::Function,
                    SemanticNodeKind::Class,
                    SemanticNodeKind::SourceFile,
                ]
            }
            ASTChangeType::Structural => {
                // For structural changes, might need broader context
                vec![
                    SemanticNodeKind::ImplBlock,
                    SemanticNodeKind::Class,
                    SemanticNodeKind::Module,
                    SemanticNodeKind::SourceFile,
                ]
            }
            ASTChangeType::Rename => {
                // For renames, show the immediate container
                vec![
                    SemanticNodeKind::Function,
                    SemanticNodeKind::Struct,
                    SemanticNodeKind::Enum,
                    SemanticNodeKind::Class,
                    SemanticNodeKind::SourceFile,
                ]
            }
            ASTChangeType::Reorder => {
                // For reorders, show the parent container
                vec![
                    SemanticNodeKind::Function,
                    SemanticNodeKind::ImplBlock,
                    SemanticNodeKind::Class,
                    SemanticNodeKind::SourceFile,
                ]
            }
        }
    }

    /// Classify leaf node relevance for context expansion
    fn classify_leaf_relevance(
        &self,
        node_kind: &SemanticNodeKind,
    ) -> crate::ast_diff::RelevanceScore {
        match node_kind {
            // Essential: Core language constructs that define behavior
            SemanticNodeKind::Function
            | SemanticNodeKind::Class
            | SemanticNodeKind::Struct
            | SemanticNodeKind::Enum
            | SemanticNodeKind::Interface => crate::ast_diff::ESSENTIAL,

            // Important: Supporting constructs and signature components
            SemanticNodeKind::ImplBlock
            | SemanticNodeKind::TypeDefinition
            | SemanticNodeKind::Variable
            | SemanticNodeKind::SignatureComponent => crate::ast_diff::IMPORTANT,

            // Background: Organizational constructs
            SemanticNodeKind::Module | SemanticNodeKind::Import => crate::ast_diff::BACKGROUND,

            // Noise: Low-value constructs for context
            SemanticNodeKind::Comment
            | SemanticNodeKind::Statement
            | SemanticNodeKind::Expression => crate::ast_diff::NOISE,

            // Other cases default to background
            SemanticNodeKind::SourceFile => crate::ast_diff::BACKGROUND,
            SemanticNodeKind::Other(_) => crate::ast_diff::NOISE,
        }
    }
}
