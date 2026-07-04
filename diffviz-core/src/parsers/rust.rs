//! Rust language parser — `RustDescriptor` + `RustParser` newtype wrapper.
//!
//! `RustDescriptor` implements `LanguageDescriptor` with Rust-specific kind tables.
//! `RustParser` wraps `GenericSemanticTreeBuilder<RustDescriptor>` and overrides
//! only `get_context_boundaries` with the Rust-specific boundary logic.

use crate::ast_diff::ASTChangeType;
use crate::common::{LanguageParser, ProgrammingLanguage, Result, SemanticNodeKind};
use crate::parsers::descriptor::LanguageDescriptor;
use crate::parsers::generic_builder::GenericSemanticTreeBuilder;
use crate::semantic_ast::{SemanticError, SemanticTree};
use tree_sitter::{Language, Node, Tree};

// ── RustDescriptor ────────────────────────────────────────────────────────────

/// Language descriptor for Rust — static kind tables consumed by
/// `GenericSemanticTreeBuilder`.
pub struct RustDescriptor;

/// Maps tree-sitter node kinds to `SemanticNodeKind`.
///
/// Serves two purposes:
/// - Tree construction: Function/Struct/Enum/ImplBlock/Module/Import/Variable
///   kinds produce typed `SemanticNode`s in the semantic tree.
/// - Classification: all other kinds (Statement, Expression, SignatureComponent,
///   Comment, SourceFile) are returned by `classify_node_kind` only.
///
/// Note: `visibility_modifier` and `function_modifiers` appear here for
/// classification (→ `SignatureComponent`) and in `RUST_TRIVIAL_KINDS` to be
/// skipped during tree construction.  Trivial-kinds governs tree building;
/// this map governs `classify_node_kind`.
static RUST_SEMANTIC_KIND_MAP: &[(&str, SemanticNodeKind)] = &[
    // ── Construction kinds ────────────────────────────────────────────────
    ("function_item", SemanticNodeKind::Function),
    ("struct_item", SemanticNodeKind::Struct),
    ("enum_item", SemanticNodeKind::Enum),
    ("impl_item", SemanticNodeKind::ImplBlock),
    ("trait_item", SemanticNodeKind::Interface),
    ("mod_item", SemanticNodeKind::Module),
    ("use_declaration", SemanticNodeKind::Import),
    ("const_item", SemanticNodeKind::Variable),
    ("static_item", SemanticNodeKind::Variable),
    ("let_declaration", SemanticNodeKind::Variable),
    // ── Classification-only: statements ──────────────────────────────────
    ("expression_statement", SemanticNodeKind::Statement),
    ("assignment_expression", SemanticNodeKind::Statement),
    // ── Classification-only: expressions ─────────────────────────────────
    ("call_expression", SemanticNodeKind::Expression),
    ("method_call_expression", SemanticNodeKind::Expression),
    ("binary_expression", SemanticNodeKind::Expression),
    ("unary_expression", SemanticNodeKind::Expression),
    ("field_expression", SemanticNodeKind::Expression),
    ("index_expression", SemanticNodeKind::Expression),
    // ── Classification-only: type definitions ────────────────────────────
    ("type_alias", SemanticNodeKind::TypeDefinition),
    // ── Classification-only: signature components ─────────────────────────
    ("visibility_modifier", SemanticNodeKind::SignatureComponent),
    ("function_modifiers", SemanticNodeKind::SignatureComponent),
    ("parameters", SemanticNodeKind::SignatureComponent),
    ("return_type", SemanticNodeKind::SignatureComponent),
    ("type_parameters", SemanticNodeKind::SignatureComponent),
    ("generic_type", SemanticNodeKind::SignatureComponent),
    ("type_parameter", SemanticNodeKind::SignatureComponent),
    // ── Classification-only: comments ────────────────────────────────────
    ("line_comment", SemanticNodeKind::Comment),
    ("block_comment", SemanticNodeKind::Comment),
    // ── Root ─────────────────────────────────────────────────────────────
    ("source_file", SemanticNodeKind::SourceFile),
];

