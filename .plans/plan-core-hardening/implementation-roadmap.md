# Implementation Roadmap — core-hardening

**Strategy**: TDD — Phase 0 builds the safety net (behavior-pinning tests +
failing reproductions for the filed bugs), then every phase goes red→green.
Each phase is a complete deliverable: the workspace builds, the full suite
passes (minus explicitly `#[ignore]`d not-yet-fixed reproductions), zero
warnings.

**Total phases**: 7 (Phase 0 + 6). Phases 0–2 are specified in full detail;
phases 3–6 carry enough to start and defer fine-grained decisions per Last
Responsible Moment. Line references are in `code-context.md`.

**Standing acceptance criteria for every phase** (not repeated below):
`cargo build --workspace` / `cargo test --workspace` green,
`cargo clippy --workspace` and `cargo fmt --all -- --check` clean,
`diffviz-review` and `diffviz-cli` compile with zero source changes unless the
phase explicitly says otherwise.

---

## Phase 0 — Safety net (tests only, no production code)

**Deliverable**: Every behavior this plan must preserve is pinned by a test;
every defect this plan must fix has a failing reproduction, `#[ignore]`d with a
reference to this plan. `bugs.md` lists them as active.

### Objectives

1. **Reconcile with the filed bugs** — the bug-filing pass ran in a separate
   conversation; inventory `tests/bug_*.rs` and `bugs.md` first. For each of
   the five verified findings below, reuse the filed test if it exists,
   otherwise create it (failing, `#[ignore = "fixed by plan-core-hardening phase N"]`):
   - Myers drops lines: old `["a"]` → new `["a","a","a"]` renders an empty
     diff; inserting a blank line next to an existing blank line loses lines.
     Drive through the public path (`create_reviewable_diff_from_range` on a
     Modification + `RenderableDiff::try_from`), asserting rendered lines
     reconstruct both sources. *(fixed by Phase 2)*
   - Decompose path drops the unit ending on the range's end line (range
     covering exactly two complete functions yields 1 diff). *(Phase 3)*
   - `line_range` off-by-one for `OwnedNodeData` starting at column 0.
     *(Phase 3)*
   - Cross-impl mispairing: `impl A { fn get }` / `impl B { fn get }`, editing
     `B::get` pairs against `A::get`. *(Phase 4)*
   - Python module-level assignment range → `NoUnitsInRange`. *(Phase 5)*
2. **Add the reconstruction property test** (`tests/diff_reconstruction.rs`) —
   `proptest` as workspace dev-dependency: for random `(Vec<String>, Vec<String>)`
   pairs, the rendered Modified diff's lines must reconstruct the old source
   (context + deleted + Modify-old lines in order) and the new source (context
   + added + Modify-new lines in order). `#[ignore]` until Phase 2 (it fails
   today — that is the point).
3. **Pin currently-correct behavior** that later phases must not break
   (only where the existing 57 tests leave gaps — check first):
   - Single-source rendering (Added boundary): line numbering, annotations,
     `should_fold` behavior on a known fixture.
   - Expand path of `create_reviewable_diff_from_range`: single-unit range
     returns that unit; identical-content units are skipped in the decompose
     path.
   - `boundary_name` extraction for fn/struct/enum boundaries.
   - Anchor *consumption*: a Modify pair (same anchor, different content)
     renders as Delete-then-Add adjacent lines. Coordinate with
     `plan-semantic-anchors-tree-sitter` Phase 0 tests — reuse, don't duplicate.

### Acceptance Criteria
- All new reproductions fail when un-ignored (verified once, then re-ignored).
- Suite green with reproductions ignored; `bugs.md` updated to list them as
  active with plan reference.

---

## Phase 1 — Dead-code sweep, error cleanup, micro-simplifications

**Deliverable**: The legacy semantic-pairing API surface is gone, error types
meet the project standard, duplicated helpers are consolidated. Pure
deletion/mechanics — no behavior change (suite is the witness).

### Objectives

