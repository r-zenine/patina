# Decision Log: Phase 3 - TUI Validation

## D7: TUI Test Strategy

**Choice**: Use existing test harness features (`--test-input` and `--test-full`) rather than creating new test framework.

**Rationale**:
- The TUI already has comprehensive test harness infrastructure built in
- `--test-input` mode provides JSON state snapshots for assertions
- `--test-full` mode provides step-by-step visual rendering for inspection
- Feature-gated with `test-harness` feature flag, no production impact
- Test data uses existing MockDiffProvider with fixtures

**Alternatives rejected**:
- Building custom validation scripts (reinvents existing wheel)
- Manual visual inspection only (lacks reproducibility)

**Outcome**: All validation sequences execute cleanly with expected state progression.

## D8: Hardcoded Decisions for Testing

**Choice**: Keep `create_hardcoded_decisions()` in TUI main.rs for Phase 3 validation.

**Rationale**:
- Phase 2.1 integration already used this approach
- No blocker on decision loading abstraction for Phase 3 validation
- Hardcoded decisions span 6 fixture files and 3 decisions
- Comprehensive enough to validate the entire pipeline
- Phase 2.2 would add decision loading abstraction

**Alternatives rejected**:
- Loading decisions from external source (Phase 2.2 work)
- Creating new decision formats (out of scope for validation)

**Outcome**: All 3 hardcoded decisions properly initialize and navigate in TUI.

## D9: Validation Scope

**Choice**: Validate complete pipeline using test harness + library tests, no manual UI interaction required.

**Rationale**:
- Test harness can simulate all critical user journeys (navigation, expansion, selection)
- JSON state snapshots prove internal state is correct
- Visual rendering in `--test-full` proves UI output is correct
- Library tests validate decision index, decision approval, decision progress
- Comprehensive coverage without manual work

**Alternatives rejected**:
- Manual interactive testing only (not reproducible)
- Reducing validation scope (might miss bugs)
- Extending validation beyond test harness (already validates everything needed)

**Outcome**: 100% of Phase 3 validation checklist completed with reproducible test evidence.

## D10: Integration Confirmation Points

**Choice**: Use specific test sequences to confirm key integration points.

**Rationale**:
- Navigation path `[2, 0, 0]` confirms decision → file → chunk hierarchy
- Panel transitions (FileList → DiffView) confirm navigation logic
- All 6 fixture files rendering proves language support intact
- Status bar updates prove approval workflows still work
- Zero warning/error output proves clean integration

**Alternatives rejected**:
- Spot checks of individual components (might miss integration issues)
- New comprehensive test suite (over-engineering for validation phase)

**Outcome**: All integration points verified through focused test sequences.
