//! Bug: decompose path silently drops semantic units ending on the range's end line
//!
//! `line_to_byte_offset(source, end_line)` in decision_based_diff.rs returns the byte
//! offset of the START of `end_line`, not its end. `find_contained_units_recursive`
//! requires `node_range.end <= end_byte`, so any unit whose last line IS the range's
//! end line is not "contained" and is silently omitted from the result.
//!
//! The existing test bug_decompose_path_unchanged_units.rs works around this with a
//! "load-bearing" trailing blank line in its fixture — this test makes the underlying
//! off-by-one explicit.
//!
//! Expected: a line range that fully covers N top-level units yields N ReviewableDiffs.
//!
//! Actual: the unit ending on the range's last line is dropped — a range covering
//! exactly two complete functions yields only 1 diff.

#[cfg(test)]
mod bug_range_end_line_excluded {
    use diffviz_core::ast_diff::SourceCode;
    use diffviz_core::common::ProgrammingLanguage;
    use diffviz_core::decision_based_diff::create_reviewable_diff_from_range;
    use diffviz_core::parsers::RustParser;

    // 7 lines, NO trailing blank line — line 7 is the closing `}` of fn second:
    // 1: fn first() {
    // 2:     1;
    // 3: }
    // 4: (blank)
    // 5: fn second() {
    // 6:     2;
    // 7: }
    const NEW_SOURCE: &str = "fn first() {\n    1;\n}\n\nfn second() {\n    2;\n}\n";

    #[test]
    #[ignore = "bug: end_byte = start of end_line, so units ending on the end line are dropped"]
    fn range_covering_two_functions_yields_two_diffs() {
        let new_provider = SourceCode::new(NEW_SOURCE.to_string());
        let parser = RustParser::new();

        // Lines 1..=7 cover BOTH functions completely. No old source, so both
        // units classify as Addition (no identical-content skipping involved).
        let diffs = create_reviewable_diff_from_range(
            "src/lib.rs",
            1,
            7,
            None,
            &new_provider,
            ProgrammingLanguage::Rust,
            &parser,
        )
        .expect("diff creation should succeed");

        assert_eq!(
            diffs.len(),
            2,
            "range covering both functions must yield 2 diffs — \
             fn second ends on line 7 (the range's end line) and is silently dropped"
        );
    }
}
