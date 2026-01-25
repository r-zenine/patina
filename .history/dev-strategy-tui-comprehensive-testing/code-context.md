# Code Context: TUI Comprehensive Testing

## Relevant Files for Test Development

### Test Infrastructure
- **diffviz-review-tui/src/test_harness/mod.rs:1-21** - Test harness module exports
- **diffviz-review-tui/src/test_harness/input_test.rs:1-86** - InputTestHarness for state validation
- **diffviz-review-tui/src/test_harness/render_test.rs** - RenderTestHarness for visual validation
- **diffviz-review-tui/src/test_harness/combined.rs** - CombinedTestHarness for full integration
- **diffviz-review-tui/src/test_harness/input_parser.rs** - Input sequence parsing (supports `"jjk"`, `<Space>`, `<Enter>`, etc.)
- **diffviz-review-tui/src/test_harness/snapshot.rs** - StateSnapshot for test assertions

### Existing Test Examples
- **diffviz-review-tui/tests/keybinding_tests.rs:1-352** - Navigation, focus, rendering tests
- **diffviz-review-tui/tests/decision_approval_tests.rs:1-446** - Decision approval workflow tests

### TUI Application Core
- **diffviz-review-tui/src/app.rs:1-99** - ReviewTuiApp main coordinator, HeadlessApp for testing
- **diffviz-review-tui/src/state.rs:1-100** - UiState structure (focus, input modes, navigation state)
- **diffviz-review-tui/src/main.rs:1-150** - Test entry point with hardcoded decisions and fixtures

### Event System
- **diffviz-review-tui/src/events/input.rs:1-100** - UiEvent enum and keyboard mapping
- **diffviz-review-tui/src/events/business.rs** - UiEvent → BusinessEvent conversion
- **diffviz-review-tui/src/command.rs** - Command enum for side effects

### Navigation
- **diffviz-review-tui/src/decision_navigation.rs** - DecisionNavigationTree, TreePath
- **diffviz-review-tui/src/navigation.rs** - Legacy flat navigation (deprecated)

### UI Components
- **diffviz-review-tui/src/ui/mod.rs** - Main draw function
- **diffviz-review-tui/src/ui/components/decision_tree.rs** - Decision tree panel rendering
- **diffviz-review-tui/src/ui/components/decision_details_panel.rs** - Decision details inline panel
- **diffviz-review-tui/src/ui/components/diff_view.rs** - Diff display with depth routing
- **diffviz-review-tui/src/ui/components/file_list.rs** - Legacy file list (deprecated)
- **diffviz-review-tui/src/ui/components/status_bar.rs** - Status bar with approval progress
- **diffviz-review-tui/src/ui/components/help_overlay.rs** - Help modal
- **diffviz-review-tui/src/ui/components/which_key.rs** - Leader key hints
- **diffviz-review-tui/src/ui/components/input_modal.rs** - Text input overlay
- **diffviz-review-tui/src/ui/components/renderable_diff_widget.rs** - Rich diff renderer

### Test Fixtures
- **diffviz-review/tests/fixtures/** - MockDiffProvider JSON fixtures
  - typescript_interface_property.json
  - python_sync_to_async.json
  - python_class_inheritance.json
  - rust_trait_impl.json
  - typescript_react_component.json
  - typescript_generic_constraint.json
  - rust_error_handling.json
  - rust_async_conversion.json

### Review Engine Integration
- **diffviz-review/src/engines/review_engine.rs** - ReviewEngine methods (approve, reject, state queries)
- **diffviz-review/src/providers/mock_provider.rs** - MockDiffProvider for testing

## Key Patterns from Existing Tests

### Creating Test Engine
```rust
fn create_test_engine() -> ReviewEngine {
    let mock_provider = MockDiffProvider::from_review_fixtures()
        .expect("Failed to load test fixtures");
    let review_engine_builder = ReviewEngineBuilder::new(
        Box::new(mock_provider),
        "test-user".to_string()
    );
    let diff_query = DiffQuery::new(GitRef::Head, GitRef::Unstaged);
    let mut review_engine = review_engine_builder
        .build(diff_query)
        .expect("Failed to build ReviewEngine");

    // Add decisions
    review_engine.set_decisions_with_index(decisions);
    review_engine
}
```

### Using InputTestHarness
```rust
let mut harness = InputTestHarness::new(engine);
let snapshot = harness.run_sequence_final_state("jjk").unwrap();
assert_eq!(snapshot.decision_tree_path.0, 1);
```

### Using RenderTestHarness
```rust
let harness = RenderTestHarness::new();
let visual = harness.render(&mut ui_state, &engine).expect("Render failed");
assert!(visual.contains("Decisions"));
```

### Using CombinedTestHarness
```rust
let mut harness = CombinedTestHarness::new(engine);
let results = harness.run_sequence_with_renders("jj<Space>ad").expect("Combined test");
for result in &results {
    assert!(!result.visual.is_empty());
    assert!(!result.state.focused_panel.is_empty());
}
```

## Input Sequence Notation

- Basic keys: `"jjk"` (j, j, k)
- Special keys: `<Space>`, `<Enter>`, `<Esc>`, `<Tab>`
- Arrow keys: `<Up>`, `<Down>`, `<Left>`, `<Right>`
- Modifiers: `<C-j>` (Ctrl+j), `<S-q>` (Shift+q), `<A-x>` (Alt+x)

## StateSnapshot Fields

- `decision_tree_path` - TreePath (decision_index, file_index, chunk_index)
- `focused_panel` - "FileList" or "DiffView"
- `input_mode` - "Navigation", "Instruction", or "Edit"
- `cursor_index` - Current cursor position
- `scroll_offset` - Scroll position
- `should_quit` - Quit flag
- `leader_active` - Leader key state
- `show_all_context` - Context display toggle
