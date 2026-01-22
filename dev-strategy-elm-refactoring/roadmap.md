# Implementation Roadmap: ELM Architecture Refactoring

**Strategy**: Steel Thread with Refactor Steps
**Crate**: diffviz-review-tui
**Estimated Complexity**: Medium (5 phases)

## Overview

Refactor diffviz-review-tui to comply with pure ELM architecture patterns by fixing violations in a specific order that maintains working application at every step. Each phase includes its own refactor step to prevent technical debt.

## Phase 1: Pure View Functions (V1 Fix)

**Goal**: Change all view function signatures to use immutable state references.

**What to Build**:
Change view function signatures from accepting `&mut UiState` to `&UiState` throughout the UI layer. This makes the immutability contract explicit and prevents accidental mutations in view logic.

**Files to Modify**:
- `src/ui/mod.rs` - Main draw function (line 11)
- `src/ui/components/diff_view.rs` - render function (line 21)
- `src/ui/components/status_bar.rs` - render function
- `src/ui/components/decision_detail_modal.rs` - render function
- `src/ui/components/input_modal.rs` - render function
- `src/ui/components/file_list.rs` - render function (if exists)
- `src/ui/components/help_overlay.rs` - render function
- `src/ui/components/which_key.rs` - render function
- `src/app.rs` - Update call sites (line 89-91 in `render()`)

**Implementation Steps**:

1. **Update main draw signature**
   - Change: `pub fn draw(f: &mut Frame, ui_state: &mut UiState, ...)`
   - To: `pub fn draw(f: &mut Frame, ui_state: &UiState, ...)`

2. **Update all component render signatures**
   - For each component in `src/ui/components/`:
     - Change `ui_state: &mut UiState` to `ui_state: &UiState`
   - Note: decision_tree.rs already uses `&UiState` (correct!)

3. **Update call sites in ReviewTuiApp**
   - Change `ui::draw(f, &mut self.ui_state, &self.review_engine)`
   - To: `ui::draw(f, &self.ui_state, &self.review_engine)`

4. **Update HeadlessApp (if using draw functions)**
   - Apply same changes to test infrastructure

**Verification**:
```bash
# Should compile without errors
cargo check --package diffviz-review-tui

# Views can't mutate state - compiler enforces
```

**Success Criteria**:
- ✅ All view functions use `&UiState`
- ✅ Code compiles
- ✅ No behavioral changes

**Refactor Step**: None needed (this is the refactor)

---

## Phase 2: Encapsulate State Mutations (V4 Fix)

**Goal**: Add dedicated methods to UiState for all nested state operations, eliminating direct field access from event handlers.

**What to Build**:
Add methods to UiState that encapsulate common operations on nested state (especially DecisionNavigationTree). Update event handlers to use these methods instead of directly accessing fields.

**Files to Modify**:
- `src/state.rs` - Add new methods
- `src/app.rs` - Update event handlers (lines 189, 292-296, others)
- `src/decision_navigation.rs` - Document public API

**New UiState Methods**:

1. **navigate_to_first_in_tree()**
   - Encapsulates: `self.decision_tree.selected_path = flatten()[0].path`
   - Used in: NavigateToTop event handling

2. **navigate_to_last_in_tree()**
   - Encapsulates: `self.decision_tree.selected_path = flatten().last().path`
   - Used in: NavigateToBottom event handling

3. **close_modal_if_open()**
   - Encapsulates: `if self.decision_tree.show_decision_modal { close... }`
   - Used in: ExitInputMode, CancelInput handling

4. **is_modal_open() -> bool**
   - Getter for: `self.decision_tree.show_decision_modal`
   - Used in: Multiple modal-related checks

**Implementation Steps**:

1. **Add methods to UiState (src/state.rs)**
   - Add `navigate_to_first_in_tree(&mut self)`
   - Add `navigate_to_last_in_tree(&mut self)`
   - Add `close_modal_if_open(&mut self)`
   - Add `is_modal_open(&self) -> bool`

2. **Update event handlers in app.rs**
   - Find all instances of direct `decision_tree` field access
   - Replace with method calls
   - Example: `self.ui_state.navigate_to_first_in_tree()` instead of `self.ui_state.decision_tree.selected_path = ...`

3. **Update HeadlessApp**
   - Same changes to duplicated event handler code

4. **Document public API**
   - Add doc comments to new UiState methods
   - Update onboarding.md if needed

**Verification**:
```bash
# Should compile and run
cargo check --package diffviz-review-tui
cargo run --bin review-tui

# Manual test: Navigate with 'g' and 'G', verify first/last selection
```

**Success Criteria**:
- ✅ No direct field access to `decision_tree` in app.rs
- ✅ All state updates go through UiState methods
- ✅ Navigation behavior unchanged

