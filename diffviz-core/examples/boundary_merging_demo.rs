//! Demonstration of boundary merging in ReviewableDiff pipeline
//!
//! This example specifically showcases how multiple changes within the same
//! semantic boundary get merged into a single ReviewableDiff structure.
//!
//! Run with: cargo run --example boundary_merging_demo

use std::error::Error;
use tree_sitter::Parser;

use diffviz_core::common::ProgrammingLanguage;
use diffviz_core::{
    NodeLike,
    ast_diff::{ChangeDetectionStrategies, ESSENTIAL, SourceCode, diff_ast_trees_with_strategies},
    common::LanguageParser,
    parsers::RustParser,
    renderable_diff::RenderableDiff,
    reviewable_diff::expand_changes_to_reviewable_diffs,
};

/// Original version with a single function containing multiple statements
const OLD_CODE: &str = r#"
fn calculate_discount(price: f64, customer_type: &str) -> f64 {
    let base_discount = 0.1;
    let loyalty_bonus = 0.05;
    
    if customer_type == "premium" {
        price * (base_discount + loyalty_bonus)
    } else if customer_type == "regular" {
        price * base_discount
    } else {
        0.0
    }
}

fn process_order(items: Vec<&str>) -> String {
    let mut result = String::new();
    for item in items {
        result.push_str(item);
        result.push(' ');
    }
    result
}
"#;

/// Modified version with multiple changes within the SAME function
const NEW_CODE: &str = r#"
fn calculate_discount(price: f64, customer_type: &str) -> f64 {
    let base_discount = 0.15;     // Changed: 0.1 -> 0.15
    let loyalty_bonus = 0.08;     // Changed: 0.05 -> 0.08
    let max_discount = 0.25;      // Added: new variable
    
    let calculated_discount = if customer_type == "premium" {
        price * (base_discount + loyalty_bonus)
    } else if customer_type == "regular" {
        price * base_discount  
    } else {
        0.0
    };
    
    // Added: clamp the discount to maximum
    if calculated_discount > price * max_discount {
        price * max_discount
    } else {
        calculated_discount
    }
}

fn process_order(items: Vec<&str>) -> String {
    let mut result = String::new();
    for item in items {
        result.push_str(item);
        result.push(' ');
    }
    result
}
"#;

