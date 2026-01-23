# diffviz-review-tui

Clean TUI for code review built on ReviewEngine architecture providing terminal-based interaction with DiffViz's decision-based review capabilities.

**Updated: 2026-01-23** - Added ELM architecture, decision-based navigation tree, automated test harness, theme system, and command pattern for side effects.

## What This Module Does
Infrastructure layer TUI that provides terminal user interface for decision-based code review with interactive navigation through decisions, files, and code chunks.

## Before You Code Here
**Existing Patterns:** ELM architecture with pure view functions, Command pattern for side effects, two-tier event system (UiEvent → BusinessEvent), decision tree-based navigation
**Reusable DTOs/Types:** `DecisionNavigationTree`, `UiState`, `Command`, `UiEvent`, `BusinessEvent`, `StateSnapshot`
**Integration Points:** ReviewEngine for business logic, diffviz-core for RenderableDiff rendering - don't duplicate review state or navigation logic

## Key Abstractions to Reuse

**DecisionNavigationTree**: Tree-based navigation model for decision-first review hierarchy. Models decisions containing files containing chunks with persistent expansion state. Prevents synchronization bugs by maintaining actual tree structure instead of flat indices into dynamically rebuilt lists.

**UiState**: Pure UI navigation and display state completely separated from business logic. Tracks panel focus, scroll positions, input modes, selection state, and leader key states. Never contains business logic or review state.

**Command Enum**: Side effect descriptions for ELM architecture. Represents I/O operations (WriteFile, ShowMessage) that should happen after state updates. Executed by runtime, not update logic.

**Two-Tier Event System**: UiEvent for navigation/display changes, BusinessEvent for operations requiring ReviewEngine coordination. Clean conversion pipeline prevents business logic leaking into UI layer.

**Test Harness Infrastructure**: InputTestHarness for state validation, RenderTestHarness for visual validation, CombinedTestHarness for full integration testing. Enables automated testing without interactive terminal sessions.

## Architectural Constraints

**ELM Architecture Purity**: View functions must be pure (`&UiState`, not `&mut UiState`). Update functions return Commands, never perform I/O directly. State mutations only in update functions, never in views.

**Clean Architecture Separation**: UiState contains zero business logic. All review operations route through ReviewEngine via BusinessEvent conversion. No direct ReviewEngine mutations from UI code.

**Decision-First Navigation**: Primary navigation pattern is DecisionNavigationTree, not flat file lists. All file/chunk selection happens through decision hierarchy.

**Command Pattern for Side Effects**: File writes, notifications, and I/O operations must use Command enum. Update logic describes side effects, runtime executes them.

**Modal Input Handling**: Input routing distinguishes navigation mode from input modes (Instruction, Edit) with completely different key binding behaviors.

**Test Harness Feature Gating**: All test harness code must be feature-gated with `test-harness` to exclude from production builds.

## Core Capabilities

- **Decision-Based Navigation**: Three-level tree navigation (decisions → files → chunks) with persistent expansion state
- **ELM Architecture**: Pure functional state management with immutable updates and Command-based side effects
- **Semantic Diff Visualization**: Rich rendering of RenderableDiff instances with syntax highlighting and semantic anchors
- **Review Workflow Integration**: Direct ReviewEngine integration for approval, commenting, and instruction operations
- **Leader Key System**: Vim-style leader key with submenus and timeout handling
- **Automated Test Harness**: Input sequence testing, render validation, and combined integration testing
- **Theme System**: Centralized Colors, Icons, and Styles for consistent visual design
- **Export Capabilities**: JSON export of instructions with file/single/all scopes

## Directory Map

