//! Demonstration of top-level import merging in ReviewableDiff pipeline
//!
//! This example specifically showcases how multiple import changes at the top-level
//! get merged into a single ReviewableDiff structure, while nested imports in modules
//! remain separate.
//!
//! Run with: cargo run --example import_merging_demo

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

/// Original version with some imports
const OLD_CODE: &str = r#"
use std::collections::HashMap;
use serde::Serialize;

fn main() {
    println!("Hello, world!");
}

mod utils {
    use super::*;
    
    pub fn helper() {
        println!("helper function");
    }
}

mod networking {
    use std::net::TcpStream;
    
    pub fn connect() {
        println!("connecting");
    }
}
"#;

/// Modified version with multiple top-level import changes
const NEW_CODE: &str = r#"
use std::collections::{HashMap, BTreeMap};
use serde::{Serialize, Deserialize};
use tokio::sync::Mutex;
use std::fs;

fn main() {
    println!("Hello, world!");
}

mod utils {
    use super::*;
    use std::path::Path;
    
    pub fn helper() {
        println!("helper function");
    }
}

mod networking {
    use std::net::{TcpStream, UdpSocket};
    use std::io::{Read, Write};
    
    pub fn connect() {
        println!("connecting");
    }
}
"#;

fn main() -> Result<(), Box<dyn Error>> {
    println!("🔗 Import Merging Demo");
    println!("======================\n");

    // Step 1: Setup source code objects
    println!("📄 Step 1: Creating source code objects...");
    let old_source = SourceCode::new(OLD_CODE);
    let new_source = SourceCode::new(NEW_CODE);
    println!("   Old code: {} lines", OLD_CODE.lines().count());
    println!("   New code: {} lines", NEW_CODE.lines().count());
    println!("   Expected: Multiple top-level import changes + module-scoped import changes");
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

    let import_changes: Vec<_> = ast_diff
        .changes
        .iter()
        .enumerate()
        .filter(|(_, change)| {
            use diffviz_core::ast_diff::ASTChange;
            match change {
                ASTChange::Addition(node) | ASTChange::Deletion(node) => {
                    node.kind() == "use_declaration"
                }
                ASTChange::ContentChange { new, .. }
                | ASTChange::StructuralChange { new, .. }
                | ASTChange::KindChange { new, .. } => new.kind() == "use_declaration",
                ASTChange::Reorder { parent, .. } => parent.kind() == "use_declaration",
            }
        })
        .collect();

    println!(
        "   🔍 Import-related changes: {} out of {}",
        import_changes.len(),
        ast_diff.changes.len()
    );
    for (i, change) in import_changes.iter() {
        println!(
            "     Import Change {}: {:?}",
            i + 1,
            get_change_summary(change)
        );
    }
    println!();

    // Step 4: Expand with semantic context and observe import merging
    println!("🎯 Step 4: Expanding changes with semantic context...");
    println!("   This is where top-level import merging happens!");

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
        "   📊 Import Merging Effect: {} total changes → {} ReviewableDiffs",
        ast_diff.changes.len(),
        reviewable_diffs.len()
    );

    // Analyze what got merged
    let top_level_import_diff = reviewable_diffs.iter().find(|diff| {
        // Check if this diff contains top-level imports
        diff.boundary.node_type == "top-level-imports"
            || (diff.boundary.node_type == "use_declaration"
                && diff.boundary.semantic_kind == diffviz_core::common::SemanticNodeKind::Import)
    });

    if let Some(import_diff) = top_level_import_diff {
        println!("   ✅ Found merged top-level imports ReviewableDiff:");
        println!(
            "      - Contains {} changes",
            import_diff.metadata.total_changes
        );
        println!(
            "      - Essential nodes: {}",
            import_diff.metadata.essential_node_count
        );
    }

    println!();

    // Step 5: Display results showing merged vs separate changes
    println!("📋 Step 5: ReviewableDiff Results (Showing Import Merging)");
    println!("===========================================================");

    if reviewable_diffs.is_empty() {
        println!("No changes detected or all changes filtered out.");
        return Ok(());
    }

    for (i, diff) in reviewable_diffs.iter().enumerate() {
        println!(
            "\n📊 ReviewableDiff {} of {} - {}",
            i + 1,
            reviewable_diffs.len(),
            if diff.boundary.node_type == "top-level-imports" {
                "🔗 MERGED TOP-LEVEL IMPORTS"
            } else {
                &diff.boundary.node_type
            }
        );

        let is_import_related =
            diff.boundary.semantic_kind == diffviz_core::common::SemanticNodeKind::Import;

        println!(
            "   📍 Boundary: {} (semantic: {:?})",
            diff.boundary.node_type, diff.boundary.semantic_kind
        );
        println!("   📈 Changes merged: {}", diff.metadata.total_changes);
        println!(
            "   📊 Essential nodes: {}",
            diff.metadata.essential_node_count
        );

        if is_import_related {
            println!("   🎯 This is an import-related change!");
            if diff.boundary.node_type == "top-level-imports" {
                println!("   ✅ SUCCESS: Multiple top-level imports were merged!");
            } else {
                println!("   📍 This appears to be a nested/module-level import (kept separate)");
            }
        }

        // Convert to RenderableDiff to show the actual code
        let renderable: RenderableDiff = diff.into();
        let essential_lines = renderable
            .lines
            .iter()
            .filter(|line| line.max_relevance() == ESSENTIAL)
            .count();

        println!(
            "   📝 Renderable: {} lines, {} essential",
            renderable.lines.len(),
            essential_lines
        );
    }

    // Step 6: Summary of import merging effectiveness
    println!("\n✅ Import Merging Analysis:");
    println!("   Original AST changes: {}", ast_diff.changes.len());
    println!("   Import-related changes: {}", import_changes.len());
    println!(
        "   Final ReviewableDiff structures: {}",
        reviewable_diffs.len()
    );

    if import_changes.len() > 1 {
        let expected_without_merging = ast_diff.changes.len();
        let actual_with_merging = reviewable_diffs.len();
        println!(
            "   📊 Consolidation: {expected_without_merging} → {actual_with_merging} structures"
        );

        if actual_with_merging < expected_without_merging {
            println!("   ✅ SUCCESS: Import merging reduced the number of review items!");
        } else {
            println!("   ⚠️  Note: May not have had multiple top-level imports to merge");
        }
    }

    println!("\n💡 Key Insight: Import merging now handles multiple scopes:");
    println!("   • Top-level imports (direct children of source_file) are grouped together");
    println!("   • Module-scoped imports are grouped by their parent module");
    println!("   • Different modules keep their imports separate for proper context");
    println!("   • Function-scoped imports remain individual (no grouping)");

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
