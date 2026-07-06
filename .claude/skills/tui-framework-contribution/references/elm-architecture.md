# Elm Architecture Detailed Guide

## Core Principles

Elm architecture consists of:
1. **Model**: Single source of truth (immutable state)
2. **View**: Pure functions `State → UI` (no mutations, no side effects)
3. **Update**: Pure functions `(State, Message) → (State, Command)` (no side effects)
4. **Commands**: Descriptions of side effects executed by runtime

## Command System Implementation

### Command Types

```rust
pub enum Command {
    WriteFile { path: String, content: String },
    ShowMessage { message: String },
    Batch(Vec<Command>),
    None,
}
```

### Command Execution

Commands are executed in the main loop after state updates:

```rust
pub fn run(&mut self) -> Result<()> {
    loop {
        self.render()?;
        let command = self.handle_events()?;
        execute_command(command)?;  // Execute side effects here
        if self.ui_state.should_quit { break; }
    }
    Ok(())
}
```

### Creating Commands

```rust
// In handle_business_event
BusinessEvent::ExportInstructions { scope } => {
    let json = self.review_engine.export_instructions_json(scope)?;
    let filename = format!("instructions-{}.json", scope);

    Ok(Command::Batch(vec![
        Command::WriteFile {
            path: filename.clone(),
            content: json
        },
        Command::ShowMessage {
            message: format!("Exported to {}", filename)
        },
    ]))
}
```

## State Encapsulation Patterns

### Why Encapsulation Matters

Direct field access breaks encapsulation:
- Makes state transitions harder to track
- Allows inconsistent state updates
- Couples code to internal representation

### Adding Encapsulation Methods

When you need to update nested state, add a method to `UiState`:

```rust
// BAD - direct field access in app.rs
self.ui_state.decision_tree.selected_path = first.path.clone();

// GOOD - add method to UiState in state.rs
impl UiState {
    pub fn navigate_to_first_in_tree(&mut self) {
        let flattened = self.decision_tree.flatten();
        if let Some(first) = flattened.first() {
            self.decision_tree.selected_path = first.path.clone();
            self.reset_scroll();  // Coordinated state update
        }
    }
}

// Use the method in app.rs
self.ui_state.navigate_to_first_in_tree();
```

### Existing Encapsulation Methods

**Navigation**:
- `navigate_to_first_in_tree()`, `navigate_to_last_in_tree()`
- `reset_scroll()`

**Modal Management**:
- `is_modal_open()`, `close_modal_if_open()`

**Input Modes**:
- `start_instruction_input(reviewable_id)`
- `start_edit_mode(reviewable_id)`
- `exit_input_mode()`
- `is_in_input_mode()`

**Leader Key**:
- `activate_leader()`, `deactivate_leader()`
- `enter_leader_submenu(char)`
- `is_leader_timed_out()`, `leader_timeout_remaining()`

**Getters** (computed from tree):
- `current_reviewable_id()`, `current_file_path()`

## Event Flow Details

### Event Hierarchy

```
KeyEvent (crossterm)
    ↓
UiEvent (UI navigation/display changes)
    ↓
BusinessEvent (ReviewEngine operations)
    ↓
Command (I/O side effects)
```

### UiEvent vs BusinessEvent

**UiEvent**: Pure UI state changes, no business logic
- Navigation (up/down/left/right)
- Scrolling
- Focus switching
- Input mode changes
- Leader key activation

**BusinessEvent**: Requires ReviewEngine interaction
- Approve/reject reviewable
- Add comment/instruction
- Export data
- File operations

### Conversion Pattern

```rust
// In events/business.rs
pub fn ui_event_to_business_event(
    ui_event: &UiEvent,
    ui_state: &UiState,
) -> Option<BusinessEvent> {
    match ui_event {
        UiEvent::ToggleApprove => {
            ui_state.current_reviewable_id().map(|id| {
                BusinessEvent::ToggleApprove { reviewable_id: id }
            })
        }
        UiEvent::SubmitInput => {
            match &ui_state.input_mode {
                InputMode::Instruction { reviewable_id } => {
                    Some(BusinessEvent::AddInstruction {
                        reviewable_id: *reviewable_id,
                        content: ui_state.input_buffer.clone(),
                    })
                }
                _ => None,
            }
        }
        _ => None,
    }
}
```