```
diffviz-review-tui/
├── src/
│   ├── app.rs                         # ReviewTuiApp coordinator (ELM runtime)
│   ├── state.rs                       # Pure UI state (ELM Model)
│   ├── command.rs                     # Command enum + executor (ELM Commands)
│   ├── decision_navigation.rs         # Decision tree navigation model
│   ├── navigation.rs                  # Legacy flat navigation (deprecated)
│   ├── theme.rs                       # Colors, Icons, Styles
│   ├── events/
│   │   ├── input.rs                   # Keyboard → UiEvent mapping
│   │   └── business.rs                # UiEvent → BusinessEvent conversion
│   ├── ui/
│   │   ├── mod.rs                     # Main draw function (ELM View)
│   │   ├── layout.rs                  # Terminal layout
│   │   └── components/                # Pure view components
│   │       ├── decision_tree.rs       # Decision tree panel
│   │       ├── decision_list.rs       # Decision list view
│   │       ├── decision_detail_modal.rs # Decision detail overlay
│   │       ├── diff_view.rs           # Diff display panel
│   │       ├── file_list.rs           # Legacy file navigation
│   │       ├── status_bar.rs          # Status/help display
│   │       ├── help_overlay.rs        # Help modal
│   │       ├── which_key.rs           # Leader key hints
│   │       ├── input_modal.rs         # Text input overlay
│   │       └── renderable_diff_widget.rs # Rich diff renderer
│   ├── test_harness/
│   │   ├── input_test.rs              # Input sequence testing
│   │   ├── render_test.rs             # Visual rendering tests
│   │   ├── combined.rs                # Full integration tests
│   │   ├── snapshot.rs                # State snapshot serialization
│   │   └── input_parser.rs            # Input notation parser
│   ├── formatting/                    # Text formatting utilities
│   ├── diff/                          # Diff processing helpers
│   └── bin/
│       └── renderable_diff_demo.rs    # Standalone widget demo
```

## Development Rules

**ELM Purity Enforcement**: View functions receive `&UiState`, never `&mut UiState`. Update functions return `(State, Command)`, never perform I/O. All side effects go through Command enum.

**Decision Tree is Truth**: Navigation state lives in DecisionNavigationTree, not derived from ReviewEngine queries. Build tree once, navigate by tree paths, not dynamic lookups.

**Zero Business Logic in UI**: UiState tracks navigation, focus, scroll, input modes only. Review state, approval status, instructions live in ReviewEngine exclusively.

**Test Before Breaking**: Run full test suite before architectural changes. Add test harness tests for new features using `--test-input` and `--test-full` modes.

**Theme Consistency**: Use Colors, Icons, and Styles from theme.rs. Never inline color values or style definitions in components.

## Testing Strategy

**Automated Test Harness**: Primary testing through InputTestHarness (state validation), RenderTestHarness (visual validation), and CombinedTestHarness (full integration). Feature-gated with `test-harness` flag.

**Input Sequence Notation**: Tests use string notation for keyboard sequences: `"jjk"` for key presses, `<Space>`, `<Enter>`, `<C-d>` for special keys and modifiers.

**State Snapshot Validation**: StateSnapshot provides JSON serialization of UiState for programmatic assertion in tests. Captures decision tree paths, focus, scroll, and input modes.

**Demo-Driven Development**: Interactive demo binaries (main.rs, renderable_diff_demo.rs) use curated fixtures for manual validation and exploratory testing.

**Widget Isolation Testing**: Standalone renderable_diff_demo allows focused testing of diff visualization without full application overhead.

## Integration Patterns

**ELM Architecture Flow**: Keyboard input → UiEvent (navigation/display) → pure update function → BusinessEvent (review operations) → ReviewEngine methods → new state + Command → command execution → UI re-render.

**Command Execution Cycle**: Update functions return Command describing side effects. Main loop executes commands after state updates. Separates pure logic from I/O.

**ReviewEngine Coordination**: TUI initializes DecisionNavigationTree from ReviewEngine on startup. All review operations (approve, comment, instruct) flow through ReviewEngine methods. UI state updates reactively based on ReviewEngine responses.

**Event Flow Architecture**: Input → UiEvent → handle_ui_event (pure state transitions) → ui_event_to_business_event → handle_business_event (ReviewEngine coordination) → Command → execute_command (side effects).

## Development Tools

**Test Harness CLI**: `cargo run --features test-harness -- --test-input "jjk"` for input testing, `--test-full "jj<Enter>"` for combined testing with visual validation.

**Predictable Test Environment**: Main binary provides consistent manual testing with curated fixtures that never change, enabling reliable regression testing.

**Keyboard Mapping Flexibility**: Event system supports easy key binding modifications through centralized mapping in events/input.rs.

**State Snapshot Debugging**: StateSnapshot JSON output enables debugging state transitions and validating complex navigation sequences.
