//! Bug: SourceCode::line_range is off by one for nodes starting at column 0
//!
//! `line_range_from_bytes` (ast_diff/source.rs) computes the start line as
//! `prefix.lines().count()`, but `str::lines()` ignores a trailing newline. When a
//! node starts at column 0 of line N (its prefix ends with '\n' — the common case for
//! top-level items), the reported start_line is N-1. Mid-line offsets are correct,
//! which is why this survives casual testing.
//!
//! This path is hit by every `RenderableDiff.overall_line_range`, because boundary
//! nodes are `OwnedNodeData` (no tree-sitter position info).
//!
//! Expected: a node starting at byte offset of line N column 0 reports start_line = N.
//!
//! Actual: it reports start_line = N-1 (e.g. 2 instead of 3).

#[cfg(test)]
mod bug_line_range_column_zero_off_by_one {
    use diffviz_core::ast_diff::{OwnedNodeData, SourceCode, SourceProvider};

    #[test]
    fn node_starting_at_column_zero_reports_correct_start_line() {
        //                             0123456789...
        let source = SourceCode::new("line one\nline two\nline three\n");

        // byte 9 = 'l' of "line two" → line 2, column 0
        let node_line2 = OwnedNodeData {
            start_byte: 9,
            end_byte: 17,
            kind: "test_node".to_string(),
            identifier: None,
        };
        let range = source.line_range(&node_line2);
        assert_eq!(
            range.start_line, 2,
            "node at column 0 of line 2 must report start_line=2"
        );

        // byte 18 = 'l' of "line three" → line 3, column 0
        let node_line3 = OwnedNodeData {
            start_byte: 18,
            end_byte: 28,
            kind: "test_node".to_string(),
            identifier: None,
        };
        let range = source.line_range(&node_line3);
        assert_eq!(
            range.start_line, 3,
            "node at column 0 of line 3 must report start_line=3"
        );
    }
}
