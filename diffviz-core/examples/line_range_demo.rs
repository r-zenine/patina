//! Demo of the new line_range functionality in SourceProvider
//!
//! This example shows how to get line numbers for any node in a ReviewableDiff,
//! which is useful for identifying what lines are under review and which changed.

use std::error::Error;
use tree_sitter::Parser;

use diffviz_core::common::ProgrammingLanguage;
use diffviz_core::{
    LineRange, SourceCode,
    ast_diff::{ChangeDetectionStrategies, diff_ast_trees_with_strategies},
    common::LanguageParser,
    parsers::RustParser,
    reviewable_diff::expand_changes_to_reviewable_diffs,
};

const OLD_CODE: &str = r#"fn hello() {
    println!("Hello");
    let x = 42;
    return x;
}"#;

const NEW_CODE: &str = r#"fn hello() {
    println!("Hello, World!");
    let x = 42;
    let y = 100;
    return x + y;
}"#;

fn main() -> Result<(), Box<dyn Error>> {
    println!("🎯 LineRange Demo: Getting Line Numbers from ReviewableDiff");
    println!("============================================================\n");

    // Step 1: Setup source code objects
    let old_source = SourceCode::new(OLD_CODE);
    let new_source = SourceCode::new(NEW_CODE);

    println!("📄 Old code ({} lines):", OLD_CODE.lines().count());
    for (i, line) in OLD_CODE.lines().enumerate() {
        println!("   {}: {}", i + 1, line);
    }
    println!();

    println!("📄 New code ({} lines):", NEW_CODE.lines().count());
    for (i, line) in NEW_CODE.lines().enumerate() {
        println!("   {}: {}", i + 1, line);
    }
    println!();

    // Step 2: Parse AST and detect changes
    let parser_impl: Box<dyn LanguageParser> = Box::new(RustParser::new());
    let mut ts_parser = Parser::new();
    ts_parser.set_language(parser_impl.get_language())?;

    let old_tree = ts_parser
        .parse(OLD_CODE, None)
        .ok_or("Failed to parse old code")?;
    let new_tree = ts_parser
        .parse(NEW_CODE, None)
        .ok_or("Failed to parse new code")?;

    // Detect changes
    let strategies = ChangeDetectionStrategies::default_strategies();
    let ast_diff =
        diff_ast_trees_with_strategies(&old_tree, &new_tree, OLD_CODE, NEW_CODE, strategies);

    // Step 3: Create ReviewableDiffs
    let reviewable_diffs = expand_changes_to_reviewable_diffs(
        &ast_diff.changes,
        parser_impl.as_ref(),
        &old_source,
        &new_source,
        ProgrammingLanguage::Rust,
    );

    println!("🔍 Found {} reviewable diff(s)", reviewable_diffs.len());
    println!();

    // Step 4: Demonstrate line range functionality
    for (i, diff) in reviewable_diffs.iter().enumerate() {
        println!("📋 ReviewableDiff {} of {}", i + 1, reviewable_diffs.len());

        // Get line range for the root boundary node
        let boundary_line_range = demonstrate_boundary_line_range(diff);
        println!(
            "   🎯 Boundary covers lines {}-{}",
            boundary_line_range.start_line, boundary_line_range.end_line
        );

        // Walk through the diff tree and show line ranges for changed nodes
        println!("   📝 Changed nodes and their line ranges:");
        demonstrate_changed_nodes_line_ranges(diff);

        println!();
    }

    println!("✅ Demo complete!");
    println!(
        "💡 Users can now call source_provider.line_range(&node) to get line numbers for any node"
    );

    Ok(())
}

fn demonstrate_boundary_line_range(
    diff: &diffviz_core::reviewable_diff::ReviewableDiff,
) -> LineRange {
    // Get the primary node from the boundary
    let boundary_node = match &diff.boundary.change_status {
        diffviz_core::reviewable_diff::NodeChangeStatus::Added { node, .. } => node,
        diffviz_core::reviewable_diff::NodeChangeStatus::Deleted { node, .. } => node,
        diffviz_core::reviewable_diff::NodeChangeStatus::Modified { new_node, .. } => new_node,
        diffviz_core::reviewable_diff::NodeChangeStatus::Moved { new_node, .. } => new_node,
        diffviz_core::reviewable_diff::NodeChangeStatus::Reordered { new_node, .. } => new_node,
        diffviz_core::reviewable_diff::NodeChangeStatus::Unchanged { node, .. } => node,
    };

    // Use the new line_range method!
    diff.new_source.line_range(boundary_node)
}

fn demonstrate_changed_nodes_line_ranges(diff: &diffviz_core::reviewable_diff::ReviewableDiff) {
    collect_changed_nodes_recursive(&diff.boundary, diff, 0);
}

fn collect_changed_nodes_recursive(
    node: &diffviz_core::reviewable_diff::DiffNode,
    diff: &diffviz_core::reviewable_diff::ReviewableDiff,
    depth: usize,
) {
    let indent = "      ".repeat(depth);

    // Check if this node has changes
    match &node.change_status {
        diffviz_core::reviewable_diff::NodeChangeStatus::Added { node: node_ref, .. } => {
            let line_range = diff.new_source.line_range(node_ref);
            println!(
                "{}➕ Added {} at lines {}-{} ({:?})",
                indent,
                node.node_type,
                line_range.start_line,
                line_range.end_line,
                node.semantic_kind
            );
        }
        diffviz_core::reviewable_diff::NodeChangeStatus::Deleted { node: node_ref, .. } => {
            let line_range = diff.old_source.line_range(node_ref);
            println!(
                "{}➖ Deleted {} at lines {}-{} ({:?})",
                indent,
                node.node_type,
                line_range.start_line,
                line_range.end_line,
                node.semantic_kind
            );
        }
        diffviz_core::reviewable_diff::NodeChangeStatus::Modified { new_node, .. } => {
            let line_range = diff.new_source.line_range(new_node);
            println!(
                "{}🔄 Modified {} at lines {}-{} ({:?})",
                indent,
                node.node_type,
                line_range.start_line,
                line_range.end_line,
                node.semantic_kind
            );
        }
        diffviz_core::reviewable_diff::NodeChangeStatus::Moved { new_node, .. } => {
            let line_range = diff.new_source.line_range(new_node);
            println!(
                "{}📦 Moved {} at lines {}-{} ({:?})",
                indent,
                node.node_type,
                line_range.start_line,
                line_range.end_line,
                node.semantic_kind
            );
        }
        diffviz_core::reviewable_diff::NodeChangeStatus::Reordered { new_node, .. } => {
            let line_range = diff.new_source.line_range(new_node);
            println!(
                "{}🔀 Reordered {} at lines {}-{} ({:?})",
                indent,
                node.node_type,
                line_range.start_line,
                line_range.end_line,
                node.semantic_kind
            );
        }
        diffviz_core::reviewable_diff::NodeChangeStatus::Unchanged { .. } => {
            // Don't print unchanged nodes to keep output clean
        }
    }

    // Recursively process children
    for child in &node.children {
        collect_changed_nodes_recursive(child, diff, depth + 1);
    }
}
