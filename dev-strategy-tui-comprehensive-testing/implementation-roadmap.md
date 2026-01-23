# Implementation Roadmap: TUI Comprehensive Testing

## Development Strategy: Steel Thread

Build end-to-end test coverage incrementally, starting with the simplest complete workflow and expanding feature coverage progressively. Each phase validates a complete user journey from start to finish.

## Phase 1: Foundation - Core Navigation Steel Thread

**Goal**: Validate basic navigation works end-to-end

**Test File**: `tests/core_navigation_tests.rs`

**Scenarios** (progressive complexity):
1. Single key navigation (j, k)
2. Multi-key navigation sequences (jjj, kkk, jjkk)
3. Boundary navigation (top, bottom, wraparound)
4. Arrow key navigation (Up, Down)
5. Jump navigation (gg for top, G for bottom if implemented)

**Execution Pattern**:
- Create test file with module structure
- For each scenario:
  - Run manually with InputTestHarness
  - Verify StateSnapshot.decision_tree_path changes correctly
  - Codify as passing test or skip with #[ignore]
  - Document expected vs actual behavior

**Validation**:
- decision_tree_path.0 moves correctly
- Cursor doesn't go out of bounds
- Navigation is responsive (state changes on each input)

**Deliverable**: `tests/core_navigation_tests.rs` with ~10-15 tests

---

## Phase 2: Panel Management Steel Thread

**Goal**: Validate focus switching and multi-panel coordination

**Test File**: `tests/panel_management_tests.rs`

**Scenarios**:
1. Panel focus switching (Left/Right arrows)
2. Focus switching with navigation (switch + navigate + switch back)
3. Scroll in diff view panel
4. Scroll in decision tree panel
5. Page up/down scrolling
6. Inactive panel scrolling (Ctrl+j/k)
7. Scroll state persistence across focus switches

**Execution Pattern**:
- Test each panel operation in isolation
- Test combinations (navigate + switch focus + scroll)
- Validate both panels maintain independent scroll state
- Test with CombinedTestHarness to verify visual rendering

**Validation**:
- focused_panel switches correctly
- scroll_offset updates appropriately
- Visual output shows correct panel highlighted
- Scroll state independent per panel

**Deliverable**: `tests/panel_management_tests.rs` with ~10-15 tests

---

## Phase 3: Decision Tree Expansion Steel Thread

**Goal**: Validate tree expansion/collapse and depth-based navigation

**Test File**: `tests/decision_tree_tests.rs`

**Scenarios**:
1. Tab toggles expansion of current decision
2. Enter expands current node
3. Navigate through expanded tree (depth 0→1→2)
4. Navigate through collapsed tree (skip files/chunks)
5. Expansion state persists during navigation
6. Visual indicators show expansion state (▶/▼)
7. Navigate to different depths and verify display routing

**Execution Pattern**:
- Start with single decision
- Test expansion/collapse behavior
- Verify TreePath.depth() changes correctly
- Use RenderTestHarness to validate expansion icons
- Test persistent expansion across navigation

**Validation**:
- decision_tree_path.depth() reflects actual depth
- Expansion state in decision_tree matches visual
- Navigation respects collapsed state
- Visual output shows correct expansion icons

**Deliverable**: `tests/decision_tree_tests.rs` with ~15-20 tests

---

## Phase 4: Approval Workflow Steel Thread

**Goal**: Validate all approval operations (already partially covered)

**Test File**: Enhance existing `tests/decision_approval_tests.rs`

**Additional Scenarios**:
1. Approve chunk at depth 2 (Space+a+a)
2. Approve file at depth 1 or 2 (Space+a+f)
3. Approve decision at depth 0 (Space+a+d)
4. Visual approval indicators in decision tree
5. Visual approval indicators in diff view
6. Status bar approval progress updates
7. Approval cascading (all chunks → decision auto-approved)
8. Reverse cascade (decision approved → all chunks approved)
9. Mixed approval states (some chunks approved)
10. Unapprove workflows (toggle twice)

**Execution Pattern**:
- Extend existing test file with new scenarios
- Focus on visual validation (icons, progress counters)
- Test cascading behavior thoroughly
- Validate approval state queries

**Validation**:
- review_engine.state().is_approved() returns correct state
- Visual indicators match approval state
- Cascading logic works bidirectionally
- Progress counters accurate

**Deliverable**: Enhanced `tests/decision_approval_tests.rs` with ~30+ total tests

---

## Phase 5: Leader Key System Steel Thread

**Goal**: Validate leader key menus and timeout behavior

**Test File**: `tests/leader_key_tests.rs`

