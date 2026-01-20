# Code Context for Decision-Based Review

## Relevant Classes and Functions

### diffviz-review (Decision modeling will go here)

- **ReviewableDiffId** (`diffviz-review/src/entities/reviewable_diff_id.rs`) - Universal identifier (DiffQuery + file_path + LineRange) for semantic diff units. Key for mapping decisions to code.
- **ReviewState** (`diffviz-review/src/state/mod.rs`) - Centralized state container with BTreeMap of ReviewableDiffs. Will need decision_index extension.
- **ReviewEngine** (`diffviz-review/src/engines/review_engine.rs`) - Core review operations with caching. Entry point for decision-based navigation.
- **ReviewEngineBuilder** (`diffviz-review/src/review_engine_builder.rs`) - Factory for creating ReviewEngine from git queries. May need to accept decision mapping.

### diffviz-review-tui (TUI changes)

- **ReviewTuiApp** (`diffviz-review-tui/src/app.rs`) - Main application coordinator bridging ReviewEngine with TUI.
- **UiState** (`diffviz-review-tui/src/state.rs`) - Pure UI state for navigation, scroll, input modes. Will need decision view mode.
- **NavigationState** (`diffviz-review-tui/src/navigation.rs`) - File/diff navigation with collapsible lists. Model for DecisionNavigationState.
- **main.rs** (`diffviz-review-tui/src/main.rs`) - Mock binary using MockDiffProvider for testing. Entry point for Phase 1 validation.

### diffviz-review entities (Reference patterns)

- **Comment** (`diffviz-review/src/entities/comment.rs`) - Pattern for entity indexed by ReviewableDiffId
- **Approval** (`diffviz-review/src/entities/approval.rs`) - Pattern for review metadata entity
- **Instruction** (`diffviz-review/src/entities/instruction.rs`) - Pattern for entity with content and author

### dev-contribute skill (Phase 3)

- **SKILL.md** (`agent-skills/skills/dev-contribute/SKILL.md`) - Skill definition, lists mandatory artifacts
- **reference.md** (`agent-skills/skills/dev-contribute/reference.md`) - Step-by-step contribution process
- **context-handoff-template.md** (`agent-skills/skills/dev-contribute/templates/context-handoff-template.md`) - Template with file references

## Key Files to Reference

### Data Model Design
- `diffviz-review/src/entities/mod.rs` - Entity module organization pattern
- `diffviz-review/src/entities/comment.rs` - Entity with ReviewableDiffId indexing pattern
- `diffviz-review/src/state/mod.rs` - ReviewState structure and collections

### TUI Navigation
- `diffviz-review-tui/src/navigation.rs` - NavigationState implementation
- `diffviz-review-tui/src/state.rs` - UiState and view mode management
- `diffviz-review-tui/src/events/input.rs` - Key binding patterns

### Test Infrastructure
- `diffviz-review/src/providers/mock_provider.rs` - MockDiffProvider for testing
- `diffviz-review-tui/src/main.rs` - Test binary entry point
