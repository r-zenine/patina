# Context Handoff - Phase 3 Sufficient Design

## Core Result (What agents get from this work)

**Built**: Two sufficient design improvements to `diffviz-cli/src/commands/debug.rs`:

1. **Phase 1 & 2 stubs removed** — both now return `None` (honest: ReviewEngine has no semantic tree or pairing data; `None` causes serde to omit the field from JSON output)
2. **`serialize_impact_phase` helper extracted** — phases 6 & 7 shared ~80 lines of near-identical logic; now a single private method parameterized by `type_label`

**Net change**: -28 lines. 9 tests pass. Zero clippy warnings.

## Deferred: Phase 3b (State Persistence Move)

Phase 3b (move `PersistedApproval`, `ReviewStateFile`, `load_review_state`, `save_review_state` from `main.rs` to `diffviz-review::persistence`) was deferred — it crosses crate boundaries and requires design coordination (use `design-contribute` skill before implementing).

## Current State

**All three roadmap goals addressed**:
- Phase 1 (--phase filtering): shipped ✓
- Phase 2 (YAGNI removals): shipped ✓  
- Phase 3a (extract duplicate serialization): shipped ✓
- Phase 3c (assess stubs): decided & shipped ✓ (removed — no data source)
- Phase 3b (state persistence move): deferred to design phase

**Remaining**: Phase 3b is the only outstanding item. It's a clean architecture improvement, not a bug fix.

## Next Agent Guidance

If Phase 3b is to be tackled:
1. Use `design-contribute` skill to design the `diffviz-review::persistence` API first
2. Key questions to resolve: module location (`persistence.rs` at crate root vs under engines), public API surface (`ReviewEngine` method vs standalone functions), backward compat of `review-state.json` format
3. Only after design is signed off: move the 5 items from `main.rs:100-203` to the new module

## Integration Points

**Phase 7 JSON shape change**: `"summary"` key renamed to `"impacted_areas"` (matches phase 6). Both are debug output; this is not a stable public contract. If downstream tooling parses `diffviz debug` output and relies on `"summary"`, it needs updating.

## Reference Links
- [decision-log.yaml](decision-log.yaml) — Technical decisions and code impacts
- [Plan implementation roadmap](../../implementation-roadmap.md) — Full three-phase roadmap
