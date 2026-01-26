//! Test for function signature pairing bug
//!
//! This test reproduces a bug where DiffViz incorrectly classifies a modified
//! function signature as Deleted + Added instead of Modified. When a function's
//! return type or parameters change, the entire signature should be marked as
//! a single Modified change, not split into separate deletion and addition.
//!
//! Issue: Function signatures with return type changes are being shown as:
//! - Line 1: DELETED "fn read_config(path: &str) -> String"
//! - Line 1: ADDED "fn read_config(path: &str) -> Result<String, io::Error>"
//!
//! Expected: Should be detected as a single MODIFIED change.

use diffviz_core::{
    ast_diff::SourceCode,
    common::{LanguageParser, ProgrammingLanguage},
    parsers::RustParser,
    renderable_diff::RenderableDiff,
    reviewable_diff_from_semantic::semantic_pairs_to_reviewable_diffs,
    semantic_ast::build_semantic_pairs,
};

/// Test that function signature with return type change is marked as Modified
///
/// This test reproduces the bug where a simple return type change is incorrectly
/// split into separate Deleted and Added classifications instead of being
/// recognized as a single Modified change.
#[test]
fn bug_function_signature_return_type_pairing() {
    let old_code = r#"use std::fs;
use std::io;

fn read_config(path: &str) -> String {
    let file = std::fs::read_to_string(path).unwrap();
    file
}
"#;

    let new_code = r#"use std::fs;
use std::io;

fn read_config(path: &str) -> Result<String, io::Error> {
    let contents = fs::read_to_string(path)?;
    Ok(contents)
}
"#;

    let old_source = SourceCode::new(old_code);
    let new_source = SourceCode::new(new_code);

    // Parse and build semantic trees
    let parser = RustParser::new();
    let old_tree = parser
        .try_parse(old_code)
        .expect("Failed to parse old code");
    let new_tree = parser
        .try_parse(new_code)
        .expect("Failed to parse new code");

    let old_semantic = parser
        .build_semantic_tree(&old_tree, old_code)
        .expect("Failed to build old semantic tree");
    let new_semantic = parser
        .build_semantic_tree(&new_tree, new_code)
        .expect("Failed to build new semantic tree");

    // Build semantic pairs
    let semantic_pairs = build_semantic_pairs(
        &old_semantic,
        &new_semantic,
        &old_source,
        &new_source,
        &parser,
    )
    .expect("Failed to build semantic pairs");

    // Convert to reviewable diffs
    let reviewable_diffs = semantic_pairs_to_reviewable_diffs(
        &semantic_pairs,
        ProgrammingLanguage::Rust,
        &old_source,
        &new_source,
        &parser,
    );

    println!("Number of reviewable diffs: {}", reviewable_diffs.len());
    for (i, diff) in reviewable_diffs.iter().enumerate() {
        println!(
            "  {}. Type: {} (changes: {})",
            i + 1,
            diff.boundary.node_type,
            diff.metadata.total_changes
        );

        // Print the renderable lines to see the classification
        let renderable: RenderableDiff = diff.into();
        for line in &renderable.lines {
            println!(
                "    Line {}: {:?} - {}",
                line.line_number,
                line.primary_change_type(),
                line.content
            );
        }
    }

    // Should have exactly 1 reviewable diff (the function)
    assert_eq!(
        reviewable_diffs.len(),
        1,
        "Expected 1 reviewable diff for the function, got {}",
        reviewable_diffs.len()
    );

    // The function signature line should be marked as Modified, not Deleted + Added
    let diff = &reviewable_diffs[0];
    let renderable: RenderableDiff = diff.into();

    // Find the function signature lines (should contain "fn read_config")
    let signature_lines: Vec<_> = renderable
        .lines
        .iter()
        .filter(|l| l.content.contains("fn read_config"))
        .collect();

    println!(
        "\nFunction signature lines found: {}",
        signature_lines.len()
    );
    for line in &signature_lines {
        println!(
            "  Line {}: {:?}",
            line.line_number,
            line.primary_change_type()
        );
    }

    // The critical assertion: should have exactly TWO lines for the function signature
    // (one deleted, one added) showing the change was semantically paired, not split
    assert_eq!(
        signature_lines.len(),
        2,
        "Expected 2 function signature lines (old + new), but found {}",
        signature_lines.len()
    );

    use diffviz_core::renderable_diff::ChangeType;
    assert_eq!(
        signature_lines[0].primary_change_type(),
        Some(&ChangeType::Deleted),
        "Old function signature should be marked as Deleted, got {:?}",
        signature_lines[0].primary_change_type()
    );

    assert_eq!(
        signature_lines[1].primary_change_type(),
        Some(&ChangeType::Added),
        "New function signature should be marked as Added, got {:?}",
        signature_lines[1].primary_change_type()
    );
}

