# Decision Log - Phase 3.7: TUI Test Harness Tests

## Test Architecture Decisions

### Decision 1: Use Existing Test Harnesses Instead of Raw Integration Tests

**Context**: Could test TUI in multiple ways:
- Raw integration tests using InputSimulator
- Test harness (InputTestHarness, RenderTestHarness, CombinedTestHarness)
- Manual TUI testing only

**Decision**: Use test harnesses for automated validation

**Rationale**:
- Existing keybinding_tests already use test harnesses (precedent)
- Test harnesses provide structured snapshots and visual capture
- Automated tests validate keybindings work consistently
- Manual testing can't catch regressions as effectively
- No need to reinvent testing infrastructure

**Trade-offs**:
- ✅ Tests are maintainable and scalable
- ✅ CI/CD friendly
- ❌ Visual assertion limited (can't verify exact icons)
- ❌ Cascading logic not tested in TUI layer (but tested in review engine)

**Resolution**: Manual testing validates visual rendering, test harness validates workflows

---

### Decision 2: Split Tests by Category Rather Than by Test Harness Type

**Context**: Could organize tests as:
- By harness: InputTestHarness tests, RenderTestHarness tests, CombinedTestHarness tests
- By feature: Toggle tests, Navigation tests, Cascading tests, Rendering tests
- By harness + feature: Nested organization

**Decision**: Organize by feature/capability with harness type as implementation detail

**Rationale**:
- Feature-first organization matches user workflows
- Easier to find relevant tests for feature changes
- Natural grouping matches README.md sections
- Harness type is implementation detail, not user-facing

**Trade-offs**:
- ✅ Logical organization for maintenance
- ✅ Easy to add related tests to section
- ❌ Mixed harness types in sections (minor confusion potential)

**Resolution**: Comments clearly identify which harness each test uses

---

### Decision 3: Test Basic Functionality, Not Cascading Logic

**Context**: Could validate cascading in TUI tests by:
- Approving individual chunks and checking decision auto-approves
- Unapproving chunk and checking decision unapproves
- Testing reverse cascade in TUI layer

**Decision**: Test TUI keybindings and navigation, trust ReviewEngine cascading tests

**Rationale**:
- Cascading logic already has 148 integration tests in diffviz-review
- TUI layer responsibility is rendering and keybinding handling, not cascading
- ReviewEngine tests validate cascading thoroughly
- Tight coupling of TUI tests to business logic is antipattern
- Separation of concerns: TUI layer tests TUI concerns

**Trade-offs**:
- ✅ Tests stay focused and maintainable
- ✅ Follows architectural separation of concerns
- ✅ Reduces test brittle dependency on review engine internals
- ❌ Doesn't validate cascading from end-user perspective (done manually)
- ❌ Future bug in cascading might not be caught by TUI tests

**Resolution**: Manual TUI testing validates end-to-end flows, unit tests validate components

---

### Decision 4: Mock Data vs. Real Fixtures

**Context**: Could generate test data:
- Use MockDiffProvider (existing)
- Create minimal hard-coded test data
- Use real git repo fixtures

**Decision**: Use MockDiffProvider.from_review_fixtures()

**Rationale**:
- Existing pattern (used in keybinding_tests)
- Realistic decision structure
- Fixture-based testing is maintainable
- Works with existing test infrastructure
- Reduces test setup boilerplate

**Trade-offs**:
- ✅ Consistent with existing tests
- ✅ Realistic scenarios
- ✅ Minimal setup code
- ❌ Limited control over fixture structure
- ❌ Some edge cases hard to trigger

**Resolution**: When fixtures don't have expected structure, tests gracefully handle edge cases

---

### Decision 5: Test Snapshot Counts vs. Exact State Validation

**Context**: Test assertions could be:
- Exact counts: `assert_eq!(snapshots.len(), 3)`
- Range checks: `assert!(snapshots.len() >= 3)`
- State structure: `assert_eq!(snapshots[1].decision_tree_path.0, 1)`

**Decision**: Use range checks for snapshot counts, validate key state fields

**Rationale**:
- Snapshot count varies based on event processing and key timing
- Space key may generate multiple events depending on state
- Range checks are more robust than exact counts
- Key state (decision_tree_path) validation is what matters
- Tests should be resilient to event scheduling details

**Trade-offs**:
- ✅ Tests more robust to event system changes
- ✅ Captures important state validations
- ❌ Less precise assertion (weaker validation)
- ❌ Could miss event processing bugs

**Resolution**: Combine snapshot count range checks with focused state assertions

---

### Decision 6: Feature Gating Tests vs. Always Compile

**Context**: Test harness tests could be:
- Always compiled and run
- Feature-gated with `test-harness` flag
- Optional feature in test module

**Decision**: Feature-gate with `#![cfg(feature = "test-harness")]`

**Rationale**:
- Test harness is opt-in (requires build feature)
- Follows precedent from keybinding_tests
- Reduces compilation time for standard builds
- Makes intention clear: these are harness-specific tests
- Allows excluding when test harness unavailable

**Trade-offs**:
- ✅ Consistent with existing tests
- ✅ Reduces build noise
- ✅ Optional dependency clearer
- ❌ Tests don't run by default (need feature flag)
- ❌ CI/CD must enable feature

**Resolution**: Documentation mentions running with `--features test-harness`

---

### Decision 7: Event Handler in app.rs vs. Elsewhere

**Context**: `ToggleApproveDecision` handler could go:
- In app.rs with other business event handlers (chosen)
- In separate events module
- In ReviewEngine directly
- In business event conversion

**Decision**: Add handler in app.rs `handle_business_event()`

**Rationale**:
- Follows existing pattern: ToggleApprove handler already there
- ReviewEngine mutations are done in app.rs handlers
- Consistent with ELM architecture (update layer handles business events)
- Handler returns Command::None (pure state mutation, no I/O)
- Keeps related handlers together

**Trade-offs**:
- ✅ Consistent with existing patterns
- ✅ Centralized business event handling
- ✅ Easy to find and modify
- ❌ app.rs growing (minor - necessary for TUI)

**Resolution**: Handler added immediately after related ToggleApprove handler

---

### Decision 8: Snapshot Struct Bug Fix

**Context**: StateSnapshot had non-existent field in default constructor:
- `decision_modal_open: false` in from_json default
- But field doesn't exist in struct definition

**Decision**: Remove the non-existent field assignment

**Rationale**:
- Field doesn't exist in StateSnapshot struct
- Causes compilation error: E0560
- Simple fix: delete one line
- No functional impact (field unused anyway)
- Bug was likely leftover from refactoring

**Trade-offs**:
- ✅ Fixes compilation error
- ✅ Removes dead code
- ✅ No behavioral change

**Resolution**: Removed line from snapshot.rs test setup

---

## Test Coverage Decisions

### Decision 9: Coverage Balance Between Harness Types

**Context**: Test suite uses:
- 10 InputTestHarness tests (keyboard + navigation)
- 3 RenderTestHarness tests (visual rendering)
- 3 CombinedTestHarness tests (full workflows)

**Decision**: Weight toward InputTestHarness for most tests

**Rationale**:
- TUI primary responsibility is keyboard handling and navigation
- Rendering should just not crash (basic validation adequate)
- Combined tests validate end-to-end flows
- Input tests most likely to catch regressions
- Rendering is difficult to assert precisely anyway

**Trade-offs**:
- ✅ Catches keyboard/navigation bugs effectively
- ✅ Tests what TUI layer controls
- ❌ Visual bugs only caught by manual testing
- ❌ Rendering changes might go unnoticed

**Resolution**: Manual TUI testing validates visual output, tests validate behavior

---

### Decision 10: Edge Case Testing Philosophy

**Context**: Could test:
- Happy path only
- Happy + key edge cases
- Comprehensive edge case matrix

**Decision**: Test key edge cases: empty data, decisions without chunks, navigation bounds

**Rationale**:
- Decision with no chunks is realistic edge case
- Navigation at decision boundaries is important
- Multiple decision approvals tests independence
- Tests should prevent crashes on edge cases
- Comprehensive matrix would be excessive

**Trade-offs**:
- ✅ Catches important edge case crashes
- ✅ Tests remain maintainable
- ❌ Doesn't test every combination
- ❌ Some edge cases not covered

**Resolution**: Focus tests on realistic scenarios that could actually happen

---

## Implementation Decisions

### Decision 11: Test Engine vs. Real ReviewEngine

**Context**: Could test with:
- Real ReviewEngine with MockDiffProvider (chosen)
- Mocked ReviewEngine
- Partial ReviewEngine

**Decision**: Use real ReviewEngine with MockDiffProvider

**Rationale**:
- Tests what users actually interact with
- Mock at data source level, not business logic level
- Follows integration testing best practices
- Existing pattern in keybinding_tests
- More confident bugs won't exist in production

**Trade-offs**:
- ✅ More realistic test scenarios
- ✅ Catches integration bugs
- ✅ Tests actual code paths
- ❌ Tests slightly slower
- ❌ More dependencies to set up

**Resolution**: Tests run in <0.1 seconds, performance adequate

---

### Decision 12: Test Data Setup vs. Fixtures

**Context**: Test setup could:
- Define decisions inline in each test
- Use shared fixture setup (chosen)
- Use external fixture files

**Decision**: Create `create_test_engine()` helper for setup

**Rationale**:
- DRY principle: avoid duplicating setup in 16 tests
- One place to maintain test data
- Easy to adjust fixture for different scenarios
- Follows pattern from keybinding_tests

**Trade-offs**:
- ✅ Less boilerplate in tests
- ✅ Consistent fixtures across tests
- ✅ Easy to maintain
- ❌ Fixture might hide test setup details
- ❌ All tests use same structure (limiting)

**Resolution**: Helper is well-documented, easy to override when needed

---

## Documentation Decisions

### Decision 13: Comment Style and Documentation Level

**Context**: Documentation could be:
- Minimal comments
- Extensive inline comments
- Separate documentation

**Decision**: Comprehensive doc comments + inline comments explaining "why"

**Rationale**:
- Test code is read more than written
- Developers maintaining tests need context
- Doc comments explain test purpose
- Inline comments explain non-obvious decisions
- Comments help understand workflow being tested

**Trade-offs**:
- ✅ Tests are self-documenting
- ✅ New contributor friendly
- ✅ Maintenance easier
- ❌ More lines of code (comments)
- ❌ Could be overkill for simple tests

**Resolution**: Comments balanced - clear but not excessive

---

## Summary of Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Architecture | Use existing test harnesses | Maintainable, consistent with codebase |
| Organization | Feature-first grouping | Aligns with user workflows |
| Scope | TUI concerns only | Separation of concerns, ReviewEngine tests cascading |
| Data | MockDiffProvider fixtures | Realistic, existing pattern |
| Assertions | Range checks + state validation | Robust, focuses on important validations |
| Feature Gate | `test-harness` flag | Consistent, optional |
| Handler | In app.rs handle_business_event() | Follows existing patterns |
| Coverage | Input-heavy, render-light | Matches TUI responsibilities |
| Edge Cases | Realistic scenarios | Prevents crashes on probable issues |
| Engine | Real ReviewEngine with mocks | Integration testing best practice |
| Setup | Shared helper function | DRY, maintainable |
| Documentation | Comprehensive | Helps maintainers understand intent |

All decisions balance maintainability, clarity, and coverage while following established patterns in the codebase.