**Scenarios**:
1. Space activates leader (leader_active becomes true)
2. Leader key shows which-key overlay
3. Leader submenu navigation (Space → a/i/t/e)
4. Actions submenu (Space+a → a/f/d)
5. Instructions submenu (Space+i → ...)
6. Toggles submenu (Space+t → ...)
7. Export submenu (Space+e → f/s/a)
8. Leader timeout behavior
9. Esc cancels leader
10. Invalid leader key deactivates leader
11. Leader key visual hints update

**Execution Pattern**:
- Test leader activation/deactivation
- Test each submenu independently
- Test timeout mechanism (may need special handling)
- Validate which-key visual output
- Test full leader sequences

**Validation**:
- leader_active flag correct
- leader_submenu tracks current submenu
- Visual which-key hints show correct options
- Timeout resets leader state
- Actions execute correctly from leader

**Deliverable**: `tests/leader_key_tests.rs` with ~20-25 tests

---

## Phase 6: Input Modes Steel Thread

**Goal**: Validate instruction and edit mode text input

**Test File**: `tests/input_mode_tests.rs`

**Scenarios**:
1. Enter instruction mode (Space+i+...)
2. Enter edit mode (Space+i+e or similar)
3. Type text in input buffer
4. Cursor movement (left/right arrows, Home/End)
5. Word-wise cursor movement (Ctrl+left/right)
6. Character deletion (Backspace, Delete)
7. Submit input (Enter)
8. Cancel input (Esc)
9. Input mode visual modal
10. Exit input mode returns to navigation
11. Input buffer cleared after submit/cancel

**Execution Pattern**:
- Test mode transitions
- Test text editing operations
- Validate input_buffer and input_cursor state
- Test visual modal rendering
- Test mode exit behavior

**Validation**:
- input_mode transitions correctly
- input_buffer contains typed text
- input_cursor position accurate
- Visual modal shows input buffer
- Mode exit cleans up state

**Deliverable**: `tests/input_mode_tests.rs` with ~20-25 tests

---

## Phase 7: Help and Context Display Steel Thread

**Goal**: Validate help overlay and context toggles

**Test File**: `tests/help_and_context_tests.rs`

**Scenarios**:
1. ? toggles help overlay
2. Help overlay shows keybindings
3. Help overlay can be dismissed
4. Context toggle (if applicable)
5. show_all_context flag behavior
6. Visual rendering with/without context
7. Help overlay doesn't interfere with navigation

**Execution Pattern**:
- Test help toggle
- Validate visual help content
- Test help dismissal
- Test context display toggle

**Validation**:
- show_help flag correct
- Visual output contains help content
- Help can be toggled on/off
- show_all_context affects rendering

**Deliverable**: `tests/help_and_context_tests.rs` with ~10-15 tests

---

## Phase 8: Export Functions Steel Thread

**Goal**: Validate export command execution

**Test File**: `tests/export_tests.rs`

**Scenarios**:
1. Export file (Space+e+f)
2. Export single instruction (Space+e+s)
3. Export all (Space+e+a)
4. Export command generation (Command enum)
5. Export with different selection contexts
6. Export with no instructions (edge case)

**Execution Pattern**:
- Test export sequences through leader key
- Validate Command return values
- Test different export scopes
- Test edge cases (no data to export)

**Validation**:
- Correct Command variant returned
- Export command includes right scope
- Commands execute without error
- Edge cases handled gracefully

**Deliverable**: `tests/export_tests.rs` with ~10-15 tests

---

## Phase 9: Edge Cases and Error Handling Steel Thread

**Goal**: Validate boundary conditions and error scenarios

**Test File**: `tests/edge_cases_tests.rs`

**Scenarios**:
1. Empty decision tree (no decisions)
2. Single decision, single file, single chunk
3. Very deep navigation (many j presses)
4. Long file paths in rendering
5. Unicode content in diffs
6. Special characters in input
7. Invalid input sequences (unknown keys)
8. Rapid input sequences
9. All decisions collapsed
10. All decisions expanded

**Execution Pattern**:
- Create custom test engines with edge case data
- Test boundary conditions
- Test error scenarios don't crash
- Validate graceful degradation

**Validation**:
- No panics or errors
- State remains consistent
- Visual output handles edge cases
- Navigation bounded correctly

**Deliverable**: `tests/edge_cases_tests.rs` with ~15-20 tests

---

## Phase 10: Complex Integration Workflows Steel Thread

**Goal**: Validate multi-feature scenarios end-to-end

**Test File**: `tests/integration_workflows_tests.rs`

