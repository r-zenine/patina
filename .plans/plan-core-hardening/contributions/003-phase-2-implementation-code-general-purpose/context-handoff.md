# Context Handoff - Phase 2 Implementation

## 🎯 Core Result
**Built**: `myers_diff.rs` (broken hand-rolled Myers) replaced by
`renderable_diff/line_diff.rs`, a `similar`-backed engine (Patience algorithm)
with index-carrying `DiffOp` (`Keep{old_idx,new_idx}`, `Add{new_idx}`,
`Delete{old_idx}`, `Modify{old_idx,new_idx}`) plus a standalone
`align_by_anchors` post-pass. `create_line_by_line_diff_for_modified` (mod.rs)
now indexes directly into `old_lines`/`new_lines` — `find_original_line_content`
(the wrong-content-fallback hack) is gone.

**Key insight**: `similar::capture_diff_slices` already groups contiguous
delete+insert runs into `Replace` ops, which is exactly the "adjacent
Delete-run/Add-run pair" shape the anchor-alignment post-pass needs — but we
still expand everything to our own flat `DiffOp` list first and re-detect runs
generically in `align_by_anchors`, so that function stays independently
testable and decoupled from `similar`'s API shape.

## 🚦 Current State
**✅ Solid foundation**:
- `diff_lines`/`align_by_anchors` in `line_diff.rs`, unit-tested directly
  (reconstruction via indices, duplicate-line handling, anchor match/mismatch/none).
- CRLF fix is shared: `line_utils::line_byte_spans` computes terminator-width-accurate
  (`\n` vs `\r\n`) content-only line spans, used by both the single-source path
  (`split_into_lines_with_positions`) and the Modified path. Both were broken
  independently by the same "+1 for newline" assumption; both are fixed now.
- Zero benign assertion drift: `renderable_diff_anchor_tests.rs` needed **no**
  changes — every existing anchor-consumption assertion holds unmodified under
  the new engine. `git diff --stat` on that file is empty.
- Reconstruction property test green over 1024 proptest cases; both Myers bug
  reproductions and the CRLF reproduction pass un-ignored.
- Test count: 53→67 passed, 11→7 ignored (diffviz-core). Workspace-wide
  build/clippy/fmt/tests all clean.

**⚠️ Needs attention**: none carried forward from this phase's own work.

**⏸️ Deferred** (unchanged from decision log, not touched this phase):
- `RenderableLine.byte_range` in the Modified path stays relative
  `(0, content.len())`, NOT unified with the single-source path's absolute
  byte_range. That unification is decision D012's explicit scope (TUI-affecting
  coordinate-frame change, its own plan) — don't casually "fix" this later
  without checking D012 first.
- `ByteRangeAnnotation`/`NodeAnnotation`/`RenderableLine.byte_range` are still
  `(usize, usize)` tuples — Phase 3 retypes to `Range<usize>`.

## 👥 Next Agent Guidance
**Phase 3 (LineIndex + typed half-open ranges)**: You'll build the general
`LineIndex` on `SourceCode` and retype byte ranges to `Range<usize>`. Note that
`line_utils::line_byte_spans(text: &str) -> Vec<(usize, usize)>` (added this
phase) already solves line↔byte conversion for *already-extracted, in-memory*
text — it is not the same thing as the `LineIndex` you're building (which
indexes a `SourceCode`/full-file source for the decompose-path arithmetic bugs
in `decision_based_diff.rs`/`ast_diff/source.rs`). Don't conflate the two or
try to unify them prematurely; `line_byte_spans` can very likely be reimplemented
in terms of `LineIndex` once it exists, but that's a nice-to-have, not required
by Phase 3's acceptance criteria — check code-context.md's Phase 3 section
before deciding whether it's in scope.

**Phase 4 (qualified-name matching)**: Unaffected by this phase; `line_diff.rs`
doesn't touch `decision_based_diff.rs`'s unit-matching logic at all.

## 🔗 Integration Points
**Expects**: `SemanticAnchor`/`SemanticAnchorType` (mod.rs) unchanged from
before this phase — anchor *extraction* (`semantic_anchors.rs`) was not
touched, only *consumption* (the post-pass), per the roadmap's coordination
note with `plan-semantic-anchors-tree-sitter`.

**Provides**: `line_diff::{DiffOp, diff_lines, align_by_anchors}` as the sole
diff-computation surface for Modified-boundary rendering; `similar` as a new
workspace-level runtime dependency.

## 📋 Reference Links
- [decision-log.yaml](decision-log.yaml) — API choice (capture_diff_slices vs
  diff_lines), CRLF-fix scope decision, byte_range non-unification rationale.
