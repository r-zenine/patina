# Context Document: TUI Comprehensive Testing

## Behavioral Specification

A comprehensive test suite for the diffviz-review-tui that validates all UI capabilities using the test harness infrastructure. The test suite will use the steel thread method to progressively build complexity from simple navigation tests to complex multi-feature integration scenarios.

Each test will be:
1. Run manually first using the test harness
2. Codified as a passing test if it succeeds
3. Codified as a skipped test with `#[ignore]` if it fails (like bug tracking)

Tests will progressively build from simple navigation to complex multi-feature scenarios.

## Architectural Summary

### ELM Architecture
The TUI follows ELM (Elm Language Model) architecture:
- **Model**: UiState - pure navigation and display state
- **View**: ui/components/* - pure rendering functions
- **Update**: events/* - pure state transformations returning (State, Command)
- **Commands**: Side effect descriptions executed separately

### Three-Tier Navigation
1. **Decisions** (depth 0) - Architectural decisions grouping code changes
2. **Files** (depth 1) - Files affected by a decision
3. **Chunks** (depth 2) - Individual diff chunks within files

The TreePath depth determines what displays in panels.

### Two-Tier Event System
- **UiEvent**: Navigation and display changes (j/k, focus switching, scrolling)
- **BusinessEvent**: Operations requiring ReviewEngine (approve, reject, instruction)

Clean separation prevents business logic leaking into UI layer.

### Test Harness Infrastructure
Three test harness types enable comprehensive testing:

1. **InputTestHarness**: State validation without rendering
   - Run input sequences through HeadlessApp
   - Capture StateSnapshot for assertions
   - Fast, focused on state transitions

2. **RenderTestHarness**: Visual validation without input
   - Render UiState to string representation
   - Validate visual output contains expected elements
   - Test rendering logic separately

3. **CombinedTestHarness**: Full integration
   - Combines input processing + rendering
   - Validates both state and visual output
   - End-to-end workflow testing

### Existing Test Coverage

**keybinding_tests.rs** covers:
- Basic navigation (j/k up/down)
- Panel focus switching (left/right arrows)
- Context display toggling
- Quit functionality
- Rendering at different sizes
- Special keys (Space, Enter, Esc, Tab)
- Modifier keys (Ctrl, Shift, Alt)
- State snapshot serialization

**decision_approval_tests.rs** covers:
- Decision approval toggling (Space+a+d at depth 0)
- Approval state persistence
- Multiple decision independence
- Progress calculation
- Edge cases (decisions with no chunks)
- Navigation around approved decisions
- Visual rendering with approval data
- Complete workflows combining navigation and approval

### Fixture System

MockDiffProvider loads from diffviz-review/tests/fixtures/:
- typescript_interface_property.json
- python_sync_to_async.json
- python_class_inheritance.json
- rust_trait_impl.json
- typescript_react_component.json
- typescript_generic_constraint.json
- rust_error_handling.json
- rust_async_conversion.json

Fixtures provide realistic diff data with semantic analysis results. Main.rs creates hardcoded Decision objects that map to these fixtures via line range overlaps.

## Key Technical Decisions

### Feature Gating
All test harness code is feature-gated with `#[cfg(feature = "test-harness")]` to exclude from production builds.

### Test Organization
- Tests live in `diffviz-review-tui/tests/` directory (integration tests)
- Each test file focuses on a feature area
- Tests use consistent naming: `test_<feature>_<scenario>_<expected>`
- Helper functions extracted to module-level for reuse

### Input Notation
Test harness uses string notation for keyboard input:
- Basic: `"jjk"` = three key presses
- Special: `<Space>`, `<Enter>`, `<Esc>`, `<Tab>`
- Arrows: `<Up>`, `<Down>`, `<Left>`, `<Right>`
- Modifiers: `<C-j>`, `<S-q>`, `<A-x>`

### Assertion Strategy
Tests primarily assert on StateSnapshot fields:
- Navigation: `decision_tree_path.0` (decision index)
- Focus: `focused_panel` ("FileList" or "DiffView")
- Modes: `input_mode`, `leader_active`
- State: `should_quit`, `show_all_context`, `cursor_index`

Visual tests assert on rendered string output containing expected UI elements.

## Gaps in Test Coverage

### Uncovered Features
Based on onboarding.md analysis, these features lack dedicated tests:

1. **Leader Key System**
   - Submenu navigation (Space → a/i/t/e)
   - Timeout behavior
   - Nested leader keys
   - All leader key actions (not just approval)

2. **Input Modes**
   - Instruction mode entry/exit
   - Edit mode entry/exit
   - Text input handling
   - Cursor movement in input
   - Input submission/cancellation

3. **Decision Tree Expansion**
   - Tab to toggle expansion
   - Enter to expand current node
   - Persistent expansion state
   - Navigating collapsed nodes

4. **Scrolling**
   - Scroll up/down in diff view
   - Page up/down
   - Inactive panel scrolling (Ctrl+j/k)
   - Auto-scroll on navigation

5. **Selection and Highlighting**
   - Range selection mode (v key)
   - Semantic highlighting toggle
   - Selection anchoring
   - Visual selection feedback

6. **Help System**
   - Help overlay toggle (?)
   - Help content rendering
   - Help navigation

7. **Export Functions**
   - Export file (Space+e+f)
   - Export single instruction (Space+e+s)
   - Export all (Space+e+a)
   - JSON output validation

8. **Edge Cases**
   - Empty decision tree
   - Single decision/file/chunk
   - Very large diffs
   - Long file paths
   - Unicode content
   - Terminal resize behavior

9. **Error Handling**
   - Invalid input sequences
   - Missing fixtures
   - ReviewEngine errors
   - Rendering failures

10. **Multi-Feature Workflows**
    - Navigate + approve + add instruction
    - Expand decision + navigate files + approve chunk
    - Toggle focus + scroll + approve
    - Input mode + cancel + navigate
    - Leader key timeout + resume navigation

### Fixture Gaps
Current fixtures focus on semantic diff analysis. May need additional fixtures for:
- Empty files (test edge case)
- Very large files (test scrolling)
- Multiple files in single diff (test navigation)
- Files with many chunks (test pagination)
- Unicode/special characters (test rendering)

## Implementation Notes

### Steel Thread Approach
Each phase builds end-to-end functionality:

**Phase 1: Core Navigation Steel Thread**
- Simple j/k navigation works end-to-end
- Can verify cursor moves through decisions

**Phase 2: Expand Navigation Steel Thread**
- Add panel focus, scrolling, expansion
- All navigation features work together

**Phase 3: Approval Steel Thread**
- Add chunk/file/decision approval
- Full approval workflow integrated with navigation

**Phase 4: Input Steel Thread**
- Add instruction/edit modes
- Text input fully integrated

**Phase 5: Complete Feature Steel Thread**
- Add leader keys, help, export
- All features work together in complex scenarios

### Test-First Approach
For each steel thread phase:
1. Write test for expected behavior
2. Run test to see current state
3. If passes: keep as passing test
4. If fails: mark with `#[ignore = "Bug #N: description"]`
5. Move to next test in phase

This creates a living documentation of what works and what doesn't.

### Progressive Complexity
Within each phase, start simple:
- Single key press
- Simple sequences
- Basic assertions

Then build complexity:
- Multi-key sequences
- State transitions
- Visual validation
- Integration scenarios

## Success Criteria

1. **Test Coverage**: All TUI features have corresponding tests
2. **Test Organization**: Tests organized by feature with clear naming
3. **Fixture Validation**: Fixtures cover all needed scenarios
4. **Documentation**: Passing/failing tests clearly documented
5. **Maintainability**: Tests easy to understand and modify
6. **Reliability**: Tests consistently pass/fail (no flakiness)
