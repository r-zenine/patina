# Code Context: ELM Architecture Refactoring

## Key Types and Their Roles

### UiState (src/state.rs)
Pure UI state container for navigation, focus, and display preferences.

**Current signature issues**:
- Line 34-87: Struct definition with public fields including `decision_tree`
- Line 115-405: Methods that mutate self (correct for Model in ELM)

**Direct field access problems** (in app.rs):
- app.rs:189 - `self.ui_state.decision_tree.selected_path = first.path.clone()`
- app.rs:292 - `if self.ui_state.decision_tree.show_decision_modal`

**Methods needing encapsulation**:
- Lines 122-124: `reset_scroll()` - simple mutation
- Lines 127-129: `current_reviewable_id()` - getter, good
- Lines 131-134: `current_file_path()` - getter, good
- Need new methods: `navigate_to_first_in_tree()`, `navigate_to_last_in_tree()`, `close_modal_if_open()`

### ReviewTuiApp (src/app.rs)
Main coordinator between ReviewEngine and UI.

**Current event loop** (lines 73-85):
```
pub fn run(&mut self) -> Result<()> {
    loop {
        self.render()?;
        if self.handle_events()? { break; }
    }
    Ok(())
}
```

**Needs to become**:
```
pub fn run(&mut self) -> Result<()> {
    loop {
        self.render()?;
        let command = self.handle_events()?;
        self.execute_command(command)?;
        if self.ui_state.should_quit { break; }
    }
    Ok(())
}
```

**Time-based side effect** (lines 96-110):
- Line 98-99: Direct mutation when leader times out
- Should emit LeaderTimeout event instead

**Update functions needing refactoring**:
- Lines 138-418: `handle_ui_event` - currently mutates, needs to stay mutation-based (updates Model)
- Lines 420-481: `handle_business_event` - currently executes side effects, needs to return Command

**Side effects to extract** (lines 448-472):
```rust
BusinessEvent::ExportInstructions { ref scope } => {
    let json = self.review_engine.export_instructions_json(scope.clone())?;
    let filename = /* ... */;

    // SIDE EFFECT - needs to become Command::WriteFile
    std::fs::write(&filename, json)
        .map_err(diffviz_review::errors::DiffVizError::Io)?;

    // SIDE EFFECT - needs to become Command::ShowMessage
    eprintln!("Exported instructions to {filename}");
}
```

### View Functions (src/ui/mod.rs, src/ui/components/*.rs)

**Main draw function** (ui/mod.rs:11):
```rust
pub fn draw(f: &mut Frame, ui_state: &mut UiState, review_engine: &ReviewEngine)
```
Should be:
```rust
pub fn draw(f: &mut Frame, ui_state: &UiState, review_engine: &ReviewEngine)
```

**All component render functions** need same change:
- ui/components/diff_view.rs:21 - `render(f, area, ui_state: &mut UiState, engine)`
- ui/components/decision_tree.rs:19 - `render(f, area, ui_state: &UiState, engine)` (already correct!)
- ui/components/status_bar.rs - needs checking
- ui/components/decision_detail_modal.rs - needs checking
- ui/components/input_modal.rs - needs checking
- ui/components/help_overlay.rs - needs checking
- ui/components/which_key.rs - needs checking

### Event System (src/events/)

**UiEvent** (events/input.rs:11-86):
Already well-structured, needs one addition:
```rust
pub enum UiEvent {
    // ... existing variants
    LeaderTimeout,  // NEW - for time-based behavior
}
```

**BusinessEvent** (events/business.rs:12-40):
Already well-structured, no changes needed.

**Event conversion** (events/business.rs:43-90):
`ui_event_to_business_event()` - pure function, good!

### DecisionNavigationTree (src/decision_navigation.rs)

**Tree building** (lines 146-192):
```rust
pub fn build_from_review_engine(review_engine: &ReviewEngine) -> Self {
    // Complex logic grouping decisions, files, chunks
    // This belongs in business layer or needs better encapsulation
}
```

**Public field access** (line 13-19):
```rust
pub struct DecisionNavigationTree {
    pub nodes: Vec<DecisionNode>,
    pub selected_path: TreePath,  // Direct access in app.rs:189
    pub show_decision_modal: bool, // Direct access in app.rs:292
}
```

Should have methods instead:
- `select_first()`, `select_last()`
- `is_modal_open()`, `toggle_modal()`

## HeadlessApp (src/app.rs:520-937)
Test-only struct behind `test-harness` feature.

**Critical**: Same refactoring needed as ReviewTuiApp
- Lines 543-566: `process_key_event` - duplicated from ReviewTuiApp
- Lines 569-848: `handle_ui_event` - duplicated from ReviewTuiApp
- Lines 850-911: `handle_business_event` - duplicated from ReviewTuiApp

**Strategy**: Extract shared logic to methods that both ReviewTuiApp and HeadlessApp can use.

## Testing Infrastructure

**Keybinding Tests** (tests/keybinding_tests.rs):
Integration tests using HeadlessApp to verify key bindings without terminal.

**Requirements**:
- Must continue working after refactoring
- Uses `test-harness` feature flag
- Creates HeadlessApp, sends key events, asserts state changes

**Example test pattern**:
```rust
#[test]
fn test_navigation_keys() {
    let mut app = HeadlessApp::new(review_engine);
    app.process_key_event(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE))?;
    // Assert state changed as expected
}
```

## Dependencies Between Violations

**Dependency Graph**:
```
V1 (View signatures) ← Independent, can fix first
V2 (Side effects) ← Requires Command system
V3 (Time-based) ← Requires event system update
V4 (Direct access) ← Requires UiState method additions
V5 (Tree building) ← Can defer or do separately
V6 (ReviewEngine) ← Pragmatic compromise, optional
```

**Recommended Order**:
1. V1 - Easy, no dependencies
2. V4 - Encapsulate state access (needed before other changes)
3. V2 - Add Command system (core architectural change)
4. V3 - Model time as messages (uses event system)
5. V5 - Move tree building (optional, separate concern)
6. V6 - Command-ify ReviewEngine (optional, major change)