1. **Delete dead API** (verified zero users outside `diffviz-core/src`):
   `is_semantically_identical` + `unit_types_are_identical` +
   `compare_optional_nodes`; `can_pair_with`; `filtered_units` +
   `collect_filtered_units` + `should_include_unit` + `estimate_node_size` +
   `find_units_by_type`; `SemanticTree.source_ranges`;
   `ChangeClassification::Deletion` + `create_deletion_diff` (unreachable, and
   wrong — reads `new_unit`); `_node_has_keyword` (rust.rs);
   `extract_expanded_source_text` (line_utils.rs). Remove the corresponding
   re-exports from `lib.rs` (`ChangeClassification` loses a variant — keep the
   enum, it's re-exported). Delete tests that exist solely to exercise deleted
   items.
2. **Consolidate `get_display_node`** → `impl NodeChangeStatus` with two
   methods: `display_node(&self) -> &OwnedNodeData` and
   `display_node_with_source<'a>(&'a self, old: &'a dyn SourceProvider, new: &'a dyn SourceProvider)
   -> (&'a OwnedNodeData, &'a dyn SourceProvider)` (the Deleted-aware variant
   from `line_utils.rs:86-97` is the canonical one). Replace the three copies;
   fix `extract_boundary_name` and `overall_line_range` to use the
   source-aware variant.
3. **Consolidate `ranges_overlap`** to one helper (kept tuple-based this phase;
   retyped in Phase 3).
4. **Error types**: `SemanticError` → `#[derive(thiserror::Error)]`;
   `DecisionDiffError::ParseError`/`SemanticError` carry
   `#[source] ASTError` / `#[source] SemanticError` instead of `String`
   (callers at decision_based_diff.rs:509-516,569-578,647-654 stop
   `format!`-ing).
5. **Micro-simplifications** (each with a small test if uncovered):
   - `determine_line_relevance_with_precedence` → single
     `.filter(overlaps).map(relevance).min().unwrap_or(ESSENTIAL)` chain, with
     a comment documenting the deliberate ESSENTIAL default (decision D011).
   - `parameter_count` → `named_child_count()` (generic_builder.rs:202-204);
     assert `fn f(a: i32, b: i32)` reports 2.

### Acceptance Criteria
- Suite green with no test deletions beyond those covering deleted API.
- `grep` finds no references to the deleted symbols anywhere in the workspace.

---

## Phase 2 — Diff engine replacement (fixes: Myers drops lines; CRLF drift)

**Deliverable**: `myers_diff.rs` is replaced by a `similar`-backed engine with
index-carrying ops and an anchor-alignment post-pass. The reconstruction
property test and the Myers bug reproductions are un-ignored and green.

### Objectives

1. **Add `similar`** (workspace dependency, `diffviz-core` consumer).
2. **New module `renderable_diff/line_diff.rs`** (replaces `myers_diff.rs`):
   - `DiffOp` becomes index-carrying:
     `Keep { old_idx: usize, new_idx: usize }`, `Add { new_idx: usize }`,
     `Delete { old_idx: usize }`,
     `Modify { old_idx: usize, new_idx: usize }`.
   - `diff_lines(old: &[&str], new: &[&str]) -> Vec<DiffOp>` via
     `TextDiff::configure().algorithm(Algorithm::Patience)`, mapping
     `ChangeTag::{Equal, Delete, Insert}` + `old_index()`/`new_index()` to ops.
   - `align_by_anchors(ops, old_anchors: &[Option<SemanticAnchor>], new_anchors: &[Option<SemanticAnchor>]) -> Vec<DiffOp>`
     — pure post-pass: within each adjacent Delete-run/Add-run pair, convert
     positionally-matching pairs whose anchors satisfy today's
     `semantically_related` predicate into `Modify`. Unit-test the post-pass
     directly (anchored pair → Modify; anchor mismatch → untouched;
     no anchors → untouched).
   - Delete `merge_identical_add_delete_pairs`, `lines_should_align`, and the
     hand-rolled forward pass/backtrack entirely.
3. **Rewrite `create_line_by_line_diff_for_modified`** (renderable_diff/mod.rs):
   - Line content comes from `old_lines[op.old_idx]` / `new_lines[op.new_idx]`
     — **delete `find_original_line_content`**.
   - Byte ranges come from precomputed per-line offsets of the extracted texts
     (accumulate `line.len() + actual newline width`, or reuse Phase 3's
     LineIndex when it lands — don't block on it), offset by
     `new_node.start_byte`. This kills the hardcoded `+1 for newline` and
     fixes CRLF drift; add a CRLF fixture test.
4. **Un-ignore and green**: the Myers reproductions from Phase 0, the
   reconstruction property test, and the existing
   `renderable_diff_anchor_tests.rs` suite (anchors consumed by the post-pass
   must preserve its observable behavior — if any assertion legitimately
   changes because Patience output differs benignly from broken-Myers output,
   document each change in the contribution notes).

### Acceptance Criteria
- Property test green over 1000+ proptest cases.
- Myers bug reproductions green and moved to fixed in `bugs.md`.
- `myers_diff.rs` no longer exists.

---

## Phase 3 — LineIndex + typed half-open ranges (fixes: end-line exclusion; column-0 off-by-one)

**Deliverable**: One line-offset index owns every line↔byte conversion; byte
ranges are `Range<usize>` throughout; the two arithmetic reproductions are
green.

### Objectives

1. **`LineIndex`** (new `ast_diff/line_index.rs`, owned lazily or eagerly by
   `SourceCode`): `Vec<usize>` of line-start offsets built in one pass;
   `line_count()`, `line_start(line) -> Option<usize>`,
   `byte_to_line(offset) -> usize` (partition_point),
   `byte_range_of_lines(start_line..=end_line) -> Range<usize>` — the ONLY
   place inclusive lines convert to half-open bytes: `line_start(a) ..
   line_start(b+1).unwrap_or(EOF)`. TDD: unit tests first (empty source,
   no trailing newline, trailing newline, CRLF, single line, out-of-bounds).
2. **Replace the four hand-rolled conversions**: `line_to_byte_offset`,
   `count_lines`, `clamp_line_range` (decision_based_diff.rs) and
   `line_range_from_bytes` (source.rs) — `line_range` for `OwnedNodeData`
   computes both endpoints through the index (fixes column-0 off-by-one).
3. **Fix the decompose range semantics**: `end_byte` becomes the end of
   `end_line` via `byte_range_of_lines`. Then test whether
   `find_units_touching_range_recursive` is still reachable; if its fallback
   never fires under the corrected semantics (expected), delete it and its
   error path.
4. **Retype byte ranges** as `Range<usize>`: `ByteRangeAnnotation`,
   `NodeAnnotation`, `RenderableLine.byte_range`, `LineInfo.byte_range`, the
   consolidated `ranges_overlap` (or replace with a small
   `overlaps(&Range, &Range)` in one util). Mechanical; suite is the witness.
   Evaluate merging `build_byte_range_annotations` with
   `collect_all_annotations` while both are open.

### Acceptance Criteria
- End-line-exclusion and column-0 reproductions green; `bugs.md` updated.
- No `(usize, usize)` byte-range tuples remain in the crate.

---

## Phase 4 — Qualified-name unit matching (fixes: cross-impl mispairing)

**Deliverable**: Unit identity is (container path, unit type, name); the
mispairing reproduction is green.

### Objectives

1. **Carry a qualified path**: extend the builder to accumulate a container
   path (seeded by `build_impl_container`'s existing `parent_context`, extended
   to named modules — and to classes once Phase 5 recurses into them). Store it
   on `SemanticNode` (e.g. `qualified_name: Option<String>`, `"Type::name"` /
   `"mod::name"`) and mirror it through `OwnedNodeData` the same way
   `identifier` already flows.
2. **Match on it**: `find_semantic_unit_by_name` compares qualified name +
   type discriminant. Nameless-unit matching (`(None, "")`) keeps current
   behavior. Both call sites in `create_reviewable_diff_from_range` follow.
3. **Tests first**: the Phase-0 mispairing reproduction, plus: same-named fns
   in sibling `mod`s; method vs free function with the same name; renamed
   container (old counterpart correctly NOT found → Addition).

**Coordination**: `plan-semantic-anchors-tree-sitter` also touches
`extract_identifier`; rebase over whatever has landed before starting.

### Acceptance Criteria
- Mispairing reproduction green; `bugs.md` updated.
- Documented limitation (decision D007): same qualified name + same unit type
  at the same nesting level (e.g. TS declaration merging) still first-match.

---

## Phase 5 — Container recursion mechanism + core 4 languages (fixes: invisible Python assignments/class bodies)

**Deliverable**: The generic builder recurses into data-structure bodies and
statement wrappers via `container_body_field`; Rust, Python, Go, TypeScript
descriptors are wired and tested; the Python reproduction is green.

### Objectives

1. **Mechanism (generic_builder.rs)**:
   - `build_data_structure` collects children from the node's body (via
     `container_body_field`, replacing the hardcoded `"body"` in the two
     existing container builders too), passing the data structure's name as
     `parent_context` so `is_method` becomes true for class methods.
   - Statement-wrapper recursion: the `_ => None` arm of `build_typed_node`
     recurses into wrapper kinds (per-descriptor list or reuse of
     `build_container_children`) so `expression_statement → assignment`
     surfaces as Variable. Decide wrapper-kind representation when
     implementing (Design note: prefer a `statement_wrapper_kinds()` descriptor
     method over hardcoding, consistent with the descriptor pattern).
   - Revisit `field_count`'s hardcoded Rust kinds while in here (per-language
     or drop the count — decide at implementation, it has no consumer logic).
