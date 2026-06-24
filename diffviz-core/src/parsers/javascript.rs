//! JavaScript language parser — `JavaScriptDescriptor` + `JavaScriptParser` newtype wrapper.
//!
//! `JavaScriptDescriptor` implements `LanguageDescriptor` with JavaScript-specific kind tables.
//! `JavaScriptParser` wraps `GenericSemanticTreeBuilder<JavaScriptDescriptor>`.
//! Promoted from stub to full descriptor-based parser.

use crate::common::{LanguageParser, ProgrammingLanguage, Result, SemanticNodeKind};
use crate::parsers::descriptor::LanguageDescriptor;
use crate::parsers::generic_builder::GenericSemanticTreeBuilder;
use crate::semantic_ast::{SemanticError, SemanticTree};
use tree_sitter::{Language, Tree};

// ── JavaScriptDescriptor ──────────────────────────────────────────────────────

pub struct JavaScriptDescriptor;

static JAVASCRIPT_SEMANTIC_KIND_MAP: &[(&str, SemanticNodeKind)] = &[
    // ── Construction kinds ────────────────────────────────────────────────
    ("function_declaration", SemanticNodeKind::Function),
    ("method_definition", SemanticNodeKind::Function),
    ("arrow_function", SemanticNodeKind::Function),
    ("class_declaration", SemanticNodeKind::Class),
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
    // ── Classification-only: signature components ─────────────────────────
    ("formal_parameters", SemanticNodeKind::SignatureComponent),
    ("rest_parameter", SemanticNodeKind::SignatureComponent),
    // ── Classification-only: comments ────────────────────────────────────
    ("comment", SemanticNodeKind::Comment),
    // ── Root ─────────────────────────────────────────────────────────────
    ("program", SemanticNodeKind::SourceFile),
];

static JAVASCRIPT_TRIVIAL_KINDS: &[&str] = &[
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
    "=>",
    // Keywords
    "function",
    "class",
    "import",
    "export",
    "const",
    "let",
    "var",
    "if",
    "else",
    "for",
    "while",
    "do",
    "switch",
    "case",
    "default",
    "try",
    "catch",
    "finally",
    "return",
    "yield",
    "break",
    "continue",
    "async",
    "await",
    "new",
    "delete",
    "typeof",
    "instanceof",
    "void",
    "in",
    "of",
    "throw",
    // Literals and identifiers
    "string",
    "number",
    "regex",
    "template_string",
    "true",
    "false",
    "null",
    "undefined",
    "identifier",
    "property_identifier",
    // Comments
    "comment",
    "line_comment",
    "block_comment",
    // Error nodes
    "ERROR",
    "MISSING",
];

impl LanguageDescriptor for JavaScriptDescriptor {
    fn ts_language(&self) -> Language {
        tree_sitter_javascript::LANGUAGE.into()
    }

    fn programming_language(&self) -> ProgrammingLanguage {
        ProgrammingLanguage::JavaScript
    }

    fn semantic_kind_map(&self) -> &[(&'static str, SemanticNodeKind)] {
        JAVASCRIPT_SEMANTIC_KIND_MAP
    }

    fn trivial_kinds(&self) -> &[&'static str] {
        JAVASCRIPT_TRIVIAL_KINDS
    }

    fn container_body_field(&self, _kind: &str) -> Option<&'static str> {
        None
    }

    fn metadata_kind(&self) -> Option<&'static str> {
        Some("decorator")
    }
}

// ── JavaScriptParser (newtype wrapper) ────────────────────────────────────────

pub struct JavaScriptParser(GenericSemanticTreeBuilder<JavaScriptDescriptor>);

impl JavaScriptParser {
    pub fn new() -> Self {
        Self(GenericSemanticTreeBuilder::new(JavaScriptDescriptor))
    }
}

impl Default for JavaScriptParser {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageParser for JavaScriptParser {
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
    fn test_javascript_parser_creation() {
        let parser = JavaScriptParser::new();
        let _language = parser.get_language();
    }

    #[test]
    fn test_javascript_parse_and_build_tree() {
        let parser = JavaScriptParser::new();
        let code = r#"
function MessageQueue() {
    this.config = {
        maxQueues: 100,
        enableLogging: true
    };
}

MessageQueue.prototype.createQueue = function(name) {
    return new Queue(name);
};
"#;
        let tree = parser.try_parse(code).expect("parse should succeed");
        let semantic_tree = parser
            .build_semantic_tree(&tree, code)
            .expect("build_semantic_tree must not return UnsupportedLanguage for JavaScript");
        assert!(!semantic_tree.root.children.is_empty());
    }

    #[test]
    fn test_javascript_classify_node_kinds() {
        let parser = JavaScriptParser::new();
        assert!(matches!(
            parser.classify_node_kind("function_declaration"),
            SemanticNodeKind::Function
        ));
        assert!(matches!(
            parser.classify_node_kind("class_declaration"),
            SemanticNodeKind::Class
        ));
        assert!(matches!(
            parser.classify_node_kind("program"),
            SemanticNodeKind::SourceFile
        ));
        assert!(matches!(
            parser.classify_node_kind("unknown_kind"),
            SemanticNodeKind::Other(_)
        ));
    }
}