/// Test that body line changes are also correctly paired as Modified
///
/// Related to the above bug: when a variable assignment changes (like the `let file` line),
/// it should also be detected as a single Modified change, not split into Deleted + Added.
#[test]
fn bug_body_line_variable_assignment_pairing() {
    let old_code = r#"fn process_data(input: &str) -> Result<String, String> {
    let data = std::fs::read_to_string(input).unwrap();
    Ok(data)
}
"#;

    let new_code = r#"fn process_data(input: &str) -> Result<String, String> {
    let data = std::fs::read_to_string(input)?;
    Ok(data)
}
"#;

    let old_source = SourceCode::new(old_code);
    let new_source = SourceCode::new(new_code);

    let parser = RustParser::new();
    let old_tree = parser
        .try_parse(old_code)
        .expect("Failed to parse old code");
    let new_tree = parser
        .try_parse(new_code)
        .expect("Failed to parse new code");

    let old_semantic = parser
        .build_semantic_tree(&old_tree, old_code)
        .expect("Failed to build old semantic tree");
    let new_semantic = parser
        .build_semantic_tree(&new_tree, new_code)
        .expect("Failed to build new semantic tree");

    let semantic_pairs = build_semantic_pairs(
        &old_semantic,
        &new_semantic,
        &old_source,
        &new_source,
        &parser,
    )
    .expect("Failed to build semantic pairs");

    let reviewable_diffs = semantic_pairs_to_reviewable_diffs(
        &semantic_pairs,
        ProgrammingLanguage::Rust,
        &old_source,
        &new_source,
        &parser,
    );

    assert_eq!(
        reviewable_diffs.len(),
        1,
        "Expected 1 reviewable diff, got {}",
        reviewable_diffs.len()
    );

    let diff = &reviewable_diffs[0];
    let renderable: RenderableDiff = diff.into();

    // Find lines with "let data"
    let data_lines: Vec<_> = renderable
        .lines
        .iter()
        .filter(|l| l.content.contains("let data"))
        .collect();

    println!("Variable assignment lines found: {}", data_lines.len());
    for line in &data_lines {
        println!(
            "  Line {}: {:?} - {}",
            line.line_number,
            line.primary_change_type(),
            line.content
        );
    }

    // Should have exactly 2 lines for "let data" (old + new) showing semantic pairing
    assert_eq!(
        data_lines.len(),
        2,
        "Expected 2 'let data' lines (old + new) but found {}",
        data_lines.len()
    );

    use diffviz_core::renderable_diff::ChangeType;
    assert_eq!(
        data_lines[0].primary_change_type(),
        Some(&ChangeType::Deleted),
        "Old variable assignment should be marked as Deleted, got {:?}",
        data_lines[0].primary_change_type()
    );

    assert_eq!(
        data_lines[1].primary_change_type(),
        Some(&ChangeType::Added),
        "New variable assignment should be marked as Added, got {:?}",
        data_lines[1].primary_change_type()
    );
}
