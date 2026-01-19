use diffviz_core::{LanguageParser, parsers::java::JavaParser};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let code = r#"
package com.example;

import java.util.List;

public class Calculator {
    private int value;

    public Calculator(int initialValue) {
        this.value = initialValue;
    }

    public int add(int number) {
        return value + number;
    }
}
"#;

    let parser = JavaParser::new();

    // Parse the Java code
    let tree = parser.try_parse(code)?;
    let semantic_tree = parser.build_semantic_tree(&tree, code)?;

    println!("🌲 Java AST structure:");
    println!("Root node kind: {}", tree.root_node().kind());

    let all_units = semantic_tree.all_units();
    println!("\n📊 All semantic units ({}):", all_units.len());
    for (i, unit) in all_units.iter().enumerate() {
        println!(
            "  {}: {:?} (kind: {})",
            i,
            unit.unit_type_name(),
            unit.tree_sitter_node.kind()
        );
    }

    let filtered_units = semantic_tree.filtered_units();
    println!("\n🔍 Filtered semantic units ({}):", filtered_units.len());
    for (i, unit) in filtered_units.iter().enumerate() {
        println!(
            "  {}: {:?} (kind: {})",
            i,
            unit.unit_type_name(),
            unit.tree_sitter_node.kind()
        );
        if let Some(name_node) = unit.name_node {
            if let Ok(name) = name_node.utf8_text(code.as_bytes()) {
                println!("     Name: {name}");
            }
        }
    }

    Ok(())
}
