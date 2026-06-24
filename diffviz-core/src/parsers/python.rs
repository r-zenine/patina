//! Python language parser — `PythonDescriptor` + `PythonParser` newtype wrapper.
//!
//! `PythonDescriptor` implements `LanguageDescriptor` with Python-specific kind tables.
//! `PythonParser` wraps `GenericSemanticTreeBuilder<PythonDescriptor>` and overrides
//! `get_context_boundaries` and `extract_visibility` for Python semantics.

use crate::ast_diff::ASTChangeType;
use crate::common::{LanguageParser, ProgrammingLanguage, Result, SemanticNodeKind};
use crate::parsers::descriptor::LanguageDescriptor;
use crate::parsers::generic_builder::GenericSemanticTreeBuilder;
use crate::semantic_ast::{SemanticError, SemanticTree};
use tree_sitter::{Language, Node, Tree};

// ── PythonDescriptor ──────────────────────────────────────────────────────────

pub struct PythonDescriptor;

static PYTHON_SEMANTIC_KIND_MAP: &[(&str, SemanticNodeKind)] = &[
    // ── Construction kinds ────────────────────────────────────────────────
    ("function_definition", SemanticNodeKind::Function),
    ("async_function_definition", SemanticNodeKind::Function),
    ("class_definition", SemanticNodeKind::Class),
    ("module", SemanticNodeKind::Module),
    ("import_statement", SemanticNodeKind::Import),
    ("import_from_statement", SemanticNodeKind::Import),
    ("assignment", SemanticNodeKind::Variable),
    ("augmented_assignment", SemanticNodeKind::Variable),
    // ── Classification-only: statements ──────────────────────────────────
    ("expression_statement", SemanticNodeKind::Statement),
    ("assert_statement", SemanticNodeKind::Statement),
    ("return_statement", SemanticNodeKind::Statement),
    ("pass_statement", SemanticNodeKind::Statement),
    ("break_statement", SemanticNodeKind::Statement),
    ("continue_statement", SemanticNodeKind::Statement),
    // ── Classification-only: expressions ─────────────────────────────────
    ("call", SemanticNodeKind::Expression),
    ("attribute", SemanticNodeKind::Expression),
    ("subscript", SemanticNodeKind::Expression),
    ("binary_operator", SemanticNodeKind::Expression),
    ("unary_operator", SemanticNodeKind::Expression),
    ("comparison_operator", SemanticNodeKind::Expression),
    // ── Classification-only: signature components ─────────────────────────
    ("parameters", SemanticNodeKind::SignatureComponent),
    ("parameter", SemanticNodeKind::SignatureComponent),
    ("default_parameter", SemanticNodeKind::SignatureComponent),
    ("typed_parameter", SemanticNodeKind::SignatureComponent),
    // ── Classification-only: comments ────────────────────────────────────
    ("comment", SemanticNodeKind::Comment),
    // ── Root ─────────────────────────────────────────────────────────────
    ("source_file", SemanticNodeKind::SourceFile),
];

/// Node kinds skipped during tree construction.
static PYTHON_TRIVIAL_KINDS: &[&str] = &[
    // Punctuation
    "(",
    ")",
    "[",
    "]",
    "{",
    "}",
    ",",
    ";",
    ":",
    ".",
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
    "def",
    "class",
    "import",
    "from",
    "if",
    "else",
    "elif",
    "for",
    "while",
    "with",
    "try",
    "except",
    "finally",
    "return",
    "yield",
    "pass",
    "break",
    "continue",
    "lambda",
    "and",
    "or",
    "not",
    "in",
    "is",
    "async",
    "await",
    // Literals and identifiers
    "string",
    "integer",
    "float",
    "boolean",
    "none",
    "identifier",
    // Comments
    "comment",
    "line_comment",
    // Error nodes
    "ERROR",
    "MISSING",
];

impl LanguageDescriptor for PythonDescriptor {
    fn ts_language(&self) -> Language {
        tree_sitter_python::LANGUAGE.into()
    }

    fn programming_language(&self) -> ProgrammingLanguage {
        ProgrammingLanguage::Python
    }

    fn semantic_kind_map(&self) -> &[(&'static str, SemanticNodeKind)] {
        PYTHON_SEMANTIC_KIND_MAP
    }

    fn trivial_kinds(&self) -> &[&'static str] {
        PYTHON_TRIVIAL_KINDS
    }

    fn container_body_field(&self, _kind: &str) -> Option<&'static str> {
        None
    }

    fn metadata_kind(&self) -> Option<&'static str> {
        Some("decorator")
    }

    fn extract_identifier<'a>(&self, node: Node<'a>, source: &str) -> Option<String> {
        match node.kind() {
            "assignment" | "augmented_assignment" => {
                // x = 42: left field is a pattern; only handle simple identifier
                let left = node.child_by_field_name("left")?;
                if left.kind() == "identifier" {
                    left.utf8_text(source.as_bytes()).ok().map(str::to_string)
                } else {
                    None // tuple/attribute assignments: skip
                }
            }
            _ => node
                .child_by_field_name("name")
                .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                .map(str::to_string),
        }
    }

    /// Python visibility is determined by naming convention:
    /// - `__name__` → "magic"
    /// - `_name` → "private"
    /// - `name` → "public"
    fn extract_visibility<'a>(&self, node: Node<'a>, source: &str) -> String {
        if let Some(name_node) = node.child_by_field_name("name")
            && let Ok(name) = name_node.utf8_text(source.as_bytes())
        {
            return if name.starts_with("__") && name.ends_with("__") {
                "magic".to_string()
            } else if name.starts_with('_') {
                "private".to_string()
            } else {
                "public".to_string()
            };
        }
        "public".to_string()
    }
}

// ── PythonParser (newtype wrapper) ────────────────────────────────────────────

pub struct PythonParser(GenericSemanticTreeBuilder<PythonDescriptor>);

impl PythonParser {
    pub fn new() -> Self {
        Self(GenericSemanticTreeBuilder::new(PythonDescriptor))
    }
}

impl Default for PythonParser {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageParser for PythonParser {
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

    fn get_context_boundaries(
        &self,
        _change_type: &ASTChangeType,
        change_node_kind: &SemanticNodeKind,
    ) -> Vec<SemanticNodeKind> {
        match change_node_kind {
            SemanticNodeKind::Function => vec![
                SemanticNodeKind::Class,
                SemanticNodeKind::Module,
                SemanticNodeKind::SourceFile,
            ],
            SemanticNodeKind::Class => {
                vec![SemanticNodeKind::Module, SemanticNodeKind::SourceFile]
            }
            SemanticNodeKind::Import => vec![
                SemanticNodeKind::Function,
                SemanticNodeKind::Module,
                SemanticNodeKind::SourceFile,
            ],
            SemanticNodeKind::Variable
            | SemanticNodeKind::Statement
            | SemanticNodeKind::Expression => vec![
                SemanticNodeKind::Function,
                SemanticNodeKind::Class,
                SemanticNodeKind::Module,
                SemanticNodeKind::SourceFile,
            ],
            _ => vec![
                SemanticNodeKind::Function,
                SemanticNodeKind::Class,
                SemanticNodeKind::Module,
                SemanticNodeKind::SourceFile,
            ],
        }
    }
}
