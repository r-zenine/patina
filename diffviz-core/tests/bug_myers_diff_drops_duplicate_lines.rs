//! Bug: Myers diff drops lines when one old line equals several consecutive new lines
//!
//! In `shortest_edit_script_semantic` (renderable_diff/myers_diff.rs), the greedy
//! "snake" loop advances only `y` while `x` stays fixed, comparing the SAME old line
//! against successive new lines. Correct Myers extends diagonally (`x += 1; y += 1`
//! while `a[x] == b[y]`). Whenever an old line matches several consecutive new lines
//! (duplicate blank lines, repeated statements, repeated `}` lines), the recorded
//! diagonal run does not correspond to actually-equal lines, and the backtrack
//! produces an invalid edit script that silently drops lines.
//!
//! Expected: the rendered diff reconstructs both sources exactly — filtering out
//! Deleted lines yields the new text, filtering out Added lines yields the old text.
//!
//! Actual: lines vanish from the output. E.g. duplicating a statement `x();` twice
//! renders a diff missing `fn f() {` and showing no additions at all; inserting a
//! blank line next to an existing blank line renders `    a();` as deleted and never
//! shows the addition.

#[cfg(test)]
mod bug_myers_diff_drops_duplicate_lines {
    use diffviz_core::ast_diff::SourceCode;
    use diffviz_core::common::ProgrammingLanguage;
    use diffviz_core::decision_based_diff::create_reviewable_diff_from_range;
    use diffviz_core::parsers::RustParser;
    use diffviz_core::renderable_diff::{ChangeType, RenderableDiff, RenderableLine};

    /// Reconstruct the new text from rendered lines (everything except Deleted lines).
    fn reconstruct_new<'a>(lines: &'a [RenderableLine<'a>]) -> Vec<&'a str> {
        lines
            .iter()
            .filter(|l| !matches!(l.primary_change_type(), Some(ChangeType::Deleted)))
            .map(|l| l.content)
            .collect()
    }

    /// Reconstruct the old text from rendered lines (everything except Added lines).
    fn reconstruct_old<'a>(lines: &'a [RenderableLine<'a>]) -> Vec<&'a str> {
        lines
            .iter()
            .filter(|l| !matches!(l.primary_change_type(), Some(ChangeType::Added)))
            .map(|l| l.content)
            .collect()
    }

    fn render_modification<'a>(
        old_source: &'a SourceCode,
        new_source: &'a SourceCode,
    ) -> Vec<diffviz_core::ReviewableDiff> {
        let parser = RustParser::new();
        create_reviewable_diff_from_range(
            "src/lib.rs",
            1,
            2,
            Some(old_source),
            new_source,
            ProgrammingLanguage::Rust,
            &parser,
        )
        .expect("diff creation should succeed")
    }

    // Bug: repeated statement insertion loses lines.
    // Expected rendered new text: fn f() { / x(); / x(); / x(); / }
    // Actual: only [`    x();`, `}`] — `fn f() {` and both added lines are dropped.
    #[test]
    fn repeated_statement_insertion_reconstructs_both_sources() {
        let old = SourceCode::new("fn f() {\n    x();\n}\n");
        let new = SourceCode::new("fn f() {\n    x();\n    x();\n    x();\n}\n");

        let diffs = render_modification(&old, &new);
        assert_eq!(diffs.len(), 1);
        let renderable = RenderableDiff::try_from(&diffs[0]).expect("rendering should succeed");

        assert_eq!(
            reconstruct_new(&renderable.lines),
            vec!["fn f() {", "    x();", "    x();", "    x();", "}"],
            "non-deleted rendered lines must reconstruct the new source"
        );
        assert_eq!(
            reconstruct_old(&renderable.lines),
            vec!["fn f() {", "    x();", "}"],
            "non-added rendered lines must reconstruct the old source"
        );
    }

    // Bug: inserting a blank line next to an existing blank line corrupts the diff.
    // Expected: one Added blank line, everything else Kept.
    // Actual: `    a();` is rendered as deleted and the added blank line never appears.
    #[test]
    fn blank_line_insertion_reconstructs_both_sources() {
        let old = SourceCode::new("fn f() {\n    a();\n\n    b();\n}\n");
        let new = SourceCode::new("fn f() {\n    a();\n\n\n    b();\n}\n");

        let diffs = render_modification(&old, &new);
        assert_eq!(diffs.len(), 1);
        let renderable = RenderableDiff::try_from(&diffs[0]).expect("rendering should succeed");

        assert_eq!(
            reconstruct_new(&renderable.lines),
            vec!["fn f() {", "    a();", "", "", "    b();", "}"],
            "non-deleted rendered lines must reconstruct the new source"
        );
        assert_eq!(
            reconstruct_old(&renderable.lines),
            vec!["fn f() {", "    a();", "", "    b();", "}"],
            "non-added rendered lines must reconstruct the old source"
        );
    }
}
