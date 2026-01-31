# Behavioral Spec: Comprehensive TUI Testing with Steel Thread Method

## What We're Building

A comprehensive test suite for the diffviz-review-tui that validates all UI capabilities using the test harness infrastructure. The test suite will use the steel thread method to progressively build complexity from simple navigation tests to complex multi-feature integration scenarios.

## Core Behaviors

### Test Execution Pattern
- Run each test scenario manually first using the test harness to verify expected behavior
- If the test passes, codify it as a passing test in the test suite
- If the test fails, codify it as a skipped test (like bug tracking) with `#[ignore]` attribute and descriptive message
- Each test should be self-contained and use the InputTestHarness, RenderTestHarness, or CombinedTestHarness appropriately

### Progressive Complexity Building
Tests will start simple and build in complexity:
1. **Basic navigation** (j/k movement, cursor positioning)
2. **Panel focus** (left/right switching, multi-panel coordination)
3. **Decision tree navigation** (expansion, depth-based display)
4. **Approval workflows** (chunk approval, file approval, cascading)
5. **Input modes** (instruction entry, edit mode)
6. **Leader key system** (space-based submenus, timeouts)
7. **Complex integrations** (multi-feature scenarios combining navigation, approval, and input)

### Test Organization
- Group tests by feature area in separate test modules
- Use consistent naming: `test_<feature>_<scenario>_<expected_result>`
- Include visual rendering validation where appropriate
- Validate both state changes (via StateSnapshot) and visual output (via RenderTestHarness)

### Fixture Validation
- Audit existing MockDiffProvider fixtures to ensure they cover all needed test scenarios
- Identify gaps in test data coverage
- Extend fixtures if needed to test edge cases (empty files, large diffs, nested structures, etc.)

## What Success Looks Like

1. **Audit Phase**: Complete understanding of TUI capabilities documented from codebase analysis
2. **Fixture Phase**: Validated fixtures that cover all test scenarios, with any gaps identified and filled
3. **Test Suite Phase**: Comprehensive test coverage organized by feature area with progressive complexity
   - All passing tests properly codified
   - All failing tests marked with `#[ignore]` and linked to issues
   - Clear test organization following steel thread methodology

## Non-Goals

- Not creating new TUI features, only testing existing ones
- Not refactoring the TUI code itself
- Not changing the test harness infrastructure (it's already built and working)
- Not creating manual testing documentation (automated tests only)

## Key Constraints

- Must use existing test harness (InputTestHarness, RenderTestHarness, CombinedTestHarness)
- Tests must be feature-gated with `#[cfg(feature = "test-harness")]`
- Must follow existing test patterns from decision_approval_tests.rs and keybinding_tests.rs
- Each test must be runnable independently
- Must use MockDiffProvider fixtures for consistent test data
