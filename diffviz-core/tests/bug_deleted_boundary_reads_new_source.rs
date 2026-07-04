//! Bug: Deleted boundaries read the wrong source in name extraction and line ranges
//!
//! `extract_boundary_name` (renderable_diff/name_extractors.rs) and the
//! `overall_line_range` computation in `RenderableDiff::try_from` always read from
//! `new_source`, even when the boundary is `Deleted` and its byte ranges belong to the
//! OLD file. `line_utils::get_display_node_with_source` handles this correctly for
//! content extraction; the other two call sites don't.
//!
//! Currently a landmine rather than a live bug — `ChangeClassification::Deletion` is
//! never constructed by `create_reviewable_diff_from_range` — but any code path that
//! builds a Deleted boundary (as the review layer legitimately may) gets a garbage
//! name and, if the old byte range exceeds the new source, a nonsense line range.
//!
//! Expected: for a Deleted function boundary, boundary_name is the deleted function's
//! name, extracted from old_source.
//!
//! Actual: node_text against new_source fails (or reads unrelated bytes) and the name
//! falls back to the debug string of the semantic kind ("function").

#[cfg(test)]
mod bug_deleted_boundary_reads_new_source {
    use diffviz_core::ast_diff::{OwnedNodeData, SourceCode};
    use diffviz_core::common::{ProgrammingLanguage, SemanticNodeKind};
    use diffviz_core::renderable_diff::RenderableDiff;
    use diffviz_core::reviewable_diff::{DiffMetadata, DiffNode, NodeChangeStatus, ReviewableDiff};
    use std::collections::HashMap;

    #[test]
    #[ignore = "bug: extract_boundary_name/overall_line_range use new_source for Deleted nodes"]
    fn deleted_function_boundary_name_comes_from_old_source() {
        let old_source = "fn deleted_one() {\n    1;\n}\n";
        let new_source = ""; // the function was deleted; new file is empty

        let boundary = DiffNode {
            node_type: "Callable".to_string(),
            semantic_kind: SemanticNodeKind::Function,
            change_status: NodeChangeStatus::Deleted {
                node: OwnedNodeData {
                    start_byte: 0,
                    end_byte: old_source.len(),
                    kind: "function_item".to_string(),
                    identifier: Some("deleted_one".to_string()),
                },
            },
            relevance: 0,
            children: Vec::new(),
        };

        let reviewable = ReviewableDiff {
            language: ProgrammingLanguage::Rust,
            boundary,
            old_source: Box::new(SourceCode::new(old_source.to_string())),
            new_source: Box::new(SourceCode::new(new_source.to_string())),
            metadata: DiffMetadata {
                total_changes: 1,
                change_summary: HashMap::new(),
                essential_node_count: 1,
                analysis_duration_ms: 0,
            },
        };

        let renderable = RenderableDiff::try_from(&reviewable).expect("rendering should succeed");

        assert_eq!(
            renderable.metadata.boundary_name, "deleted_one",
            "boundary name of a Deleted function must come from old_source"
        );
    }
}
