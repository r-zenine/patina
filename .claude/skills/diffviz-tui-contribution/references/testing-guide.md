# Test Harness Detailed Guide

## Test Harness Architecture

The test harness enables headless testing of TUI behavior without requiring an interactive terminal.

### Three-Layer System

**Input Layer** (InputTestHarness):
- Validates keyboard processing
- Returns state snapshots after each key
- No rendering overhead

**Render Layer** (RenderTestHarness):
- Validates visual output
- Tests widget rendering
- No event handling

**Combined Layer** (CombinedTestHarness):
- Full integration testing
- Input → State → Visual
- Most comprehensive

### Key Components

**HeadlessApp** (`src/app.rs`):
- Terminal-independent version of ReviewTuiApp
- Behind `test-harness` feature flag
- Same event handling as production

**InputParser** (`src/test_harness/input_parser.rs`):
- Converts vim-style notation to KeyEvents
- Supports special keys, modifiers, delays

**StateSnapshot** (`src/test_harness/snapshot.rs`):
- JSON-serializable state representation
- Contains all UI state fields for assertions

## Input Notation Reference

### Basic Keys

```
j, k, h, l          → Single character keys
a, b, c, ...        → Letter keys
1, 2, 3, ...        → Number keys
```

### Special Keys

```
<Space>             → Space bar
<Enter>             → Enter/Return
<Esc>               → Escape
<Tab>               → Tab
<Backspace>         → Backspace
<Delete>            → Delete
<Up>, <Down>        → Arrow keys
<Left>, <Right>     → Arrow keys
<PageUp>            → Page up
<PageDown>          → Page down
<Home>, <End>       → Home/End
```

### Modifiers

```
<C-j>               → Ctrl+j
<S-?>               → Shift+? (produces '?')
<A-x>               → Alt+x
<C-S-k>             → Ctrl+Shift+k
```

### Special Sequences

```
<Wait:100>          → Wait 100ms (for timeout testing)
```

### Example Sequences

```
jjk                 → Down, down, up
<Space>a<Enter>     → Space, 'a', enter
<C-u><C-d>          → Ctrl+u, Ctrl+d
gg                  → 'g' twice (navigate to top)
```

## CLI Testing Commands

### Mode 1: Input Testing

**Basic usage**:
```bash
cargo run --bin review-tui --features test-harness -- --test-input "jjk"
```

**With jq for specific fields**:
```bash
# Check focused panel
cargo run --bin review-tui --features test-harness -- --test-input "jjk" | jq .focused_panel

# Check decision tree path
cargo run --bin review-tui --features test-harness -- --test-input "j" | jq .decision_tree_path

# Check input mode
cargo run --bin review-tui --features test-harness -- --test-input "i" | jq .input_mode
```

**Empty sequence for initial state**:
```bash
cargo run --bin review-tui --features test-harness -- --test-input "" | jq .
```

### Mode 2: Integration Testing

**Full workflow testing**:
```bash
cargo run --bin review-tui --features test-harness -- --test-full "jjk<Space>a<Enter>"
```

**Capture to file**:
```bash
cargo run --bin review-tui --features test-harness -- --test-full "sequence" > test-output.txt
```

**Before/after comparison**:
```bash
# Before changes
cargo run --bin review-tui --features test-harness -- --test-full "jjk" > before.txt

# Make code changes

# After changes
cargo run --bin review-tui --features test-harness -- --test-full "jjk" > after.txt

# Compare
diff before.txt after.txt
```

### Mode 3: Unit Tests

**Run all tests**:
```bash
cargo test --package diffviz-review-tui --features test-harness
```

**Run specific test**:
```bash
cargo test --package diffviz-review-tui --features test-harness test_navigation_down
```

**Run with output**:
```bash
cargo test --package diffviz-review-tui --features test-harness -- --nocapture
```

## State Snapshot Fields

The JSON state snapshot includes:

```json
{
  "focused_panel": "FileList" | "DiffView",
  "input_mode": "Navigation" | {"Instruction": {...}} | {"Edit": {...}},
  "scroll_offset": 0,
  "input_buffer": "",
  "input_cursor": 0,
  "show_all_context": true,
  "cursor_index": 0,
  "selection_range": null | [start, end],
  "show_instructions": false,
  "leader_active": false,
  "show_help": false,
  "decision_tree_path": [0, 1, ...],
  "decision_tree_selected_id": "chunk-id" | null,
  "decision_tree_selected_file": "path/to/file.rs" | null,
  "decision_tree_modal_open": false
}
```

## Writing Integration Tests

### Test Structure

```rust
#[test]
fn test_feature_name() {
    // 1. Create test engine with fixture data
    let review_engine = create_test_engine();

    // 2. Create test harness
    let mut harness = InputTestHarness::new(review_engine);

    // 3. Run key sequence
    let snapshots = harness.run_sequence("jjk<Space>").unwrap();

    // 4. Assert on expected state
    let final_state = snapshots.last().unwrap();
    assert_eq!(final_state.focused_panel, "FileList");
}
```

### Test Engine Setup

All tests need a properly configured ReviewEngine:

