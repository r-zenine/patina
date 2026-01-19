//! Integration tests for semantic Myers diff algorithm through the full pipeline

use diffviz_core::{
    LanguageParser,
    ast_diff::{LineRange, SourceError, SourceProvider},
    common::ProgrammingLanguage,
    parsers::rust::RustParser,
    renderable_diff::RenderableDiff,
    reviewable_diff_from_semantic::semantic_pairs_to_reviewable_diffs,
    semantic_ast::build_semantic_pairs,
};

#[test]
fn test_semantic_anchors_in_full_pipeline() {
    let old_code = r#"
fn calculate_total(items: Vec<Item>) -> i32 {
    let mut total = 0;
    for item in items {
        total += item.price;
    }
    total
}

fn process_data(data: &str) {
    println!("Processing: {}", data);
}
"#;

    let new_code = r#"
fn calculate_total(items: Vec<Item>, discount: f32) -> i32 {
    let mut total = 0;
    for item in items {
        total += item.price;
    }
    (total as f32 * (1.0 - discount)) as i32
}

fn process_data(data: &str) {
    println!("Processing data: {}", data);
}
"#;

    // Parse the code
    let parser = RustParser::new();
    let old_tree = parser
        .try_parse(old_code)
        .expect("Failed to parse old code");
    let new_tree = parser
        .try_parse(new_code)
        .expect("Failed to parse new code");

    // Build semantic trees
    let old_semantic = parser
        .build_semantic_tree(&old_tree, old_code)
        .expect("Failed to build old semantic tree");
    let new_semantic = parser
        .build_semantic_tree(&new_tree, new_code)
        .expect("Failed to build new semantic tree");

    #[derive(Clone)]
    struct SimpleSource {
        content: String,
    }

    impl SourceProvider for SimpleSource {
        fn node_text(
            &self,
            node: &dyn diffviz_core::ast_diff::NodeLike,
        ) -> Result<&str, SourceError> {
            let start = node.start_byte();
            let end = node.end_byte();
            Ok(&self.content.as_str()[start..end])
        }

        fn line_range(&self, node: &dyn diffviz_core::ast_diff::NodeLike) -> LineRange {
            // Try to use TreeSitter's position info if available
            if let Some(ts_node) = node.as_tree_sitter_node() {
                let start_pos = ts_node.start_position();
                let end_pos = ts_node.end_position();
                LineRange {
                    start_line: start_pos.row + 1,
                    end_line: end_pos.row + 1,
                    start_column: start_pos.column,
                    end_column: end_pos.column,
                }
            } else {
                // Calculate from byte positions for OwnedNodeData
                let content_str = &self.content;
                let start_byte = node.start_byte();
                let end_byte = node.end_byte();

                // Find start position
                let start_pos = content_str[..start_byte.min(content_str.len())]
                    .lines()
                    .count();
                let start_line = if start_byte == 0 { 1 } else { start_pos };
                let start_column = content_str[..start_byte.min(content_str.len())]
                    .rfind('\n')
                    .map(|pos| start_byte - pos - 1)
                    .unwrap_or(start_byte);

                // Find end position
                let end_content = &content_str[..end_byte.min(content_str.len())];
                let end_pos = end_content.lines().count();
                let end_line = if end_byte == 0 { 1 } else { end_pos };
                let end_column = end_content
                    .rfind('\n')
                    .map(|pos| end_byte - pos - 1)
                    .unwrap_or(end_byte);

                LineRange {
                    start_line,
                    end_line,
                    start_column,
                    end_column,
                }
            }
        }

        fn clone_box(&self) -> Box<dyn diffviz_core::SourceProvider> {
            Box::new(self.clone())
        }
    }

    let old_source = SimpleSource {
        content: old_code.to_string(),
    };
    let new_source = SimpleSource {
        content: new_code.to_string(),
    };

    // Build semantic pairs
    let pairs = build_semantic_pairs(
        &old_semantic,
        &new_semantic,
        &old_source,
        &new_source,
        &parser,
    )
    .expect("Failed to build semantic pairs");

    let reviewable_diffs = semantic_pairs_to_reviewable_diffs(
        &pairs,
        ProgrammingLanguage::Rust,
        &old_source,
        &new_source,
    );

    // Convert to renderable diffs and check semantic anchors across all diffs
    let mut total_anchored_lines = 0;
    let mut total_function_lines_with_anchors = 0;

    for reviewable in &reviewable_diffs {
        let renderable = RenderableDiff::from(reviewable);

        // Count anchored lines across all diffs
        let anchored_lines = renderable
            .lines
            .iter()
            .filter(|line| line.semantic_anchor.is_some())
            .count();
        total_anchored_lines += anchored_lines;

        // Count function signature lines with anchors
        let function_lines_with_anchors = renderable
            .lines
            .iter()
            .filter(|line| line.content.trim().starts_with("fn calculate_total"))
            .filter(|line| line.semantic_anchor.is_some())
            .count();
        total_function_lines_with_anchors += function_lines_with_anchors;
    }

    // We should have semantic anchors overall
    assert!(
        total_anchored_lines > 0,
        "Should have extracted semantic anchors"
    );

    // We should have at least one function signature with anchor
    assert!(
        total_function_lines_with_anchors > 0,
        "At least one function signature should have semantic anchor"
    );
}

