# Context Handoff — Phase 2 Implementation

## 🎯 Core Result
**Built**: `semantic_anchors.rs` rewritten from string/regex scanning to a pure DiffNode tree walk. `extract_semantic_anchor(line, reviewable, line_byte_start)` finds the deepest DiffNode whose own declaration starts within `[line_byte_start, line_byte_start + line.len())` and maps its `SemanticNodeKind` + `identifier` to a `SemanticAnchor`.
**Key insight**: The roadmap's original "byte containment" design (point-in-range) breaks on any indented line, because leading whitespace isn't covered by any tree-sitter node's byte range — the point falls in an inter-statement gap and the walk bottoms out on an enclosing container. Matching on "does the candidate node's `start_byte` fall on this line" instead of "does this line's start byte fall inside the candidate" sidesteps that entirely and needed no string trimming to fix.

## 🚦 Current State
**✅ Solid foundation**: All 17 non-ignored tests in `renderable_diff_anchor_tests.rs` pass (Phase 0 behavioral suite + Phase 1 identifier-propagation gate tests), including the three variable-binding tests (`rust_let_binding`, `rust_let_mut_binding`, `rust_const_item`) that exercise identifiers on non-boundary DiffNode children. 49 tests pass workspace-wide in diffviz-core (57 baseline − 8 deleted string-scanner unit tests), 12 ignored (pre-existing, unrelated), zero clippy warnings, `cargo fmt --check` clean.
**⚠️ Needs attention**: None for Phase 2 scope. The Myers-diff path (`create_line_by_line_diff_for_modified` in `renderable_diff/mod.rs`) still passes literal `0` for `line_byte_start` at all 5 call sites — this is unchanged from before and is explicitly Phase 3's job.
**⏸️ Deferred**: Same items Phase 1 deferred — Import/Module identifier extraction, the enum-misclassified-as-struct bug (`#[ignore]`d test `rust_enum_decl`), and everything Phase 3 owns.

## 👥 Next Agent Guidance
**Phase 3 Implementer (007)**: Fix the 5 zero-offset call sites in `renderable_diff/mod.rs`'s `create_line_by_line_diff_for_modified`. The anchor resolution logic in `semantic_anchors.rs` is already correct and needs no changes — it just needs real byte offsets fed in. Follow the same pattern used in this phase's fix to `line_utils.rs`: track an absolute byte position (boundary_start + running offset) rather than 0. Note none of the current Phase 0/1/2 tests exercise the Modified/Myers path at all (`old_source: None` in every test helper forces the Addition classification) — Phase 3 will need its own test fixtures that go through `NodeChangeStatus::Modified` to verify the fix.

## 🔗 Integration Points
**Expects**: `OwnedNodeData.identifier` populated on every DiffNode (boundary and children alike) for the 4 core languages, via `LanguageParser::extract_identifier` → `LanguageDescriptor::extract_identifier`.
**Provides**: `extract_semantic_anchor` with zero string/regex operations — satisfies the diffviz-core CLAUDE.md tree-sitter-only rule. `SemanticAnchorType` no longer has `FieldAssignment`/`MethodCall` variants (dropped, confirmed unused elsewhere).

## Key File Locations
- New anchor walk: `src/renderable_diff/semantic_anchors.rs` (full rewrite, 76 lines)
- Absolute-offset fix: `src/renderable_diff/line_utils.rs:61-66`
- `extract_identifier` trait method + default: `src/common.rs:137-172` (on `LanguageParser`)
- Delegating override: `src/parsers/generic_builder.rs` (`impl LanguageParser for GenericSemanticTreeBuilder<D>`)
- Per-language delegation to the wrapped builder: `src/parsers/{rust,go,python,typescript}.rs`
- Child DiffNode identifier population: `src/decision_based_diff.rs` — `build_child_nodes_recursive` (now takes `source: &str`), `build_reviewable_diff_from_unit_with_data` and its 3 callees (`create_addition_diff`, `create_deletion_diff`, `create_modification_diff_with_data`)
