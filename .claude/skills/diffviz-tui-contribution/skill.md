---
name: diffviz-tui-contribution
description: Mandatory skill to use whenever you interract with diffviz-review-tui, wether to add code to it, test it or plan a change
---

# Contributing to diffviz-review-tui

Comprehensive guide for making architecture-compliant contributions to the TUI crate with proper testing.

## What This Skill Does

Ensures contributions to diffviz-review-tui follow:
- Elm architecture patterns (pure views, command system, event flow)
- Proper state encapsulation through methods
- Test-driven development with the test harness
- Verification of behavior before and after changes

## When to Use This Skill

✅ **Use when:**
- Implementing new TUI features or keybindings
- Modifying existing UI components or state management
- Fixing bugs in the TUI layer
- Reviewing TUI pull requests
- Testing UI workflows and navigation
- Debugging TUI behavior or visual rendering

❌ **Don't use when:**
- Working on other crates (diffviz-core, diffviz-review, etc.)
- Making changes to business logic in ReviewEngine
- Writing pure backend code without UI impact

## Quick Reference

| Component | Rule | Location |
|-----------|------|----------|
| View | `&UiState` (immutable) | `src/ui/**/*.rs` |
| Update | Mutates `UiState`, returns `Command` | `src/app.rs` |
| Command | I/O side effects only | `src/command.rs` |
| Events | `KeyEvent` → `UiEvent` → `BusinessEvent` | `src/events/*.rs` |
| Tests | Test harness with fixture data | `tests/keybinding_tests.rs` |

## Architecture Compliance Rules

### V1: Pure View Functions

**Rule**: All view/render functions must accept `&UiState`, never `&mut UiState`.

```rust
// CORRECT
pub fn render(f: &mut Frame, area: Rect, ui_state: &UiState, engine: &ReviewEngine)

// VIOLATION
pub fn render(f: &mut Frame, area: Rect, ui_state: &mut UiState, engine: &ReviewEngine)
```

**Verification**: `rg "ui_state: &mut UiState" diffviz-review-tui/src/ui/`

### V2: Command System for Side Effects

**Rule**: Update handlers return `Command` for I/O operations, not execute inline.

```rust
// CORRECT
fn handle_business_event(&mut self, event: BusinessEvent) -> Result<Command> {
    match event {
        BusinessEvent::ExportInstructions { scope } => {
            let json = self.review_engine.export_instructions_json(scope)?;
            Ok(Command::WriteFile { path: filename, content: json })
        }
    }
}

// VIOLATION - inline I/O
std::fs::write(&filename, json)?;  // Direct side effect in handler
eprintln!("Exported to {}", filename);  // Direct console output
```

**Available Commands**: `WriteFile`, `ShowMessage`, `Batch`, `None`

### V3: Time as Events

**Rule**: Time-based behavior flows through event system in `handle_events()`.

```rust
// CORRECT
if self.ui_state.leader_active && self.ui_state.is_leader_timed_out() {
    self.ui_state.deactivate_leader();
}
```

### V4: State Encapsulation

**Rule**: Access nested state through `UiState` methods.

```rust
// CORRECT
self.ui_state.navigate_to_first_in_tree();
self.ui_state.close_modal_if_open();

// VIOLATION - direct field access
self.ui_state.decision_tree.selected_path = first.path.clone();
```

### V5: Event Flow Architecture

**Flow**: `KeyEvent` → `UiEvent` → `BusinessEvent` → `Command`

**Adding features**:
1. Add `UiEvent` variant in `src/events/input.rs`
2. Map key to event in `handle_key_event()`
3. Handle in `handle_ui_event()` for UI changes
4. Add `BusinessEvent` if ReviewEngine interaction needed
5. Return `Command` for side effects in `handle_business_event()`

## Testing Your Changes

### Three Testing Modes

**Mode 1: Input Testing** - State validation
```bash
cargo run --bin review-tui --features test-harness -- --test-input "jjk<Space>"
```
Returns JSON state snapshot after each key.

**Mode 2: Integration Testing** - Full workflow
```bash
cargo run --bin review-tui --features test-harness -- --test-full "sequence"
```
Returns state + visual output at each step.

**Mode 3: Unit Tests** - Regression prevention
```bash
cargo test --package diffviz-review-tui --features test-harness
```

### Testing Workflow

```
[ ] Step 1: Identify feature/keybinding to test
[ ] Step 2: Look up keybinding in src/events/input.rs
[ ] Step 3: Run CLI test (--test-input or --test-full)
[ ] Step 4: Verify expected state changes in JSON output
[ ] Step 5: Write integration test in tests/keybinding_tests.rs
[ ] Step 6: Run full test suite to confirm
```

### Input Notation

Vim-style compact notation:
- Single keys: `j`, `k`, `a`
- Special keys: `<Space>`, `<Enter>`, `<Esc>`, `<Up>`, `<Down>`
- Modifiers: `<C-j>` (Ctrl), `<S-?>` (Shift), `<A-x>` (Alt)
- Delays: `<Wait:100>`

Example: `jjk<Space>a<Enter>` = down, down, up, toggle, approve, enter

### Debugging Patterns

**Test single keybinding**:
```bash
cargo run --bin review-tui --features test-harness -- --test-input "j" | jq .focused_panel
```

**Capture before/after behavior**:
```bash
cargo run --bin review-tui --features test-harness -- --test-full "sequence" > before.txt
# Make changes
cargo run --bin review-tui --features test-harness -- --test-full "sequence" > after.txt
diff before.txt after.txt
```

**Verify test engine setup**:
```bash
cargo run --bin review-tui --features test-harness -- --test-input "" | jq .
```

## Code Review Rejection Criteria

Reject contributions that:

1. Add `&mut UiState` to view function signatures
2. Perform file I/O or console output in update handlers without Command
3. Access `decision_tree` fields directly outside `state.rs`
4. Add time-based logic outside event handling flow
5. Skip event system for user interactions
6. Don't include tests for new functionality

## Verification Commands

```bash
# Architecture compliance checks
rg "ui_state: &mut UiState" diffviz-review-tui/src/ui/
rg "std::fs::|eprintln!|println!" diffviz-review-tui/src/app.rs
rg "\.decision_tree\.(selected_path|show_decision_modal)" diffviz-review-tui/src/ --glob '!state.rs'

# Run tests
cargo test --package diffviz-review-tui --features test-harness

# Compile check
cargo check --package diffviz-review-tui

# Clippy
cargo clippy --package diffviz-review-tui
```

## Accepted Pragmatic Compromises

**ReviewEngine mutations**: Direct `self.review_engine.approve()` calls in handlers are acceptable. These are synchronous domain operations, not I/O side effects.

```rust
// ACCEPTABLE
BusinessEvent::ToggleApprove { reviewable_id } => {
    if self.review_engine.state().is_approved(&reviewable_id) {
        self.review_engine.reject(reviewable_id, None)?;
    }
    Ok(Command::None)
}
```

## Reference Documentation

For detailed information, see:
- **Architecture principles**: [references/elm-architecture.md](references/elm-architecture.md)
- **Test harness guide**: [references/testing-guide.md](references/testing-guide.md)
- **Onboarding**: `diffviz-review-tui/onboarding.md`
- **Architecture audit**: `diffviz-review-tui/ARCHITECTURE_AUDIT.md`
- **Test harness**: `diffviz-review-tui/TEST_HARNESS.md`
