//! C language parser — `CDescriptor` + `CParser` newtype wrapper.
//!
//! `CDescriptor` implements `LanguageDescriptor` with C-specific kind tables.
//! `CParser` wraps `GenericSemanticTreeBuilder<CDescriptor>`.

use crate::common::{LanguageParser, ProgrammingLanguage, Result, SemanticNodeKind};
use crate::parsers::descriptor::LanguageDescriptor;
use crate::parsers::generic_builder::GenericSemanticTreeBuilder;
use crate::semantic_ast::{SemanticError, SemanticTree};
use tree_sitter::{Language, Tree};

// ── CDescriptor ───────────────────────────────────────────────────────────────

pub struct CDescriptor;

static C_SEMANTIC_KIND_MAP: &[(&str, SemanticNodeKind)] = &[
    // ── Construction kinds ────────────────────────────────────────────────
    ("function_definition", SemanticNodeKind::Function),
    ("struct_specifier", SemanticNodeKind::Struct),
    ("enum_specifier", SemanticNodeKind::Enum),
    ("preproc_include", SemanticNodeKind::Import),
    ("preproc_def", SemanticNodeKind::Import),
    ("declaration", SemanticNodeKind::Variable),
    ("init_declarator", SemanticNodeKind::Variable),
    // ── Classification-only: statements ──────────────────────────────────
    ("expression_statement", SemanticNodeKind::Statement),
    ("assignment_expression", SemanticNodeKind::Statement),
    // ── Classification-only: expressions ─────────────────────────────────
    ("call_expression", SemanticNodeKind::Expression),
    ("field_expression", SemanticNodeKind::Expression),
    ("binary_expression", SemanticNodeKind::Expression),
    // ── Classification-only: signature components ─────────────────────────
    ("parameter_list", SemanticNodeKind::SignatureComponent),
    (
        "parameter_declaration",
        SemanticNodeKind::SignatureComponent,
    ),
    ("pointer_declarator", SemanticNodeKind::SignatureComponent),
    // ── Classification-only: type definitions ────────────────────────────
    ("typedef_declaration", SemanticNodeKind::TypeDefinition),
    // ── Classification-only: comments ────────────────────────────────────
    ("comment", SemanticNodeKind::Comment),
    // ── Root ─────────────────────────────────────────────────────────────
    ("translation_unit", SemanticNodeKind::SourceFile),
];

static C_TRIVIAL_KINDS: &[&str] = &[
    // Punctuation (note: ";" is intentionally absent — C uses it as a statement terminator)
    "(",
    ")",
    "[",
    "]",
    "{",
    "}",
    ",",
    ":",
    "::",
    ".",
    "->",
    "?",
    "!",
    "#",
    "@",
    "$",
    "%",
    "^",
    "&",
    "*",
    "-",
    "=",
    "+",
    "|",
    "\\",
    "/",
    "<",
    ">",
    // Keywords
    "if",
    "else",
    "for",
    "while",
    "do",
    "switch",
    "case",
    "default",
    "break",
    "continue",
    "return",
    "goto",
    "sizeof",
    "typedef",
    "struct",
    "union",
    "enum",
    "static",
    "extern",
    "auto",
    "register",
    "const",
    "volatile",
    "inline",
    "restrict",
    // Type specifiers
    "void",
    "char",
    "short",
    "int",
    "long",
    "float",
    "double",
    "signed",
    "unsigned",
    // Literals and identifiers
    "string_literal",
    "char_literal",
    "number_literal",
    "identifier",
    "field_identifier",
    "type_identifier",
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
    "init_declarator",
    "field_declaration",
    "parameter_declaration",
    "pointer_declarator",
    "function_declarator",
    "field_declaration_list",
    "parameter_list",
    "enumerator_list",
    "enumerator",
    "system_lib_string",
    // Error nodes
    "ERROR",
    "MISSING",
];

impl LanguageDescriptor for CDescriptor {
    fn ts_language(&self) -> Language {
        tree_sitter_c::language()
    }

    fn programming_language(&self) -> ProgrammingLanguage {
        ProgrammingLanguage::C
    }

    fn semantic_kind_map(&self) -> &[(&'static str, SemanticNodeKind)] {
        C_SEMANTIC_KIND_MAP
    }

    fn trivial_kinds(&self) -> &[&'static str] {
        C_TRIVIAL_KINDS
    }

    fn container_body_field(&self, _kind: &str) -> Option<&'static str> {
        None
    }

    fn metadata_kind(&self) -> Option<&'static str> {
        None
    }
}

// ── CParser (newtype wrapper) ─────────────────────────────────────────────────

pub struct CParser(GenericSemanticTreeBuilder<CDescriptor>);

impl CParser {
    pub fn new() -> Self {
        Self(GenericSemanticTreeBuilder::new(CDescriptor))
    }
}

impl Default for CParser {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageParser for CParser {
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
