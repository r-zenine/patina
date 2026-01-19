//! Complete Java semantic code review demo
//!
//! This demonstrates the Java parser's ability to detect and analyze
//! semantic changes in Java code including classes, generics, annotations,
//! inheritance patterns, and package organization.

use std::error::Error;
use tree_sitter::Parser;

use diffviz_core::common::{LanguageParser, ProgrammingLanguage};
use diffviz_core::{
    ast_diff::SourceCode, parsers::java::JavaParser, renderable_diff::RenderableDiff,
    reviewable_diff_from_semantic::semantic_pairs_to_reviewable_diffs,
    semantic_ast::build_semantic_pairs,
};

/// Original version - Basic Java class with traditional patterns
const OLD_CODE: &str = r#"package com.example.processor;

import java.util.List;
import java.util.ArrayList;

public class DataProcessor {
    private List data;
    private int capacity;
    
    public DataProcessor(int cap) {
        this.capacity = cap;
        this.data = new ArrayList();
    }
    
    public void addItem(Object item) {
        if (data.size() < capacity) {
            data.add(item);
        }
    }
    
    public int getSize() {
        return data.size();
    }
    
    public void process() {
        for (Object item : data) {
            System.out.println("Processing: " + item);
        }
    }
}

enum Status {
    IDLE, PROCESSING, COMPLETE
}"#;

/// Enhanced version - Modern Java with generics, annotations, and enhanced features
const NEW_CODE: &str = r#"package com.example.processor;

import java.util.List;
import java.util.ArrayList;
import java.util.concurrent.CopyOnWriteArrayList;

/**
 * Enhanced data processor with generic support and thread safety
 */
public class DataProcessor<T> {
    private final List<T> data;
    private final int capacity;
    
    public DataProcessor(int cap) {
        this.capacity = cap;
        this.data = new CopyOnWriteArrayList<>();
    }
    
    @Override
    public String toString() {
        return "DataProcessor[capacity=" + capacity + ", size=" + data.size() + "]";
    }
    
    public void addItem(T item) {
        if (item == null) {
            throw new IllegalArgumentException("Item cannot be null");
        }
        if (data.size() < capacity) {
            data.add(item);
        }
    }
    
    public int getSize() {
        return data.size();
    }
    
    public void process() {
        for (T item : data) {
            System.out.println("Processing: " + item.toString());
        }
    }
    
    @SafeVarargs
    public final void addAll(T... items) {
        for (T item : items) {
            addItem(item);
        }
    }
}

public enum Status {
    IDLE("Not active"),
    PROCESSING("Currently processing"),
    COMPLETE("Processing complete");
    
    private final String description;
    
    Status(String description) {
        this.description = description;
    }
    
    public String getDescription() {
        return description;
    }
}"#;

fn analyze_tree_structure(node: tree_sitter::Node, source: &str, indent: usize, max_depth: usize) {
    if indent > max_depth {
        return;
    }

    let indent_str = "  ".repeat(indent);
    let node_text = &source[node.start_byte()..node.end_byte().min(node.start_byte() + 50)];

    println!("{}├─ {} → {:?}", indent_str, node.kind(), node_text);

    if node.child_count() <= 3 && indent < max_depth {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            analyze_tree_structure(child, source, indent + 1, max_depth);
        }
    } else if node.child_count() > 3 {
        println!(
            "{}  └─ ... {} more children",
            indent_str,
            node.child_count()
        );
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("⚙️  Java Semantic Code Review Demo");
    println!();

    // Configuration
    let language = ProgrammingLanguage::Java;

    // Setup source code objects
    let old_source = SourceCode::new(OLD_CODE);
    let new_source = SourceCode::new(NEW_CODE);

    // Parse AST trees
    let java_parser = JavaParser::new();
    let mut ts_parser = Parser::new();
    ts_parser.set_language(java_parser.get_language())?;

    let old_tree = ts_parser
        .parse(OLD_CODE, None)
        .ok_or("Failed to parse old Java code")?;
    let new_tree = ts_parser
        .parse(NEW_CODE, None)
        .ok_or("Failed to parse new Java code")?;

    // Debug: Show AST structure
    println!("🌲 OLD AST structure:");
    analyze_tree_structure(old_tree.root_node(), OLD_CODE, 0, 2);
    println!("\n🌲 NEW AST structure:");
    analyze_tree_structure(new_tree.root_node(), NEW_CODE, 0, 2);

    let old_semantic_tree = java_parser
        .build_semantic_tree(&old_tree, OLD_CODE)
        .map_err(|e| format!("Failed to build old Java semantic tree: {e}"))?;
    let new_semantic_tree = java_parser
        .build_semantic_tree(&new_tree, NEW_CODE)
        .map_err(|e| format!("Failed to build new Java semantic tree: {e}"))?;

    println!("\n✅ Successfully built semantic trees!");

    // Debug: Show semantic tree structure
    println!(
        "📊 Old semantic tree has {} children",
        old_semantic_tree.root.children.len()
    );
    println!(
        "📊 New semantic tree has {} children",
        new_semantic_tree.root.children.len()
    );

    // Build semantic pairs and convert to reviewable diffs
    let semantic_pairs = build_semantic_pairs(
        &old_semantic_tree,
        &new_semantic_tree,
        &old_source,
        &new_source,
        &java_parser,
    )
    .map_err(|e| format!("Failed to build semantic pairs: {e}"))?;

    let semantic_reviewable_diffs =
        semantic_pairs_to_reviewable_diffs(&semantic_pairs, language, &old_source, &new_source);

    // First pass: filter out changes with no visible content
    let visible_changes: Vec<&_> = semantic_reviewable_diffs
        .iter()
        .filter(|reviewable_diff| {
            let renderable: RenderableDiff = (*reviewable_diff).into();
            let visible_lines = renderable
                .lines
                .iter()
                .filter(|line| !line.should_fold())
                .count();
            visible_lines > 0
        })
        .collect();

    for (i, reviewable_diff) in visible_changes.iter().enumerate() {
        // Convert to RenderableDiff for display
        let renderable: RenderableDiff = (*reviewable_diff).into();

        let changed_lines = renderable
            .lines
            .iter()
            .filter(|line| line.has_changes())
            .count();
        let hidden_lines = renderable
            .lines
            .iter()
            .filter(|line| line.should_fold())
            .count();

        println!(
            "\n🔸 Change {} of {}: {}",
            i + 1,
            visible_changes.len(),
            renderable.metadata.boundary_name
        );

        if changed_lines > 0 {
            println!("   📊 {changed_lines} changed lines, {hidden_lines} hidden lines");
        }

        // Display source code with syntax highlighting
        println!("   📝 Source Code:");

        let mut hidden_count = 0;
        for line in &renderable.lines {
            if line.should_fold() {
                hidden_count += 1;
                continue;
            }

            if hidden_count > 0 {
                println!("   \x1b[37m  ... {hidden_count} lines hidden ...\x1b[0m");
                hidden_count = 0;
            }

            let (prefix, color) = line.get_display_style();
            println!("   {}{} {}\x1b[0m", color, prefix, line.content);
        }

        if hidden_count > 0 {
            println!("   \x1b[37m  ... {hidden_count} lines hidden ...\x1b[0m");
        }
    }

    println!(
        "\n✅ Review complete: {} changes detected",
        visible_changes.len()
    );

    Ok(())
}
