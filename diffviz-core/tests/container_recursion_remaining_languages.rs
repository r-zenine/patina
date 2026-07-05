//! Phase 6 (plan-core-hardening): container recursion for the remaining 4 languages
//! (JavaScript, Java, C, C++), reusing the Phase 5 mechanism
//! (`container_body_field` + `is_recursable_container`) with the same per-language
//! test triple: (a) a range over a method inside a class/namespace resolves to
//! that method, not the whole container; (b) a variable-like unit resolves
//! correctly; (c) the byte-coverage/containment invariant holds.
//!
//! Java has no free-standing module-level variables (fields must live in a
//! class), so its (b) is "a field inside a class resolves to a Variable unit"
//! rather than a true module-level case.
//!
//! C++ top-level variable declarations are not classified at all today (the
//! `declaration` node kind is trivial/unmapped in `CppDescriptor`, a pre-existing
//! gap independent of container recursion) — (b) is intentionally not tested for
//! C++ here; it is not in this phase's scope (container recursion), which only
//! wires `class_specifier`/`struct_specifier`/`namespace_definition` bodies.

#[cfg(test)]
mod container_recursion_remaining_languages {
    use diffviz_core::ast_diff::SourceCode;
    use diffviz_core::common::{LanguageParser, ProgrammingLanguage};
    use diffviz_core::decision_based_diff::create_reviewable_diff_from_range;
    use diffviz_core::parsers::generic_builder::assert_byte_coverage_invariant;
    use diffviz_core::parsers::{CParser, CppParser, JavaParser, JavaScriptParser};

    // ── JavaScript: method inside class resolves to method, not class (a) ─────

    const JS_CLASS_SOURCE: &str = r#"class Foo {
    bar() {
        return 1;
    }

    baz() {
        return 2;
    }
}
"#;

    #[test]
    fn javascript_method_inside_class_resolves_to_method_not_class() {
        let provider = SourceCode::new(JS_CLASS_SOURCE.to_string());
        let parser = JavaScriptParser::new();

        // Lines 6..=8 = baz only (including its closing brace).
        let diffs = create_reviewable_diff_from_range(
            "foo.js",
            6,
            8,
            None,
            &provider,
            ProgrammingLanguage::JavaScript,
            &parser,
        )
        .expect("diff creation should succeed");

        assert_eq!(diffs.len(), 1);
        assert_eq!(
            diffs[0].boundary.node_type, "Callable",
            "range over method `baz` must resolve to the method, not the class (got {})",
            diffs[0].boundary.node_type
        );
    }

    // ── JavaScript: top-level variable visibility (b) ──────────────────────────

    #[test]
    fn javascript_top_level_const_resolves_to_variable_unit() {
        let source = SourceCode::new("const maxSize = 100;\n".to_string());
        let parser = JavaScriptParser::new();

        let diffs = create_reviewable_diff_from_range(
            "foo.js",
            1,
            1,
            None,
            &source,
            ProgrammingLanguage::JavaScript,
            &parser,
        )
        .expect("range over a top-level const must resolve to a Variable unit");

        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].boundary.node_type, "Variable");
    }

    // ── Java: method inside class resolves to method, not class (a) ───────────

    const JAVA_CLASS_SOURCE: &str = r#"class Foo {
    int bar() {
        return 1;
    }

    int baz() {
        return 2;
    }
}
"#;

    #[test]
    fn java_method_inside_class_resolves_to_method_not_class() {
        let provider = SourceCode::new(JAVA_CLASS_SOURCE.to_string());
        let parser = JavaParser::new();

        // Lines 6..=8 = baz only (including its closing brace).
        let diffs = create_reviewable_diff_from_range(
            "Foo.java",
            6,
            8,
            None,
            &provider,
            ProgrammingLanguage::Java,
            &parser,
        )
        .expect("diff creation should succeed");

        assert_eq!(diffs.len(), 1);
        assert_eq!(
            diffs[0].boundary.node_type, "Callable",
            "range over method `baz` must resolve to the method, not the class (got {})",
            diffs[0].boundary.node_type
        );
    }

    // ── Java: field inside class resolves to a Variable unit (b) ──────────────
    // Java has no module-level variables — fields must live in a class.

    #[test]
    fn java_field_inside_class_resolves_to_variable_unit() {
        let source = SourceCode::new("class Foo {\n    int maxSize = 100;\n}\n".to_string());
        let parser = JavaParser::new();

        let diffs = create_reviewable_diff_from_range(
            "Foo.java",
            2,
            2,
            None,
            &source,
            ProgrammingLanguage::Java,
            &parser,
        )
        .expect("range over a field must resolve to a Variable unit");

        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].boundary.node_type, "Variable");
    }

    // ── C: function resolution unaffected (no container recursion needed) ─────
    // C has no methods/classes; this pins that function resolution still works.

    #[test]
    fn c_top_level_function_resolves_to_function() {
        let source = SourceCode::new("int bar(void) {\n    return 1;\n}\n".to_string());
        let parser = CParser::new();

        let diffs = create_reviewable_diff_from_range(
            "foo.c",
            1,
            3,
            None,
            &source,
            ProgrammingLanguage::C,
            &parser,
        )
        .expect("range over a C function must resolve to that function");

        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].boundary.node_type, "Callable");
    }

    // ── C: top-level variable visibility (b) ───────────────────────────────────

    #[test]
    fn c_top_level_declaration_resolves_to_variable_unit() {
        let source = SourceCode::new("int maxSize = 100;\n".to_string());
        let parser = CParser::new();

        let diffs = create_reviewable_diff_from_range(
            "foo.c",
            1,
            1,
            None,
            &source,
            ProgrammingLanguage::C,
            &parser,
        )
        .expect("range over a top-level declaration must resolve to a Variable unit");

        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].boundary.node_type, "Variable");
    }

    // ── C++: method inside class resolves to method, not class (a) ────────────

    const CPP_CLASS_SOURCE: &str = r#"class Foo {