/// Node kinds with no semantic value — skipped during tree construction.
///
/// Intentional overlaps with `RUST_SEMANTIC_KIND_MAP`:
/// - `visibility_modifier`, `function_modifiers` → skipped in the tree but
///   classified as `SignatureComponent` by `classify_node_kind`.
/// - `doc_comment` → skipped so that `find_units_touching_range_recursive`
///   returns the enclosing function, not the comment node.
static RUST_TRIVIAL_KINDS: &[&str] = &[
    // Visibility / modifiers — skipped in tree, classifiable via kind map
    "visibility_modifier",
    "function_modifiers",
    // Literals
    "string_literal",
    "raw_string_literal",
    "integer_literal",
    "float_literal",
    "boolean_literal",
    "char_literal",
    // Identifiers
    "identifier",
    "field_identifier",
    "type_identifier",
    // Type tokens
    "primitive_type",
    "reference_type",
    "pointer_type",
    "array_type",
    "tuple_type",
    // Expression components
    "binary_operator",
    "unary_operator",
    "assignment_operator",
    "compound_assignment_expr",
    "range_expression",
    // Comments (including doc comments) — kept trivial so range queries find
    // the enclosing function rather than the doc comment node
    "line_comment",
    "block_comment",
    "doc_comment",
    // Error nodes
    "ERROR",
    "MISSING",
];

impl LanguageDescriptor for RustDescriptor {
    fn ts_language(&self) -> Language {
        tree_sitter_rust::LANGUAGE.into()
    }

    fn programming_language(&self) -> ProgrammingLanguage {
        ProgrammingLanguage::Rust
    }

    fn semantic_kind_map(&self) -> &[(&'static str, SemanticNodeKind)] {
        RUST_SEMANTIC_KIND_MAP
    }

    fn trivial_kinds(&self) -> &[&'static str] {
        RUST_TRIVIAL_KINDS
    }

    fn container_body_field(&self, kind: &str) -> Option<&'static str> {
        match kind {
            "mod_item" | "impl_item" => Some("body"),
            _ => None,
        }
    }

    fn metadata_kind(&self) -> Option<&'static str> {
        Some("attribute_item")
    }

    // `extract_visibility` uses the default: scan `node.children` for a
    // `"visibility_modifier"` child.  In the Rust grammar, visibility modifiers
    // are direct children of the item node (function_item, struct_item, …),
    // so the default is correct.

    // `collect_metadata` uses the default: scan backwards through `parent.children`
    // collecting `"attribute_item"` siblings immediately preceding the node.

    fn extract_identifier<'a>(&self, node: Node<'a>, source: &str) -> Option<String> {
        match node.kind() {
            "let_declaration" => {
                let pattern = node.child_by_field_name("pattern")?;
                match pattern.kind() {
                    "identifier" => pattern
                        .utf8_text(source.as_bytes())
                        .ok()
                        .map(str::to_string),
                    "mut_pattern" => {
                        // mut_pattern has no named fields — walk children for the identifier
                        let mut cursor = pattern.walk();
                        pattern
                            .children(&mut cursor)
                            .find(|c| c.kind() == "identifier")
                            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                            .map(str::to_string)
                    }
                    _ => None, // tuple/struct patterns: multiple names, skip
                }
            }
            _ => node
                .child_by_field_name("name")
                .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                .map(str::to_string),
        }
    }
}

// ── RustParser (newtype wrapper) ──────────────────────────────────────────────

/// Rust language parser.
///
/// Thin newtype over `GenericSemanticTreeBuilder<RustDescriptor>`.
/// Delegates all `LanguageParser` methods except `get_context_boundaries`,
/// which carries Rust-specific boundary semantics.
pub struct RustParser(GenericSemanticTreeBuilder<RustDescriptor>);

impl RustParser {
    pub fn new() -> Self {
        Self(GenericSemanticTreeBuilder::new(RustDescriptor))
    }
}

impl Default for RustParser {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageParser for RustParser {
    fn try_parse(&self, content: &str) -> Result<Tree> {
        self.0.try_parse(content)
    }

    fn get_language(&self) -> Language {
        self.0.get_language()
    }

