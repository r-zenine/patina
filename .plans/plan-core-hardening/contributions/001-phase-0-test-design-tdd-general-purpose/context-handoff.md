# Context Handoff - Phase 0 Test Design (Safety Net)

## 🎯 Core Result
**Built**: The reconstruction property test (`tests/diff_reconstruction.rs`, `#[ignore]`d)
and one new behavior pin (`tests/pin_added_boundary_rendering.rs`, active). Confirmed the
5 headline bugs plus 7 more were already filed with `#[ignore]`d failing tests by an
earlier bug-filing pass — reused them as-is, no duplication. Cross-referenced every filed
bug in `bugs.md` with the plan phase (or scope decision) that resolves it.

**Key insight**: `myers_diff_semantic` is a private module — the only way to exercise the
diff engine from a test is through the full `create_reviewable_diff_from_range` +
`RenderableDiff::try_from` pipeline, wrapping generated lines inside a synthetic
`fn f() { ... }`. Arbitrary proptest content breaks tree-sitter's unit boundaries (parens
especially — unbalanced parens fragment the wrapper into multiple nodes, producing
`NoUnitsInRange`, not the diff bug you're trying to trigger). The strategy had to be
narrowed to a small colliding alphabet (`a`, `b`, `stmt`, empty) to make duplicate lines
common — that's what the Myers bug actually needs to fire.

## 🚦 Current State
**✅ Solid foundation**: Baseline is clean (52 passed, 13 ignored, clippy/fmt clean).
Every bug this plan must fix already has a real, verified-failing `#[ignore]`d
reproduction. The reconstruction property test is verified to fail today for the right
reason (a reconstruction mismatch, not a setup error) — checked with
`cargo test --test diff_reconstruction -- --ignored`.

**⚠️ Needs attention**: When Phase 2 lands, re-run the reconstruction test's line
strategy — it's deliberately narrow (4 tokens) to reliably trigger today's bug. Phase 2
may want to widen it (more tokens, longer bodies) now that engine correctness no longer
depends on avoiding parser edge cases in the *diff* logic — but keep the "no braces/no
unbalanced parens" constraint on the wrapper body regardless, that's a tree-sitter
limitation, not a Myers-engine one.

**⏸️ Deferred**: Did not add tests for boundary_name extraction (fn/struct/enum),
decompose-path identical-unit skipping, or single-unit-range expansion — all three
already have real, non-ignored coverage (see decision-log.yaml #4). Did not touch
`bug_mixed_line_number_coordinate_systems.rs` — out of scope per D012, left exactly as
filed.

## 👥 Next Agent Guidance
**Phase 1 (dead-code sweep, error cleanup)**: No blockers from this phase. Proceed per
the roadmap's Phase 1 objectives directly — delete the legacy pairing API, consolidate
`get_display_node`/`ranges_overlap`, migrate `SemanticError`/`DecisionDiffError` to
`thiserror`. Deleting the unreachable `Deletion` path also resolves the
`bug_deleted_boundary_reads_new_source` landmine per D009 — bugs.md already notes this.

**Phase 2 (diff engine replacement)**: `tests/diff_reconstruction.rs` and
`tests/bug_myers_diff_drops_duplicate_lines.rs` are your exit criteria — un-ignore both,
they must go green. `renderable_diff_anchor_tests.rs`'s Myers-path tests
(`myers_path_signature_anchor`, `myers_path_variable_binding_anchor`) must keep passing
through whatever anchor-alignment post-pass replaces `lines_should_align`.

## 🔗 Integration Points
**Expects**: `proptest` now available workspace-wide as a dev-dependency
(`proptest = { workspace = true }` in any crate's `[dev-dependencies]`).
**Provides**: A permanent regression guard for the diff-engine invariant, and a
documentation trail in `bugs.md` linking every filed bug to the phase (or the D012
scope decision) that governs it.

## 📋 Reference Links
- [decision-log.yaml](decision-log.yaml) — decisions made this phase
- `diffviz-core/bugs.md` — updated with phase cross-references
- `diffviz-core/tests/diff_reconstruction.rs`, `diffviz-core/tests/pin_added_boundary_rendering.rs`
