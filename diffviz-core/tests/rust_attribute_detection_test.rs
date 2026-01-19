//! Test for Rust attribute detection in semantic parsing
//!
//! This test verifies that attributes like #[cfg(test)] and #[allow(dead_code)]
//! are properly detected as part of their semantic units.

use diffviz_core::{
    LanguageParser,
    ast_diff::{ASTChange, diff_ast_trees},
    parsers::rust::RustParser,
};

#[test]
fn test_function_attribute_detection() {
    let old_code = r#"fn test_function() {
    println!("test");
}"#;

    let new_code = r#"#[cfg(test)]
#[allow(dead_code)]
fn test_function() {
    println!("test");
}"#;

    let parser = RustParser::new();
    let old_tree = parser
        .try_parse(old_code)
        .expect("Failed to parse old code");
    let new_tree = parser
        .try_parse(new_code)
        .expect("Failed to parse new code");

    println!("=== Old Tree ===");
    println!("{}", old_tree.root_node().to_sexp());
    println!("\n=== New Tree ===");
    println!("{}", new_tree.root_node().to_sexp());

    let diff = diff_ast_trees(&old_tree, &new_tree);

    println!("\n=== Diff Result ===");
    println!("Has changes: {}", diff.has_changes());
    println!("Total changes: {}", diff.total_changes());

    for change in &diff.changes {
        match change {
            ASTChange::Addition(node_ref) => {
                println!(
                    "Addition: {} at {:?}",
                    node_ref.node.kind(),
                    node_ref.node.range()
                );
            }
            ASTChange::Deletion(node_ref) => {
                println!(
                    "Deletion: {} at {:?}",
                    node_ref.node.kind(),
                    node_ref.node.range()
                );
            }
            ASTChange::ContentChange { old, new } => {
                println!(
                    "ContentChange: {} -> {} at {:?}",
                    old.node.kind(),
                    new.node.kind(),
                    old.node.range()
                );
            }
            _ => {
                println!("Other change type: {change:?}");
            }
        }
    }

    // The key assertion: we should detect changes when attributes are added
    assert!(diff.has_changes(), "Should detect attribute additions");
    assert!(
        diff.total_changes() > 0,
        "Should have at least one change detected"
    );
}

#[test]
fn test_struct_attribute_detection() {
    let old_code = r#"struct User {
    name: String,
    age: u32,
}"#;

    let new_code = r#"#[derive(Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct User {
    name: String,
    age: u32,
}"#;

    let parser = RustParser::new();
    let old_tree = parser
        .try_parse(old_code)
        .expect("Failed to parse old code");
    let new_tree = parser
        .try_parse(new_code)
        .expect("Failed to parse new code");

    let diff = diff_ast_trees(&old_tree, &new_tree);

    println!("\n=== Struct Attribute Test ===");
    println!("Has changes: {}", diff.has_changes());
    println!("Total changes: {}", diff.total_changes());

    assert!(
        diff.has_changes(),
        "Should detect struct attribute additions"
    );
    assert!(
        diff.total_changes() > 0,
        "Should have at least one change detected"
    );
}

#[test]
fn test_attributes_show_in_ast_diff() {
    // This test verifies that when attributes are added, the AST diff
    // detects them as attribute_item additions
    let old_code = "fn simple() {}";
    let new_code = "#[test]\nfn simple() {}";

    let parser = RustParser::new();
    let old_tree = parser.try_parse(old_code).expect("Failed to parse old");
    let new_tree = parser.try_parse(new_code).expect("Failed to parse new");

    let diff = diff_ast_trees(&old_tree, &new_tree);

    println!("\n=== Simple Attribute Addition Test ===");
    println!("Changes detected: {}", diff.total_changes());

    let mut found_attribute_related_change = false;
    for change in &diff.changes {
        match change {
            ASTChange::Addition(node_ref) => {
                println!("Added: {}", node_ref.node.kind());
                if node_ref.node.kind() == "attribute_item" {
                    found_attribute_related_change = true;
                    println!("  → Found attribute_item addition! ✅");
                }
            }
            ASTChange::KindChange { old, new } => {
                println!("KindChange: {} -> {}", old.node.kind(), new.node.kind());
                if new.node.kind() == "attribute_item" || old.node.kind() == "attribute_item" {
                    found_attribute_related_change = true;
                    println!("  → Found attribute-related KindChange! ✅");
                }
            }
            _ => {
                println!("Other change: {change:?}");
            }
        }
    }

    // The key point: attributes are now being detected in the AST diff!
    assert!(
        found_attribute_related_change,
        "Should detect attribute-related changes"
    );
    assert!(
        diff.has_changes(),
        "Should detect changes when attribute is added"
    );
}
