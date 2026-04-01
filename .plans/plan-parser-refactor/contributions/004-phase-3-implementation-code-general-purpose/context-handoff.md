# Context Handoff — Phase 3 Implementation

## What Was Done

Phase 3 is complete. The parser refactor plan is fully finished.

**Caller verification:**
- `diffviz-review/src/review_engine_builder.rs` — already used the new public names
  (`RustParser::new()`, `PythonParser::new()`, etc.) unchanged from Phase 2. No edits required.
- `diffviz-core/tests/test_utils.rs` — no changes required; factory compiles with refactored types.

**Final quality gate (pre-doc):**
- `cargo build --workspace` — clean (0 warnings)
- `cargo test --workspace` — 240 passed, 1 ignored (pre-existing doctest in `semantic_ast.rs`)
- `cargo clippy --workspace` — no issues

**Documentation updated:**
- `diffviz-core/CLAUDE.md` — Parser Architecture section rewritten for descriptor pattern; bug section updated
- `diffviz-core/onboarding.md` — Phase 1-2 updated to reference `LanguageDescriptor`/`GenericSemanticTreeBuilder`; directory map updated; Recent Changes section added
- `diffviz-core/bugs.md` — impl_block, struct_range_expansion, TypeScript classification, and JavaScript stub bugs moved to Fixed

## Key Decisions

**No caller changes were needed.** Phase 2 preserved all public names (`RustParser`, `PythonParser`,
etc.), so `review_engine_builder.rs` required zero edits. The newtype wrapper pattern — each
language parser is a thin struct holding `GenericSemanticTreeBuilder<XxxDescriptor>` — was the right
abstraction boundary: callers see the same interface, the internals are completely replaced.

## Plan Status

All three phases complete:
- Phase 1 ✅ — `LanguageDescriptor` trait + `GenericSemanticTreeBuilder` + `RustDescriptor`
- Phase 2 ✅ — 6 remaining language descriptors + JavaScript promoted from stub
- Phase 3 ✅ — Caller verification + documentation

**The parser refactor plan is done.** No further phases remain.

## Remaining Known Limitations

These were consciously out of scope for this plan (see Phase 2 context-handoff):

1. **Non-Rust import parsing** — `parse_use_declaration` in `generic_builder.rs` is Rust-specific. Import metadata for Python, Go, TypeScript, Java may be absent or malformed. Do not add callers that depend on import metadata for non-Rust languages without fixing this first.

2. **DataStructure nodes have no method children** — `build_data_structure` does not recurse into class bodies for non-Rust languages. Python/TypeScript/Java class methods are not children of their `DataStructure` node in the semantic tree. Only Rust's `ImplBlock` correctly encloses methods.

3. **TypeScript/JS review-layer classification bug** — The parser now works correctly. The "modified files shown as New file" symptom that surfaced in the review layer is a separate bug in `diffviz-review` and should be filed there if not already tracked.