**Refactor Step**: Review UiState for other potential encapsulation improvements (defer to future work if found)

---

## Phase 3: Command System Foundation (V2 Fix Part 1)

**Goal**: Introduce Command enum and execution infrastructure, wire into main loop.

**What to Build**:
Create a Command type that describes side effects (file writes, notifications). Add command execution logic to main loop. Convert handle_business_event to return Command instead of executing side effects directly.

**Files to Create**:
- `src/command.rs` - Command enum and execute_command function

**Files to Modify**:
- `src/lib.rs` - Export command module
- `src/app.rs` - Update run loop, handle_business_event signature
- `src/events/business.rs` - No changes (BusinessEvent stays same)

**Command Type Design**:

```rust
// In src/command.rs
pub enum Command {
    /// Write content to a file
    WriteFile { path: String, content: String },

    /// Show a message to the user (currently via eprintln, could be UI notification later)
    ShowMessage { message: String },

    /// Batch multiple commands
    Batch(Vec<Command>),

    /// No operation
    None,
}
```

**Implementation Steps**:

1. **Create src/command.rs**
   - Define Command enum
   - Implement `execute_command(command: Command) -> Result<()>`
   - Handle WriteFile: `std::fs::write(path, content)?`
   - Handle ShowMessage: `eprintln!(message)`
   - Handle Batch: execute each sub-command
   - Handle None: no-op

2. **Update src/lib.rs**
   - Add `pub mod command;`
   - Export Command type

3. **Update ReviewTuiApp::run() in src/app.rs**
   - Change loop to call `execute_command()` after `handle_events()`
   - Pattern:
     ```rust
     loop {
         self.render()?;
         let command = self.handle_events()?;
         self.execute_command(command)?;
         if self.ui_state.should_quit { break; }
     }
     ```

4. **Update handle_events() signature**
   - Change: `fn handle_events(&mut self) -> Result<bool>`
   - To: `fn handle_events(&mut self) -> Result<Command>`
   - Return Command::None for most events
   - Pass through commands from process_key_event

5. **Update process_key_event() signature**
   - Change: `pub fn process_key_event(&mut self, key: KeyEvent) -> Result<()>`
   - To: `pub fn process_key_event(&mut self, key: KeyEvent) -> Result<Command>`
   - Collect command from handle_business_event
   - Return to caller

6. **Add execute_command() method to ReviewTuiApp**
   - Call `command::execute_command(command)?`
   - Wrap in method for future extensibility

**Verification**:
```bash
# Should compile but commands not yet returned
cargo check --package diffviz-review-tui

# App still works (returns Command::None everywhere)
cargo run --bin review-tui
```

**Success Criteria**:
- ✅ Command infrastructure in place
- ✅ Main loop calls execute_command
- ✅ App behavior unchanged (all None commands)

**Refactor Step**: None needed (next phase will use the infrastructure)

---

## Phase 4: Convert Side Effects to Commands (V2 Fix Part 2)

**Goal**: Refactor handle_business_event to return Commands instead of executing side effects directly.

**What to Build**:
Update ExportInstructions handler and any other side-effecting business event handlers to return Command values instead of performing I/O directly. This completes the separation of update logic from side effects.

**Files to Modify**:
- `src/app.rs` - handle_business_event implementation (lines 420-481)
- HeadlessApp - duplicate implementation

**Implementation Steps**:

1. **Update handle_business_event signature**
   - Change: `fn handle_business_event(&mut self, event: BusinessEvent) -> Result<()>`
   - To: `fn handle_business_event(&mut self, event: BusinessEvent) -> Result<Command>`

2. **Refactor ExportInstructions handler**
   - Current (lines 448-472):
     ```rust
     BusinessEvent::ExportInstructions { ref scope } => {
         let json = self.review_engine.export_instructions_json(scope)?;
         let filename = /* determine */;
         std::fs::write(&filename, json)?;  // SIDE EFFECT
         eprintln!("Exported to {}", filename);  // SIDE EFFECT
     }
     ```
   - New:
     ```rust
     BusinessEvent::ExportInstructions { ref scope } => {
         let json = self.review_engine.export_instructions_json(scope)?;
         let filename = /* determine */;
         Ok(Command::Batch(vec![
             Command::WriteFile { path: filename.clone(), content: json },
             Command::ShowMessage { message: format!("Exported to {}", filename) },
         ]))
     }
     ```

3. **Update other handlers**
   - ToggleApprove, ApproveFile, AddInstruction: return `Command::None`
   - EditContent, SaveSession, LoadSession: return `Command::None` (not implemented)

