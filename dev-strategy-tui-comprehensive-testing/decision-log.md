# Decision Log: TUI Comprehensive Testing

## Development Strategy

**Question**: Which development strategy should we use?

**Answer**: **Steel Thread**

**Rationale**: The steel thread method is ideal for this testing project because:
- We need to build test coverage progressively from simple to complex
- Each phase should deliver a complete, working test suite for a feature area
- Early phases provide immediate value (basic navigation tests work right away)
- Later phases can build on earlier test patterns
- We want to discover fixture gaps early (can address in parallel with test development)
- The progressive complexity matches how users actually use the TUI (navigate first, then advanced features)

## Test Harness Selection

**Question**: Which test harness should we use for each scenario?

**Answer**: Use harness based on what we're validating:
- **InputTestHarness**: State transitions without visual validation (fast, focused)
- **RenderTestHarness**: Visual output without input sequences (rendering logic only)
- **CombinedTestHarness**: Full integration tests (state + visual, slower but comprehensive)

**Rationale**:
- Using the right harness keeps tests focused and fast
- State-only tests run faster and are easier to debug
- Visual tests validate rendering logic
- Combined tests catch integration issues
- Mix provides comprehensive coverage without redundancy

## Test Organization

**Question**: Where should tests live? How should they be organized?

**Answer**:
- Tests in `diffviz-review-tui/tests/` directory (integration tests)
- One file per feature area (following steel thread phases)
- Module-level helper functions for common setup
- Feature-gated with `#[cfg(feature = "test-harness")]`

**Rationale**:
- Integration test location matches existing pattern (keybinding_tests.rs, decision_approval_tests.rs)
- Feature-based organization makes tests easy to find
- Helper functions reduce duplication
- Feature gating keeps test code out of production

## Fixture Strategy

**Question**: Should we create new fixtures or use existing ones?

**Answer**:
- Start with existing MockDiffProvider fixtures
- Create new fixtures only when existing ones don't cover test scenarios
- Document fixture-to-test mapping

**Rationale**:
- Existing fixtures provide realistic data
- Creating new fixtures only when needed reduces maintenance burden
- Documentation prevents duplicate fixture creation
- Realistic fixtures better than synthetic test data

## Skipped Test Pattern

**Question**: How should we handle failing tests?

**Answer**: Mark with `#[ignore = "Bug #N: description"]` and document expected vs. actual behavior

**Rationale**:
- Matches the bug tracking pattern from CLAUDE.md
- Provides living documentation of what works vs. what doesn't
- Tests can be unskipped when bugs are fixed
- Clear communication of known issues
- Tests serve as bug reproduction cases

## Manual Execution Pattern

**Question**: How should we run tests manually before codifying?

**Answer**:
- Use `--test-input` flag for quick state validation
- Use `--test-full` flag for visual validation
- Document expected behavior before running
- Codify immediately after manual verification

**Rationale**:
- Manual execution catches issues before codifying
- Test harness CLI flags make manual testing easy
- Documenting expectations first ensures tests validate right behavior
- Immediate codification prevents forgetting details

## Progressive Complexity

**Question**: How complex should tests start? How should they progress?

**Answer**:
Start simple within each phase:
1. Single operation (one key press)
2. Simple sequences (2-3 keys)
3. State validation (basic assertions)
4. Visual validation (simple contains checks)
5. Complex scenarios (multi-feature integration)

**Rationale**:
- Simple tests easier to debug when they fail
- Progressive complexity builds confidence
- Early simple tests provide quick wins
- Complex tests catch integration issues
- Matches steel thread methodology

## Assertion Strategy

**Question**: What should tests assert on?

**Answer**:
- **State tests**: Assert on StateSnapshot fields (decision_tree_path, focused_panel, etc.)
- **Visual tests**: Assert on rendered output containing expected UI elements
- **Integration tests**: Assert on both state and visual output
- **Error tests**: Assert tests don't panic/error

**Rationale**:
- StateSnapshot provides reliable state validation
- Visual assertions validate user-facing output
- Combined assertions catch state/visual mismatches
- No-panic assertions validate robustness

## Test Naming

**Question**: How should tests be named?

**Answer**: `test_<feature>_<scenario>_<expected_result>`

Examples:
- `test_navigation_down_moves_cursor`
- `test_approval_toggle_twice_returns_to_unapproved`
- `test_leader_timeout_deactivates_leader`

**Rationale**:
- Clear naming makes test purpose obvious
- Consistent pattern easy to follow
- Expected result in name documents behavior
- Matches existing test patterns in codebase

## Helper Function Strategy

**Question**: Should we create helper functions? Where should they live?

**Answer**:
- Create module-level helpers for common setup (create_test_engine, etc.)
- Extract helpers when used in 3+ tests
- Don't create helpers for one-off operations
- Document helpers with clear comments

**Rationale**:
- Helpers reduce duplication
- Module-level location keeps tests focused
- Three-use rule prevents premature abstraction
- Documentation makes helpers discoverable

## Coverage Goal

**Question**: How much coverage is enough?

**Answer**: Every user-facing feature should have at least one test covering:
- Happy path (expected behavior)
- Edge case (boundary conditions)
- Error case (invalid input)

**Rationale**:
- Every feature tested builds confidence
- Happy path + edge + error provides comprehensive coverage
- More than one test per category only if significantly different scenarios
- Quality over quantity (focused tests better than many redundant tests)
