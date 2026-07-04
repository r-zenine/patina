//! Bug: class bodies have no semantic children — methods are invisible to range lookup
//!
//! `GenericSemanticTreeBuilder::build_data_structure` never collects children, so for
//! class-based languages (Python/TypeScript/Java/C++) every method inside a class is
//! absent from the semantic tree. The `("function_definition", Function)` /
//! `("method_definition", Function)` mappings only fire for top-level functions.
//! Rust escapes only because methods live in `impl` blocks, which are special-cased
//! as Module containers that recurse into their body.
//!
//! Expected: a line range covering one method inside a class resolves to that method
//! (a Callable unit).
//!
//! Actual: the smallest semantic unit containing the range is the whole class
//! (a DataStructure unit) — a one-method change produces a whole-class diff,
//! defeating step-by-step review for OOP languages.

#[cfg(test)]
mod bug_class_bodies_have_no_semantic_children {
    use diffviz_core::ast_diff::SourceCode;
    use diffviz_core::common::ProgrammingLanguage;
    use diffviz_core::decision_based_diff::create_reviewable_diff_from_range;
    use diffviz_core::parsers::PythonParser;

    const SOURCE: &str = r#"class Foo:
    def bar(self):
        return 1

    def baz(self):
        return 2
"#;

    #[test]
    #[ignore = "bug: build_data_structure collects no children, class methods invisible"]
    fn range_over_python_method_resolves_to_method_not_class() {
        let provider = SourceCode::new(SOURCE.to_string());
        let parser = PythonParser::new();

        // Lines 5..=6 = def baz only.
        let diffs = create_reviewable_diff_from_range(
            "foo.py",
            5,
            6,
            None,
            &provider,
            ProgrammingLanguage::Python,
            &parser,
        )
        .expect("diff creation should succeed");

        assert_eq!(diffs.len(), 1);
        assert_eq!(
            diffs[0].boundary.node_type, "Callable",
            "range over `def baz` must resolve to the method (Callable), \
             not the entire class (got {})",
            diffs[0].boundary.node_type
        );
    }
}
