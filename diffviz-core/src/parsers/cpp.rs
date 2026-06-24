//! C++ language parser — `CppDescriptor` + `CppParser` newtype wrapper.
//!
//! `CppDescriptor` implements `LanguageDescriptor` with C++-specific kind tables.
//! `CppParser` wraps `GenericSemanticTreeBuilder<CppDescriptor>`.

use crate::common::{LanguageParser, ProgrammingLanguage, Result, SemanticNodeKind};
use crate::parsers::descriptor::LanguageDescriptor;
use crate::parsers::generic_builder::GenericSemanticTreeBuilder;
use crate::semantic_ast::{SemanticError, SemanticTree};
use tree_sitter::{Language, Tree};

// ── CppDescriptor ─────────────────────────────────────────────────────────────

pub struct CppDescriptor;

static CPP_SEMANTIC_KIND_MAP: &[(&str, SemanticNodeKind)] = &[
    // ── Construction kinds ────────────────────────────────────────────────
    ("function_definition", SemanticNodeKind::Function),
    ("class_specifier", SemanticNodeKind::Class),
    // ── Classification-only: signature components ─────────────────────────
    ("parameter_list", SemanticNodeKind::SignatureComponent),
    (
        "parameter_declaration",
        SemanticNodeKind::SignatureComponent,
    ),
    (
        "template_parameter_list",
        SemanticNodeKind::SignatureComponent,
    ),
    ("template_parameter", SemanticNodeKind::SignatureComponent),
    ("pointer_declarator", SemanticNodeKind::SignatureComponent),
    ("reference_declarator", SemanticNodeKind::SignatureComponent),
    // ── Root ─────────────────────────────────────────────────────────────
    ("translation_unit", SemanticNodeKind::SourceFile),
];

static CPP_TRIVIAL_KINDS: &[&str] = &[
    // Literals and identifiers
    "string_literal",
    "char_literal",
    "number_literal",
    "raw_string_literal",
    "identifier",
    "field_identifier",
    "type_identifier",
    "namespace_identifier",
    "template_type",
    "template_function",
    "primitive_type",
    // Comments and preprocessor
    "comment",
    "preproc_directive",
    // Statement-level constructs (trivial for tree construction)
    "expression_statement",
    "assignment_expression",
    "binary_expression",
    "call_expression",
    "field_expression",
    "parenthesized_expression",
    "unary_expression",
    "conditional_expression",
    "compound_statement",
    "if_statement",
    "return_statement",
    "declaration",
    "parameter_declaration",
    "pointer_declarator",
    "reference_declarator",
    "function_declarator",
    "parameter_list",
    "argument_list",
    "field_declaration_list",
    "enumerator_list",
    "enumerator",
    "access_specifier",
    "base_class_clause",
    "initializer_list",
    "template_parameter_list",
    "template_argument_list",
    // Error nodes
    "ERROR",
    "MISSING",
];

impl LanguageDescriptor for CppDescriptor {
    fn ts_language(&self) -> Language {
        tree_sitter_cpp::LANGUAGE.into()
    }

    fn programming_language(&self) -> ProgrammingLanguage {
        ProgrammingLanguage::Cpp
    }

    fn semantic_kind_map(&self) -> &[(&'static str, SemanticNodeKind)] {
        CPP_SEMANTIC_KIND_MAP
    }

    fn trivial_kinds(&self) -> &[&'static str] {
        CPP_TRIVIAL_KINDS
    }

    fn container_body_field(&self, _kind: &str) -> Option<&'static str> {
        None
    }

    fn metadata_kind(&self) -> Option<&'static str> {
        None
    }
}

// ── CppParser (newtype wrapper) ───────────────────────────────────────────────

pub struct CppParser(GenericSemanticTreeBuilder<CppDescriptor>);

impl CppParser {
    pub fn new() -> Self {
        Self(GenericSemanticTreeBuilder::new(CppDescriptor))
    }
}

impl Default for CppParser {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageParser for CppParser {
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
