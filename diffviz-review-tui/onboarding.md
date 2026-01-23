# diffviz-review-tui

Clean TUI for code review built on ReviewEngine architecture providing terminal-based interaction with DiffViz's decision-based review capabilities.

**Updated: 2026-01-23** - Removed decision detail modal, replaced with inline decision details panel. Decision display now routes by tree depth (0=decision details, 1=file placeholder, 2=diff content).

## What This Module Does
Infrastructure layer TUI that provides terminal user interface for decision-based code review with interactive navigation through decisions, files, and code chunks.

## Before You Code Here
**Existing Patterns:** ELM architecture with pure view functions, Command pattern for side effects, two-tier event system (UiEvent → BusinessEvent), decision tree-based navigation with depth-routed display
**Reusable DTOs/Types:** `DecisionNavigationTree`, `UiState`, `Command`, `UiEvent`, `BusinessEvent`, `StateSnapshot`, `TreePath` (with depth calculation)
**Integration Points:** ReviewEngine for business logic (approval state, instructions), diffviz-core for RenderableDiff rendering - don't duplicate review state or navigation logic

## Key Abstractions to Reuse

**DecisionNavigationTree**: Tree-based navigation model for decision-first review hierarchy. Models decisions containing files containing chunks with persistent expansion state. TreePath depth determines display routing (0=decision, 1=file, 2=chunk). Prevents synchronization bugs by maintaining actual tree structure instead of flat indices into dynamically rebuilt lists.

**TreePath with Depth**: Navigation path that encodes position in tree (decision_index, file_index, chunk_index). Depth method (0/1/2) drives display routing logic - which panel to show for current selection. Critical for decision-first navigation flow.

**UiState**: Pure UI navigation and display state completely separated from business logic. Tracks panel focus, scroll positions, input modes, selection state, and leader key states. Never contains business logic or review state. All approval status queries go through ReviewEngine.

**Command Enum**: Side effect descriptions for ELM architecture. Represents I/O operations (WriteFile, ShowMessage) that should happen after state updates. Executed by runtime, not update logic.

**Two-Tier Event System**: UiEvent for navigation/display changes, BusinessEvent for operations requiring ReviewEngine coordination. Clean conversion pipeline prevents business logic leaking into UI layer. Approval operations always convert through BusinessEvent.

**Test Harness Infrastructure**: InputTestHarness for state validation, RenderTestHarness for visual validation, CombinedTestHarness for full integration testing. Enables automated testing without interactive terminal sessions.

## Architectural Constraints

**ELM Architecture Purity**: View functions must be pure (`&UiState`, not `&mut UiState`). Update functions return Commands, never perform I/O directly. State mutations only in update functions, never in views.

**Clean Architecture Separation**: UiState contains zero business logic. All review operations route through ReviewEngine via BusinessEvent conversion. No direct ReviewEngine mutations from UI code. Approval state lives in ReviewEngine exclusively.

**Decision-First Navigation**: Primary navigation pattern is DecisionNavigationTree, not flat file lists. All file/chunk selection happens through decision hierarchy. Tree depth determines what displays in diff view panel.

**Depth-Routed Display**: diff_view.rs routes rendering based on TreePath.depth(): depth 0 shows decision details inline, depth 1 shows file placeholder, depth 2 shows chunk diff. This is the canonical pattern for extending display capabilities.

**Command Pattern for Side Effects**: File writes, notifications, and I/O operations must use Command enum. Update logic describes side effects, runtime executes them.

**Modal Input Handling**: Input routing distinguishes navigation mode from input modes (Instruction, Edit) with completely different key binding behaviors.

**Test Harness Feature Gating**: All test harness code must be feature-gated with `test-harness` to exclude from production builds.

## Core Capabilities

- **Decision-Based Navigation**: Three-level tree navigation (decisions → files → chunks) with persistent expansion state and depth-routed display
- **ELM Architecture**: Pure functional state management with immutable updates and Command-based side effects
- **Semantic Diff Visualization**: Rich rendering of RenderableDiff instances with syntax highlighting and semantic anchors
- **Review Workflow Integration**: Direct ReviewEngine integration for approval, commenting, and instruction operations
- **Approval Management**: Toggle approve/unapprove for individual chunks (Space+a+a) or entire files (Space+a+f). All approval state managed by ReviewEngine, queried via `review_engine.state().is_approved()`
- **Decision Display**: Inline decision details panel shows title, summary, code impacts with confidence levels, and navigation hints when decision node selected (depth 0)
- **Leader Key System**: Vim-style leader key (Space) with submenus and timeout handling. Actions submenu for approvals, Instructions submenu for adding content, Toggles for view options, Export for JSON output
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
│   │       ├── decision_details_panel.rs # Inline decision details (NEW)
│   │       ├── diff_view.rs           # Diff display panel with depth routing
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

**Zero Business Logic in UI**: UiState tracks navigation, focus, scroll, input modes only. Review state, approval status, instructions live in ReviewEngine exclusively. Always query approval via `review_engine.state().is_approved()`.

**Depth-Routed Display Pattern**: When extending diff view display, follow the depth routing pattern: check `ui_state.decision_tree.selected_path.depth()` and route to appropriate render function. Don't add conditionals in existing branches.

**Approval Operations Through BusinessEvent**: All approval operations (toggle, approve file) must convert UiEvent → BusinessEvent → ReviewEngine method call. Never call ReviewEngine directly from UI event handlers.

**Test Before Breaking**: Run full test suite before architectural changes. Add test harness tests for new features using `--test-input` and `--test-full` modes.