#[test]
fn test_variable_assignment_anchors() {
    let old_code = r#"
fn main() {
    let config = Config::new();
    let config = config.with_timeout(30);
    let config = config.with_retries(3);
    config.run();
}
"#;

    let new_code = r#"
fn main() {
    let config = Config::builder();
    let config = config.timeout(60);
    let config = config.retries(5).build();
    config.run();
}
"#;

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

    #[derive(Clone)]
    struct SimpleSource {
        content: String,
    }

    impl SourceProvider for SimpleSource {
        fn node_text(
            &self,
            node: &dyn diffviz_core::ast_diff::NodeLike,
        ) -> Result<&str, SourceError> {
            let start = node.start_byte();
            let end = node.end_byte();
            Ok(&self.content.as_str()[start..end])
        }

        fn line_range(&self, node: &dyn diffviz_core::ast_diff::NodeLike) -> LineRange {
            // Try to use TreeSitter's position info if available
            if let Some(ts_node) = node.as_tree_sitter_node() {
                let start_pos = ts_node.start_position();
                let end_pos = ts_node.end_position();
                LineRange {
                    start_line: start_pos.row + 1,
                    end_line: end_pos.row + 1,
                    start_column: start_pos.column,
                    end_column: end_pos.column,
                }
            } else {
                // Calculate from byte positions for OwnedNodeData
                let content_str = &self.content;
                let start_byte = node.start_byte();
                let end_byte = node.end_byte();

                // Find start position
                let start_pos = content_str[..start_byte.min(content_str.len())]
                    .lines()
                    .count();
                let start_line = if start_byte == 0 { 1 } else { start_pos };
                let start_column = content_str[..start_byte.min(content_str.len())]
                    .rfind('\n')
                    .map(|pos| start_byte - pos - 1)
                    .unwrap_or(start_byte);

                // Find end position
                let end_content = &content_str[..end_byte.min(content_str.len())];
                let end_pos = end_content.lines().count();
                let end_line = if end_byte == 0 { 1 } else { end_pos };
                let end_column = end_content
                    .rfind('\n')
                    .map(|pos| end_byte - pos - 1)
                    .unwrap_or(end_byte);

                LineRange {
                    start_line,
                    end_line,
                    start_column,
                    end_column,
                }
            }
        }

        fn clone_box(&self) -> Box<dyn diffviz_core::SourceProvider> {
            Box::new(self.clone())
        }
    }

    let old_source = SimpleSource {
        content: old_code.to_string(),
    };
    let new_source = SimpleSource {
        content: new_code.to_string(),
    };

    let pairs = build_semantic_pairs(
        &old_semantic,
        &new_semantic,
        &old_source,
        &new_source,
        &parser,
    )
    .expect("Failed to build semantic pairs");

    let reviewable_diffs = semantic_pairs_to_reviewable_diffs(
        &pairs,
        ProgrammingLanguage::Rust,
        &old_source,
        &new_source,
    );

    for reviewable in &reviewable_diffs {
        let renderable = RenderableDiff::from(reviewable);

        // Check that config assignment lines have semantic anchors
        for line in &renderable.lines {
            if line.content.trim().starts_with("let config") {
                assert!(
                    line.semantic_anchor.is_some(),
                    "Variable assignment should have semantic anchor for: {}",
                    line.content
                );
            }
        }
    }
}

