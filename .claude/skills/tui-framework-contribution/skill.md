---
name: tui-framework-contribution
description: Mandatory skill to use whenever you interact with any TUI built on the tui-harness/tui-elm/tui-design framework (diffviz-review-tui, patina-detect-tui, and future ELM-style TUIs in this workspace), whether to add code to it, test it, or plan a change
---

# Contributing to a framework TUI

Comprehensive guide for making architecture-compliant contributions to any
`*-tui` crate built on `tui-harness` (ELM runtime, headless testing, agent
discovery), `tui-elm` (keybinding registry, leader state, overlays), and
`tui-design` (theme, stylesheet, card/drillnav rendering primitives).

This skill states framework law once, for every consuming crate. Rule text
and rejection criteria below must never hardcode a specific crate's paths or
types — those go in that crate's own **Per-crate addendum** at the bottom.

## What This Skill Does

Ensures contributions to a framework TUI follow:
- Elm architecture patterns (pure views, command system, event flow)
- Proper state encapsulation through methods
- Test-driven development with the test harness
- Verification of behavior before and after changes

## When to Use This Skill

✅ **Use when:**
- Implementing new TUI features or keybindings in any `*-tui` crate
- Modifying existing UI components or state management in any `*-tui` crate
- Fixing bugs in a TUI layer
- Reviewing TUI pull requests
- Testing UI workflows and navigation
- Debugging TUI behavior or visual rendering
- Extracting or modifying shared rendering code in `tui-harness`/`tui-elm`/`tui-design`

❌ **Don't use when:**
- Working on a domain/business-logic crate with no UI surface (e.g.
  `diffviz-core`, `diffviz-review`, `patina-detect`)
- Writing pure backend code without UI impact

## Quick Reference

| Component | Rule | Location |
|-----------|------|----------|
| View | `&UiState` (immutable) | `<tui-crate>/src/ui/**/*.rs` |
| Update | Mutates `UiState`, returns `Command` | `<tui-crate>/src/app.rs` |
| Command | I/O side effects only | `<tui-crate>/src/command.rs` |
| Events | `KeyEvent` → `UiEvent` → `BusinessEvent` | `<tui-crate>/src/events/*.rs` |
| Keybindings | One registry row per binding (feeds everything) | `<tui-crate>/src/events/bindings.rs` |
| Tests | Test harness with fixture data | `<tui-crate>/tests/*.rs` |
| Discovery | `--describe` / `--agent-repl` on the binary | provided by `tui-harness` |

`<tui-crate>` is whichever framework TUI you're contributing to (e.g.
`apps/diffviz/review-tui`, `apps/patina-detect-tui`) — substitute its actual
path. See the per-crate addendum for that crate's exact binary name and
package name.

## Architecture Compliance Rules

### V1: Pure View Functions

**Rule**: All view/render functions must accept `&UiState`, never `&mut UiState`.

```rust
// CORRECT
pub fn render(f: &mut Frame, area: Rect, ui_state: &UiState, engine: &Engine)

// VIOLATION
pub fn render(f: &mut Frame, area: Rect, ui_state: &mut UiState, engine: &Engine)
```

**Verification**: `rg "ui_state: &mut UiState" <tui-crate>/src/ui/`

Generic rendering primitives that live in `tui-design` (card chrome,
drillnav Browse/Drill, status bar, input modal) take plain data
(`&DrillGroup`/`&DrillChunk` impls, pre-formatted strings) instead of any
app's `UiState` at all — they don't know a `UiState` type exists. This rule
applies to the domain-side adapter functions that build that data as much as
to any hand-rolled view.

### V2: Command System for Side Effects

**Rule**: Update handlers return `Command` for I/O operations, not execute inline.

