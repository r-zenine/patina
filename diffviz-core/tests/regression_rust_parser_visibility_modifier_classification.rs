/*
Test for visibility_modifier nodes being incorrectly classified as NOISE

This test reproduces a bug where the RustParser classifies visibility_modifier nodes
(and other function signature components) as NOISE instead of ESSENTIAL. This causes
function signatures to be folded away during context folding, which is incorrect since
the function signature is critical for understanding the boundary context.

The bug affects visibility_modifiers on multiple Rust constructs including:
- Functions
- Structs
- Enums
- Traits
- Modules
- Constants
- Static variables
- Impl blocks
*/

use diffviz_core::common::LanguageParser;
use diffviz_core::parsers::RustParser;
use tree_sitter::Parser;

/// Explores which Rust constructs have signature-related nodes
///
/// This test helps us understand the scope of the bug by checking which signature
/// components (visibility_modifier, function_modifiers, parameters, return_type, etc.)
/// are misclassified across different Rust constructs.
#[test]
fn explore_signature_component_scope() {
    let code = r#"
pub async unsafe fn async_unsafe_function() {}
pub const unsafe fn const_unsafe_function() {}
pub struct GenericStruct<T> { field: T }
pub enum GenericEnum<T> { Variant(T) }
pub trait GenericTrait<T> { fn method(&self) -> T; }
pub mod public_module {}
pub const PUBLIC_CONST: i32 = 42;
pub static PUBLIC_STATIC: i32 = 42;

impl<T> GenericStruct<T> {
    pub async fn public_async_method(&self) {}
    pub unsafe fn public_unsafe_method(&self) {}
}

impl GenericTrait<i32> for GenericStruct<i32> {
    fn method(&self) -> i32 { 42 }
}
"#;

    let rust_parser = RustParser::new();
    let mut ts_parser = Parser::new();
    ts_parser
        .set_language(&rust_parser.get_language())
        .expect("Failed to set language");

    let tree = ts_parser.parse(code, None).expect("Failed to parse code");

    // Collect all signature-related nodes
    let mut signature_nodes = Vec::new();
    let signature_node_kinds = [
        "visibility_modifier",
        "function_modifiers",
        "parameters",
        "return_type",
        "type_parameters",
        "generic_type",
        "type_parameter",
    ];

    fn find_signature_nodes(
        node: tree_sitter::Node,
        parent_kind: Option<&str>,
        signature_kinds: &[&str],
        nodes: &mut Vec<(String, String, String)>, // (parent_kind, node_kind, text)
    ) {
        if signature_kinds.contains(&node.kind()) {
            let parent_kind = parent_kind.unwrap_or("unknown").to_string();
            let node_kind = node.kind().to_string();
            nodes.push((parent_kind, node_kind, "sig-component".to_string()));
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            find_signature_nodes(child, Some(node.kind()), signature_kinds, nodes);
        }
    }

    find_signature_nodes(
        tree.root_node(),
        None,
        &signature_node_kinds,
        &mut signature_nodes,
    );

    eprintln!("Found {} signature-related nodes:", signature_nodes.len());
    let mut grouped: std::collections::HashMap<(String, String), usize> =
        std::collections::HashMap::new();
    for (parent, node_kind, _) in &signature_nodes {
        *grouped
            .entry((parent.clone(), node_kind.clone()))
            .or_insert(0) += 1;
    }

    for ((parent, node_kind), count) in grouped.iter() {
        let semantic_kind = rust_parser.classify_node_kind(node_kind);
        let relevance = rust_parser.classify_leaf_relevance(&semantic_kind);
        eprintln!(
            "  - Parent: {:30} | Node: {:20} | Count: {} | SemanticKind: {:?} | Relevance: {} {}",
            parent,
            node_kind,
            count,
            semantic_kind,
            relevance,
            if relevance == 3 {
                "← NOISE (BUG!)"
            } else {
                ""
            }
        );
    }

    assert!(
        !signature_nodes.is_empty(),
        "Expected to find signature-related nodes"
    );
}

/// Test that function signature components are classified as IMPORTANT
///
/// Previously, RustParser::classify_node_kind() didn't have explicit cases
/// for nodes like "visibility_modifier", so they fell through to the default case and
/// got classified as SemanticNodeKind::Other, which has NOISE relevance.
///
/// After the fix:
/// - Signature components are now explicitly classified as SignatureComponent
/// - They get IMPORTANT relevance at the leaf level
/// - During context expansion, they inherit their parent construct's relevance
/// - This ensures complete signatures are shown when their parent is in context
#[test]
fn signature_components_classified_as_important() {
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
        .set_language(&rust_parser.get_language())
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

    // Verify fix: visibility_modifier should now be found
    assert!(
        visibility_modifier_found,
        "visibility_modifier node not found in parsed tree"
    );

    let relevance = visibility_modifier_relevance.expect("Failed to get relevance");
    let expected_relevance = diffviz_core::ast_diff::IMPORTANT; // 1

    // After fix: visibility_modifier is classified as SignatureComponent with IMPORTANT relevance
    assert_eq!(
        relevance, expected_relevance,
        "visibility_modifier should be classified as IMPORTANT ({expected_relevance}), not NOISE (3)"
    );
}