#[test]
fn test_function_signature_changes_are_shown() {
    let old_code = r#"
fn make_request(&self, method: &str, url: &str, body: Option<String>) -> Result<String, HttpError> {
    println!("Making {} request to: {}", method, url);
    Ok("response".to_string())
}
"#;

    let new_code = r#"
async fn make_request(&mut self, method: &str, url: &str, body: Option<String>) -> Result<String, HttpError> {
    self.request_count += 1;
    println!("Making async {} request to: {}", method, url);
    Ok("response".to_string())
}
"#;

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

    #[derive(Clone)]
    struct SimpleSource {
        content: String,
    }

    impl SourceProvider for SimpleSource {
        fn node_text(
            &self,
            node: &dyn diffviz_core::ast_diff::NodeLike,
        ) -> Result<&str, SourceError> {
            let start = node.start_byte();
            let end = node.end_byte();
            Ok(&self.content.as_str()[start..end])
        }

        fn line_range(&self, node: &dyn diffviz_core::ast_diff::NodeLike) -> LineRange {
            // Try to use TreeSitter's position info if available
            if let Some(ts_node) = node.as_tree_sitter_node() {
                let start_pos = ts_node.start_position();
                let end_pos = ts_node.end_position();
                LineRange {
                    start_line: start_pos.row + 1,
                    end_line: end_pos.row + 1,
                    start_column: start_pos.column,
                    end_column: end_pos.column,
                }
            } else {
                // Calculate from byte positions for OwnedNodeData
                let content_str = &self.content;
                let start_byte = node.start_byte();
                let end_byte = node.end_byte();

                // Find start position
                let start_pos = content_str[..start_byte.min(content_str.len())]
                    .lines()
                    .count();
                let start_line = if start_byte == 0 { 1 } else { start_pos };
                let start_column = content_str[..start_byte.min(content_str.len())]
                    .rfind('\n')
                    .map(|pos| start_byte - pos - 1)
                    .unwrap_or(start_byte);

                // Find end position
                let end_content = &content_str[..end_byte.min(content_str.len())];
                let end_pos = end_content.lines().count();
                let end_line = if end_byte == 0 { 1 } else { end_pos };
                let end_column = end_content
                    .rfind('\n')
                    .map(|pos| end_byte - pos - 1)
                    .unwrap_or(end_byte);

                LineRange {
                    start_line,
                    end_line,
                    start_column,
                    end_column,
                }
            }
        }

        fn clone_box(&self) -> Box<dyn diffviz_core::SourceProvider> {
            Box::new(self.clone())
        }
    }

    let old_source = SimpleSource {
        content: old_code.to_string(),
    };
    let new_source = SimpleSource {
        content: new_code.to_string(),
    };

    let pairs = build_semantic_pairs(
        &old_semantic,
        &new_semantic,
        &old_source,
        &new_source,
        &parser,
    )
    .expect("Failed to build semantic pairs");

    let reviewable_diffs = semantic_pairs_to_reviewable_diffs(
        &pairs,
        ProgrammingLanguage::Rust,
        &old_source,
        &new_source,
    );

    // Convert to renderable diffs and verify signature changes are shown
    // Check across all diffs combined for both old and new signatures
    let mut found_old_signature = false;
    let mut found_new_signature = false;

    for reviewable in &reviewable_diffs {
        let renderable = RenderableDiff::from(reviewable);

        for line in &renderable.lines {
            if line.content.contains("fn make_request(&self") {
                found_old_signature = true;
            }
            if line.content.contains("async fn make_request(&mut self") {
                found_new_signature = true;
            }
        }
    }

    // We should see both the old signature (deleted) and new signature (added) across all diffs
    assert!(
        found_old_signature && found_new_signature,
        "Function signature changes should be shown as both delete and add operations across all diffs. \
        Found old: {found_old_signature}, Found new: {found_new_signature}"
    );
}
