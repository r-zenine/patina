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
| Keybindings | One registry row per binding (feeds everything) | `src/events/bindings.rs` |
| Tests | Test harness with fixture data | `tests/keybinding_tests.rs` |
| Discovery | `--describe` / `--agent-repl` on the binary | provided by `tui-harness` |

## Architecture Compliance Rules

### V1: Pure View Functions

**Rule**: All view/render functions must accept `&UiState`, never `&mut UiState`.

```rust
// CORRECT
pub fn render(f: &mut Frame, area: Rect, ui_state: &UiState, engine: &ReviewEngine)

// VIOLATION
pub fn render(f: &mut Frame, area: Rect, ui_state: &mut UiState, engine: &ReviewEngine)
```

**Verification**: `rg "ui_state: &mut UiState" apps/diffviz/review-tui/src/ui/`

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
self.ui_state.navigate_up();
self.ui_state.drill_in();

// VIOLATION - direct field access
self.ui_state.drill_nav = DrillNavState::Browse { cursor: 0 };
```

### V5: Event Flow Architecture

**Flow**: `KeyEvent` → registry lookup (`src/events/bindings.rs`) → `UiEvent` → `BusinessEvent` → `Command`

**Adding features**:
1. Add `UiEvent` variant in `src/events/input.rs`
2. Add a `Binding` row in `src/events/bindings.rs` (keys, event, notation,
   description) — dispatch, which-key, help overlay, `--describe`, and
   affordances all pick it up from this one row
3. Handle in `handle_ui_event()` for UI changes
4. Add `BusinessEvent` if ReviewEngine interaction needed
5. Return `Command` for side effects in `handle_business_event()`

**Never** add key matching outside the registry — `handle_key_event` is a
lookup plus two documented fallbacks, nothing else.

## Testing Your Changes

### Discover First

The binary describes itself — start here instead of reading source:
```bash
cargo run --bin review-tui -- --describe
```
Returns a JSON manifest: all keybindings per mode, the input-sequence
notation grammar, and the JSON Schema of the state snapshot.

### Four Testing Modes

**Mode 1: Input Testing** - State validation
```bash
cargo run --bin review-tui -- --test-input "jjk<Space>"
```
Returns the final JSON state snapshot.

**Mode 2: Integration Testing** - Full workflow
```bash
cargo run --bin review-tui -- --test-full "sequence"
```
Returns state + visual + affordances (keys meaningful right now) at each step.

**Mode 3: Persistent Session** - Multi-turn exploration without prefix replay
```bash
cargo run --bin review-tui -- --agent-repl
```
NDJSON on stdin/stdout: `{"cmd":"keys","input":"jj"}`, `{"cmd":"render"}`,
`{"cmd":"describe"}`, `{"cmd":"quit"}`. Every response carries
`state` + `visual` + `affordances`. Errors never end the session.

**Mode 4: Unit Tests** - Regression prevention
```bash
cargo test --package diffviz-review-tui --features test-harness
```
(The `test-harness` feature is only needed for `cargo test`; the CLI modes
above work on any build.)

### Testing Workflow

```
[ ] Step 1: Identify feature/keybinding to test
[ ] Step 2: Find it in --describe output (or add its row to src/events/bindings.rs)
[ ] Step 3: Run CLI test (--test-input, --test-full, or an --agent-repl session)
[ ] Step 4: Verify expected state changes in JSON output
[ ] Step 5: Write integration test in tests/keybinding_tests.rs
[ ] Step 6: Run full test suite to confirm
```

### Input Notation

Vim-style compact notation (full grammar in `--describe` under `notation`):
- Single keys: `j`, `k`, `a`
- Special keys: `<Space>`, `<Enter>`, `<Esc>`, `<Up>`, `<Down>`
- Modifiers: `<C-j>` (Ctrl), `<S-?>` (Shift), `<A-x>` (Alt)
- Delays: `<Wait:100>` — sleeps and runs `on_tick`, so timeouts really fire

Example: `jjk<Space>a<Enter>` = down, down, up, leader, actions submenu, submit

### Debugging Patterns

**Test single keybinding**:
```bash
cargo run --bin review-tui -- --test-input "j" | jq .nav_mode
```

**Capture before/after behavior**:
```bash
cargo run --bin review-tui -- --test-full "sequence" > before.txt
# Make changes
cargo run --bin review-tui -- --test-full "sequence" > after.txt
diff before.txt after.txt
```

**Verify test engine setup**:
```bash
cargo run --bin review-tui -- --test-input "" | jq .
```

**Drive a live session** (state persists between commands):
```bash
printf '%s\n' '{"cmd":"keys","input":"<Enter>"}' '{"cmd":"keys","input":"j"}' '{"cmd":"quit"}' \
  | cargo run --quiet --bin review-tui -- --agent-repl
```

## Code Review Rejection Criteria

Reject contributions that:

1. Add `&mut UiState` to view function signatures
2. Perform file I/O or console output in update handlers without Command
3. Access `drill_nav`/`drill_index` fields directly outside `state.rs`
4. Add time-based logic outside event handling flow
5. Skip event system for user interactions
6. Don't include tests for new functionality
7. Add key matching outside `src/events/bindings.rs` (dispatch, overlays,
   manifest, and affordances must all read the same registry row)
8. Edit `tests/dispatch_characterization_tests.rs` to make a refactor pass
   (that file pins behavior — changing it means behavior changed)

## Verification Commands

```bash
# Architecture compliance checks
rg "ui_state: &mut UiState" apps/diffviz/review-tui/src/ui/
rg "std::fs::|eprintln!|println!" apps/diffviz/review-tui/src/app.rs
rg "\.drill_nav\b" apps/diffviz/review-tui/src/ --glob '!state.rs'

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
- **Onboarding**: `apps/diffviz/review-tui/onboarding.md`
- **Architecture audit**: `apps/diffviz/review-tui/ARCHITECTURE_AUDIT.md`
- **Test harness**: `apps/diffviz/review-tui/TEST_HARNESS.md`
