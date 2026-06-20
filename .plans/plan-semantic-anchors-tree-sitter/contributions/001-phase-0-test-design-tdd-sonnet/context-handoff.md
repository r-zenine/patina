# Context Handoff — Phase 0: Behavioral Test Coverage

## What Was Done

Created `diffviz-core/tests/renderable_diff_anchor_tests.rs` with 12 behavioral integration tests covering all anchor cases from the Phase 0 table. Tests drive the production pipeline end-to-end: `create_reviewable_diff_from_range` → `RenderableDiff::try_from` → check `line.semantic_anchor`.

All 11 active tests pass with the current string-scanner implementation. 1 test is `#[ignore]`d.

## Key Discovery: rust_enum_decl Bug

`rust_enum_decl` is ignored due to a pre-existing classification bug unrelated to string-scanning:

- `GenericSemanticTreeBuilder` uses `SemanticUnitType::DataStructure` for both structs and enums
- `unit_type_to_semantic_kind` maps `DataStructure` → `SemanticNodeKind::Struct` (no Enum arm)
- `extract_semantic_anchor` dispatches to `extract_struct_anchor` for the enum boundary
- `extract_struct_anchor` checks `starts_with("struct ")` → false for `"enum Color {"` → returns `None`

**Impact on Phase 2**: Phase 2's `kind_to_anchor_type` uses `node.semantic_kind` which also comes from `unit_type_to_semantic_kind`. If left unfixed, enum boundaries will produce `StructDeclaration` (wrong), not `EnumDeclaration`. Phase 2 implementer should either fix `unit_type_to_semantic_kind` to add an Enum arm, or confirm this is out of scope and document it.

## What Phase 1 Should Focus On

Phase 1 implements `extract_identifier` on `LanguageDescriptor` and adds `identifier: Option<String>` to `OwnedNodeData`. The Phase 0 tests will continue to pass throughout Phase 1 (string scanner is still in place, only infrastructure is added).

Phase 1 verification: add targeted assertions in the test file to confirm that `OwnedNodeData` nodes inside the DiffNode tree carry correct identifiers for all binding cases (the `RUST_BINDINGS_SOURCE` constant in the test file is the right fixture for this).

## Test File Navigation

- `anchors_for_range` helper (line 17): the reusable entry point — takes source, range, language, parser
- `anchor_at` helper (line 50): extracts anchor by 1-based line number relative to boundary
- Rust function tests (lines 57–85)
- Rust type declaration tests (lines 89–114): `rust_enum_decl` is `#[ignore]` here
- Rust binding tests (lines 118–158): use `RUST_BINDINGS_SOURCE` constant
- Go tests (lines 162–193): require `package main` header; line range targets the function, not file root
- Python test (line 197–210)
- Negative cases (lines 214–231): comment and empty line both return `None`