## Time-Based Behavior

### The Pattern

Time checks happen in `handle_events()`, state updates through methods:

```rust
fn handle_events(&mut self) -> Result<Command> {
    // Check timeout
    if self.ui_state.leader_active && self.ui_state.is_leader_timed_out() {
        self.ui_state.deactivate_leader();
        return Ok(Command::None);
    }

    // Poll for input
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            return self.process_key_event(key);
        }
    }

    Ok(Command::None)
}
```

### Why Not Direct Time Checks?

```rust
// VIOLATION - scattered time logic
if Instant::now() > self.some_deadline {
    self.state.some_field = false;
}
```

Problems:
- Time logic scattered across codebase
- Hard to test
- Breaks event-driven pattern
- Inconsistent with message-based architecture

## View Purity

### The Contract

Views are functions `State → UI`. They:
- Read state
- Compute UI representation
- Never mutate state
- Never perform side effects

### Why Immutable References?

```rust
// With &mut, compiler allows this (BAD):
pub fn render(f: &mut Frame, ui_state: &mut UiState) {
    ui_state.scroll_offset += 10;  // Accidental mutation during render!
    // ... render code
}

// With &, compiler prevents mutations:
pub fn render(f: &mut Frame, ui_state: &UiState) {
    // ui_state.scroll_offset += 10;  // Compile error!
    // ... render code
}
```

Benefits:
- Compiler-enforced immutability
- Clearer intent
- Easier to reason about
- Matches Elm pattern exactly

## Common Patterns

### Pattern: Coordinated State Updates

When multiple state fields must update together, use a method:

```rust
impl UiState {
    pub fn navigate_to_first_in_tree(&mut self) {
        let flattened = self.decision_tree.flatten();
        if let Some(first) = flattened.first() {
            // Coordinated updates
            self.decision_tree.selected_path = first.path.clone();
            self.reset_scroll();  // Keep scroll consistent
        }
    }
}
```

### Pattern: Optional Command Returns

Most handlers return `Command::None`:

```rust
UiEvent::NavigateUp => {
    self.ui_state.cursor_up();
    // No command needed, just state change
}
```

Only return commands for I/O:

```rust
BusinessEvent::ExportInstructions { scope } => {
    let json = self.review_engine.export_instructions_json(scope)?;
    Ok(Command::WriteFile {
        path: filename,
        content: json
    })
}
```

### Pattern: Handler Chains

UI events may trigger business events:

```rust
pub fn process_key_event(&mut self, key: KeyEvent) -> Result<Command> {
    if let Some(ui_event) = handle_key_event(key, ...) {
        // Handle UI state changes
        self.handle_ui_event(&ui_event)?;

        // Convert to business event if needed
        if let Some(business_event) = ui_event_to_business_event(&ui_event, &self.ui_state) {
            return self.handle_business_event(business_event);
        }
    }
    Ok(Command::None)
}
```

## Testing Elm Patterns

### Test View Purity

Views should be testable without side effects:

```rust
#[test]
fn test_view_purity() {
    let ui_state = UiState::default();
    let engine = create_test_engine();

    // Should be able to call multiple times with same result
    let mut buf1 = Buffer::empty(Rect::new(0, 0, 100, 50));
    draw(&mut buf1, &ui_state, &engine);

    let mut buf2 = Buffer::empty(Rect::new(0, 0, 100, 50));
    draw(&mut buf2, &ui_state, &engine);

    assert_eq!(buf1, buf2);  // Deterministic output
}
```

### Test Command Generation

Update logic should be testable without executing I/O:

```rust
#[test]
fn test_export_generates_command() {
    let mut app = create_test_app();
    let event = BusinessEvent::ExportInstructions {
        scope: InstructionScope::All
    };

    let command = app.handle_business_event(event).unwrap();

    match command {
        Command::WriteFile { path, content } => {
            assert!(path.ends_with(".json"));
            assert!(!content.is_empty());
        }
        _ => panic!("Expected WriteFile command"),
    }
}
```
