//! Test for false positive phantom changes bug
//!
//! This test reproduces a bug where DiffViz incorrectly identifies unchanged functions
//! as having changes. Specifically, when only one function is modified, DiffViz may
//! incorrectly show changes in other completely unchanged functions.

use diffviz_core::{
    ast_diff::SourceCode,
    common::{LanguageParser, ProgrammingLanguage},
    parsers::RustParser,
    reviewable_diff_from_semantic::semantic_pairs_to_reviewable_diffs,
    semantic_ast::build_semantic_pairs,
};

/// Test that unchanged functions are not incorrectly marked as changed
///
/// This test reproduces the false positive where build_impl_items was shown
/// as changed when only compare_semantic_units was actually modified.
#[test]
#[ignore = "Bug #1: Phantom changes detected in unchanged functions"]
fn bug_1_phantom_changes_in_unchanged_functions() {
    // Create old version of rust.rs with original compare_semantic_units implementation
    let old_code = create_old_rust_parser_code();

    // Create new version with only compare_semantic_units modified
    let new_code = create_new_rust_parser_code();

    let old_source = SourceCode::new(&old_code);
    let new_source = SourceCode::new(&new_code);

    // Build semantic trees and pairs
    let parser = RustParser::new();
    let old_tree = parser
        .try_parse(&old_code)
        .expect("Failed to parse old code");
    let new_tree = parser
        .try_parse(&new_code)
        .expect("Failed to parse new code");

    let old_semantic = parser
        .build_semantic_tree(&old_tree, &old_code)
        .expect("Failed to build old semantic tree");
    let new_semantic = parser
        .build_semantic_tree(&new_tree, &new_code)
        .expect("Failed to build new semantic tree");

    let semantic_pairs = build_semantic_pairs(
        &old_semantic,
        &new_semantic,
        &old_source,
        &new_source,
        &parser,
    )
    .expect("Failed to build semantic pairs");

    // Convert to reviewable diffs
    let reviewable_diffs = semantic_pairs_to_reviewable_diffs(
        &semantic_pairs,
        ProgrammingLanguage::Rust,
        &old_source,
        &new_source,
    );

    // Print debug info for investigation
    println!(
        "Number of reviewable diffs created: {}",
        reviewable_diffs.len()
    );
    for (i, diff) in reviewable_diffs.iter().enumerate() {
        println!(
            "  {}. Type: {} (changes: {})",
            i + 1,
            diff.boundary.node_type,
            diff.metadata.total_changes
        );
    }

    // ASSERTIONS - These should pass but currently FAIL due to the bug

    // Should detect exactly 1 reviewable diff (only compare_semantic_units function)
    // If this fails with 2 diffs, it confirms the false positive bug
    assert_eq!(
        reviewable_diffs.len(),
        1,
        "Expected only 1 reviewable diff (compare_semantic_units), but got {} diffs. This indicates a false positive bug!",
        reviewable_diffs.len()
    );

    // If we only have 1 diff, it should have changes (not be incorrectly filtered)
    if reviewable_diffs.len() == 1 {
        assert!(
            reviewable_diffs[0].metadata.total_changes > 0,
            "The single detected diff should have changes, but shows 0 changes"
        );
    }
}

/// Create the old version of the rust parser code (before our changes)
fn create_old_rust_parser_code() -> String {
    r#"use crate::ast_diff::SourceProvider;
use crate::common::{LanguageParser, SemanticNodeKind};
use crate::semantic_ast::{SemanticNode, SemanticSimilarity, SemanticUnitType};
use tree_sitter::{Node, Parser, Tree};

pub struct RustParser {
    parser: Parser,
}

impl RustParser {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_rust::language())
            .expect("Error loading Rust grammar");
        Self { parser }
    }
    
    fn build_impl_items<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        parent: Option<Node<'a>>,
    ) -> std::result::Result<Vec<SemanticNode<'a>>, crate::semantic_ast::SemanticError> {
        // Extract the target type for context
        let target_type = node
            .child_by_field_name("type")
            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
            .unwrap_or("Unknown");

        let mut methods = Vec::new();

        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                if child.kind() == "function_item" {
                    if let Ok(method_node) =
                        self.build_function_node(child, source, parent, Some(target_type))
                    {
                        methods.push(method_node);
                    }
                }
            }
        }

        Ok(methods)
    }
    
    fn build_function_node<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        parent: Option<Node<'a>>,
        impl_target: Option<&str>,
    ) -> std::result::Result<SemanticNode<'a>, crate::semantic_ast::SemanticError> {
        // Simplified function node building for test
        Ok(SemanticNode::new(
            node,
            node.child_by_field_name("name"),
            SemanticUnitType::Callable {
                is_generic: false,
                parameter_count: 0,
                return_type: None,
                is_async: false,
                visibility: "private".to_string(),
                is_method: impl_target.is_some(),
                signature_node: None,
                metadata: std::collections::HashMap::new(),
            },
            Vec::new(),
        ))
    }
}

