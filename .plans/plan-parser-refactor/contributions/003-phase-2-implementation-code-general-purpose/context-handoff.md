# Context Handoff — Phase 2 Implementation

## What Was Done

Phase 2 is complete. The 6 remaining bespoke parsers (Python, Go, TypeScript, Java, C, C++) and the JavaScript stub have been replaced with `LanguageDescriptor` + `GenericSemanticTreeBuilder` newtypes.

**Final state:**
- 58 tests pass (+10 vs Phase 1 baseline), 0 clippy warnings
- 4 previously-ignored bug tests now pass (2 TypeScript, 2 JavaScript)
- 1 ignored remains — a pre-existing doctest in `semantic_ast.rs`, unrelated to this work

## Critical Decisions for Phase 3

**Go and Python visibility is name-based.**
`GoDescriptor` and `PythonDescriptor` both override `extract_visibility` by reading `child_by_field_name("name")`. For Go: capitalised first letter = public. For Python: `_name` = private, `__name__` = magic, else public. Phase 3 callers that display visibility should be aware of this.

**No import parsing for non-Rust languages.**
The generic builder's `parse_use_declaration` is Rust-specific (strips `use `, splits on `::`). For Python `import_statement`, Go `import_declaration`, TypeScript `import_statement`, Java `import_declaration`, the method may return garbage data or fail silently (returning `None` via `.ok()`). Import nodes may be absent or malformed in non-Rust semantic trees. Phase 3 should not introduce callers that depend on accurate import metadata for non-Rust languages without fixing this first.

**DataStructure nodes have no children.**
The generic builder's `build_data_structure` does not recurse into the node body to build method children. Python class methods, TypeScript class methods, and Java class methods are therefore not children of their enclosing DataStructure node in the semantic tree. Only Rust's `ImplBlock` (mapped as `Module{Namespace}`) correctly encloses method nodes. Phase 3 must not assume that class-level changes will resolve to method nodes for non-Rust languages.

**TypeScript/JS classification bug is still open in the review layer.**
The original TS "modified files shown as New file" bug lives in `diffviz-review`, not in the parser. The parser tests now verify `build_semantic_tree` succeeds — that's the correct scope for Phase 2. The review-layer bug is tracked separately and should be filed if it has not been already.

## What Phase 3 Needs to Do

Port callers outside `diffviz-core/src/parsers/`:

1. `diffviz-review/src/review_engine_builder.rs` (lines 225–232) — update the parser factory to instantiate the new newtype structs. The public names (`RustParser`, `PythonParser`, etc.) are unchanged, so only the construction sites matter.
2. `diffviz-core/tests/test_utils.rs` — `get_parser_for_language` already uses the correct public names; verify it still compiles with the refactored types (no changes expected).
3. Run `cargo build --workspace` + `cargo test --workspace` + `cargo clippy --workspace` to confirm zero regressions across all crates.
4. Update `diffviz-core/CLAUDE.md` and `diffviz-core/onboarding.md` to describe the new descriptor architecture.
5. Update `diffviz-core/bugs.md` to mark the resolved bugs as fixed (impl_block, struct_range, TS classification parser-layer, JS unsupported-language).

## Files to Read

- `diffviz-core/src/parsers/rust.rs` — canonical reference implementation
- `diffviz-core/src/parsers/descriptor.rs` — `LanguageDescriptor` trait
- `diffviz-core/src/parsers/generic_builder.rs` — full builder logic
- `diffviz-review/src/review_engine_builder.rs` — primary Phase 3 target

## Success Criteria for Phase 3

- `cargo build --workspace` succeeds with zero warnings
- `cargo test --workspace` passes all tests
- `cargo clippy --workspace` clean
- No references to old bespoke parser internals (`build_source_file_node`, `build_function_node`, `is_trivial_syntax_token`, etc.)
- `review_engine_builder.rs` correctly instantiates parsers for all supported extensions