    fn build_semantic_tree<'a>(
        &self,
        ast: &'a Tree,
        source: &str,
    ) -> std::result::Result<SemanticTree<'a>, SemanticError> {
        self.0.build_semantic_tree(ast, source)
    }

    fn classify_node_kind(&self, node_kind: &str) -> SemanticNodeKind {
        self.0.classify_node_kind(node_kind)
    }

    fn extract_identifier(&self, node: tree_sitter::Node, source: &str) -> Option<String> {
        self.0.extract_identifier(node, source)
    }

    fn get_context_boundaries(
        &self,
        change_type: &ASTChangeType,
        change_node_kind: &SemanticNodeKind,
    ) -> Vec<SemanticNodeKind> {
        match (change_type, change_node_kind) {
            (_, SemanticNodeKind::Function) => vec![
                SemanticNodeKind::ImplBlock,
                SemanticNodeKind::Struct,
                SemanticNodeKind::Enum,
                SemanticNodeKind::Module,
                SemanticNodeKind::SourceFile,
            ],
            (_, SemanticNodeKind::Struct | SemanticNodeKind::Enum) => {
                vec![SemanticNodeKind::Module, SemanticNodeKind::SourceFile]
            }
            (_, SemanticNodeKind::ImplBlock) => vec![
                SemanticNodeKind::Struct,
                SemanticNodeKind::Enum,
                SemanticNodeKind::Module,
                SemanticNodeKind::SourceFile,
            ],
            (_, SemanticNodeKind::Interface) => {
                vec![SemanticNodeKind::Module, SemanticNodeKind::SourceFile]
            }
            (_, SemanticNodeKind::Import) => vec![
                SemanticNodeKind::Function,
                SemanticNodeKind::Module,
                SemanticNodeKind::SourceFile,
            ],
            (
                _,
                SemanticNodeKind::Variable
                | SemanticNodeKind::Statement
                | SemanticNodeKind::Expression,
            ) => vec![
                SemanticNodeKind::Function,
                SemanticNodeKind::ImplBlock,
                SemanticNodeKind::Module,
                SemanticNodeKind::SourceFile,
            ],
            (_, SemanticNodeKind::Module) => {
                vec![SemanticNodeKind::Module, SemanticNodeKind::SourceFile]
            }
            (_, SemanticNodeKind::TypeDefinition) => {
                vec![SemanticNodeKind::Module, SemanticNodeKind::SourceFile]
            }
            _ => vec![
                SemanticNodeKind::Function,
                SemanticNodeKind::ImplBlock,
                SemanticNodeKind::Struct,
                SemanticNodeKind::Module,
                SemanticNodeKind::SourceFile,
            ],
        }
    }
}

// The default `get_context_boundaries` implementation in `LanguageParser` is a
// no-op (returns empty). `RustParser` overrides it with Rust-specific logic, so
// we must NOT delegate to `self.0` for that method — the delegation above is
// intentionally limited to the four methods that have no language-specific logic.

// ── Unused node helper — kept for tree-sitter traversal parity ───────────────

/// Walk `node.children()` looking for `keyword` (including inside
/// `function_modifiers` children).
fn _node_has_keyword(node: Node<'_>, keyword: &str) -> bool {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == keyword {
            return true;
        }
        if child.kind() == "function_modifiers" && _node_has_keyword(child, keyword) {
            return true;
        }
    }
    false
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ASTChangeType;

    #[test]
    fn test_rust_parser_creation() {
        let parser = RustParser::new();
        let _language = parser.get_language();
    }

    #[test]
    fn test_classify_node_kinds() {
        let parser = RustParser::new();

        assert!(matches!(
            parser.classify_node_kind("function_item"),
            SemanticNodeKind::Function
        ));
        assert!(matches!(
            parser.classify_node_kind("struct_item"),
            SemanticNodeKind::Struct
        ));
        assert!(matches!(
            parser.classify_node_kind("enum_item"),
            SemanticNodeKind::Enum
        ));
        assert!(matches!(
            parser.classify_node_kind("impl_item"),
            SemanticNodeKind::ImplBlock
        ));
        assert!(matches!(
            parser.classify_node_kind("unknown"),
            SemanticNodeKind::Other(_)
        ));
    }

    #[test]
    fn test_parse_simple_rust() {
        let parser = RustParser::new();
        let code = "fn main() { println!(\"hello\"); }";

        let tree = parser.try_parse(code).unwrap();
        let root = tree.root_node();

        assert_eq!(root.kind(), "source_file");
        assert!(root.child_count() > 0);
    }

    #[test]
    fn test_context_boundaries() {
        let parser = RustParser::new();

        let boundaries =
            parser.get_context_boundaries(&ASTChangeType::Content, &SemanticNodeKind::Function);

        assert!(boundaries.contains(&SemanticNodeKind::ImplBlock));
        assert!(boundaries.contains(&SemanticNodeKind::Module));

        let struct_boundaries =
            parser.get_context_boundaries(&ASTChangeType::Structural, &SemanticNodeKind::Struct);

        assert!(struct_boundaries.contains(&SemanticNodeKind::Module));
        assert!(struct_boundaries.contains(&SemanticNodeKind::SourceFile));
    }
}
