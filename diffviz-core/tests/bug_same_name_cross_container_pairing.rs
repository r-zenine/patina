//! Bug: same-named units pair across containers — old side of the diff is the wrong unit
//!
//! `find_semantic_unit_by_name` in decision_based_diff.rs matches old-tree counterparts
//! by (unit-type discriminant, name text) over a flat scan of ALL units and returns the
//! first hit. Container context (impl target, module path) is ignored. With two impl
//! blocks each defining a method of the same name — ubiquitous in Rust (`fn new`,
//! `fn get`, trait impls) — a change to `B::get` is paired against `A::get`, and the
//! rendered old/new diff is fiction.
//!
//! Expected: the old counterpart of `B::get` is `B::get` from the old tree.
//!
//! Actual: the first name match wins — `A::get` (body `1`) is returned as the old node
//! instead of `B::get` (body `100`).

#[cfg(test)]
mod bug_same_name_cross_container_pairing {
    use diffviz_core::ast_diff::SourceCode;
    use diffviz_core::common::ProgrammingLanguage;
    use diffviz_core::decision_based_diff::create_reviewable_diff_from_range;
    use diffviz_core::parsers::RustParser;
    use diffviz_core::reviewable_diff::NodeChangeStatus;

    const OLD_SOURCE: &str = r#"struct A;
struct B;

impl A {
    fn get(&self) -> u32 {
        1
    }
}

impl B {
    fn get(&self) -> u32 {
        100
    }
}
"#;

    // Only B::get changed (100 -> 200); A::get is untouched.
    const NEW_SOURCE: &str = r#"struct A;
struct B;

impl A {
    fn get(&self) -> u32 {
        1
    }
}

impl B {
    fn get(&self) -> u32 {
        200
    }
}
"#;

    #[test]
    #[ignore = "bug: old counterpart matched by name only, ignores containing impl/module"]
    fn modified_method_pairs_with_same_impl_counterpart() {
        let old_provider = SourceCode::new(OLD_SOURCE.to_string());
        let new_provider = SourceCode::new(NEW_SOURCE.to_string());
        let parser = RustParser::new();

        // Lines 11..=13 = body of B::get in the new file.
        let diffs = create_reviewable_diff_from_range(
            "src/lib.rs",
            11,
            13,
            Some(&old_provider),
            &new_provider,
            ProgrammingLanguage::Rust,
            &parser,
        )
        .expect("diff creation should succeed");

        assert_eq!(diffs.len(), 1);

        match &diffs[0].boundary.change_status {
            NodeChangeStatus::Modified { old_node, .. } => {
                let old_text = diffs[0]
                    .old_source
                    .node_text(old_node)
                    .expect("old node text should be extractable");
                assert!(
                    old_text.contains("100"),
                    "old counterpart of B::get must be B::get (body 100), \
                     but A::get was paired instead:\n{old_text}"
                );
            }
            other => panic!("expected Modification for B::get, got {other:?}"),
        }
    }
}
