//! Go language parser — `GoDescriptor` + `GoParser` newtype wrapper.
//!
//! `GoDescriptor` implements `LanguageDescriptor` with Go-specific kind tables.
//! `GoParser` wraps `GenericSemanticTreeBuilder<GoDescriptor>`.
//! Go has no visibility keywords; visibility is derived from naming convention
//! (capitalised identifier = exported/public).

use crate::common::{LanguageParser, ProgrammingLanguage, Result, SemanticNodeKind};
use crate::parsers::descriptor::LanguageDescriptor;
use crate::parsers::generic_builder::GenericSemanticTreeBuilder;
use crate::semantic_ast::{SemanticError, SemanticTree};
use tree_sitter::{Language, Node, Tree};

// ── GoDescriptor ──────────────────────────────────────────────────────────────

pub struct GoDescriptor;

static GO_SEMANTIC_KIND_MAP: &[(&str, SemanticNodeKind)] = &[
    // ── Construction kinds ────────────────────────────────────────────────
    ("function_declaration", SemanticNodeKind::Function),
    ("method_declaration", SemanticNodeKind::Function),
    ("struct_type", SemanticNodeKind::Struct),
    ("interface_type", SemanticNodeKind::Interface),
    ("package_clause", SemanticNodeKind::Module),
    ("import_declaration", SemanticNodeKind::Import),
    ("var_declaration", SemanticNodeKind::Variable),
    ("const_declaration", SemanticNodeKind::Variable),
    ("short_var_declaration", SemanticNodeKind::Variable),
    // ── Classification-only: statements ──────────────────────────────────
    ("expression_statement", SemanticNodeKind::Statement),
    ("assignment_statement", SemanticNodeKind::Statement),
    ("inc_statement", SemanticNodeKind::Statement),
    ("dec_statement", SemanticNodeKind::Statement),
    // ── Classification-only: expressions ─────────────────────────────────
    ("call_expression", SemanticNodeKind::Expression),
    ("selector_expression", SemanticNodeKind::Expression),
    ("index_expression", SemanticNodeKind::Expression),
    ("binary_expression", SemanticNodeKind::Expression),
    // ── Classification-only: signature components ─────────────────────────
    ("parameter_list", SemanticNodeKind::SignatureComponent),
    (
        "parameter_declaration",
        SemanticNodeKind::SignatureComponent,
    ),
    ("result", SemanticNodeKind::SignatureComponent),
    // ── Classification-only: comments ────────────────────────────────────
    ("comment", SemanticNodeKind::Comment),
    // ── Root ─────────────────────────────────────────────────────────────
    ("source_file", SemanticNodeKind::SourceFile),
];

static GO_TRIVIAL_KINDS: &[&str] = &[
    // Literals and identifiers
    "string_literal",
    "int_literal",
    "float_literal",
    "identifier",
    "field_identifier",
    "type_identifier",
    "package_identifier",
    // Comments
    "comment",
    "line_comment",
    "block_comment",
    // Error nodes
    "ERROR",
    "MISSING",
];

impl LanguageDescriptor for GoDescriptor {
    fn ts_language(&self) -> Language {
        tree_sitter_go::LANGUAGE.into()
    }

    fn programming_language(&self) -> ProgrammingLanguage {
        ProgrammingLanguage::Go
    }

    fn semantic_kind_map(&self) -> &[(&'static str, SemanticNodeKind)] {
        GO_SEMANTIC_KIND_MAP
    }

    fn trivial_kinds(&self) -> &[&'static str] {
        GO_TRIVIAL_KINDS
    }

    fn container_body_field(&self, _kind: &str) -> Option<&'static str> {
        None
    }

    fn metadata_kind(&self) -> Option<&'static str> {
        None
    }

    fn extract_identifier<'a>(&self, node: Node<'a>, source: &str) -> Option<String> {
        match node.kind() {
            "short_var_declaration" => {
                // := left side is expression_list — take first identifier child
                let left = node.child_by_field_name("left")?;
                let mut cursor = left.walk();
                left.children(&mut cursor)
                    .find(|c| c.kind() == "identifier")
                    .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                    .map(str::to_string)
            }
            "var_declaration" | "const_declaration" | "type_declaration" => {
                // Container nodes: no "name" field — drill into first named child (spec)
                let mut cursor = node.walk();
                node.named_children(&mut cursor)
                    .next()
                    .and_then(|spec| spec.child_by_field_name("name"))
                    .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                    .map(str::to_string)
            }
            _ => node
                .child_by_field_name("name")
                .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                .map(str::to_string),
        }
    }

    /// Go visibility is determined by naming convention: capitalised = exported (public).
    fn extract_visibility<'a>(&self, node: Node<'a>, source: &str) -> String {
        if let Some(name_node) = node.child_by_field_name("name")
            && let Ok(name) = name_node.utf8_text(source.as_bytes())
        {
            return if name.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
                "public".to_string()
            } else {
                "private".to_string()
            };
        }
        "private".to_string()
    }
}

// ── GoParser (newtype wrapper) ────────────────────────────────────────────────

pub struct GoParser(GenericSemanticTreeBuilder<GoDescriptor>);

impl GoParser {
    pub fn new() -> Self {
        Self(GenericSemanticTreeBuilder::new(GoDescriptor))
    }
}

impl Default for GoParser {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageParser for GoParser {
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
}
