//! Pins today's correct behavior for single-source (Added-boundary) rendering:
//! sequential line numbering, an annotation on every line (including blank ones)
//! carrying `ChangeType::Added`, and — as a direct consequence — `should_fold()`
//! is false for every line, since a wholly new unit has no line without a change
//! annotation to fold. `plan-core-hardening` must not disturb this while
//! reworking the diff engine (Phase 2) or the annotation/byte-range plumbing
//! (Phase 3).

use diffviz_core::ast_diff::SourceCode;
use diffviz_core::common::ProgrammingLanguage;
use diffviz_core::decision_based_diff::create_reviewable_diff_from_range;
use diffviz_core::parsers::RustParser;
use diffviz_core::renderable_diff::{ChangeType, RenderableDiff};

#[test]
fn added_boundary_lines_are_numbered_annotated_and_never_fold() {
    // A brand-new top-level function (old_source = None => Addition), with a
    // blank line in the body so the fixture can prove blank lines still carry
    // an annotation instead of being silently dropped.
    let source = "fn greet() {\n    let name = \"world\";\n\n    println!(\"{}\", name);\n}\n";
    let new_source = SourceCode::new(source.to_string());
    let parser = RustParser::new();

    let diffs = create_reviewable_diff_from_range(
        "src/lib.rs",
        1,
        5,
        None,
        &new_source,
        ProgrammingLanguage::Rust,
        &parser,
    )
    .expect("diff creation should succeed");

    assert_eq!(diffs.len(), 1, "a single new top-level fn is one diff");
    let renderable = RenderableDiff::try_from(&diffs[0]).expect("rendering should succeed");

    let expected_lines: Vec<&str> = source.lines().collect();
    assert_eq!(renderable.lines.len(), expected_lines.len());

    for (i, (line, expected_content)) in renderable
        .lines
        .iter()
        .zip(expected_lines.iter())
        .enumerate()
    {
        let expected_line_number = i + 1;
        assert_eq!(
            line.line_number, expected_line_number,
            "line numbering must be sequential starting at 1"
        );
        assert_eq!(&line.content, expected_content);
        assert!(
            !line.annotations.is_empty(),
            "line {expected_line_number} ({:?}) must carry an annotation even if blank",
            line.content
        );
        assert_eq!(
            line.primary_change_type(),
            Some(&ChangeType::Added),
            "every line of a wholly new boundary is Added"
        );
        assert!(
            !line.should_fold(),
            "line {expected_line_number} must not fold: it always has an Added annotation"
        );
    }
}
