//! Phase 5 (plan-core-hardening): container recursion mechanism + core 4 languages.
//!
//! Per-language triple: (a) a range over a method inside a class/impl resolves to
//! that method, not the whole container; (b) module-level variable ranges resolve
//! to Variable units; (c) the byte-coverage/containment invariant holds.
//!
//! Python's (a)/(b) are pinned by `bug_class_bodies_have_no_semantic_children.rs`
//! and `bug_python_module_level_assignment_invisible.rs` respectively — not
//! duplicated here. Rust's (a) (impl method resolution) is pinned by Phase-0/
//! pre-existing decompose tests — not duplicated here.

#[cfg(test)]
mod container_recursion_core_languages {
    use diffviz_core::ast_diff::SourceCode;
    use diffviz_core::common::{LanguageParser, ProgrammingLanguage};
    use diffviz_core::decision_based_diff::create_reviewable_diff_from_range;
    use diffviz_core::parsers::generic_builder::assert_byte_coverage_invariant;
    use diffviz_core::parsers::{GoParser, PythonParser, RustParser, TypeScriptParser};

    // ── Rust: module-level variable visibility (b) ─────────────────────────────

    #[test]
    fn rust_module_level_const_resolves_to_variable_unit() {
        let source = SourceCode::new("const MAX_SIZE: u32 = 100;\n".to_string());
        let parser = RustParser::new();

        let diffs = create_reviewable_diff_from_range(
            "lib.rs",
            1,
            1,
            None,
            &source,
            ProgrammingLanguage::Rust,
            &parser,
        )
        .expect("range over a module-level const must resolve to a Variable unit");

        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].boundary.node_type, "Variable");
    }

    // ── Go: method-with-receiver resolution (a) unaffected by this mechanism ───
    // Go has no class bodies — methods are top-level with a receiver — so there is
    // no container to recurse into. This pins that method resolution still works.

    #[test]
    fn go_method_with_receiver_resolves_to_method() {
        let source = SourceCode::new(
            "package main\n\ntype Foo struct {\n\tX int\n}\n\nfunc (f Foo) Bar() int {\n\treturn 1\n}\n"
                .to_string(),
        );
        let parser = GoParser::new();

        // Lines 7..=9 = the Bar method body.
        let diffs = create_reviewable_diff_from_range(
            "main.go",
            7,
            9,
            None,
            &source,
            ProgrammingLanguage::Go,
            &parser,
        )
        .expect("range over a Go method must resolve to that method");

        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].boundary.node_type, "Callable");
    }

    // ── Go: module-level variable visibility (b) ───────────────────────────────

    #[test]
    fn go_package_level_var_resolves_to_variable_unit() {
        let source = SourceCode::new("package main\n\nvar MaxSize = 100\n".to_string());
        let parser = GoParser::new();

        let diffs = create_reviewable_diff_from_range(
            "main.go",
            3,
            3,
            None,
            &source,
            ProgrammingLanguage::Go,
            &parser,
        )
        .expect("range over a package-level var must resolve to a Variable unit");

        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].boundary.node_type, "Variable");
    }

    // ── TypeScript: method inside class resolves to method, not class (a) ─────
    // New capability this phase: class_declaration now recurses into its body via
    // container_body_field, so a range over one method no longer expands to the
    // whole class.

    const TS_CLASS_SOURCE: &str = r#"class Foo {
    bar(): number {
        return 1;
    }

    baz(): number {
        return 2;
    }
}
"#;

    #[test]
    fn typescript_method_inside_class_resolves_to_method_not_class() {
        let provider = SourceCode::new(TS_CLASS_SOURCE.to_string());
        let parser = TypeScriptParser::new();

        // Lines 6..=8 = baz only (including its closing brace).
        let diffs = create_reviewable_diff_from_range(
            "foo.ts",
            6,
            8,
            None,
            &provider,
            ProgrammingLanguage::TypeScript,
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

    // ── TypeScript: module-level variable visibility (b) ───────────────────────

    #[test]
    fn typescript_top_level_const_resolves_to_variable_unit() {
        let source = SourceCode::new("const maxSize = 100;\n".to_string());
        let parser = TypeScriptParser::new();

        let diffs = create_reviewable_diff_from_range(
            "foo.ts",
            1,
            1,
            None,
            &source,
            ProgrammingLanguage::TypeScript,
            &parser,
        )
        .expect("range over a top-level const must resolve to a Variable unit");

        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].boundary.node_type, "Variable");
    }

    // ── Byte-coverage invariant (c), all 4 core languages ──────────────────────

    #[test]
    fn byte_coverage_invariant_holds_for_core_languages() {
        let cases: &[(&str, &dyn LanguageParser, &str)] = &[
            (
                "rust",
                &RustParser::new(),
                "struct Foo;\n\nimpl Foo {\n    fn bar(&self) -> i32 {\n        1\n    }\n}\n\nconst X: i32 = 1;\n",
            ),
            (
                "python",
                &PythonParser::new(),
                "class Foo:\n    def bar(self):\n        return 1\n\nX = 1\n",
            ),
            (
                "go",
                &GoParser::new(),
                "package main\n\ntype Foo struct {\n\tX int\n}\n\nfunc (f Foo) Bar() int {\n\treturn 1\n}\n\nvar Y = 1\n",
            ),
            (
                "typescript",
                &TypeScriptParser::new(),
                "class Foo {\n    bar(): number {\n        return 1;\n    }\n}\n\nconst x = 1;\n",
            ),
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
