//! Bug: RenderableDiff mixes two line-number coordinate systems
//!
//! Lines produced by the Modified rendering path are numbered 1..n relative to the
//! boundary (create_line_by_line_diff_for_modified restarts `line_number` at 1), and
//! `changed_line_numbers` stores those relative numbers — while
//! `metadata.overall_line_range` is file-absolute. Consumers correlating the two
//! (e.g. "which file lines changed?") mix frames and point at the wrong lines.
//!
//! Expected: every entry of changed_line_numbers() falls within overall_line_range
//! (both file-absolute).
//!
//! Actual: for a function starting at file line 5, changed_line_numbers contains
//! small relative values (e.g. 2) outside overall_line_range.

#[cfg(test)]
mod bug_mixed_line_number_coordinate_systems {
    use diffviz_core::ast_diff::SourceCode;
    use diffviz_core::common::ProgrammingLanguage;
    use diffviz_core::decision_based_diff::create_reviewable_diff_from_range;
    use diffviz_core::parsers::RustParser;
    use diffviz_core::renderable_diff::RenderableDiff;

    const OLD_SOURCE: &str = "fn pad_one() {\n    0;\n}\n\nfn target() {\n    1;\n}\n";
    const NEW_SOURCE: &str = "fn pad_one() {\n    0;\n}\n\nfn target() {\n    2;\n}\n";

    #[test]
    #[ignore = "bug: changed_line_numbers are boundary-relative, overall_line_range is file-absolute"]
    fn changed_line_numbers_fall_within_overall_line_range() {
        let old_provider = SourceCode::new(OLD_SOURCE.to_string());
        let new_provider = SourceCode::new(NEW_SOURCE.to_string());
        let parser = RustParser::new();

        // Lines 5..=6 = fn target, which starts at file line 5.
        let diffs = create_reviewable_diff_from_range(
            "src/lib.rs",
            5,
            6,
            Some(&old_provider),
            &new_provider,
            ProgrammingLanguage::Rust,
            &parser,
        )
        .expect("diff creation should succeed");

        assert_eq!(diffs.len(), 1);
        let renderable = RenderableDiff::try_from(&diffs[0]).expect("rendering should succeed");

        let range = renderable.line_range();
        assert!(
            !renderable.changed_line_numbers().is_empty(),
            "the 1 -> 2 change must produce changed lines"
        );
        for &line in renderable.changed_line_numbers() {
            assert!(
                (range.start_line..=range.end_line).contains(&line),
                "changed line {} is outside overall_line_range {}..={} — \
                 relative vs absolute coordinate mismatch",
                line,
                range.start_line,
                range.end_line
            );
        }
    }
}
