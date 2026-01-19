//! Demo of the new line range functionality in RenderableDiff
//!
//! This example shows how RenderableDiff now includes overall line range
//! information and specific line numbers that contain changes.

use std::error::Error;
use tree_sitter::Parser;

use diffviz_core::common::ProgrammingLanguage;
use diffviz_core::{
    RenderableDiff, SourceCode,
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
    println!("🎯 RenderableDiff Line Range Demo");
    println!("=================================\n");

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

    // Step 2: Create ReviewableDiffs and convert to RenderableDiffs
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

    // Create ReviewableDiffs
    let reviewable_diffs = expand_changes_to_reviewable_diffs(
        &ast_diff.changes,
        parser_impl.as_ref(),
        &old_source,
        &new_source,
        ProgrammingLanguage::Rust,
    );

    // Step 3: Convert to RenderableDiffs and demonstrate line range functionality
    println!("🔍 Found {} reviewable diff(s)", reviewable_diffs.len());

    for (i, reviewable_diff) in reviewable_diffs.iter().enumerate() {
        println!(
            "\n📋 RenderableDiff {} of {}",
            i + 1,
            reviewable_diffs.len()
        );

        // Convert to RenderableDiff
        let renderable_diff = RenderableDiff::from(reviewable_diff);

        // Demonstrate new line range functionality
        demonstrate_line_range_info(&renderable_diff);
    }

    println!("\n✅ Demo complete!");
    println!("💡 RenderableDiff now provides comprehensive line range information!");

    Ok(())
}

fn demonstrate_line_range_info(diff: &RenderableDiff) {
    // 1. Overall line range
    let overall_range = diff.line_range();
    println!(
        "   🎯 Overall range: lines {}-{} (covers {} total lines)",
        overall_range.start_line,
        overall_range.end_line,
        overall_range.end_line - overall_range.start_line + 1
    );

    // 2. Changed line numbers
    let changed_lines = diff.changed_line_numbers();
    if !changed_lines.is_empty() {
        println!("   📝 Lines with changes: {changed_lines:?}");
        println!("   📊 Total changed lines: {}", diff.changed_line_count());

        // 3. Compact range of changes
        if let Some((min, max)) = diff.changed_line_range() {
            if min == max {
                println!("   📍 Changes concentrated on line {min}");
            } else {
                println!("   📍 Changes span from line {min} to line {max}");
            }
        }

        // 4. Individual line inspection
        println!("   🔍 Inspecting specific lines:");
        for line_num in changed_lines {
            if diff.line_has_changes(*line_num) {
                println!("      ✅ Line {line_num} has changes");
            }
        }

        // 5. Show actual changed lines with content
        println!("   📄 Changed line content:");
        for (i, line) in diff.changed_lines().iter().enumerate() {
            let (prefix, _color) = line.get_display_style();
            println!("      {}{}: {}", prefix, line.line_number, line.content);

            // Show first few annotations for this line
            if !line.annotations.is_empty() && i == 0 {
                println!(
                    "         └─ {} annotation(s) on this line",
                    line.annotations.len()
                );
                for (j, annotation) in line.annotations.iter().enumerate().take(2) {
                    println!(
                        "            • {:?} ({:?})",
                        annotation.semantic_kind, annotation.change_type
                    );
                    if j == 1 && line.annotations.len() > 2 {
                        println!("            • ... and {} more", line.annotations.len() - 2);
                        break;
                    }
                }
            }
        }
    } else {
        println!("   📝 No lines have changes (context only)");
    }

    // 6. Summary statistics
    println!("   📈 Renderable diff statistics:");
    println!("      • Total lines: {}", diff.lines.len());
    println!("      • Lines with changes: {}", diff.changed_line_count());
    println!(
        "      • Essential lines: {}",
        diff.metadata.essential_line_count
    );
    println!("      • Change summary: {:?}", diff.metadata.change_summary);
}