```rust
// CORRECT
fn handle_business_event(&mut self, event: BusinessEvent) -> Result<Command> {
    match event {
        BusinessEvent::Export { scope } => {
            let json = self.engine.export_json(scope)?;
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
if self.ui_state.leader.is_active() && self.ui_state.is_leader_timed_out() {
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
1. Add a `UiEvent` variant in `src/events/input.rs`
2. Add a `Binding` row in `src/events/bindings.rs` (keys, event, notation,
   description) — dispatch, which-key, help overlay, `--describe`, and
   affordances all pick it up from this one row (`tui_elm::Registry`)
3. Handle in `handle_ui_event()` for UI changes
4. Add a `BusinessEvent` if engine interaction is needed
5. Return `Command` for side effects in `handle_business_event()`

**Never** add key matching outside the registry — `handle_key_event` is a
lookup plus two documented fallbacks, nothing else.

## Testing Your Changes

### Discover First

The binary describes itself — start here instead of reading source:
```bash
cargo run --bin <tui-binary> -- --describe
```
Returns a JSON manifest: all keybindings per mode, the input-sequence
notation grammar, and the JSON Schema of the state snapshot.

### Four Testing Modes

**Mode 1: Input Testing** - State validation
```bash
cargo run --bin <tui-binary> -- --test-input "jjk<Space>"
```
Returns the final JSON state snapshot.

**Mode 2: Integration Testing** - Full workflow
```bash
cargo run --bin <tui-binary> -- --test-full "sequence"
```
Returns state + visual + affordances (keys meaningful right now) at each step.

**Mode 3: Persistent Session** - Multi-turn exploration without prefix replay
```bash
cargo run --bin <tui-binary> -- --agent-repl
```
NDJSON on stdin/stdout: `{"cmd":"keys","input":"jj"}`, `{"cmd":"render"}`,
`{"cmd":"describe"}`, `{"cmd":"quit"}`. Every response carries
`state` + `visual` + `affordances`. Errors never end the session.

**Mode 4: Unit Tests** - Regression prevention
```bash
cargo test --package <tui-package> --features test-harness
```
(The `test-harness` feature is only needed for `cargo test`; the CLI modes
above work on any build.)

### Testing Workflow

```
[ ] Step 1: Identify feature/keybinding to test
[ ] Step 2: Find it in --describe output (or add its row to src/events/bindings.rs)
[ ] Step 3: Run CLI test (--test-input, --test-full, or an --agent-repl session)
[ ] Step 4: Verify expected state changes in JSON output
[ ] Step 5: Write an integration test alongside the crate's existing ones
[ ] Step 6: Run the full test suite to confirm
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
cargo run --bin <tui-binary> -- --test-input "j" | jq .nav_mode
```

**Capture before/after behavior**:
```bash
cargo run --bin <tui-binary> -- --test-full "sequence" > before.txt
# Make changes
cargo run --bin <tui-binary> -- --test-full "sequence" > after.txt
diff before.txt after.txt
```

**Verify test engine setup**:
```bash
cargo run --bin <tui-binary> -- --test-input "" | jq .
```

**Drive a live session** (state persists between commands):
```bash
printf '%s\n' '{"cmd":"keys","input":"<Enter>"}' '{"cmd":"keys","input":"j"}' '{"cmd":"quit"}' \
  | cargo run --quiet --bin <tui-binary> -- --agent-repl
```

## Code Review Rejection Criteria

Reject contributions that:

1. Add `&mut UiState` to view function signatures
2. Perform file I/O or console output in update handlers without Command
3. Access internal navigation/focus fields directly outside `state.rs`
4. Add time-based logic outside event handling flow
5. Skip event system for user interactions
6. Don't include tests for new functionality
7. Add key matching outside `src/events/bindings.rs` (dispatch, overlays,
   manifest, and affordances must all read the same registry row)
8. Edit a characterization test file to make a refactor pass (that file pins
   behavior — changing it means behavior changed; check whether the crate
   marks such files, e.g. `tests/dispatch_characterization_tests.rs`)

## Verification Commands

```bash
# Architecture compliance checks
rg "ui_state: &mut UiState" <tui-crate>/src/ui/
rg "std::fs::|eprintln!|println!" <tui-crate>/src/app.rs
rg "\.drill_nav\b" <tui-crate>/src/ --glob '!state.rs'

# Run tests
cargo test --package <tui-package> --features test-harness

# Compile check
cargo check --package <tui-package>

# Clippy
cargo clippy --package <tui-package>
```

## Accepted Pragmatic Compromises

**Direct engine mutations**: Direct `self.engine.approve()`-style calls in
handlers are acceptable. These are synchronous domain operations, not I/O
side effects.

```rust
// ACCEPTABLE
BusinessEvent::ToggleApprove { id } => {
    if self.engine.state().is_approved(&id) {
        self.engine.reject(id, None)?;
    }
    Ok(Command::None)
}
```

## Reference Documentation

For detailed information, see:
- **Architecture principles**: [references/elm-architecture.md](references/elm-architecture.md)
- **Test harness guide**: [references/testing-guide.md](references/testing-guide.md)

---

## Per-crate addenda

Each framework TUI keeps its own onboarding/testing specifics here instead
of duplicating rule text above.

### `diffviz-review-tui`

- Binary: `review-tui` (`cargo run --bin review-tui`)
- Package: `diffviz-review-tui` (`cargo test --package diffviz-review-tui --features test-harness`)
- Characterization test that pins behavior: `tests/dispatch_characterization_tests.rs`
- Onboarding: `apps/diffviz/review-tui/onboarding.md`
- Architecture audit: `apps/diffviz/review-tui/ARCHITECTURE_AUDIT.md`
- Test harness notes: `apps/diffviz/review-tui/TEST_HARNESS.md`
- Its Browse/Drill views are generic `tui_design::drillnav` renderers behind
  a `DecisionGroup`/`ChunkAdapter` adapter
  (`src/ui/components/drillnav_{browse,drill}.rs`) implementing
  `DrillGroup`/`DrillChunk` over `ReviewEngine`'s `Decision`/`ReviewableDiff`
  model; `note_for`/`note_rows`/`line_change_type`/`annotations_for` in
  `drillnav_common.rs` stay domain-specific because they reach into
  `ReviewEngine`/`Instruction`/`RenderableLine`.
- Its which-key and help overlays call `tui_elm::which_key::render` /
  `tui_elm::help::render` directly from `src/ui/mod.rs` — no per-crate
  wrapper, since those two are already fully generic over `Registry<E>` and
  `LeaderState`.

### `patina-detect-tui`

- Not yet built — add its addendum here once its first phase lands
  (binary name, package name, characterization test file, and how its
  `Symptom`/`Site` model adapts to `DrillGroup`/`DrillChunk`).
