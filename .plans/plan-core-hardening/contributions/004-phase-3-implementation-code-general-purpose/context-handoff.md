# Context Handoff - Phase 3 Implementation

## 🎯 Core Result
**Built**: `LineIndex` (`diffviz-core/src/ast_diff/line_index.rs`) — a newline-start table
built in one pass, owned by `SourceCode`. It is now the only place line<->byte conversions
happen in the crate, replacing four hand-rolled and mutually inconsistent implementations.
**Key insight**: the end-line-exclusion bug and the touching-fallback are the same
mechanism seen from two sides. Fixing `end_byte` to cover the *full* end line (not just its
start) fixes the decompose bug, but it also means `end_byte` can now exceed a container
node's own tree-sitter span when the range's last line is the file's last line and the file
ends in a newline (the node itself typically doesn't span that trailing newline).
`find_units_touching_range_recursive` is what recovers a result in that case — it is
proven still reachable (see decision #4), not dead code as the roadmap suspected.

## 🚦 Current State
**✅ Solid foundation**: `LineIndex` is fully unit-tested (empty source, no/with trailing
newline, CRLF, single line, out-of-bounds, the column-0 regression fixture) and wired into
both `SourceCode::line_range_from_bytes` and `decision_based_diff.rs`'s range resolution.
Byte ranges are `Range<usize>` everywhere in the crate now — no more `(usize, usize)`
byte-range tuples anywhere.
**⚠️ Needs attention**: none blocking. `build_byte_range_annotations`
(`renderable_diff/mod.rs`) and `collect_all_annotations` (`renderable_diff/line_utils.rs`)
are still two near-identical tree walks over `DiffNode`, now both typed with
`Range<usize>` — a future cleanup could merge them, but it wasn't attempted here since it's
orthogonal to this phase and not obviously low-risk (they collect into different
`*Annotation` struct shapes for different consumers).
**⏸️ Deferred**: the annotation-collection merge above; `LineNr`/`ByteOffset` newtypes
(decision D006, explicitly deferred).

## 👥 Next Agent Guidance
**Phase 4 (qualified-name unit matching)**: works on `SemanticNode`/`OwnedNodeData`
identity, not on line/byte arithmetic — this phase doesn't change any of the types Phase 4
touches (`identifier`, `parent_context`). No interaction expected, but re-verify current
line numbers in `decision_based_diff.rs` and `parsers/generic_builder.rs` before starting,
they will have shifted again.
**Anyone touching range resolution later**: use `LineIndex`/`byte_range_for_lines`
(`decision_based_diff.rs`), don't reintroduce ad hoc line<->byte math. `LineIndex::line_start`
panics on an out-of-bounds start line by design (fail-fast per this crate's CLAUDE.md) —
callers must validate against `line_count()` first, as `byte_range_for_lines` does.

## 🔗 Integration Points
**Expects**: Phase 2's `similar`-backed diff engine and `line_byte_spans` helper
(unrelated/orthogonal — different job, see decision #1 for why they weren't merged).
**Provides**: `LineIndex` as the crate's single line<->byte conversion authority;
`Range<usize>` byte ranges as the crate-wide convention for annotations/rendering.

## 📋 Reference Links
- [decision-log.yaml](decision-log.yaml) - Technical choices made, including the
  find_units_touching_range_recursive reachability proof
- `bugs.md` - both Phase 3 bugs (end-line-exclusion, column-0 off-by-one) marked Fixed