```rust
fn create_test_engine() -> ReviewEngine {
    // Load fixture data
    let decisions = load_fixture_decisions();

    // Create mock provider
    let provider = MockDiffProvider::new();

    // Create engine
    let mut engine = ReviewEngine::new(Arc::new(provider));

    // Add decisions
    engine.set_decisions_with_index(decisions);

    engine
}
```

### Common Assertion Patterns

**Check navigation**:
```rust
assert_eq!(snapshot.focused_panel, "DiffView");
assert_eq!(snapshot.cursor_index, 5);
```

**Check input mode**:
```rust
match &snapshot.input_mode {
    InputMode::Instruction { reviewable_id } => {
        assert_eq!(reviewable_id.to_string(), "expected-id");
    }
    _ => panic!("Expected instruction mode"),
}
```

**Check leader key**:
```rust
assert!(snapshot.leader_active);
```

**Check selection**:
```rust
assert_eq!(snapshot.selection_range, Some((10, 15)));
```

## Debugging Patterns

### Verify Fixture Loading

Check if test engine has decisions:

```bash
cargo run --bin review-tui --features test-harness -- --test-input "" | jq .decision_tree_selected_id
```

If null, decisions aren't loaded. Check `create_test_engine()`.

### Trace State Changes

Run sequence step-by-step:

```bash
# Single key
cargo run --bin review-tui --features test-harness -- --test-input "j" | jq .

# Two keys
cargo run --bin review-tui --features test-harness -- --test-input "jj" | jq .

# Three keys
cargo run --bin review-tui --features test-harness -- --test-input "jjj" | jq .
```

### Visual Output Inspection

For rendering issues:

```bash
cargo run --bin review-tui --features test-harness -- --test-full "sequence" | less
```

Look for visual artifacts, alignment issues, or missing content.

### Compare Against Working Version

```bash
# Checkout working commit
git checkout working-commit

# Capture baseline
cargo run --bin review-tui --features test-harness -- --test-full "sequence" > baseline.txt

# Return to current commit
git checkout -

# Capture current
cargo run --bin review-tui --features test-harness -- --test-full "sequence" > current.txt

# Compare
diff baseline.txt current.txt
```

## Test Fixtures

### Location

Test fixtures in `apps/diffviz/review/tests/fixtures/`:
- Realistic code samples
- Pre-analyzed diffs
- Mock ReviewEngine data

### Loading Fixtures

```rust
use diffviz_review::test_utils::load_fixture_decisions;

let decisions = load_fixture_decisions();
```

### Fixture Consistency

All test binaries use identical fixtures:
- Predictable test behavior
- Reproducible results
- Reliable regression testing

## Common Test Patterns

### Pattern: Navigation Flow

```rust
#[test]
fn test_navigation_flow() {
    let mut harness = create_harness();

    // Navigate down
    let snaps = harness.run_sequence("jjj").unwrap();
    assert_eq!(snaps.last().unwrap().cursor_index, 3);

    // Navigate up
    let snaps = harness.run_sequence("k").unwrap();
    assert_eq!(snaps.last().unwrap().cursor_index, 2);
}
```

### Pattern: Focus Switching

```rust
#[test]
fn test_focus_switch() {
    let mut harness = create_harness();

    // Start in FileList
    assert_eq!(harness.current_state().focused_panel, "FileList");

    // Switch to DiffView
    let snaps = harness.run_sequence("<Tab>").unwrap();
    assert_eq!(snaps.last().unwrap().focused_panel, "DiffView");
}
```

### Pattern: Input Mode Workflow

```rust
#[test]
fn test_instruction_workflow() {
    let mut harness = create_harness();

    // Enter instruction mode
    let snaps = harness.run_sequence("i").unwrap();
    let state = snaps.last().unwrap();
    assert!(matches!(state.input_mode, InputMode::Instruction { .. }));

    // Type content
    let snaps = harness.run_sequence("test<Enter>").unwrap();
    let state = snaps.last().unwrap();

    // Should exit input mode after submit
    assert!(matches!(state.input_mode, InputMode::Navigation));
}
```

### Pattern: Leader Key Sequence

```rust
#[test]
fn test_leader_sequence() {
    let mut harness = create_harness();

    // Activate leader
    let snaps = harness.run_sequence("<Space>").unwrap();
    assert!(snaps.last().unwrap().leader_active);

    // Execute leader command
    let snaps = harness.run_sequence("a").unwrap();
    assert!(!snaps.last().unwrap().leader_active);  // Deactivated after
}
```

## Troubleshooting

### Tests Pass But UI Broken

The test harness uses HeadlessApp, which may have diverged from ReviewTuiApp.

Solution: Check for duplicated logic in `src/app.rs` and ensure both implementations match.

### State Changes Not Reflected

InputTestHarness only captures state after event processing.

Solution: Use `--test-full` to see intermediate state transitions.

### Fixture Data Missing

Engine has no decisions loaded.

Solution: Verify `create_test_engine()` calls `set_decisions_with_index()`.

### Timeout Behavior

Time-based features require `<Wait:N>` notation.

Example:
```bash
cargo run --bin review-tui --features test-harness -- --test-input "<Space><Wait:3000>j"
```

This activates leader, waits 3 seconds (timeout), then navigates.
