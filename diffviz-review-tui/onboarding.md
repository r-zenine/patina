# diffviz-review-tui

Clean TUI for code review built on ReviewEngine architecture providing terminal-based interaction with DiffViz's code review capabilities.

## Architecture Role

Infrastructure layer TUI crate that provides the terminal user interface for DiffViz code review. Depends on diffviz-review for business logic orchestration, diffviz-core for semantic analysis rendering, and diffviz-git for repository operations. Acts as a presentation layer that transforms ReviewEngine state into interactive terminal UI components while maintaining clean separation between UI events and business operations.

## Core Capabilities

- **Interactive Code Review Interface**: Two-panel layout with file list navigation and diff viewing
- **Semantic Diff Visualization**: Rich rendering of RenderableDiff instances with syntax highlighting and semantic anchors
- **Review Workflow Integration**: Direct integration with ReviewEngine for approval, commenting, and instruction operations
- **Keyboard-Driven Navigation**: Vim-style key bindings with context-sensitive behavior across different UI modes
- **Real-time Review State Management**: Live display of approval status, comments, and review progress
- **Input Mode Management**: Modal interface supporting comment entry, instruction creation, and edit operations
- **Context-Aware Display**: Toggle between full context and focused view of relevant changes

## Key Abstractions

**ReviewTuiApp**: Main application coordinator that bridges ReviewEngine business logic with terminal UI presentation. Manages terminal setup, event routing, and application lifecycle.

**UiState**: Pure UI navigation and display state container tracking panel focus, scroll positions, input modes, and selection state. Completely separated from business logic state.

**Event System**: Two-tier event handling with UiEvent for navigation/display changes and BusinessEvent for operations requiring ReviewEngine coordination.

**RenderableDiffWidget**: Rich diff visualization widget that renders RenderableDiff instances with semantic highlighting, change indicators, and optional inline old content display.

**NavigationState**: Hierarchical file/diff navigation system supporting collapsible file lists and ReviewableDiff selection with keyboard-driven traversal.

## Development Rules

**Clean Architecture Separation**: UI state (UiState) must never contain business logic. All review operations must route through ReviewEngine via BusinessEvent conversion.

**Modal Input Handling**: Input routing must distinguish between navigation mode and input modes (comment, instruction, edit) with different key binding behaviors.

**Widget Composition**: UI components must compose cleanly through the main draw function without direct inter-component dependencies.

**EdTUI Integration Constraints**: EdTUI editor routing requires specific key event filtering to avoid conflicts with application navigation controls.

**Terminal Lifecycle Management**: Application must properly clean up terminal state on exit including raw mode disable and screen restoration.

## Code Organization

```
diffviz-review-tui/
├── src/
│   ├── app.rs                    # Main ReviewTuiApp coordinator
│   ├── state.rs                  # Pure UI state management
│   ├── navigation.rs             # File/diff navigation logic
│   ├── main.rs                   # Test-only binary with fixtures
│   ├── events/
│   │   ├── mod.rs               # Event system exports
│   │   ├── input.rs             # Keyboard input to UiEvent mapping
│   │   └── business.rs          # UiEvent to BusinessEvent conversion
│   ├── ui/
│   │   ├── mod.rs               # Main draw function
│   │   ├── layout.rs            # Terminal layout management
│   │   └── components/          # Reusable UI widgets
│   │       ├── diff_view.rs     # Diff display panel
│   │       ├── file_list.rs     # File navigation panel
│   │       ├── status_bar.rs    # Status/help display
│   │       ├── input_modal.rs   # Text input overlay
│   │       └── renderable_diff_widget.rs  # Rich diff renderer
│   ├── formatting/              # Text formatting utilities
│   ├── diff/                    # Diff processing helpers
│   └── bin/
       └── renderable_diff_demo.rs  # Standalone widget demo
```

## Testing Strategy

**Demo-Driven Development**: Primary testing through interactive demo binaries that use curated fixtures for predictable manual validation. Main binary (main.rs) provides full TUI experience with MockDiffProvider fixtures.

**Widget Isolation Testing**: Standalone demo for RenderableDiffWidget (renderable_diff_demo.rs) allows focused testing of diff visualization without full application context.

**Fixture-Based Consistency**: All test binaries use identical fixtures from diffviz-review crate ensuring consistent test data across manual validation sessions.

**Manual UI Validation**: No automated UI tests - relies on systematic manual testing with known good fixtures to validate visual correctness and interaction behavior.

## Integration Patterns

**ReviewEngine Coordination**: TUI acts as thin presentation layer over ReviewEngine business logic. All review operations (approve, comment, instruct) flow through ReviewEngine methods with proper error handling.

**Event Flow Architecture**: Keyboard input → UiEvent (navigation/display) → BusinessEvent (review operations) → ReviewEngine methods → UI state updates via pure functions.

**State Synchronization**: UI state updates reactively based on ReviewEngine state changes while maintaining independent navigation and display state in UiState.

## Development Tools

**Predictable Test Environment**: Main binary provides consistent manual testing environment using curated fixtures that never change, enabling reliable regression testing.

**Widget Development Tools**: Standalone RenderableDiffWidget demo with mock data enables rapid iteration on diff visualization without full application overhead.

**Keyboard Mapping Flexibility**: Event system architecture supports easy modification of key bindings through centralized input mapping in events/input.rs.