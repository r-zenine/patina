//! Bug: Python module-level assignments never become Variable units
//!
//! In tree-sitter-python, `X = 1` at module level parses as
//! `expression_statement → assignment`. `GenericSemanticTreeBuilder::build_typed_node`
//! classifies `expression_statement` as `Statement` and its `_ => None` arm drops the
//! node WITHOUT recursing into children — so the `("assignment", Variable)` entry in
//! PYTHON_SEMANTIC_KIND_MAP is unreachable dead code.
//!
//! Expected: a line range over module-level assignments resolves to Variable semantic
//! units and yields ReviewableDiffs.
//!
//! Actual: the semantic tree contains no node for the assignments, so
//! create_reviewable_diff_from_range fails with NoUnitsInRange.

#[cfg(test)]
mod bug_python_module_level_assignment_invisible {
    use diffviz_core::ast_diff::SourceCode;
    use diffviz_core::common::ProgrammingLanguage;
    use diffviz_core::decision_based_diff::create_reviewable_diff_from_range;
    use diffviz_core::parsers::PythonParser;

    #[test]
    #[ignore = "bug: expression_statement dropped without recursion, assignment→Variable unreachable"]
    fn range_over_module_level_constants_yields_variable_units() {
        let source = SourceCode::new("MAX_SIZE = 100\nTIMEOUT = 30\n".to_string());
        let parser = PythonParser::new();

        let result = create_reviewable_diff_from_range(
            "config.py",
            1,
            2,
            None,
            &source,
            ProgrammingLanguage::Python,
            &parser,
        );

        let diffs = result.unwrap_or_else(|e| {
            panic!("range over module-level assignments must find Variable units, got: {e:?}")
        });
        assert!(
            !diffs.is_empty(),
            "expected at least one diff for the module-level assignments"
        );
    }
}
