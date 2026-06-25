# Backlog: RenderableLine.line_number is relative, not absolute

## Summary

`RenderableLine.line_number` is a 1-based offset within the rendered boundary, not the
actual file line number. This causes a coordinate mismatch whenever code that knows
absolute file positions (e.g. `CodeImpact.line_ranges`) tries to correlate with rendered
lines.

## Root cause

Two creation paths both start line numbering from 1:

- **Single-source path** (`renderable_diff/line_utils.rs:114`):
  `number: line_num + 1` — enumerates lines of the extracted boundary text from zero.
- **Myers diff path** (`renderable_diff/mod.rs:182`):
  `let mut line_number = 1` — hard-coded start regardless of where the boundary sits
  in the file.

Both paths operate on text extracted from the semantic boundary node, so line 1 always
means "first line of this function/struct/block", never "line N of the source file."

The bug is acknowledged in the existing codebase:

```rust
// renderable_diff_widget.rs
// Line numbers disabled due to alignment issues in Myers diff
// TODO: Re-enable once RenderableDiff line numbering is fixed
```

## Impact

Any feature that needs to align absolute file positions with rendered lines must perform
a manual offset conversion:

```rust
let diff_start = renderable_diff.metadata.overall_line_range.start_line;
let relative = absolute.saturating_sub(diff_start).saturating_add(1);
```

`overall_line_range.start_line` is already computed from the boundary node's absolute
position and is available on every `RenderableDiff`.

The conversion is exact for single-source boundaries (Added, Deleted, Unchanged).
For Modified boundaries rendered via Myers diff, interleaved deleted lines each consume
a `line_number` slot, so the offset is approximate when deletions precede the target
line within the boundary.

## Current workaround

The inline reasoning annotations feature (plan-inline-reasoning-annotations, Phase 3)
applies this offset in `diff_view.rs` when building `Vec<ReasoningAnnotation>`, keeping
the fix inside the TUI layer with zero diffviz-core changes.

## Proper fix

Make `RenderableLine.line_number` store the absolute new-source file line number:

- Single-source path: pass `boundary_node.start_position().row + 1` as the base and
  add it to `line_num` instead of using `line_num + 1`.
- Myers diff path: initialize `line_number` from `new_node.start_position().row + 1`
  instead of `1`. Deleted lines have no new-source position — a sentinel value (e.g. 0)
  or a separate `old_line_number: Option<usize>` field would be needed to avoid
  ambiguity.

Fixing this also enables the commented-out line number display in
`renderable_diff_widget.rs::line_to_spans`.
