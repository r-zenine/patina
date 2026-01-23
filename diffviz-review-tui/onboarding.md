# diffviz-review-tui

Clean TUI for code review built on ReviewEngine architecture providing terminal-based interaction with DiffViz's decision-based review capabilities.

**Updated: 2026-01-23** - Added decision-level approval capability. Decisions can now be approved/unapproved at depth 0 using Space+a+d, with visual progress indicators (approved/total chunks) displayed in decision tree and decision details panel. Icons::NOT_APPROVED added for unapproved state visual feedback.

## What This Module Does
Infrastructure layer TUI that provides terminal user interface for decision-based code review with interactive navigation through decisions, files, and code chunks, including decision-level approval management.

## Before You Code Here
**Existing Patterns:** ELM architecture with pure view functions, Command pattern for side effects, two-tier event system (UiEvent → BusinessEvent), decision tree-based navigation with depth-routed display, context-aware approval operations (depth-based)
**Reusable DTOs/Types:** `DecisionNavigationTree`, `UiState`, `Command`, `UiEvent`, `BusinessEvent`, `StateSnapshot`, `TreePath` (with depth calculation)
**Integration Points:** ReviewEngine for business logic (approval state at chunk and decision level, instructions), diffviz-core for RenderableDiff rendering - don't duplicate review state or navigation logic

## Key Abstractions to Reuse

**DecisionNavigationTree**: Tree-based navigation model for decision-first review hierarchy. Models decisions containing files containing chunks with persistent expansion state. TreePath depth determines display routing (0=decision, 1=file, 2=chunk). Prevents synchronization bugs by maintaining actual tree structure instead of flat indices into dynamically rebuilt lists. Provides `selected_decision_number()` to extract decision context at depth 0.

**TreePath with Depth**: Navigation path that encodes position in tree (decision_index, file_index, chunk_index). Depth method (0/1/2) drives display routing logic - which panel to show for current selection. Critical for decision-first navigation flow and context-aware approval operations.

**UiState**: Pure UI navigation and display state completely separated from business logic. Tracks panel focus, scroll positions, input modes, selection state, and leader key states. Never contains business logic or review state. All approval status queries go through ReviewEngine. Provides `current_decision_number()` to extract decision context when at depth 0.

**Command Enum**: Side effect descriptions for ELM architecture. Represents I/O operations (WriteFile, ShowMessage) that should happen after state updates. Executed by runtime, not update logic.

**Two-Tier Event System**: UiEvent for navigation/display changes, BusinessEvent for operations requiring ReviewEngine coordination. Clean conversion pipeline prevents business logic leaking into UI layer. Approval operations always convert through BusinessEvent with context-aware routing (depth 0 → ToggleApproveDecision, depth 2 → ToggleApprove).

**Test Harness Infrastructure**: InputTestHarness for state validation, RenderTestHarness for visual validation, CombinedTestHarness for full integration testing. Enables automated testing without interactive terminal sessions. Supports decision approval workflow testing.

## Architectural Constraints

**ELM Architecture Purity**: View functions must be pure (`&UiState`, not `&mut UiState`). Update functions return Commands, never perform I/O directly. State mutations only in update functions, never in views.

**Clean Architecture Separation**: UiState contains zero business logic. All review operations route through ReviewEngine via BusinessEvent conversion. No direct ReviewEngine mutations from UI code. Approval state lives in ReviewEngine exclusively at both chunk and decision levels.

**Decision-First Navigation**: Primary navigation pattern is DecisionNavigationTree, not flat file lists. All file/chunk selection happens through decision hierarchy. Tree depth determines what displays in diff view panel and what approval operations are available.

**Depth-Routed Display**: diff_view.rs routes rendering based on TreePath.depth(): depth 0 shows decision details inline, depth 1 shows file placeholder, depth 2 shows chunk diff. This is the canonical pattern for extending display capabilities.

**Context-Aware Approval Routing**: Approval operations route based on TreePath depth. Depth 0 triggers decision-level approval (ToggleApproveDecision), depth 2 triggers chunk-level approval (ToggleApprove). Same keybinding (Space+a+a or Space+a+d) performs different operations based on navigation context.

**Command Pattern for Side Effects**: File writes, notifications, and I/O operations must use Command enum. Update logic describes side effects, runtime executes them.

**Modal Input Handling**: Input routing distinguishes navigation mode from input modes (Instruction, Edit) with completely different key binding behaviors.

**Test Harness Feature Gating**: All test harness code must be feature-gated with `test-harness` to exclude from production builds.

