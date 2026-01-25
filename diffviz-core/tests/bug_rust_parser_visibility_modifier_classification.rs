//! Test for visibility_modifier nodes being incorrectly classified as NOISE
//!
//! This test reproduces a bug where the RustParser classifies visibility_modifier nodes
//! (and other function signature components) as NOISE instead of ESSENTIAL. This causes
//! function signatures to be folded away during context folding, which is incorrect since
//! the function signature is critical for understanding the boundary context.

use diffviz_core::common::LanguageParser;
use diffviz_core::parsers::RustParser;
use tree_sitter::Parser;

/// Test that function signature components are classified as ESSENTIAL
///
/// The bug occurs because RustParser::classify_node_kind() doesn't have explicit cases
/// for nodes like "visibility_modifier", so they fall through to the default case and
/// get classified as SemanticNodeKind::Other, which has NOISE relevance.
#[test]
#[ignore = "Bug: RustParser classifies visibility_modifier as NOISE instead of ESSENTIAL"]
fn bug_rust_parser_visibility_modifier_classification() {
    let code = r#"pub fn divide(a: i32, b: i32) -> Result<i32, CalcError> {
    if b == 0 {
        return Err(CalcError::DivisionByZero);
    }
    let result = a / b;
    Ok(result)
}"#;

    let rust_parser = RustParser::new();
    let mut ts_parser = Parser::new();
    ts_parser
        .set_language(rust_parser.get_language())
        .expect("Failed to set language");

    let tree = ts_parser.parse(code, None).expect("Failed to parse code");

    // Find visibility_modifier node
    let mut visibility_modifier_found = false;
    let mut visibility_modifier_relevance = None;

    fn find_visibility_node(
        node: tree_sitter::Node,
        parser: &RustParser,
        found: &mut bool,
        relevance: &mut Option<u8>,
    ) {
        if node.kind() == "visibility_modifier" {
            *found = true;
            let semantic_kind = parser.classify_node_kind(node.kind());
            *relevance = Some(parser.classify_leaf_relevance(&semantic_kind));
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            find_visibility_node(child, parser, found, relevance);
        }
    }

    find_visibility_node(
        tree.root_node(),
        &rust_parser,
        &mut visibility_modifier_found,
        &mut visibility_modifier_relevance,
    );

    // Verify the bug exists
    assert!(
        visibility_modifier_found,
        "visibility_modifier node not found in parsed tree"
    );

    let relevance = visibility_modifier_relevance.expect("Failed to get relevance");
    let expected_relevance = diffviz_core::ast_diff::ESSENTIAL; // 0

    // This assertion will FAIL, demonstrating the bug
    assert_eq!(
        relevance, expected_relevance,
        "visibility_modifier should be classified as ESSENTIAL ({}), but got {} (NOISE)",
        expected_relevance, relevance
    );
}