4. **Update call sites in process_key_event**
   - Capture returned Command
   - Return it to handle_events
   - Pattern:
     ```rust
     if let Some(business_event) = ui_event_to_business_event(&ui_event, &self.ui_state) {
         let command = self.handle_business_event(business_event)?;
         // Exit input mode, etc.
         return Ok(command);
     }
     ```

5. **Update HeadlessApp**
   - Apply same changes to test infrastructure
   - Add execute_command method (can be no-op for tests)

**Verification**:
```bash
# Should compile
cargo check --package diffviz-review-tui

# Test export functionality manually
cargo run --bin review-tui
# Press Space+e+a (export all), verify file created and message shown
```

**Success Criteria**:
- ✅ No file I/O in handle_business_event
- ✅ No eprintln in handle_business_event
- ✅ Commands returned and executed by main loop
- ✅ Export functionality works as before

**Refactor Step**: Review for any other hidden side effects in update logic

---

## Phase 5: Time as Messages (V3 Fix)

**Goal**: Model leader timeout as a message instead of direct mutation in event loop.

**What to Build**:
Add LeaderTimeout event to UiEvent enum. Update event loop to send timeout message through proper event handling instead of directly mutating state. Add handler for LeaderTimeout event.

**Files to Modify**:
- `src/events/input.rs` - Add LeaderTimeout variant
- `src/app.rs` - Update handle_events (lines 96-110), add event handler

**Implementation Steps**:

1. **Add LeaderTimeout to UiEvent**
   - In `src/events/input.rs` around line 86:
     ```rust
     pub enum UiEvent {
         // ... existing variants
         LeaderTimeout,  // Fired when leader key times out
     }
     ```

2. **Update handle_events in app.rs**
   - Current (lines 97-100):
     ```rust
     if self.ui_state.leader_active && self.ui_state.is_leader_timed_out() {
         self.ui_state.deactivate_leader();  // DIRECT MUTATION
     }
     ```
   - New:
     ```rust
     if self.ui_state.leader_active && self.ui_state.is_leader_timed_out() {
         self.handle_ui_event(&UiEvent::LeaderTimeout)?;
     }
     ```

3. **Add LeaderTimeout handler**
   - In `handle_ui_event` around line 415 (after existing handlers):
     ```rust
     UiEvent::LeaderTimeout => {
         self.ui_state.deactivate_leader();
     }
     ```

4. **Update HeadlessApp**
   - Add same LeaderTimeout handling
   - May need to expose timeout checking for tests

**Verification**:
```bash
# Should compile
cargo check --package diffviz-review-tui

# Test leader key timeout manually
cargo run --bin review-tui
# Press Space, wait 2+ seconds, verify leader mode exits
```

**Success Criteria**:
- ✅ Timeout modeled as message
- ✅ No direct mutation in event loop
- ✅ Leader key behavior unchanged

**Refactor Step**: Review event loop for any other direct mutations (should be clean after this)

---

## Final Verification

**After All Phases**:

1. **Compile and run**
   ```bash
   cargo check --package diffviz-review-tui
   cargo build --package diffviz-review-tui
   cargo run --bin review-tui
   ```

2. **Run existing tests**
   ```bash
   cargo test --package diffviz-review-tui
   ```

3. **Manual testing checklist**
   - Navigate with hjkl and arrow keys
   - Use g/G for top/bottom navigation
   - Test leader key (Space) and timeout
   - Test export functionality (Space+e+a)
   - Test modal open/close (Space+d, Escape)
   - Test instruction entry (Space+i+i)

4. **Verify ELM compliance**
   - ✅ View functions use `&UiState` (V1 fixed)
   - ✅ Side effects return Commands (V2 fixed)
   - ✅ Time as messages (V3 fixed)
   - ✅ State encapsulated (V4 fixed)
   - ⏸️ Tree building in UI layer (V5 deferred)
   - ⏸️ ReviewEngine mutations (V6 accepted compromise)

## Success Metrics

**Code Quality**:
- Pure view functions (compiler-enforced immutability)
- Testable update logic (pure functions return Commands)
- Clear separation of concerns (side effects isolated)

**Maintainability**:
- Single path for state updates (UiState methods)
- Easy to add new commands (extend enum)
- Clear event flow (KeyEvent → UiEvent → Command → Execution)

**Testing**:
- Existing tests pass
- Command logic unit testable
- Update logic testable in isolation

## Future Work

**Deferred (V5)**: Move tree building logic to business layer
- Requires changes to diffviz-review crate
- Can be separate refactoring task
- Track as technical debt

**Optional (V6)**: Command-ify ReviewEngine operations
- Major architectural change
- Diminishing returns for Rust implementation
- Only pursue if strict ELM adherence required

**Technical Debt**: Extract shared logic from HeadlessApp
- Both ReviewTuiApp and HeadlessApp have duplicate update logic
- Consider trait or shared module
- Not blocking ELM compliance
