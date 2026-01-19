//! TreeSitter AST Explorer
//!
//! This example helps analyze the TreeSitter AST structure for different code patterns.
//! Useful for understanding how to properly parse and handle various language constructs.

use diffviz_core::LanguageParser;
use diffviz_core::parsers::rust::RustParser;
use tree_sitter::Node;

fn main() {
    println!("🌳 TreeSitter AST Explorer for Rust");
    println!("=====================================");

    // Test case: HttpError from demo (OLD_CODE)
    analyze_code(
        "HttpError OLD_CODE",
        r#"#[derive(Debug)]
pub enum HttpError {
    NetworkError(String),
    ParseError(String),
}"#,
    );

    // Test case: HttpError from demo (NEW_CODE)
    analyze_code(
        "HttpError NEW_CODE",
        r#"/// Comprehensive HTTP error types with detailed context
#[derive(Debug)]
pub enum HttpError {
    NetworkError { message: String, status_code: Option<u16> },
    ParseError { message: String, line: Option<usize> },
    TimeoutError { duration: Duration },
    ConfigError { field: String, value: String },
}"#,
    );

    // Test case: Struct with derives
    analyze_code(
        "Struct with derives",
        r#"#[derive(Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct User {
    name: String,
    age: u32,
}"#,
    );

    // Test case: Simple function without attributes
    analyze_code(
        "Simple function",
        r#"fn hello() { 
    println!("world"); 
}"#,
    );
}

fn analyze_code(description: &str, source_code: &str) {
    println!("\n📋 Analyzing: {description}");
    println!("Source code:");
    for (i, line) in source_code.lines().enumerate() {
        println!("{:2}: {}", i + 1, line);
    }

    let parser = RustParser::new();
    let tree = parser.try_parse(source_code).expect("Failed to parse");

    println!("\n🌲 Full AST (S-expression):");
    println!("{}", tree.root_node().to_sexp());

    println!("\n🔍 Walking top-level nodes:");
    walk_children(tree.root_node(), source_code, 0);

    println!("\n{}", "=".repeat(60));
}

fn walk_children(node: Node, source: &str, depth: usize) {
    let indent = "  ".repeat(depth);
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        let text = child.utf8_text(source.as_bytes()).unwrap_or("<error>");
        let preview = if text.len() > 50 {
            format!("{}...", &text[..47])
        } else {
            text.to_string()
        };

        println!(
            "{}├─ {} → \"{}\"",
            indent,
            child.kind(),
            preview.replace('\n', "\\n")
        );

        // For attribute_item and function_item, dive deeper
        if matches!(child.kind(), "attribute_item" | "function_item") && depth < 2 {
            walk_children(child, source, depth + 1);
        }
    }
}
