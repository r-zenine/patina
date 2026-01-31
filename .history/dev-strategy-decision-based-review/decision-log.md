# Decision Log: Decision-Based Review Pipeline

## D1: Pipeline Location
**Choice**: New module in diffviz-core (`decision_based_diff.rs`)
**Rationale**: Keeps core analysis logic in the core crate. ReviewEngineBuilder calls into it.
**Alternatives rejected**: New path in ReviewEngineBuilder (leaks core logic into review layer), replace existing path in-place (harder to develop incrementally).

## D2: Unmapped Code Handling
**Choice**: Ignore unmapped code entirely
**Rationale**: Only what decisions specify gets reviewed. Simplifies the system — no more Decision 0, no more `create_unmapped_decision()`.
**Alternatives rejected**: Keep git discovery for unmapped (adds complexity), warn about unmapped (half-measure).

## D3: Old-Version Range Discovery
**Choice**: Parse both files, match by semantic unit name
**Rationale**: Lightweight O(n) lookup — find the same-named unit in the old file's semantic tree. Not a full pairing: we already know which unit to find from the decision's range in the new file.
**Alternatives rejected**: Same range in both versions (breaks when lines shift), diff whole file first (unnecessary overhead).

## D4: Legacy Semantic Pairing
**Choice**: Remove entirely
**Rationale**: Full replacement. Decisions are the only way to create ReviewableDiffs going forward. Not in production, so no migration concerns.
**Alternatives rejected**: Keep both paths (maintenance burden for unused code).

## D5: Implementation Strategy
**Choice**: Core-then-Integrate
**Rationale**: Not in production, so no integration risk to front-load. Build bulletproof business logic first with comprehensive tests, then wire into the system.
**Alternatives rejected**: Steel Thread (integration risk isn't the concern here), TDD (behavior is clear, uncertainty is low).

## D6: Phase Structure
**Choice**: 5 phases — Core, Integrate, TUI Validation, Remove Old Path, Cleanup
**Rationale**: TUI validation phase acts as a gate between integration and removal. We don't delete anything until the TUI works end-to-end with the new approach. Final cleanup phase sweeps all orphaned code.