**Scenarios**:
1. Navigate → expand → navigate files → approve chunk
2. Navigate → approve decision → verify cascade → navigate
3. Expand tree → scroll → switch focus → scroll → approve
4. Navigate → enter instruction mode → type → submit → verify
5. Leader key → action → verify state → continue navigation
6. Help overlay → navigate through help → dismiss → resume
7. Approval workflow → export → verify command
8. Complex leader sequence → timeout → retry
9. Multi-panel workflow (switch focus repeatedly with actions)
10. Full review workflow (navigate all, approve all, export)

**Execution Pattern**:
- Combine multiple features in single test
- Use CombinedTestHarness for full validation
- Test realistic user workflows
- Validate state consistency throughout

**Validation**:
- All state transitions correct
- Visual output consistent
- No state corruption
- Features work together seamlessly

**Deliverable**: `tests/integration_workflows_tests.rs` with ~15-20 tests

---

## Phase 11: Fixture Validation and Enhancement

**Goal**: Ensure fixtures support all test scenarios

**Tasks**:
1. Audit existing fixtures
   - List all fixture files
   - Document what each fixture provides
   - Identify coverage gaps

2. Create fixture test matrix
   - Map test scenarios to required fixture data
   - Identify missing fixture types
   - Document fixture requirements

3. Enhance fixtures if needed
   - Create empty file fixtures
   - Create large diff fixtures
   - Create multi-file fixtures
   - Create special character fixtures

4. Document fixture usage
   - Update fixture README
   - Document fixture-to-test mapping
   - Provide fixture creation guidelines

**Execution Pattern**:
- Survey existing fixtures
- Map to test requirements
- Create missing fixtures
- Validate new fixtures

**Validation**:
- All test scenarios have fixture support
- Fixtures well-documented
- Fixtures easy to extend

**Deliverable**:
- `diffviz-review/tests/fixtures/README.md`
- New fixture files as needed
- Fixture usage documentation

---

## Phase 12: Test Suite Organization and Documentation

**Goal**: Polish test suite and create comprehensive documentation

**Tasks**:
1. Review all test files
   - Consistent naming
   - Consistent structure
   - Consistent assertions
   - Proper module organization

2. Extract common test utilities
   - Shared helper functions
   - Common assertion patterns
   - Fixture loading helpers
   - Test engine builders

3. Document test patterns
   - How to write new tests
   - How to use each harness type
   - Input notation reference
   - Assertion strategies

4. Create test coverage report
   - Features tested
   - Features skipped (with #[ignore])
   - Coverage gaps
   - Future test ideas

5. CI/CD integration notes
   - How to run test suite
   - Feature flag requirements
   - Performance considerations
   - Maintenance guidelines

**Deliverable**:
- `diffviz-review-tui/tests/README.md`
- Test utilities module
- Coverage report
- CI/CD integration guide

---

## Execution Order and Dependencies

### Sequential Phases
Phases must be completed in order due to dependencies:
1. Phase 1 (navigation) → Foundation for all other tests
2. Phase 2 (panels) → Builds on navigation
3. Phase 3 (tree) → Builds on navigation + panels
4. Phase 4 (approval) → Builds on tree navigation
5. Phase 5 (leader) → Independent but uses navigation
6. Phase 6 (input) → Independent but uses navigation
7. Phase 7 (help) → Independent but uses navigation
8. Phase 8 (export) → Builds on leader keys
9. Phase 9 (edge cases) → Requires all features tested
10. Phase 10 (integration) → Requires all features tested
11. Phase 11 (fixtures) → Can run in parallel with earlier phases
12. Phase 12 (docs) → Must be last

### Parallel Opportunities
- Phases 5, 6, 7 can run in parallel (different features)
- Phase 11 can start early and run alongside test development
- Within each phase, individual tests can be developed in parallel

## Validation Checkpoints

After each phase, verify:
1. All tests compile and run
2. Passing tests consistently pass
3. Skipped tests documented with #[ignore] and issue reference
4. Test coverage documented in phase deliverable
5. No regressions in previous phases

## Final Deliverables

1. **11 test files** covering all TUI features
2. **Estimated 180-240 total tests** across all files
3. **Enhanced fixtures** supporting all test scenarios
4. **Comprehensive documentation** of test patterns and coverage
5. **CI/CD integration** guidelines
6. **Living documentation** of working vs. broken features (via passing vs. skipped tests)

## Success Metrics

- **Coverage**: Every TUI feature has at least one test
- **Clarity**: Test names clearly describe scenario
- **Reliability**: Tests consistently pass/fail (no flakiness)
- **Maintainability**: New tests easy to add following patterns
- **Documentation**: Clear guidance for contributors
- **Transparency**: Broken features clearly marked and tracked
