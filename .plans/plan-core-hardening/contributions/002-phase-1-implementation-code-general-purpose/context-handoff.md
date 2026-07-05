# Context Handoff - Phase 1 Implementation (Dead-code Sweep, Error Cleanup)

## 🎯 Core Result
**Built**: Deleted the legacy semantic-pairing API surface entirely (is_semantically_identical
chain, can_pair_with, filtered_units chain, SemanticTree.source_ranges, the unreachable
ChangeClassification::Deletion path, _node_has_keyword, extract_expanded_source_text).
Migrated SemanticError to thiserror and DecisionDiffError::ParseError/SemanticError to carry
real `#[source]` errors instead of `format!`-ed strings. Consolidated the three
get_display_node copies onto `NodeChangeStatus::display_node` / `display_node_with_source`,
and the two ranges_overlap copies onto one `pub(super)` helper. Simplified
determine_line_relevance_with_precedence to a single `min()` chain (D011). Fixed
parameter_count to use `named_child_count()`.

**Key insight**: The get_display_node consolidation wasn't purely mechanical — two of the
three copies (`name_extractors::extract_boundary_name`, `mod.rs`'s `overall_line_range`) were
calling the *source-blind* variant and always reading `new_source`, which is exactly the bug
`bug_deleted_boundary_reads_new_source.rs` pinned. Routing both through
`display_node_with_source` fixed it as a side effect — anticipated by decision D009 in the
top-level plan, not scope creep. The test is now un-ignored and passing; `bugs.md` marks it
Fixed.

## 🚦 Current State
**✅ Solid foundation**: Suite green throughout (53 passed, 11 ignored at the end — was 52/13
at Phase 0's close: -1 for the deleted `test_all_units_vs_filtered_units`, +2 for the two bugs
this phase fixed as intended/incidental). Clippy and fmt clean workspace-wide. Grepped every
deleted symbol across the *whole workspace* (not just diffviz-core/src) both before and after
deletion — zero remaining references except a stale doc-comment, which was rewritten.

**⚠️ Needs attention**: None blocking. `ranges_overlap` and `RenderableLine.byte_range` /
`LineInfo.byte_range` / `ByteRangeAnnotation.byte_range` / `NodeAnnotation.byte_range` are
still `(usize, usize)` tuples — untouched on purpose, that's Phase 3's job (typed half-open
ranges). Don't be surprised the tuple convention persists into Phase 2's diff engine work too.

**⏸️ Deferred**: Nothing new deferred this phase beyond what the top-level plan already
scopes out (D012).

## 👥 Next Agent Guidance
**Phase 2 (diff engine replacement)**: No blockers from this phase. `myers_diff.rs` is
untouched — Phase 1 only touched the *consumers* of `NodeChangeStatus` and error types, not
the diff engine itself. `create_line_by_line_diff_for_modified` (renderable_diff/mod.rs) still
calls into `myers_diff::myers_diff_semantic` exactly as before; `find_original_line_content`
is still present and still the thing Phase 2 deletes. `tests/diff_reconstruction.rs` and
`tests/bug_myers_diff_drops_duplicate_lines.rs` remain your exit criteria, unchanged by this
phase.

**Phase 3 (LineIndex)**: When retyping byte ranges to `Range<usize>`, the consolidated
`ranges_overlap(range1: (usize, usize), range2: (usize, usize))` in `line_utils.rs` (now
`pub(super)`, single copy) is the one function to retype — no more hunting two copies.

## 🔗 Integration Points
**Expects**: Nothing new — no dependency or API changes visible outside diffviz-core/src
(DecisionDiffError's variant *payload* type changed but its variant names and the crate's
public error-handling contract did not).
**Provides**: A single-source-of-truth `NodeChangeStatus::display_node[_with_source]` for any
future code needing the display node of a change status — use these, don't re-invent a fourth
copy.

## 📋 Reference Links
- [decision-log.yaml](decision-log.yaml) — decisions made this phase
- `diffviz-core/bugs.md` — two bugs marked Fixed (deleted-boundary-reads-wrong-source,
  parameter_count-counts-commas)
- `diffviz-core/src/reviewable_diff.rs` — new `impl NodeChangeStatus` block
