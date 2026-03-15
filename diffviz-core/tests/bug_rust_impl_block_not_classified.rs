#[cfg(test)]
mod rust_impl_block_not_classified {
    use diffviz_core::decision_based_diff::create_reviewable_diff_from_range;
    use diffviz_core::{ast_diff::SourceCode, common::ProgrammingLanguage};
    use diffviz_core::parsers::rust::RustParser;

    const SOURCE: &str = r#"use std::collections::HashMap;

#[derive(Debug)]
pub struct DecisionLog {
    pub decisions: Vec<String>,
}

impl DecisionLog {
    /// Parse decisions from YAML content.
    /// Returns `Err` if the content cannot be deserialized.
    pub fn parse(content: &str) -> Result<Vec<String>, String> {
        let _ = content;
        Ok(vec![])
    }
}

pub struct OtherStruct {
    pub value: u32,
}
"#;

    #[test]
    #[ignore = "Bug: Rust impl blocks are flattened in SemanticTree — ranges covering impl header expand to entire file (see bugs.md)"]
    fn impl_block_range_should_not_expand_to_module() {
        // Lines 8-15 cover the entire `impl DecisionLog { ... }` block.
        // The expected behavior is that the expansion resolves to a boundary
        // no larger than the impl block itself (~8 lines).
        //
        // Actual behavior: Unit type = Module, expanded to entire file (100x+).
        // Root cause: `build_source_file_node()` calls `build_impl_items()` which
        // extracts only the methods as direct module children, discarding the
        // `impl_item` node from the SemanticTree entirely. Any byte range that
        // includes the impl header falls in no-man's land between SemanticNodes.

        let parser = RustParser::new();
        let new_source = Box::new(SourceCode::new(SOURCE.to_string()));

        let mut diffs = create_reviewable_diff_from_range(
            "src/example.rs",
            8,  // `impl DecisionLog {`
            15, // closing `}`
            None,
            new_source.as_ref(),
            ProgrammingLanguage::Rust,
            &parser,
        )
        .expect("should find at least one semantic unit");

        assert!(!diffs.is_empty(), "should find at least one semantic unit");
        let diff = diffs.remove(0);

        let boundary_lines = {
            use diffviz_core::reviewable_diff::NodeChangeStatus;
            match &diff.boundary.change_status {
                NodeChangeStatus::Added { node } | NodeChangeStatus::Unchanged { node } => {
                    node.end_byte - node.start_byte
                }
                NodeChangeStatus::Modified { new_node, .. } => {
                    new_node.end_byte - new_node.start_byte
                }
                _ => usize::MAX,
            }
        };

        let total_bytes = SOURCE.len();

        // The expanded unit should be nowhere near the entire file
        assert!(
            boundary_lines < total_bytes / 2,
            "Expected expansion within impl block bounds, got expansion covering most of the file"
        );
    }

    #[test]
    #[ignore = "Bug: Rust impl blocks are flattened in SemanticTree — range starting at impl header expands to entire file (see bugs.md)"]
    fn range_starting_at_impl_header_should_resolve_to_method() {
        // Even a tighter range covering just the method body (lines 9-14, inside the impl)
        // expands to the full file because the impl header bytes (line 8) are not owned
        // by any SemanticNode and the method's tree_sitter_node starts at `pub fn parse`,
        // after the doc-comment bytes which are also unowned.

        let parser = RustParser::new();
        let new_source = Box::new(SourceCode::new(SOURCE.to_string()));

        let mut diffs = create_reviewable_diff_from_range(
            "src/example.rs",
            9,  // first doc comment inside impl
            14, // closing `}` of parse function
            None,
            new_source.as_ref(),
            ProgrammingLanguage::Rust,
            &parser,
        )
        .expect("should find at least one semantic unit");

        assert!(!diffs.is_empty(), "should find at least one semantic unit");
        let diff = diffs.remove(0);

        // The boundary unit type should be Callable (the parse method), not Module
        let is_callable = diff.boundary.node_type.contains("Callable") || diff.boundary.node_type.contains("parse");
        assert!(
            is_callable,
            "Expected boundary unit to be the `parse` method (Callable), got: {}",
            diff.boundary.node_type
        );
    }
}
