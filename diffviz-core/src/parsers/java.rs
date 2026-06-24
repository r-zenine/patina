//! Java language parser — `JavaDescriptor` + `JavaParser` newtype wrapper.
//!
//! `JavaDescriptor` implements `LanguageDescriptor` with Java-specific kind tables.
//! `JavaParser` wraps `GenericSemanticTreeBuilder<JavaDescriptor>`.

use crate::common::{LanguageParser, ProgrammingLanguage, Result, SemanticNodeKind};
use crate::parsers::descriptor::LanguageDescriptor;
use crate::parsers::generic_builder::GenericSemanticTreeBuilder;
use crate::semantic_ast::{SemanticError, SemanticTree};
use tree_sitter::{Language, Tree};

// ── JavaDescriptor ────────────────────────────────────────────────────────────

pub struct JavaDescriptor;

static JAVA_SEMANTIC_KIND_MAP: &[(&str, SemanticNodeKind)] = &[
    // ── Construction kinds ────────────────────────────────────────────────
    ("method_declaration", SemanticNodeKind::Function),
    ("constructor_declaration", SemanticNodeKind::Function),
    ("class_declaration", SemanticNodeKind::Class),
    ("interface_declaration", SemanticNodeKind::Interface),
    ("enum_declaration", SemanticNodeKind::Enum),
    ("import_declaration", SemanticNodeKind::Import),
    ("variable_declaration", SemanticNodeKind::Variable),
    ("field_declaration", SemanticNodeKind::Variable),
    // ── Classification-only: statements ──────────────────────────────────
    ("expression_statement", SemanticNodeKind::Statement),
    ("assignment_expression", SemanticNodeKind::Statement),
    // ── Classification-only: expressions ─────────────────────────────────
    ("method_invocation", SemanticNodeKind::Expression),
    ("field_access", SemanticNodeKind::Expression),
    ("binary_expression", SemanticNodeKind::Expression),
    // ── Classification-only: signature components ─────────────────────────
    ("formal_parameters", SemanticNodeKind::SignatureComponent),
    ("formal_parameter", SemanticNodeKind::SignatureComponent),
    ("type_parameters", SemanticNodeKind::SignatureComponent),
    ("type_parameter", SemanticNodeKind::SignatureComponent),
    ("throws", SemanticNodeKind::SignatureComponent),
    ("modifier", SemanticNodeKind::SignatureComponent),
    // ── Classification-only: comments ────────────────────────────────────
    ("comment", SemanticNodeKind::Comment),
    ("line_comment", SemanticNodeKind::Comment),
    ("block_comment", SemanticNodeKind::Comment),
    // ── Root ─────────────────────────────────────────────────────────────
    ("program", SemanticNodeKind::SourceFile),
];

/// Trivial syntax tokens skipped during tree construction.
///
/// Includes statement/expression constructs from the old parser's trivial list,
/// which governs tree building. These may also appear in the kind map for
/// `classify_node_kind` — overlap is intentional (same pattern as Rust).
static JAVA_TRIVIAL_KINDS: &[&str] = &[
    // Literals and identifiers
    "string_literal",
    "character_literal",
    "decimal_integer_literal",
    "hex_integer_literal",
    "octal_integer_literal",
    "binary_integer_literal",
    "decimal_floating_point_literal",
    "hex_floating_point_literal",
    "true",
    "false",
    "null",
    "identifier",
    "type_identifier",
    // Comments
    "comment",
    "line_comment",
    "block_comment",
    // Statement-level constructs (trivial for tree construction; classifiable via kind map)
    "expression_statement",
    "assignment_expression",
    "binary_expression",
    "method_invocation",
    "field_access",
    "array_access",
    "parenthesized_expression",
    "unary_expression",
    "conditional_expression",
    "block",
    "if_statement",
    "return_statement",
    "throw_statement",
    "local_variable_declaration",
    "enhanced_for_statement",
    "for_statement",
    "while_statement",
    "do_statement",
    // Type and generic constructs
    "type_arguments",
    "type_parameters",
    "wildcard",
    "generic_type",
    "array_type",
    "integral_type",
    "floating_point_type",
    "boolean_type",
    // Annotation elements (handled as metadata or skipped)
    "annotation",
    "marker_annotation",
    "annotation_argument_list",
    // Error nodes
    "ERROR",
    "MISSING",
];

impl LanguageDescriptor for JavaDescriptor {
    fn ts_language(&self) -> Language {
        tree_sitter_java::LANGUAGE.into()
    }

    fn programming_language(&self) -> ProgrammingLanguage {
        ProgrammingLanguage::Java
    }

    fn semantic_kind_map(&self) -> &[(&'static str, SemanticNodeKind)] {
        JAVA_SEMANTIC_KIND_MAP
    }

    fn trivial_kinds(&self) -> &[&'static str] {
        JAVA_TRIVIAL_KINDS
    }

    fn container_body_field(&self, _kind: &str) -> Option<&'static str> {
        None
    }

    fn metadata_kind(&self) -> Option<&'static str> {
        None
    }
}

// ── JavaParser (newtype wrapper) ──────────────────────────────────────────────

pub struct JavaParser(GenericSemanticTreeBuilder<JavaDescriptor>);

impl JavaParser {
    pub fn new() -> Self {
        Self(GenericSemanticTreeBuilder::new(JavaDescriptor))
    }
}

impl Default for JavaParser {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageParser for JavaParser {
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