impl LanguageParser for RustParser {
    fn compare_semantic_units(
        &self,
        old: &SemanticNode,
        new: &SemanticNode,
        old_source: &dyn SourceProvider,
        new_source: &dyn SourceProvider,
    ) -> SemanticSimilarity {
        // OLD IMPLEMENTATION - just defaults to body_change
        SemanticSimilarity::body_change()
    }
    
    fn parse<'a>(&mut self, source: &'a str) -> Option<Tree> {
        self.parser.parse(source, None)
    }
}"#
    .to_string()
}

/// Create the new version of the rust parser code (with our changes)
fn create_new_rust_parser_code() -> String {
    r#"use crate::ast_diff::SourceProvider;
use crate::common::{LanguageParser, SemanticNodeKind};
use crate::semantic_ast::{SemanticNode, SemanticSimilarity, SemanticUnitType};
use tree_sitter::{Node, Parser, Tree};

pub struct RustParser {
    parser: Parser,
}

impl RustParser {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_rust::language())
            .expect("Error loading Rust grammar");
        Self { parser }
    }
    
    fn build_impl_items<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        parent: Option<Node<'a>>,
    ) -> std::result::Result<Vec<SemanticNode<'a>>, crate::semantic_ast::SemanticError> {
        // Extract the target type for context
        let target_type = node
            .child_by_field_name("type")
            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
            .unwrap_or("Unknown");

        let mut methods = Vec::new();

        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                if child.kind() == "function_item" {
                    if let Ok(method_node) =
                        self.build_function_node(child, source, parent, Some(target_type))
                    {
                        methods.push(method_node);
                    }
                }
            }
        }

        Ok(methods)
    }
    
    fn build_function_node<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        parent: Option<Node<'a>>,
        impl_target: Option<&str>,
    ) -> std::result::Result<SemanticNode<'a>, crate::semantic_ast::SemanticError> {
        // Simplified function node building for test
        Ok(SemanticNode::new(
            node,
            node.child_by_field_name("name"),
            SemanticUnitType::Callable {
                is_generic: false,
                parameter_count: 0,
                return_type: None,
                is_async: false,
                visibility: "private".to_string(),
                is_method: impl_target.is_some(),
                signature_node: None,
                metadata: std::collections::HashMap::new(),
            },
            Vec::new(),
        ))
    }
}

impl LanguageParser for RustParser {
    fn compare_semantic_units(
        &self,
        old: &SemanticNode,
        new: &SemanticNode,
        old_source: &dyn SourceProvider,
        new_source: &dyn SourceProvider,
    ) -> SemanticSimilarity {
        // NEW IMPLEMENTATION - compares content for other unit types
        let old_name = old
            .name_node
            .and_then(|node| old_source.node_text(&node).ok())
            .map(|s| s.to_string());
        let new_name = new
            .name_node
            .and_then(|node| new_source.node_text(&node).ok())
            .map(|s| s.to_string());

        match (&old_name, &new_name) {
            (Some(old_n), Some(new_n)) if old_n == new_n => {
                // For other unit types, compare the full node content
                let old_text = old_source.node_text(&old.tree_sitter_node).unwrap_or("");
                let new_text = new_source.node_text(&new.tree_sitter_node).unwrap_or("");
                
                if old_text == new_text {
                    SemanticSimilarity::identical()
                } else {
                    SemanticSimilarity::body_change()
                }
            }
            _ => SemanticSimilarity::unrelated(),
        }
    }
    
    fn parse<'a>(&mut self, source: &'a str) -> Option<Tree> {
        self.parser.parse(source, None)
    }
}"#
    .to_string()
}
