//! TypeScript language parser — `TypeScriptDescriptor` + `TypeScriptParser` newtype wrapper.
//!
//! `TypeScriptDescriptor` implements `LanguageDescriptor` with TypeScript-specific kind tables.
//! `TypeScriptParser` wraps `GenericSemanticTreeBuilder<TypeScriptDescriptor>`.

use crate::common::{LanguageParser, ProgrammingLanguage, Result, SemanticNodeKind};
use crate::parsers::descriptor::LanguageDescriptor;
use crate::parsers::generic_builder::GenericSemanticTreeBuilder;
use crate::semantic_ast::{SemanticError, SemanticTree};
use tree_sitter::{Language, Node, Tree};

// ── TypeScriptDescriptor ──────────────────────────────────────────────────────

pub struct TypeScriptDescriptor;

static TYPESCRIPT_SEMANTIC_KIND_MAP: &[(&str, SemanticNodeKind)] = &[
    // ── Construction kinds ────────────────────────────────────────────────
    ("function_declaration", SemanticNodeKind::Function),
    ("method_definition", SemanticNodeKind::Function),
    ("arrow_function", SemanticNodeKind::Function),
    ("class_declaration", SemanticNodeKind::Class),
    ("interface_declaration", SemanticNodeKind::Interface),
    ("enum_declaration", SemanticNodeKind::Enum),
    ("import_statement", SemanticNodeKind::Import),
    ("export_statement", SemanticNodeKind::Import),
    ("variable_declaration", SemanticNodeKind::Variable),
    ("lexical_declaration", SemanticNodeKind::Variable),
    // ── Classification-only: statements ──────────────────────────────────
    ("expression_statement", SemanticNodeKind::Statement),
    ("assignment_expression", SemanticNodeKind::Statement),
    // ── Classification-only: expressions ─────────────────────────────────
    ("call_expression", SemanticNodeKind::Expression),
    ("member_expression", SemanticNodeKind::Expression),
    ("binary_expression", SemanticNodeKind::Expression),
    // ── Classification-only: type definitions ────────────────────────────
    ("type_alias_declaration", SemanticNodeKind::TypeDefinition),
    // ── Classification-only: signature components ─────────────────────────
    ("formal_parameters", SemanticNodeKind::SignatureComponent),
    ("rest_parameter", SemanticNodeKind::SignatureComponent),
    ("type_parameters", SemanticNodeKind::SignatureComponent),
    ("type_parameter", SemanticNodeKind::SignatureComponent),
    ("type_annotation", SemanticNodeKind::SignatureComponent),
    ("return_type", SemanticNodeKind::SignatureComponent),
    // ── Classification-only: comments ────────────────────────────────────
    ("comment", SemanticNodeKind::Comment),
    // ── Root ─────────────────────────────────────────────────────────────
    ("program", SemanticNodeKind::SourceFile),
];

static TYPESCRIPT_TRIVIAL_KINDS: &[&str] = &[
    // Literals and identifiers
    "string",
    "number",
    "boolean",
    "null",
    "undefined",
    "identifier",
    "property_identifier",
    "type_identifier",
    // Comments
    "comment",
    "line_comment",
    "block_comment",
    // Error nodes
    "ERROR",
    "MISSING",
];

impl LanguageDescriptor for TypeScriptDescriptor {
    fn ts_language(&self) -> Language {
        tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
    }

    fn programming_language(&self) -> ProgrammingLanguage {
        ProgrammingLanguage::TypeScript
    }

    fn semantic_kind_map(&self) -> &[(&'static str, SemanticNodeKind)] {
        TYPESCRIPT_SEMANTIC_KIND_MAP
    }

    fn trivial_kinds(&self) -> &[&'static str] {
        TYPESCRIPT_TRIVIAL_KINDS
    }

    fn container_body_field(&self, _kind: &str) -> Option<&'static str> {
        None
    }

    fn metadata_kind(&self) -> Option<&'static str> {
        Some("decorator")
    }

    fn extract_identifier<'a>(&self, node: Node<'a>, source: &str) -> Option<String> {
        match node.kind() {
            "variable_declaration" | "lexical_declaration" => {
                // const/let/var: container has no "name"; find first variable_declarator → "name"
                let mut cursor = node.walk();
                node.named_children(&mut cursor)
                    .find(|c| c.kind() == "variable_declarator")
                    .and_then(|d| d.child_by_field_name("name"))
                    .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                    .map(str::to_string)
            }
            _ => node
                .child_by_field_name("name")
                .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                .map(str::to_string),
        }
    }
}

// ── TypeScriptParser (newtype wrapper) ────────────────────────────────────────

pub struct TypeScriptParser(GenericSemanticTreeBuilder<TypeScriptDescriptor>);

impl TypeScriptParser {
    pub fn new() -> Self {
        Self(GenericSemanticTreeBuilder::new(TypeScriptDescriptor))
    }
}

impl Default for TypeScriptParser {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageParser for TypeScriptParser {
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
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typescript_parser_creation() {
        let parser = TypeScriptParser::new();
        let _language = parser.get_language();
    }

    #[test]
    fn test_typescript_parse_and_build_tree() {
        let parser = TypeScriptParser::new();
        let code = r#"
export interface User {
    id: string;
    name: string;
}

export class UserService {
    getUser(id: string): User {
        return { id, name: "test" };
    }
}
"#;
        let tree = parser.try_parse(code).expect("parse should succeed");
        let semantic_tree = parser
            .build_semantic_tree(&tree, code)
            .expect("build_semantic_tree should succeed for TypeScript");
        assert!(!semantic_tree.root.children.is_empty());
    }

    #[test]
    fn test_typescript_classify_node_kinds() {
        let parser = TypeScriptParser::new();
        assert!(matches!(
            parser.classify_node_kind("function_declaration"),
            SemanticNodeKind::Function
        ));
        assert!(matches!(
            parser.classify_node_kind("class_declaration"),
            SemanticNodeKind::Class
        ));
        assert!(matches!(
            parser.classify_node_kind("interface_declaration"),
            SemanticNodeKind::Interface
        ));
        assert!(matches!(
            parser.classify_node_kind("enum_declaration"),
            SemanticNodeKind::Enum
        ));
        assert!(matches!(
            parser.classify_node_kind("unknown_kind"),
            SemanticNodeKind::Other(_)
        ));
    }
}