2. **Per-language wiring + tests** (Rust, Python, Go, TypeScript): for each, a
   fixture test proving (a) a range over a method inside a
   class/impl resolves to that method, not the whole container; (b) module-
   level variable ranges resolve to Variable units; (c) the byte-coverage/
   containment invariant (`assert_byte_coverage_invariant`) holds on fixtures.
3. **Un-ignore** the Python module-level assignment reproduction.

### Acceptance Criteria
- Python reproduction green; `bugs.md` updated.
- For each core language: method-inside-container range test green.
- Existing decompose/expand behavior for Rust impls unchanged (Phase 0 pins).

---

## Phase 6 — Container recursion: remaining 4 languages

**Deliverable**: JavaScript, Java, C, C++ descriptors wired to the Phase-5
mechanism with the same per-language test triple.

### Objectives

1. Wire `container_body_field` / wrapper kinds for JavaScript
   (`class_declaration`/`class_body`), Java (`class_declaration`,
   `interface_declaration`, `enum_declaration` bodies), C (struct fields are
   likely sufficient as-is — verify, C has no methods; cover `declaration`
   wrappers), C++ (`class_specifier`/`struct_specifier` field lists, methods,
   namespaces). Consult each grammar's node kinds at implementation time —
   don't trust memory, print real trees for the fixtures first.
2. Same test triple per language as Phase 5.
3. Sweep each descriptor's kind map for construction kinds that are still
   unreachable after recursion lands; delete or wire them — no aspirational
   map entries left.

### Acceptance Criteria
- Per-language method-inside-container tests green for all 8 languages total.
- `bugs.md` shows no active bugs from this plan; all Phase-0 reproductions
  un-ignored and green.

---

## Deferred (explicitly out of scope — decision D012)

- GumTree-style matching / rename detection (`ASTChangeType::Rename`/`Reorder`
  remain unproduced).
- RenderableDiff line-number coordinate-frame unification (TUI-affecting).
- Anchor extraction migration (owned by `plan-semantic-anchors-tree-sitter`).
- `LineNr`/`ByteOffset` newtypes beyond `Range<usize>`.