public:
    int bar() {
        return 1;
    }

    int baz() {
        return 2;
    }
};
"#;

    #[test]
    fn cpp_method_inside_class_resolves_to_method_not_class() {
        let provider = SourceCode::new(CPP_CLASS_SOURCE.to_string());
        let parser = CppParser::new();

        // Lines 7..=9 = baz only (including its closing brace).
        let diffs = create_reviewable_diff_from_range(
            "foo.cpp",
            7,
            9,
            None,
            &provider,
            ProgrammingLanguage::Cpp,
            &parser,
        )
        .expect("diff creation should succeed");

        assert_eq!(diffs.len(), 1);
        assert_eq!(
            diffs[0].boundary.node_type, "Callable",
            "range over method `baz` must resolve to the method, not the class (got {})",
            diffs[0].boundary.node_type
        );
    }

    // ── C++: function inside namespace resolves to function, not namespace ────

    const CPP_NAMESPACE_SOURCE: &str = r#"namespace ns {
int bar() {
    return 1;
}

int baz() {
    return 2;
}
}
"#;

    #[test]
    fn cpp_function_inside_namespace_resolves_to_function_not_namespace() {
        let provider = SourceCode::new(CPP_NAMESPACE_SOURCE.to_string());
        let parser = CppParser::new();

        // Lines 5..=7 = baz only (including its closing brace).
        let diffs = create_reviewable_diff_from_range(
            "foo.cpp",
            5,
            7,
            None,
            &provider,
            ProgrammingLanguage::Cpp,
            &parser,
        )
        .expect("diff creation should succeed");

        assert_eq!(diffs.len(), 1);
        assert_eq!(
            diffs[0].boundary.node_type, "Callable",
            "range over function `baz` must resolve to the function, not the namespace (got {})",
            diffs[0].boundary.node_type
        );
    }

    // ── Byte-coverage invariant (c), all 4 remaining languages ─────────────────

    #[test]
    fn byte_coverage_invariant_holds_for_remaining_languages() {
        let cases: &[(&str, &dyn LanguageParser, &str)] = &[
            ("javascript", &JavaScriptParser::new(), JS_CLASS_SOURCE),
            ("java", &JavaParser::new(), JAVA_CLASS_SOURCE),
            (
                "c",
                &CParser::new(),
                "struct Foo {\n    int x;\n};\n\nint bar(void) {\n    return 1;\n}\n\nint y = 1;\n",
            ),
            ("cpp", &CppParser::new(), CPP_NAMESPACE_SOURCE),
        ];

        for (name, parser, source) in cases {
            let ast = parser.try_parse(source).expect("parse should succeed");
            let tree = parser
                .build_semantic_tree(&ast, source)
                .unwrap_or_else(|e| panic!("{name}: build_semantic_tree failed: {e:?}"));
            assert_byte_coverage_invariant(&tree.root, source.len());
        }
    }
}
