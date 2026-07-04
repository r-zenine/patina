# Code Context for core-hardening

All references verified against the working tree on 2026-07-04. Line numbers
drift as phases land — re-verify before each phase.

## Diff engine (Phase 2 target)

- **`myers_diff_semantic`** (`diffviz-core/src/renderable_diff/myers_diff.rs:31-73`) —
  entry point; edge cases for empty sides, then forward pass + backtrack + merge.
  **Replaced wholesale by `similar`.**
- **`shortest_edit_script_semantic`** (`myers_diff.rs:108-158`) — broken snake:
  the greedy loop advances only `y` (lines 136-144) comparing the same
  `old_lines[x]` against successive new lines; `let x = x + (y - (x - k))` then
  fabricates a diagonal. Root cause of the drops-lines bug.
- **`backtrack_operations_semantic`** (`myers_diff.rs:161-236`) — contains the
  `else { 0 }` defensive fallbacks that mask the inconsistent trace.
- **`merge_identical_add_delete_pairs`** (`myers_diff.rs:242-277`) — patch over
  the root cause; delete with the rewrite.
- **`DiffOp`** (`myers_diff.rs:12-21`) — carries `String`s today; becomes
  index-carrying (`Keep { old_idx, new_idx }`, `Add { new_idx }`,
  `Delete { old_idx }`, `Modify { old_idx, new_idx }`).
- **`lines_should_align` / `semantically_related`** (`myers_diff.rs:76-105`) —
  anchor-equality logic; survives as the predicate of the alignment post-pass.

## Renderable pipeline (Phases 2–3)

- **`create_line_by_line_diff_for_modified`** (`renderable_diff/mod.rs:143-322`) —
  consumes DiffOps; byte accounting assumes 1-byte newlines (`+1 for newline`,
  lines 219/246/296 — CRLF drift); rewritten to derive byte ranges from real
  line offsets once ops carry indices.
- **`find_original_line_content`** (`renderable_diff/mod.rs:353-378`) — content-
  matching hack with wrong-content fallback; **deleted** once ops carry indices.
- **`determine_line_relevance_with_precedence`** (`renderable_diff/mod.rs:327-345`) —
  two-pass "ESSENTIAL wins, else min"; ESSENTIAL is 0 so this is just `min()`.
- **`RenderableDiff::try_from`** (`renderable_diff/mod.rs:381-427`) — the only
  production entry into rendering; `overall_line_range` (line 403) flows through
  the buggy `line_range_from_bytes`.
- **`build_byte_range_annotations`** (`renderable_diff/mod.rs:110-140`) and
  **`collect_all_annotations`** (`renderable_diff/line_utils.rs:127-153`) — near-
  duplicate tree walks; candidates to unify when tuples become `Range<usize>`.
- **`ranges_overlap`** — two copies: `renderable_diff/mod.rs:348-350`,
  `line_utils.rs:192-194`.
- **`get_display_node`** — three copies: `renderable_diff/mod.rs:514-521`,
  `line_utils.rs:100-107`, `name_extractors.rs:82-89`. Consolidate as a method
  on `NodeChangeStatus`. Note `line_utils.rs:86-97`
  (`get_display_node_with_source`) is the only copy that picks the correct
  source for Deleted nodes.

## Line/byte arithmetic (Phase 3 target)

- **`line_to_byte_offset`** (`decision_based_diff.rs:163-186`) — O(n) scan per
  call; returns the offset of the *start* of the line, which makes the
  decompose path's `end_byte` exclude the end line's content.
- **`count_lines`** (`decision_based_diff.rs:146-152`),
  **`clamp_line_range`** (`decision_based_diff.rs:156-160`) — clamps end only.
- **`line_range_from_bytes`** (`ast_diff/source.rs:91-119`) — `lines().count()`
  undercounts by 1 when the offset sits at column 0 (prefix ends with `\n`).
- **`find_contained_units_recursive`** (`decision_based_diff.rs:69-88`) — the
  `node_range.end <= end_byte` check that drops units ending on the range's
  last line.
- **`find_units_touching_range_recursive`** (`decision_based_diff.rs:93-115`) —
  fallback that exists to paper over the single-line empty-interval case;
  likely removable once `end_byte` is the end of the end line (verify by test).