**Theme Consistency**: Use Colors, Icons, and Styles from theme.rs. Never inline color values or style definitions in components.

## How Decisions are Displayed and Navigated

**Tree-Based Navigation**: DecisionNavigationTree builds a three-level hierarchy from ReviewEngine on startup. Users navigate with j/k (vim-style) through flattened view that respects expansion state. Tab toggles expansion, Enter expands current node.

**Depth-Based Display Routing**: diff_view.rs checks `TreePath.depth()` to determine what to render:
- **Depth 0 (Decision Selected)**: Renders decision_details_panel showing title, summary, code impacts with confidence/reasoning, and impact counts
- **Depth 1 (File Selected)**: Shows placeholder (future: file summary)
- **Depth 2 (Chunk Selected)**: Renders actual diff content with RenderableDiffWidget

**Visual Indicators**: Decision tree shows expansion icons (▶/▼), selection indicator (►), decision numbers with titles, and code impact counts in brackets. Selected items highlighted with inverted colors (DarkGray background).

**Decision Details Panel**: When decision selected, shows formatted view with decision number, title (bold), summary, decision log reference, impact summary (file/impact counts), and detailed code impacts list with file paths, line ranges, change types (Addition/Modification/Deletion), confidence levels (High/Medium/Low with color coding), and reasoning text.

## How Approvals Work

**Approval Operations**:
- **Toggle Approve Chunk**: `Space + a + a` when chunk selected (depth 2). BusinessEvent::ToggleApprove checks current state via `review_engine.state().is_approved()` then calls either `review_engine.approve()` or `review_engine.reject()`
- **Approve All in File**: `Space + a + f` when file or chunk selected. BusinessEvent::ApproveFile calls `review_engine.approve_all_in_file()` to approve all chunks in current file

**Approval State Queries**: All UI components query approval state through ReviewEngine: `review_engine.state().is_approved(&reviewable_id)`. Never cache approval state in UiState.

**Visual Feedback**:
- **Diff View Title**: Shows Icons::APPROVED (✓) or Icons::PENDING (○) based on approval state
- **Decision Tree**: No approval indicators in tree view (decision-first, not approval-first)
- **Status Bar**: Calculates approval progress by filtering reviewable IDs through `is_approved()` for current file and total counts
- **File List** (deprecated): Shows approval icons per diff item

**Approval Flow**: UiEvent::ToggleApprove (from leader key) → ui_event_to_business_event() extracts current reviewable_id from UiState → BusinessEvent::ToggleApprove → handle_business_event() queries current state → calls ReviewEngine approve/reject methods → UI re-renders with updated state

**Author Attribution**: All approve operations pass `review_engine.author()` as author parameter. Rejection passes None for comment parameter.

## Relationship Between Decisions and Approvals

**Orthogonal Concerns**: Decisions organize code changes by architectural intent (what was decided). Approvals track review completion status (what was reviewed). These are separate dimensions - you navigate by decision, approve by chunk.

**Navigation First, Action Second**: Decision tree provides navigation structure to find relevant chunks. Once chunk selected, approval actions operate at chunk or file level. Decisions don't have approval state themselves.

**No Decision-Level Approval**: You cannot approve an entire decision. You approve individual chunks or all chunks in a file. This prevents accidentally approving unrelated code that happens to share a decision.

**Review State Separation**: DecisionNavigationTree maintains navigation/expansion state. ReviewEngine maintains approval state. UiState coordinates between them but owns neither. This separation enables decision tree rebuilds without losing approval state.

## Existing Infrastructure for Approving

**BusinessEvent Enum**: Defines ToggleApprove (single chunk) and ApproveFile (all chunks in file) events with appropriate parameters (reviewable_id or file_path).

**Event Conversion Pipeline**: `ui_event_to_business_event()` in business.rs extracts context from UiState (current reviewable_id, current file_path) and converts UiEvents to BusinessEvents with proper parameters.

**ReviewEngine Methods**:
- `approve(reviewable_id, author, comment)`: Marks chunk approved
- `reject(reviewable_id, comment)`: Marks chunk not approved
- `approve_all_in_file(file_path, author, comment)`: Bulk approve all chunks in file
- `state().is_approved(&reviewable_id)`: Query approval state

**Leader Key Binding**: Actions submenu (Space + a) provides:
- `a`: Approve/unapprove current chunk (ToggleApprove)
- `f`: Approve all chunks in current file (ApproveFile)

**Visual Components**:
- Icons::APPROVED / Icons::PENDING in theme.rs for status display
- Status bar calculates and displays approval progress percentages
- Diff view title shows approval icon next to file path and line range

**Command Pattern**: Approval operations return Command::None (no file I/O needed). State change happens in ReviewEngine, UI reflects updated state on next render.

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

**Approval Query Pattern**: Any component needing approval state receives `&ReviewEngine` parameter and calls `review_engine.state().is_approved(&id)`. No caching, no stale state.

## Development Tools

**Test Harness CLI**: `cargo run --features test-harness -- --test-input "jjk"` for input testing, `--test-full "jj<Enter>"` for combined testing with visual validation.

**Predictable Test Environment**: Main binary provides consistent manual testing with curated fixtures that never change, enabling reliable regression testing.

**Keyboard Mapping Flexibility**: Event system supports easy key binding modifications through centralized mapping in events/input.rs.

**State Snapshot Debugging**: StateSnapshot JSON output enables debugging state transitions and validating complex navigation sequences.
