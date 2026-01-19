use crate::common::{ASTError, LanguageParser, Result, SemanticNodeKind};
use crate::semantic_ast::{SemanticError, SemanticNode, SemanticSimilarity, SemanticTree};
use tree_sitter::{Parser, Tree};

pub struct TomlParser;

impl TomlParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TomlParser {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageParser for TomlParser {
    fn try_parse(&self, content: &str) -> Result<Tree> {
        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_toml::language())
            .map_err(|e| ASTError::TreeSitterError {
                error: format!("TOML error: {e}"),
            })?;
        parser
            .parse(content, None)
            .ok_or_else(|| ASTError::ParseError {
                message: "Failed to parse TOML".to_string(),
            })
    }

    fn get_language(&self) -> tree_sitter::Language {
        tree_sitter_toml::language()
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
            "TOML semantic tree building not implemented yet".to_string(),
        ))
    }

    fn compare_semantic_units(
        &self,
        _old: &SemanticNode,
        _new: &SemanticNode,
        _old_source: &dyn crate::ast_diff::SourceProvider,
        _new_source: &dyn crate::ast_diff::SourceProvider,
    ) -> SemanticSimilarity {
        SemanticSimilarity::unrelated()
    }
}