- **`SourceCode`** (`ast_diff/source.rs:50-120`) — home for the new `LineIndex`.
- Tuple byte ranges to retype: `ByteRangeAnnotation.byte_range`
  (`renderable_diff/mod.rs:101-105`), `NodeAnnotation.byte_range`
  (`line_utils.rs:20-27`), `RenderableLine.byte_range` + `LineInfo.byte_range`
  (`renderable_diff/mod.rs:41-47`, `line_utils.rs:12-17`).

## Unit matching (Phase 4 target)

- **`find_semantic_unit_by_name`** (`decision_based_diff.rs:192-224`) — flat
  scan over `all_units()`, matches (discriminant, name), first hit wins; the
  cross-impl mispairing lives here.
- **`get_unit_name`** (`decision_based_diff.rs:227-233`).
- **`build_impl_container`** (`parsers/generic_builder.rs:280-303`) — already
  threads the impl target type as `parent_context: Option<&str>`; the natural
  seed for a qualified-name path.
- **`SemanticNode.identifier`** (`semantic_ast.rs:73`) and
  **`OwnedNodeData.identifier`** (`ast_diff/nodes.rs:27-28`) — where identity
  is carried across tree lifetimes today.
- Callers to update: the two `find_semantic_unit_by_name` call sites in
  `create_reviewable_diff_from_range` (`decision_based_diff.rs:583,655`).

## Container recursion (Phases 5–6 target)

- **`build_data_structure`** (`parsers/generic_builder.rs:236-273`) — never
  collects children; class bodies are opaque.
- **`build_typed_node`** (`generic_builder.rs:151-186`) — the `_ => None` arm
  drops Statement-classified wrappers *without recursing*; kills Python
  module-level `expression_statement → assignment`.
- **`container_body_field`** (`parsers/descriptor.rs:45`) — dead hook, defined
  per-language but never called; the builder hardcodes `"body"` in
  `build_impl_container`/`build_module_container` (`generic_builder.rs:287,310`).
- Per-language descriptors: `parsers/{rust,python,go,typescript,javascript,java,c,cpp}.rs`
  — kind maps contain construction kinds that are currently unreachable inside
  class bodies (e.g. `method_definition` in typescript.rs:19,
  `assignment` in python.rs:26).
- **`field_count`** (`generic_builder.rs:251-256`) — hardcodes Rust kinds
  (`field_declaration`/`enum_variant`); revisit per-language while in here.
- **`parameter_count`** (`generic_builder.rs:202-204`) — `child_count() - 2`
  counts comma tokens; should be `named_child_count()`.

## Dead code to remove (Phase 1 target) — verified no users outside diffviz-core/src

- `SemanticNode::is_semantically_identical` + `unit_types_are_identical` +
  `compare_optional_nodes` (`semantic_ast.rs:463-609`)
- `SemanticNode::can_pair_with` (`semantic_ast.rs:417-459`)
- `SemanticTree::filtered_units` + `collect_filtered_units` +
  `should_include_unit` + `estimate_node_size` + `find_units_by_type`
  (`semantic_ast.rs:253-327`)
- `SemanticTree.source_ranges` (`semantic_ast.rs:29`) — always empty
- `ChangeClassification::Deletion` + `create_deletion_diff`
  (`decision_based_diff.rs:59-64,256-262,313-341`) — unreachable; the
  Deletion arm reads `new_unit` and would render wrong sources if ever reached
- `_node_has_keyword` (`parsers/rust.rs:289-300`)
- `extract_expanded_source_text` (`line_utils.rs:196-250`, already
  `#[allow(dead_code)]`)

## Error types (Phase 1 target)

- **`SemanticError`** (`semantic_ast.rs:613-653`) — hand-rolled `Display`;
  migrate to `thiserror`.
- **`DecisionDiffError`** (`decision_based_diff.rs:28-53`) —
  `ParseError(String)` / `SemanticError(String)` flatten source chains built at
  `decision_based_diff.rs:509-516,569-578,647-654`.

## Testing Patterns

- Bug reproductions: `diffviz-core/tests/bug_*.rs` (one file per bug, see
  `bugs.md` for lifecycle). Filed bugs from the 2026-07-04 review land here.
- Shared helpers: `tests/test_utils.rs`; realistic fixtures under
  `tests/fixtures/`.
- Suite baseline before this plan: 57 passed, 1 ignored.
- Consumer smoke: `cargo test --workspace` (diffviz-review's engine tests
  exercise `create_reviewable_diff_from_range` + `RenderableDiff::try_from`).