fn main() -> Result<(), Box<dyn Error>> {
    println!("🔗 Boundary Merging Demo");
    println!("========================\n");

    // Step 1: Setup source code objects
    println!("📄 Step 1: Creating source code objects...");
    let old_source = SourceCode::new(OLD_CODE);
    let new_source = SourceCode::new(NEW_CODE);
    println!("   Old code: {} lines", OLD_CODE.lines().count());
    println!("   New code: {} lines", NEW_CODE.lines().count());
    println!("   Expected: Multiple changes within calculate_discount function");
    println!();

    // Step 2: Parse AST trees
    println!("🌳 Step 2: Parsing AST trees with TreeSitter...");
    let parser_impl: Box<dyn LanguageParser> = Box::new(RustParser::new());
    let mut ts_parser = Parser::new();
    ts_parser.set_language(parser_impl.get_language())?;

    let old_tree = ts_parser
        .parse(OLD_CODE, None)
        .ok_or("Failed to parse old code")?;
    let new_tree = ts_parser
        .parse(NEW_CODE, None)
        .ok_or("Failed to parse new code")?;

    println!("   Old AST: {} nodes", count_nodes(&old_tree));
    println!("   New AST: {} nodes", count_nodes(&new_tree));
    println!();

    // Step 3: Detect changes using strategies
    println!("🔍 Step 3: Detecting changes with multiple strategies...");
    let strategies = ChangeDetectionStrategies::default_strategies();
    let ast_diff =
        diff_ast_trees_with_strategies(&old_tree, &new_tree, OLD_CODE, NEW_CODE, strategies);

    println!("   Detected {} raw AST changes", ast_diff.changes.len());
    for (i, change) in ast_diff.changes.iter().enumerate() {
        println!("     Change {}: {:?}", i + 1, get_change_summary(change));
    }
    println!();

    // Step 4: Expand with semantic context and observe merging
    println!("🎯 Step 4: Expanding changes with semantic context...");
    println!("   This is where boundary merging happens!");

    let reviewable_diffs = expand_changes_to_reviewable_diffs(
        &ast_diff.changes,
        parser_impl.as_ref(),
        &old_source,
        &new_source,
        ProgrammingLanguage::Rust,
    );

    println!(
        "   Generated {} reviewable diff structures",
        reviewable_diffs.len()
    );
    println!(
        "   📊 Merging Effect: {} raw changes → {} boundaries",
        ast_diff.changes.len(),
        reviewable_diffs.len()
    );
    println!();

    // Step 5: Display results showing merged changes
    println!("📋 Step 5: ReviewableDiff Results (Showing Merged Changes)");
    println!("==========================================================");

    if reviewable_diffs.is_empty() {
        println!("No changes detected or all changes filtered out.");
        return Ok(());
    }

    for (i, diff) in reviewable_diffs.iter().enumerate() {
        println!(
            "\n📊 Merged ReviewableDiff {} of {}",
            i + 1,
            reviewable_diffs.len()
        );
        println!("   🔗 Merging Results:");
        println!(
            "     - Total merged changes: {}",
            diff.metadata.total_changes
        );
        println!(
            "     - Essential nodes: {}",
            diff.metadata.essential_node_count
        );
        println!(
            "     - Change breakdown: {:?}",
            diff.metadata.change_summary
        );

        // Convert to RenderableDiff using idiomatic trait conversion
        println!("\n   🎯 Converting to RenderableDiff (line-based structure):");
        let renderable: RenderableDiff = diff.into();

        // Show line-based statistics
        let total_lines = renderable.lines.len();
        let essential_lines = renderable
            .lines
            .iter()
            .filter(|line| line.max_relevance() == ESSENTIAL)
            .count();
        let changed_lines = renderable
            .lines
            .iter()
            .filter(|line| line.has_changes())
            .count();
        let folded_lines = renderable
            .lines
            .iter()
            .filter(|line| line.should_fold())
            .count();

        println!("     - Total lines: {total_lines}");
        println!("     - Essential lines: {essential_lines}");
        println!("     - Changed lines: {changed_lines}");
        println!("     - Would fold: {folded_lines} lines");

        // Display using RenderableDiff line-by-line approach
        println!("\n   📝 Source Code (using RenderableDiff structure):");
        println!(
            "   \x1b[96m📦 {} ({}) - {} changes merged\x1b[0m",
            renderable.metadata.boundary_name,
            format!("{:?}", renderable.language).to_lowercase(),
            renderable.metadata.total_changes
        );
        println!("   \x1b[37m────────────────────────────────────────────────────────────\x1b[0m");

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

        println!("   {}", "─".repeat(60));
    }

    // Step 6: Summary of merging effectiveness
    println!("\n✅ Boundary Merging Summary:");
    println!("   Original AST changes: {}", ast_diff.changes.len());
    println!(
        "   Final ReviewableDiff structures: {}",
        reviewable_diffs.len()
    );
    let reduction_percentage = if !ast_diff.changes.is_empty() {
        ((ast_diff.changes.len() - reviewable_diffs.len()) as f32 / ast_diff.changes.len() as f32)
            * 100.0
    } else {
        0.0
    };
    println!(
        "   Reduction: {:.1}% (consolidated {} changes)",
        reduction_percentage,
        ast_diff.changes.len() - reviewable_diffs.len()
    );

    println!("\n💡 Key Insight: Multiple changes within the same semantic boundary");
    println!("   (like variable changes within a function) are merged into a single");
    println!("   ReviewableDiff for efficient code review!");

    Ok(())
}

/// Count total nodes in an AST tree
fn count_nodes(tree: &tree_sitter::Tree) -> usize {
    fn count_recursive(node: tree_sitter::Node) -> usize {
        let mut count = 1;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            count += count_recursive(child);
        }
        count
    }
    count_recursive(tree.root_node())
}

/// Get a summary string for an AST change
fn get_change_summary(change: &diffviz_core::ast_diff::ASTChange) -> String {
    use diffviz_core::ast_diff::ASTChange;

    match change {
        ASTChange::Addition(node) => format!("Added {}", node.kind()),
        ASTChange::Deletion(node) => format!("Deleted {}", node.kind()),
        ASTChange::ContentChange { old, new } => {
            format!("Content {} -> {}", old.kind(), new.kind())
        }
        ASTChange::StructuralChange { old, new } => {
            format!("Structure {} -> {}", old.kind(), new.kind())
        }
        ASTChange::KindChange { old, new } => format!("Kind {} -> {}", old.kind(), new.kind()),
        ASTChange::Reorder { parent, .. } => format!("Reordered children in {}", parent.kind()),
    }
}
