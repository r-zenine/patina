//! Test C semantic tree construction

use diffviz_core::{common::LanguageParser, parsers::CParser};

fn main() {
    let parser = CParser::new();

    let simple_c = r#"struct http_response {
    char *data;
    size_t size;
};

int get_data(void) {
    return 42;
}
"#;

    // First parse the AST
    let tree = parser.try_parse(simple_c).unwrap();
    println!(
        "🌲 AST root has {} children",
        tree.root_node().child_count()
    );

    // Walk AST children to see structure
    let mut cursor = tree.root_node().walk();
    for (i, child) in tree.root_node().children(&mut cursor).enumerate() {
        println!(
            "  AST Child {}: {} -> {:?}",
            i,
            child.kind(),
            &simple_c[child.start_byte()..child.end_byte().min(child.start_byte() + 50)]
        );
    }

    // Then build semantic tree
    match parser.build_semantic_tree(&tree, simple_c) {
        Ok(semantic_tree) => {
            println!("\n✅ Semantic tree built successfully");
            println!(
                "📊 Root semantic node has {} children",
                semantic_tree.root.children.len()
            );
            for (i, child) in semantic_tree.root.children.iter().enumerate() {
                println!(
                    "  Semantic Child {}: {} - TreeSitter: {}",
                    i,
                    child.unit_type_name(),
                    child.tree_sitter_node.kind()
                );
                let text = &simple_c[child.tree_sitter_node.start_byte()
                    ..child
                        .tree_sitter_node
                        .end_byte()
                        .min(child.tree_sitter_node.start_byte() + 60)];
                println!("    Text: {text:?}");
            }
        }
        Err(e) => {
            println!("\n❌ Failed to build semantic tree: {e:?}");
        }
    }
}
