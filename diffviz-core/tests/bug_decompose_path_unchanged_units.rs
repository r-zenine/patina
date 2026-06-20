//! Bug: decompose path includes unchanged semantic units as empty diffs
//!
//! When a decision line range spans multiple top-level items (triggering the
//! decompose path in create_reviewable_diff_from_range), find_contained_units_recursive
//! collects ALL semantic units within the byte range — including units that were NOT
//! changed between old and new source. Those unchanged units are classified as
//! Modification (because they exist in both trees), and Myers diff on identical
//! content produces all DiffOp::Keep ops, resulting in a ReviewableDiff whose
//! RenderableDiff has zero changed lines — an empty diff that pollutes the TUI.
//!
//! Expected: create_reviewable_diff_from_range returns only diffs for units that
//! have actual content changes. Unchanged units within the range must be omitted.
//!
//! Actual: an extra ReviewableDiff is returned for the unchanged unit, and its
//! RenderableDiff::has_changes() returns false.

#[cfg(test)]
mod bug_decompose_path_unchanged_units {
    use diffviz_core::ast_diff::SourceCode;
    use diffviz_core::common::ProgrammingLanguage;
    use diffviz_core::decision_based_diff::create_reviewable_diff_from_range;
    use diffviz_core::parsers::RustParser;
    use diffviz_core::renderable_diff::RenderableDiff;

    // old_source (8 lines, trailing blank line ensures end_byte covers both functions):
    // 1: fn changed_fn() -> i32 {
    // 2:     1
    // 3: }
    // 4:
    // 5: fn stable_fn() -> i32 {
    // 6:     42
    // 7: }
    // 8: (blank)
    //
    // The trailing blank line is load-bearing: line_to_byte_offset(source, 8) returns the
    // byte offset of the blank line, which is beyond the closing `}` of fn stable_fn.
    // Without it, end_byte = start-of-line-7 = byte of the `}` itself, so fn stable_fn
    // is not "fully contained" in [start, end) and the decompose path skips it.
    const OLD_SOURCE: &str =
        "fn changed_fn() -> i32 {\n    1\n}\n\nfn stable_fn() -> i32 {\n    42\n}\n\n";

    // new_source: only changed_fn is modified (1 → 100); stable_fn is byte-for-byte identical.
    const NEW_SOURCE: &str =
        "fn changed_fn() -> i32 {\n    100\n}\n\nfn stable_fn() -> i32 {\n    42\n}\n\n";

    #[test]
    fn test_decompose_path_does_not_produce_empty_diffs_for_unchanged_units() {
        let old_provider = SourceCode::new(OLD_SOURCE.to_string());
        let new_provider = SourceCode::new(NEW_SOURCE.to_string());
        let parser = RustParser::new();

        // Range covering both functions (lines 1–8, where line 8 is the trailing blank).
        // Because the range spans two top-level items, find_unit_recursive returns the
        // Module root → decompose path collects both fn changed_fn and fn stable_fn.
        let result = create_reviewable_diff_from_range(
            "src/lib.rs",
            1,
            8,
            Some(&old_provider),
            &new_provider,
            ProgrammingLanguage::Rust,
            &parser,
        );

        assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
        let diffs = result.unwrap();

        assert!(
            !diffs.is_empty(),
            "Expected at least one diff for the changed function"
        );

        // Every returned diff must contain at least one changed line.
        // Under the current bug, stable_fn is returned as a Modification with no
        // changed lines — changed_line_count() returns 0 for that diff.
        for diff in &diffs {
            let renderable =
                RenderableDiff::try_from(diff).expect("RenderableDiff conversion should succeed");
            assert!(
                renderable.changed_line_count() > 0,
                "Diff for '{}' has no changed lines — unchanged units must not be included \
                 in the decompose path output",
                renderable.metadata.boundary_name
            );
        }
    }

    #[test]
    fn test_decompose_path_returns_only_changed_unit_not_unchanged_sibling() {
        let old_provider = SourceCode::new(OLD_SOURCE.to_string());
        let new_provider = SourceCode::new(NEW_SOURCE.to_string());
        let parser = RustParser::new();

        // Range 1–8 fully encloses both functions (line 8 is the trailing blank).
        let result = create_reviewable_diff_from_range(
            "src/lib.rs",
            1,
            8,
            Some(&old_provider),
            &new_provider,
            ProgrammingLanguage::Rust,
            &parser,
        );

        let diffs = result.expect("Expected Ok");

        // Only fn changed_fn has actual changes — stable_fn must not appear.
        assert_eq!(
            diffs.len(),
            1,
            "Expected exactly 1 diff (for changed_fn only), got {} — \
             decompose path must skip unchanged siblings",
            diffs.len()
        );
    }
}