## Core Capabilities

- **Decision-Based Navigation**: Three-level tree navigation (decisions → files → chunks) with persistent expansion state and depth-routed display
- **ELM Architecture**: Pure functional state management with immutable updates and Command-based side effects
- **Semantic Diff Visualization**: Rich rendering of RenderableDiff instances with syntax highlighting and semantic anchors
- **Review Workflow Integration**: Direct ReviewEngine integration for approval, commenting, and instruction operations
- **Multi-Level Approval Management**:
  - Toggle approve/unapprove individual chunks (Space+a+a at depth 2)
  - Approve entire files (Space+a+f when file/chunk selected)
  - Approve entire decisions (Space+a+d at depth 0) - cascades to all chunks in decision
  - All approval state managed by ReviewEngine with queries via `review_engine.state().is_approved()` (chunks) and `review_engine.is_decision_approved()` (decisions)
- **Decision Display**: Inline decision details panel shows title, summary, code impacts with confidence levels, approval status icon, and approval progress (approved/total chunks) when decision node selected (depth 0)
- **Leader Key System**: Vim-style leader key (Space) with submenus and timeout handling. Actions submenu for approvals (context-aware based on depth), Instructions submenu for adding content, Toggles for view options, Export for JSON output
- **Automated Test Harness**: Input sequence testing, render validation, and combined integration testing with decision approval workflow coverage
- **Theme System**: Centralized Colors, Icons (APPROVED, NOT_APPROVED, PENDING), and Styles for consistent visual design
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
│   │       ├── decision_details_panel.rs # Inline decision details
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
├── tests/
│   └── decision_approval_tests.rs     # Decision approval integration tests
```

## Development Rules

**ELM Purity Enforcement**: View functions receive `&UiState`, never `&mut UiState`. Update functions return `(State, Command)`, never perform I/O. All side effects go through Command enum.

**Decision Tree is Truth**: Navigation state lives in DecisionNavigationTree, not derived from ReviewEngine queries. Build tree once, navigate by tree paths, not dynamic lookups.

**Zero Business Logic in UI**: UiState tracks navigation, focus, scroll, input modes only. Review state, approval status, instructions live in ReviewEngine exclusively. Always query approval via `review_engine.state().is_approved()` for chunks and `review_engine.is_decision_approved()` for decisions.

**Depth-Routed Display Pattern**: When extending diff view display, follow the depth routing pattern: check `ui_state.decision_tree.selected_path.depth()` and route to appropriate render function. Don't add conditionals in existing branches.

**Context-Aware Approval Pattern**: When adding approval operations, use TreePath depth to determine operation scope. Extract context via `ui_state.current_decision_number()` at depth 0, `ui_state.current_reviewable_id()` at depth 2. Route through appropriate BusinessEvent variant.

**Approval Operations Through BusinessEvent**: All approval operations (toggle chunk, approve file, toggle decision) must convert UiEvent → BusinessEvent → ReviewEngine method call. Never call ReviewEngine directly from UI event handlers.

**Test Before Breaking**: Run full test suite before architectural changes. Add test harness tests for new features using `--test-input` and `--test-full` modes. Decision approval features require integration test coverage.

**Theme Consistency**: Use Colors, Icons, and Styles from theme.rs. Never inline color values or style definitions in components.

## How Decisions are Displayed and Navigated

**Tree-Based Navigation**: DecisionNavigationTree builds a three-level hierarchy from ReviewEngine on startup. Users navigate with j/k (vim-style) through flattened view that respects expansion state. Tab toggles expansion, Enter expands current node.

**Depth-Based Display Routing**: diff_view.rs checks `TreePath.depth()` to determine what to render:
- **Depth 0 (Decision Selected)**: Renders decision_details_panel showing title, summary, code impacts with confidence/reasoning, approval status icon, approval progress (approved/total chunks), and impact counts
- **Depth 1 (File Selected)**: Shows placeholder (future: file summary)
- **Depth 2 (Chunk Selected)**: Renders actual diff content with RenderableDiffWidget

**Visual Indicators in Decision Tree**:
- Expansion icons (▶/▼) for collapsed/expanded states
- Selection indicator (►) for currently selected item
- Approval status icons (✓ for approved, ○ for not approved) at decision level
- Decision numbers with titles
- Approval progress counters (approved/total) in parentheses next to each decision
- Code impact counts in brackets showing total impacts
- Selected items highlighted with inverted colors (DarkGray background)

**Decision Details Panel**: When decision selected (depth 0), shows formatted view with:
- Approval status icon (✓/○) at the start
- Decision number and title (bold)
- Approval progress counter (approved/total chunks)
- Summary text
- Decision log reference line number
- Impact summary (file count / impact count)
- Detailed code impacts list with:
  - File paths
  - Line ranges
  - Change types (Addition/Modification/Deletion)
  - Confidence levels (High/Medium/Low with color coding)
  - Reasoning text

## How Approvals Work

**Approval Operations**:
- **Toggle Approve Chunk**: `Space + a + a` when chunk selected (depth 2). BusinessEvent::ToggleApprove checks current state via `review_engine.state().is_approved()` then calls either `review_engine.approve()` or `review_engine.reject()`
- **Toggle Approve Decision**: `Space + a + d` when decision selected (depth 0). BusinessEvent::ToggleApproveDecision checks current state via `review_engine.is_decision_approved()` then calls either `review_engine.approve_decision()` or `review_engine.reject_decision()`. Cascades to all chunks within the decision.
- **Approve All in File**: `Space + a + f` when file or chunk selected. BusinessEvent::ApproveFile calls `review_engine.approve_all_in_file()` to approve all chunks in current file

**Approval State Queries**: All UI components query approval state through ReviewEngine:
- Chunks: `review_engine.state().is_approved(&reviewable_id)`
- Decisions: `review_engine.is_decision_approved(decision_number)`
- Progress: `review_engine.state().decision_approval_progress(decision_number)` returns (approved_count, total_count)
- Never cache approval state in UiState

**Visual Feedback**:
- **Decision Tree**: Shows approval icons (✓/○) and progress counters (approved/total) for each decision
- **Decision Details Panel**: Shows approval icon and progress at top of panel when decision selected
- **Diff View Title**: Shows Icons::APPROVED (✓) or Icons::PENDING (○) based on chunk approval state
- **Status Bar**: Calculates approval progress by filtering reviewable IDs through `is_approved()` for current file and total counts
- **File List** (deprecated): Shows approval icons per diff item

**Approval Flow**:
- UiEvent::ToggleApprove (from leader key)
- → ui_event_to_business_event() checks depth via `ui_state.decision_tree.selected_path.depth()`
  - If depth 0: extracts decision_number via `ui_state.current_decision_number()` → BusinessEvent::ToggleApproveDecision
  - If depth 2: extracts reviewable_id via `ui_state.current_reviewable_id()` → BusinessEvent::ToggleApprove
- → handle_business_event() queries current state
- → calls appropriate ReviewEngine method (approve_decision/reject_decision or approve/reject)
- → UI re-renders with updated state

**Author Attribution**: All approve operations pass `review_engine.author()` as author parameter. Rejection passes None for comment parameter.

## Relationship Between Decisions and Approvals

**Hierarchical Approval Model**: Decisions can now be approved at the decision level, which cascades to all chunks within that decision. Three approval scopes exist: chunk-level, file-level, and decision-level. All are managed through the same approval infrastructure.

**Navigation First, Action Second**: Decision tree provides navigation structure to find relevant decisions/chunks. Approval scope is determined by current depth - decision approval at depth 0, chunk approval at depth 2.

**Decision-Level Approval**: Approving a decision (Space+a+d at depth 0) approves all chunks associated with that decision's code impacts. This provides a fast-path for approving architectural decisions that span multiple chunks.

**Review State Separation**: DecisionNavigationTree maintains navigation/expansion state. ReviewEngine maintains approval state at both chunk and decision levels. UiState coordinates between them but owns neither. This separation enables decision tree rebuilds without losing approval state.

**Progress Tracking**: Approval progress (approved/total chunks) is calculated dynamically by ReviewEngine and displayed in both decision tree and decision details panel, providing real-time feedback on review completion status.

## Existing Infrastructure for Approving

**BusinessEvent Enum**: Defines approval events with appropriate parameters:
- `ToggleApprove { reviewable_id }`: Single chunk approval
- `ToggleApproveDecision { decision_number }`: Decision-level approval (cascades to all chunks)
- `ApproveFile { file_path }`: All chunks in file

**Event Conversion Pipeline**: `ui_event_to_business_event()` in business.rs extracts context from UiState using depth-aware routing:
- Checks `ui_state.decision_tree.selected_path.depth()`
- At depth 0: calls `ui_state.current_decision_number()` → BusinessEvent::ToggleApproveDecision
- At depth 2: calls `ui_state.current_reviewable_id()` → BusinessEvent::ToggleApprove
- Prevents business logic leaking into UI layer

**ReviewEngine Methods**:
- `approve(reviewable_id, author, comment)`: Marks chunk approved
- `reject(reviewable_id, comment)`: Marks chunk not approved
- `approve_decision(decision_number, author)`: Approves entire decision (cascades)
- `reject_decision(decision_number)`: Rejects entire decision
- `approve_all_in_file(file_path, author, comment)`: Bulk approve all chunks in file
- `state().is_approved(&reviewable_id)`: Query chunk approval state
- `is_decision_approved(decision_number)`: Query decision approval state
- `state().decision_approval_progress(decision_number)`: Get (approved, total) tuple

**Leader Key Binding**: Actions submenu (Space + a) provides context-aware options:
- `a`: Approve/unapprove current chunk (at depth 2) or decision (at depth 0)
- `d`: Approve/unapprove current decision (visible only at depth 0 in which-key hints)
- `f`: Approve all chunks in current file

**Visual Components**:
- Icons::APPROVED (✓), Icons::NOT_APPROVED (○), Icons::PENDING (○) in theme.rs for status display
- Decision tree renders approval icons and progress counters for each decision
- Decision details panel shows approval status and progress at top
- Status bar calculates and displays approval progress percentages
- Diff view title shows approval icon next to file path and line range

**Command Pattern**: Approval operations return Command::None (no file I/O needed). State change happens in ReviewEngine, UI reflects updated state on next render.

## Testing Strategy

**Automated Test Harness**: Primary testing through InputTestHarness (state validation), RenderTestHarness (visual validation), and CombinedTestHarness (full integration). Feature-gated with `test-harness` flag.

**Decision Approval Test Coverage**: Comprehensive test suite in `tests/decision_approval_tests.rs` covering:
- Basic decision approval toggle workflows
- Progress counter calculation and display
- Multiple decision independence
- Visual rendering with approval data
- Complete integration workflows
- Edge cases (decisions with no chunks, navigation around approved decisions)
- State persistence across operations

**Input Sequence Notation**: Tests use string notation for keyboard sequences: `"jjk"` for key presses, `<Space>`, `<Enter>`, `<C-d>` for special keys and modifiers.

**State Snapshot Validation**: StateSnapshot provides JSON serialization of UiState for programmatic assertion in tests. Captures decision tree paths, focus, scroll, and input modes.

**Demo-Driven Development**: Interactive demo binaries (main.rs, renderable_diff_demo.rs) use curated fixtures for manual validation and exploratory testing.

**Widget Isolation Testing**: Standalone renderable_diff_demo allows focused testing of diff visualization without full application overhead.

## Integration Patterns

**ELM Architecture Flow**: Keyboard input → UiEvent (navigation/display) → pure update function → BusinessEvent (review operations) → ReviewEngine methods → new state + Command → command execution → UI re-render.

**Command Execution Cycle**: Update functions return Command describing side effects. Main loop executes commands after state updates. Separates pure logic from I/O.

**ReviewEngine Coordination**: TUI initializes DecisionNavigationTree from ReviewEngine on startup. All review operations (approve chunk/decision/file, comment, instruct) flow through ReviewEngine methods. UI state updates reactively based on ReviewEngine responses.

**Event Flow Architecture**: Input → UiEvent → handle_ui_event (pure state transitions) → ui_event_to_business_event (depth-aware routing) → handle_business_event (ReviewEngine coordination) → Command → execute_command (side effects).

**Approval Query Pattern**: Any component needing approval state receives `&ReviewEngine` parameter and calls appropriate query methods (`state().is_approved(&id)` for chunks, `is_decision_approved(number)` for decisions, `state().decision_approval_progress(number)` for progress). No caching, no stale state.

**Context-Aware UI Updates**: Which-key hints dynamically show/hide decision approval option based on current depth. Decision tree and decision details panel query approval progress on each render to display current state.

## Development Tools

**Test Harness CLI**:
- `cargo run --features test-harness -- --test-input "jjk"` for input testing
- `--test-full "jj<Enter>"` for combined testing with visual validation
- `cargo test --test decision_approval_tests` for decision approval integration tests

**Predictable Test Environment**: Main binary provides consistent manual testing with curated fixtures that never change, enabling reliable regression testing.

**Keyboard Mapping Flexibility**: Event system supports easy key binding modifications through centralized mapping in events/input.rs. Context-aware routing happens in business event conversion layer.

**State Snapshot Debugging**: StateSnapshot JSON output enables debugging state transitions and validating complex navigation sequences including decision approval workflows.
