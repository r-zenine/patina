/*
Test for C++ enum to enum class semantic pairing

This test reproduces Bug #256 where enum to enum class transformations were
incorrectly treated as separate delete + add operations instead of being
recognized as a single matched pair with a signature change.

The CppParser correctly stores "enum" vs "enum_class" in the metadata HashMap
during parsing, but the compare_semantic_units method wasn't checking this
metadata to detect enum→enum class transformations.

After the fix:
- The check_enum_type helper method retrieves the enum type from metadata
- The compare_semantic_units method detects enum ↔ enum class transformations
- These transformations are recognized as signature changes (similarity 0.8)
- They result in a single matched pair instead of separate add/delete operations
*/

use diffviz_core::{
    LanguageParser,
    ast_diff::{LineRange, NodeLike, SourceError, SourceProvider},
    parsers::cpp::CppParser,
    semantic_ast::build_semantic_pairs,
};

#[derive(Clone)]
struct SimpleSource {
    content: String,
}

impl SourceProvider for SimpleSource {
    fn node_text(&self, node: &dyn NodeLike) -> Result<&str, SourceError> {
        let start = node.start_byte();
        let end = node.end_byte();
        Ok(&self.content.as_str()[start..end])
    }

    fn line_range(&self, node: &dyn NodeLike) -> LineRange {
        if let Some(ts_node) = node.as_tree_sitter_node() {
            let start_pos = ts_node.start_position();
            let end_pos = ts_node.end_position();
            LineRange {
                start_line: start_pos.row + 1,
                end_line: end_pos.row + 1,
                start_column: start_pos.column,
                end_column: end_pos.column,
            }
        } else {
            LineRange {
                start_line: 1,
                end_line: 1,
                start_column: node.start_byte(),
                end_column: node.end_byte(),
            }
        }
    }

    fn clone_box(&self) -> Box<dyn SourceProvider> {
        Box::new(self.clone())
    }
}

/// Test that enum → enum class transformations are properly matched
#[test]
fn enum_to_enum_class_transformation_pairing() {
    let old_code = r#"
enum Status {
    IDLE,
    PROCESSING,
    COMPLETE
};
"#;

    let new_code = r#"
enum class Status {
    IDLE,
    PROCESSING,
    COMPLETE
};
"#;

    let parser = CppParser::new();

    // Parse AST trees
    let old_tree = parser
        .try_parse(old_code)
        .expect("Failed to parse old C++ enum code");
    let new_tree = parser
        .try_parse(new_code)
        .expect("Failed to parse new C++ enum class code");

    let old_semantic_tree = parser
        .build_semantic_tree(&old_tree, old_code)
        .expect("Failed to build old semantic tree");
    let new_semantic_tree = parser
        .build_semantic_tree(&new_tree, new_code)
        .expect("Failed to build new semantic tree");

    let old_source = SimpleSource {
        content: old_code.to_string(),
    };
    let new_source = SimpleSource {
        content: new_code.to_string(),
    };

    // Build semantic pairs
    let semantic_pairs = build_semantic_pairs(
        &old_semantic_tree,
        &new_semantic_tree,
        &old_source,
        &new_source,
        &parser,
    )
    .expect("Failed to build semantic pairs");

    // Filter out file module pairs as they're created implicitly by the semantic tree
    let semantic_pairs_without_modules = semantic_pairs
        .iter()
        .filter(|pair| match pair {
            diffviz_core::semantic_ast::SemanticPair::Matched { old_unit, .. } => !matches!(
                old_unit.unit_type,
                diffviz_core::semantic_ast::SemanticUnitType::Module { .. }
            ),
            diffviz_core::semantic_ast::SemanticPair::Addition { unit } => !matches!(
                unit.unit_type,
                diffviz_core::semantic_ast::SemanticUnitType::Module { .. }
            ),
            diffviz_core::semantic_ast::SemanticPair::Deletion { unit } => !matches!(
                unit.unit_type,
                diffviz_core::semantic_ast::SemanticUnitType::Module { .. }
            ),
        })
        .collect::<Vec<_>>();

    // The enum should be recognized as a single matched pair, not separate delete + add
    let matched_pairs = semantic_pairs_without_modules
        .iter()
        .filter(|pair| pair.should_diff())
        .count();

    assert_eq!(
        matched_pairs, 1,
        "Enum to enum class should be a single matched pair (excluding file modules)"
    );

    // Verify it's actually a semantic match, not just add/delete
    assert!(
        semantic_pairs_without_modules.iter().any(|pair| matches!(
            pair,
            diffviz_core::semantic_ast::SemanticPair::Matched { .. }
        )),
        "Enum to enum class should be a matched pair, not separate add/delete"
    );
}
