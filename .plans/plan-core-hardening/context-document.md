# Context Document for core-hardening

> Immutable reference for the diffviz-core hardening effort. Derived from a full
> code review of diffviz-core (2026-07-04) that verified 5 bugs empirically and
> identified the structural improvements below. The filed bugs are owned by this
> plan (their failing tests are acceptance criteria, no separate minimal patches).

## Behavioral Specification

Harden diffviz-core so that the diffs it renders are provably faithful to the
sources: replace the broken hand-rolled Myers diff with a well-tested engine,
make line/byte arithmetic correct by construction via a single line-offset
index, pair changed units with the *right* old counterpart via
container-qualified names, and make methods inside class bodies visible to the
semantic tree in all 8 supported languages. Along the way, delete the dead
legacy-pairing API surface and bring error types up to the project's thiserror
standard. Observable behavior for end users: diffs stop dropping/misplacing
lines, reported line ranges are exact, and reviewing a single method inside a
Python/TypeScript/Java/C++ class yields that method — not the whole class.

## Scope

**In scope**

1. **Diff engine replacement** — swap `myers_diff.rs` for the `similar` crate
   (Patience algorithm); `DiffOp` carries line *indices* instead of `String`s;
   semantic-anchor alignment becomes a post-pass over the ops; a
   reconstruction property test (replaying ops rebuilds both inputs exactly)
   guards the engine permanently.
2. **Line-offset index** — a `LineIndex` (newline table + binary search) owned
   by `SourceCode`, replacing the four inconsistent hand-rolled implementations
   (`line_to_byte_offset`, `count_lines`, `clamp_line_range`,
   `line_range_from_bytes`).
3. **Typed half-open ranges** — `std::ops::Range<usize>` for byte ranges
   everywhere `(usize, usize)` tuples are used today; one `ranges_overlap`
   helper; inclusive-line → half-open-byte conversion happens in exactly one
   place (the `LineIndex`).
4. **Qualified-name unit matching** — unit identity becomes
   (container path, unit type, name); fixes cross-`impl`/cross-class mispairing.
5. **Container recursion** — `build_data_structure` and statement wrappers
   recurse into bodies via the (currently dead) `container_body_field`
   descriptor hook, for **all 8 languages** (Rust, Python, Go, TypeScript,
   JavaScript, Java, C, C++).
6. **Dead-code sweep** — remove the legacy semantic-pairing API
   (`is_semantically_identical` chain, `can_pair_with`, `filtered_units` chain,
   `SemanticTree.source_ranges`, the unreachable `Deletion` classification
   path), consolidate the 3 copies of `get_display_node` and 2 copies of
   `ranges_overlap`, remove `_node_has_keyword` and
   `extract_expanded_source_text`.
7. **Error-handling cleanup** — `SemanticError` gets `thiserror`;
   `DecisionDiffError` preserves source chains instead of stringifying.
8. **Micro-simplifications** — `determine_line_relevance_with_precedence`
   reduces to `min()`; `parameter_count` uses `named_child_count()`.

**Out of scope (deferred, see decision log)**

- GumTree-style AST matching / rename detection (`ASTChangeType::Rename`,
  `Reorder` stay unproduced).
- Unifying `RenderableDiff` line-number coordinate frames (boundary-relative vs
  file-absolute) — behavior change for TUI consumers, own plan.
- String-based semantic anchors / boundary-name extractors — owned by the
  in-flight `plan-semantic-anchors-tree-sitter`.

## Interaction with in-flight work

`plan-semantic-anchors-tree-sitter` (TDD, phases 0–1 contributed) is migrating
`semantic_anchors.rs` string scanning to tree-sitter. This plan **must not**
touch anchor *extraction*. The diff-engine phase changes how anchors are
*consumed*: `myers_diff_semantic(&[(&str, Option<SemanticAnchor>)], …)` is
replaced by diff-then-align-post-pass. The post-pass consumes the same
`Option<SemanticAnchor>` per line, so the extraction interface both plans share
is stable. If the anchors plan lands first, its Phase-0 behavioral tests are
part of this plan's safety net; coordinate before starting Phase 2 here.

## Codebase Patterns to Follow

- **Testing**: TDD; bug reproductions live in `tests/bug_*.rs` (failing test
  first, then fix, then update `bugs.md`). Shared helpers in
  `tests/test_utils.rs`.
- **Errors**: `thiserror` with structured variants, `#[source]`/`#[from]`
  chains preserved (root CLAUDE.md).
- **Zero warnings**: `cargo fmt` + `cargo clippy` + `cargo check` clean after
  every change.
- **Core crate rules**: tree-sitter only for code analysis; no fallbacks —
  fail fast (diffviz-core CLAUDE.md). Several improvements in this plan
  *restore* compliance with these rules (removing `find_original_line_content`
  fallback, `else { 0 }` in backtrack, defensive `unwrap_or`s).
- **Parser architecture**: descriptor + generic-builder pattern
  (`LanguageDescriptor` trait, `GenericSemanticTreeBuilder<D>`); language
  newtypes override only language-specific behavior.

## Technical Constraints

- `diffviz-core` is the domain core: no dependencies on review/git layers.
  New deps this plan introduces: `similar` (runtime) and `proptest`
  (dev-dependency only). Both added at workspace level per existing convention.
- Public API consumers to keep compiling: `diffviz-review` uses
  `create_reviewable_diff_from_range` (review_engine_builder.rs:110) and
  `RenderableDiff::try_from` (engines/review_engine/mod.rs:132,160). Verified:
  no external users of `is_semantically_identical`, `can_pair_with`,
  `filtered_units`, `container_body_field` — free to delete/change.
- Workspace must stay green (build + tests + clippy) at the end of every phase.

## Research Findings

**`similar` crate** (v2.x, MIT, used by insta among others):
- `similar::TextDiff::configure().algorithm(Algorithm::Patience).diff_lines(old, new)`
  yields ops over line indices; `iter_all_changes()` gives
  `ChangeTag::{Equal, Delete, Insert}` with `old_index()`/`new_index()` —
  exactly the index-carrying shape this plan wants for `DiffOp`.
- Patience diff anchors on unique lines → the human-readable diffs review
  tooling wants; also structurally eliminates the need for
  `merge_identical_add_delete_pairs`.
- `similar` operates on `&str` lines without allocating per line; our
  `DiffOp { …_idx }` conversion is a thin mapping layer.

**Reconstruction property** (the invariant that caught the original bug):
replaying the ops must rebuild `old` exactly (Keep + Delete + Modify.old in
order) and `new` exactly (Keep + Add + Modify.new in order). With `proptest`
over `Vec<String>` pairs this is ~40 lines and permanent.

## Local Repository Skills

- `diffviz-tui-contribution` — mandatory when touching `diffviz-review-tui`
  (not expected in this plan, but Phase 2/3 renderable changes may prompt TUI
  verification; use the harness, agents have no TTY).
- `filing-bugs` — convention already followed for the filed bugs this plan
  absorbs.
