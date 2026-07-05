//! Bug: CRLF line endings cause byte-offset drift in rendered lines
//!
//! `split_into_lines_with_positions` (renderable_diff/line_utils.rs) and the Modified
//! rendering path advance byte offsets by `line.len() + 1`, but `str::lines()` strips
//! BOTH `\r` and `\n`. On CRLF sources every line after the first is attributed a byte
//! range one byte earlier per preceding line, so annotation-to-line mapping (relevance,
//! change highlighting) drifts progressively.
//!
//! Expected: each RenderableLine's byte_range starts at the actual byte offset of that
//! line within the boundary source.
//!
//! Actual: line N's byte_range is short by N-1 bytes on CRLF sources.

#[cfg(test)]
mod bug_crlf_byte_offset_drift {
    use diffviz_core::ast_diff::SourceCode;
    use diffviz_core::common::ProgrammingLanguage;
    use diffviz_core::decision_based_diff::create_reviewable_diff_from_range;
    use diffviz_core::parsers::RustParser;
    use diffviz_core::renderable_diff::RenderableDiff;

    #[test]
    fn crlf_source_line_byte_ranges_match_actual_offsets() {
        // "fn f() {\r\n    1;\r\n}\r\n"
        //  line 1 starts at byte 0, line 2 at byte 10, line 3 at byte 18
        let source = "fn f() {\r\n    1;\r\n}\r\n";
        let provider = SourceCode::new(source.to_string());
        let parser = RustParser::new();

        // No old source → Added boundary → single-source rendering path.
        let diffs = create_reviewable_diff_from_range(
            "src/lib.rs",
            1,
            3,
            None,
            &provider,
            ProgrammingLanguage::Rust,
            &parser,
        )
        .expect("diff creation should succeed");

        assert_eq!(diffs.len(), 1);
        let renderable = RenderableDiff::try_from(&diffs[0]).expect("rendering should succeed");

        assert!(renderable.lines.len() >= 3, "expected 3 rendered lines");

        // Byte ranges are relative to the boundary source, which starts at byte 0 here.
        assert_eq!(
            renderable.lines[1].byte_range.start, 10,
            "line 2 ('    1;') starts at byte 10 in CRLF source, got {}",
            renderable.lines[1].byte_range.start
        );
        assert_eq!(
            renderable.lines[2].byte_range.start, 18,
            "line 3 ('}}') starts at byte 18 in CRLF source, got {}",
            renderable.lines[2].byte_range.start
        );
    }
}
