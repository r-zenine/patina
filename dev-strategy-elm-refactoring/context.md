# Context: ELM Architecture Refactoring for diffviz-review-tui

## What We're Building

A series of refactorings to align the `diffviz-review-tui` crate with pure ELM (Elm Architecture) patterns. The codebase already follows ELM principles in spirit but has several violations that break the pure functional nature of the pattern. We need to fix critical and moderate violations while maintaining a working application throughout the refactoring.

## Current Architecture

**Location**: `diffviz-review-tui/` crate within DiffViz workspace

**Key Components**:
- **ReviewTuiApp** (`app.rs`): Main coordinator bridging ReviewEngine and terminal UI
- **UiState** (`state.rs`): Pure UI state tracking navigation, focus, input modes, and selections
- **Event System** (`events/`): Two-tier system with `UiEvent` and `BusinessEvent`
- **UI Components** (`ui/components/`): View functions rendering state to terminal

**Current Event Flow**:
```
Keyboard Input → UiEvent → BusinessEvent → ReviewEngine mutations → UI state updates
```

**What Works Well**:
- Clean separation between UI state and business logic
- Two-tier event system (UiEvent for display, BusinessEvent for operations)
- Pure view components (conceptually, though signatures allow mutation)
- No business logic in view layer

## The Problems (Violations)

### Critical Violations

**V1: View Functions Accept Mutable State**
- Location: `src/ui/mod.rs:11`, all component render functions
- Issue: Functions accept `&mut UiState` instead of `&UiState`
- Impact: Breaks immutability contract, allows accidental mutations

**V2: Side Effects in Update Logic**
- Location: `src/app.rs:448-472` (ExportInstructions handler)
- Issue: Direct file I/O and stderr printing in `handle_business_event`
- Impact: Not testable, violates pure function requirement

**V6: Direct ReviewEngine Mutations**
- Location: `src/app.rs:425-446` (all business event handlers)
- Issue: Event handlers directly call ReviewEngine methods
- Impact: Side effects embedded in update logic (pragmatic compromise vs pure ELM)

### Moderate Violations

**V3: Time-Based Side Effects in Event Loop**
- Location: `src/app.rs:98-100`
- Issue: Direct mutation based on elapsed time check
- Impact: Not following message-based subscription pattern

**V4: Direct Field Access to Nested State**
- Location: `src/app.rs:189`, `app.rs:292-296`
- Issue: Event handlers directly access/mutate `decision_tree` fields
- Impact: Breaks encapsulation, unpredictable state updates

**V5: Business Logic in UI State Layer**
- Location: `src/decision_navigation.rs:146-192`
- Issue: `build_from_review_engine` contains complex tree-building logic
- Impact: UI layer depends on business entity structure

## Architectural Goals

**Pure ELM Pattern**:
1. **Model**: Immutable state (UiState, ReviewEngine state)
2. **View**: Pure functions `State → UI` (no mutations, no side effects)
3. **Update**: Pure functions `(State, Event) → (State, Command)` (no side effects)
4. **Commands**: Descriptions of side effects executed by runtime

**Key Changes Required**:
- Introduce `Command` enum to describe side effects
- Change view signatures to use `&UiState` instead of `&mut UiState`
- Refactor update logic to return Commands instead of executing side effects
- Encapsulate all state updates through dedicated UiState methods
- Model time-based behavior as messages

## Constraints

**Must Maintain**:
- Working application at every step (no breaking phases)
- Existing test infrastructure (`test-harness` feature, keybinding tests)
- Clean Architecture separation (UI layer vs business layer)
- Two-tier event system (UiEvent, BusinessEvent)
- Manual testing workflow with fixtures

**Dependencies**:
- `ratatui` for terminal UI rendering
- `crossterm` for terminal control and event polling
- `diffviz-review` for ReviewEngine business logic
- Test-only feature flag: `test-harness` for HeadlessApp

**Testing Strategy**:
- Demo-driven development with curated fixtures
- Existing integration tests in `tests/keybinding_tests.rs`
- Manual validation through main binary (`src/main.rs`)

## Files Involved

**Core Files** (will be modified):
- `src/app.rs` - Main application loop, event handling, command execution
- `src/state.rs` - UI state with encapsulated update methods
- `src/ui/mod.rs` - Main draw function signature
- `src/ui/components/*.rs` - All view component signatures
- `src/events/input.rs` - Add LeaderTimeout event
- `src/decision_navigation.rs` - Encapsulate mutations

**New Files** (to be created):
- `src/command.rs` - Command enum and execution logic

**Cross-Crate** (may need changes):
- `diffviz-review` crate - Potentially add tree building helper (V5)

## Success Criteria

**After Refactoring**:
1. All view functions use `&UiState` (immutable reference)
2. Update logic returns Commands instead of executing side effects
3. Command execution isolated to main loop
4. Time-based behavior modeled as messages
5. All state updates go through UiState methods (no direct field access)
6. Tree building logic moved to business layer (or properly encapsulated)
7. Existing tests pass
8. Application behavior unchanged from user perspective
9. Code is more testable (pure functions, injectable commands)

## Non-Goals

**Out of Scope**:
- Changing ReviewEngine API or internal architecture
- Adding new features or UI improvements
- Optimizing performance
- Adding new automated tests (beyond maintaining existing ones)
- Full ELM runtime with subscriptions and task scheduling (pragmatic Rust approach)
