# Context Handoff — Phase 1 Implementation

## What Was Done

Phase 1 is complete. The 882-line bespoke `RustParser` has been replaced with:

- **`descriptor.rs`** — `LanguageDescriptor` trait (static kind tables + two override methods)
- **`generic_builder.rs`** — `GenericSemanticTreeBuilder<D>` implementing `LanguageParser`
- **`rust.rs`** — `RustDescriptor` (~90 lines of static data) + `RustParser` newtype wrapper

All 48 tests pass; 0 clippy warnings. Both previously-ignored bug tests now pass:
- `bug_rust_impl_block_not_classified.rs` (2 tests)
- `bug_struct_range_expansion.rs` (2 tests)

## Critical Decisions for Next Phase

**impl_item → Module(Namespace), not DataStructure.**
The decompose path in `create_reviewable_diff_from_range` is triggered only by `Module` variants.
Any future language that has impl-equivalent constructs (Python classes, Go structs with methods)
should follow the same pattern: map the container to `ImplBlock` in `semantic_kind_map`, which the
generic builder converts to `Module { Namespace }`.

**attribute_item as Unknown children.**
`find_unit_recursive` only inspects `tree_sitter_node.byte_range()` of direct children. Bytes that
belong to no child are "unowned" — ranges covering only those bytes return empty. Any annotation
node (Python `decorator`, TypeScript `decorator`) must be made an Unknown child via `metadata_kind()`.

**doc_comment is trivial, not a tree node.**
If doc-comment-equivalent nodes become Unknown children, range queries starting at documentation
return the comment as the first diff result instead of the enclosing function. Keep doc comment
node kinds in `trivial_kinds`.

**visibility_modifier lives in both lists.**
`trivial_kinds` governs tree construction. `semantic_kind_map` governs `classify_node_kind`. These
are orthogonal; an entry in both is correct and intentional.

## What Phase 2 Needs to Do

Add `LanguageDescriptor` implementations for the remaining 6 languages:
Python, Go, TypeScript, Java, C, C++. Each follows the same pattern as `RustDescriptor`:

1. Define a `FooDescriptor` struct.
2. Populate `semantic_kind_map` from the existing `FooParser.classify_node_kind` match arms.
3. Populate `trivial_kinds` from the existing `is_trivial_syntax_token` equivalent.
4. Set `metadata_kind()` appropriately (Python/TypeScript → `"decorator"`, others → `None`).
5. Replace the existing `FooParser` with a newtype over `GenericSemanticTreeBuilder<FooDescriptor>`.
6. Override `get_context_boundaries` in the newtype if the language has specific boundary logic.

The Go parser has naming-convention-based visibility (exported = `pub`, unexported = `private`).
`GoDescriptor` must override `extract_visibility` to inspect the identifier name rather than a
`visibility_modifier` child.

## Files to Read

- `diffviz-core/src/parsers/descriptor.rs` — trait API
- `diffviz-core/src/parsers/generic_builder.rs` — full builder logic + `assert_byte_coverage_invariant`
- `diffviz-core/src/parsers/rust.rs` — reference implementation; copy the pattern for each language

## Success Criteria for Phase 2

- All existing language parser unit tests pass.
- `cargo test --package diffviz-core` stays green.
- 0 clippy warnings.
- Each descriptor is ≤ 150 lines.
