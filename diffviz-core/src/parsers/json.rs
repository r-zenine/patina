use crate::common::{ASTError, LanguageParser, Result, SemanticNodeKind};
use crate::semantic_ast::{SemanticError, SemanticTree};
use tree_sitter::{Parser, Tree};

pub struct JsonParser;

impl JsonParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for JsonParser {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageParser for JsonParser {
    fn try_parse(&self, content: &str) -> Result<Tree> {
        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_json::language())
            .map_err(|e| ASTError::TreeSitterError {
                error: format!("JSON error: {e}"),
            })?;
        parser
            .parse(content, None)
            .ok_or_else(|| ASTError::ParseError {
                message: "Failed to parse JSON".to_string(),
            })
    }

    fn get_language(&self) -> tree_sitter::Language {
        tree_sitter_json::language()
    }

    fn classify_node_kind(&self, node_kind: &str) -> SemanticNodeKind {
        SemanticNodeKind::Other(node_kind.to_string())
    }

    fn build_semantic_tree<'a>(
        &self,
        _ast: &'a Tree,
        _source: &str,
    ) -> std::result::Result<SemanticTree<'a>, SemanticError> {
        Err(SemanticError::UnsupportedLanguage(
            "JSON semantic tree building not implemented yet".to_string(),
        ))
    }
}
