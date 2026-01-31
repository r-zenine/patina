# TUI Comprehensive Testing - Dev Strategy

This directory contains the complete implementation plan for building comprehensive test coverage for the diffviz-review-tui using the steel thread method.

## Planning Artifacts

### 1. behavioral-spec.md
**What we're building**: High-level description of the comprehensive test suite goal, core behaviors, and success criteria.

### 2. code-context.md
**Relevant code**: File paths and line references for test infrastructure, existing tests, TUI components, and fixtures.

### 3. context.md
**Full context**: Combines behavioral spec with architectural summary, explains ELM architecture, test harness infrastructure, existing coverage, gaps, and technical decisions.

### 4. implementation-roadmap.md
**How to build it**: 12-phase steel thread implementation plan with progressive complexity:
- Phase 1-3: Navigation foundation
- Phase 4: Approval workflows
- Phase 5-8: Advanced features (leader keys, input modes, help, export)
- Phase 9-10: Edge cases and integration
- Phase 11-12: Fixtures and documentation

### 5. decision-log.md
**Key decisions**: Documents chosen approaches for test organization, harness selection, fixture strategy, naming conventions, and coverage goals.

## Quick Start

### Understanding the Plan
1. Read `behavioral-spec.md` for the big picture
2. Read `context.md` for architectural understanding
3. Read `implementation-roadmap.md` for execution plan
4. Refer to `code-context.md` for file references
5. Check `decision-log.md` for rationale behind choices

### Executing the Plan
1. Start with Phase 1 (Core Navigation Steel Thread)
2. For each test scenario:
   - Run manually using test harness CLI (`--test-input` or `--test-full`)
   - Document expected behavior
   - Codify as passing test or skip with `#[ignore]`
3. Complete phase deliverable (test file)
4. Move to next phase
5. Validate no regressions

## Key Principles

### Steel Thread Method
Each phase builds complete end-to-end functionality before moving to the next feature. This provides:
- Working tests from day one
- Clear progress milestones
- Early feedback on fixture gaps
- Progressive complexity

### Progressive Complexity
Within each phase, start simple:
1. Single operations
2. Simple sequences
3. Basic assertions
4. Visual validation
5. Complex scenarios

### Test-First Execution
1. Document expected behavior
2. Run test manually
3. Verify behavior
4. Codify test
5. Mark passing or skip

### Living Documentation
- Passing tests document working features
- Skipped tests (with `#[ignore]`) document known issues
- Test suite serves as feature catalog
- Clear visibility into TUI capabilities

## Deliverables

### Test Files (11 total)
1. `core_navigation_tests.rs` - Basic navigation
2. `panel_management_tests.rs` - Focus and scrolling
3. `decision_tree_tests.rs` - Tree expansion and depth
4. `decision_approval_tests.rs` - Approval workflows (enhanced)
5. `leader_key_tests.rs` - Leader key system
6. `input_mode_tests.rs` - Text input modes
7. `help_and_context_tests.rs` - Help and context display
8. `export_tests.rs` - Export functions
9. `edge_cases_tests.rs` - Boundary conditions
10. `integration_workflows_tests.rs` - Multi-feature scenarios
11. Test utilities module - Shared helpers

### Documentation
- Fixture README and usage guide
- Test pattern documentation
- Coverage report
- CI/CD integration guide

### Estimated Test Count
180-240 tests total across all phases

## Dependencies

### Required
- `diffviz-review-tui` crate with `test-harness` feature
- `diffviz-review` crate with MockDiffProvider
- Existing fixture files in `diffviz-review/tests/fixtures/`

### Tools
- Test harness CLI flags: `--test-input`, `--test-render`, `--test-full`
- InputTestHarness for state validation
- RenderTestHarness for visual validation
- CombinedTestHarness for integration

## Success Metrics

- ✅ Every TUI feature has at least one test
- ✅ Test names clearly describe scenarios
- ✅ Tests consistently pass or are properly skipped
- ✅ New tests easy to add following patterns
- ✅ Clear documentation for contributors
- ✅ Known issues transparently tracked via skipped tests

## Next Steps

1. Review this plan with stakeholders
2. Begin Phase 1: Core Navigation Steel Thread
3. Execute test-first workflow for each scenario
4. Complete each phase deliverable
5. Validate no regressions between phases
6. Document coverage and gaps
7. Iterate on fixture enhancements as needed
