//! Bug: Callable parameter_count counts comma tokens
//!
//! `GenericSemanticTreeBuilder::build_callable` computes
//! `parameters_node.child_count().saturating_sub(2)`, which subtracts the surrounding
//! `(` `)` but still counts the `,` separator tokens. `fn f(a: i32, b: i32)` has
//! parameters children `(` `a: i32` `,` `b: i32` `)` → 5 - 2 = 3.
//!
//! Expected: parameter_count == number of parameters (2).
//!
//! Actual: parameter_count == 3 (parameters plus commas). Should use
//! `named_child_count()` instead.

#[cfg(test)]
mod bug_parameter_count_includes_commas {
    use diffviz_core::common::LanguageParser;
    use diffviz_core::parsers::RustParser;
    use diffviz_core::semantic_ast::SemanticUnitType;

    #[test]
    fn two_parameter_function_reports_parameter_count_two() {
        let parser = RustParser::new();
        let source = "fn f(a: i32, b: i32) -> i32 { a + b }\n";
        let tree = parser.try_parse(source).expect("parse should succeed");
        let semantic_tree = parser
            .build_semantic_tree(&tree, source)
            .expect("semantic tree should build");

        let callable = semantic_tree
            .all_units()
            .into_iter()
            .find(|u| matches!(u.unit_type, SemanticUnitType::Callable { .. }))
            .expect("fn f should be a Callable unit");

        match &callable.unit_type {
            SemanticUnitType::Callable {
                parameter_count, ..
            } => {
                assert_eq!(
                    *parameter_count, 2,
                    "fn f(a, b) has 2 parameters, got {parameter_count} (commas counted)"
                );
            }
            _ => unreachable!(),
        }
    }
}
