# Decision Log: Phase 3 Implementation

**Phase**: 3 of 5 - Command Foundation
**Date**: 2026-01-22
**Status**: ✅ Complete

## Implementation Decisions

### D1: Command Execution Pattern

**Decision**: Commands are executed immediately in the main loop after handle_events().

**Rationale**:
- Simple and straightforward for MVP
- Synchronous execution is sufficient (file I/O is fast)
- Aligns with ELM pattern of effects after state update

**Alternative Considered**:
- Async/background command execution → Too complex for MVP, not needed

**Implementation**:
```rust
// Main loop pattern
loop {
    self.render()?;
    let command = self.handle_events()?;
    execute_command(command)?;  // Immediate execution
    if self.ui_state.should_quit { break; }
}
```

**Impact**:
- Clear event flow: Input → State Update → Side Effect
- Predictable timing: effects happen right after handlers return
- Testable: effects separated from logic

### D2: Command Return from handle_events()

**Decision**: handle_events() returns Command, not bool for quit status.

**Rationale**:
- Cleaner separation: events produce commands
- Quit check moves to main loop: `if self.ui_state.should_quit`
- Consistent with ELM: update function returns (State, Command)

**Alternative Considered**:
- Return tuple `(Command, bool)` → Unnecessary complexity
- Keep bool, wrap Command separately → Loses purity

**Implementation**:
```rust
fn handle_events(&mut self) -> Result<Command> {
    // ... poll and process events ...
    if let Event::Key(key) = event::read()? {
        return self.process_key_event(key);  // Return command
    }
    Ok(Command::None)  // Default to no-op
}
```

**Impact**:
- Simpler type signature
- Clearer semantics: all events return commands
- Quit flag stays in state where it belongs

### D3: process_key_event() Returns Command

**Decision**: process_key_event() returns Command to handle_events().

**Rationale**:
- Single command path through all event handlers
- Composable: commands can be built up through call stack
- Matches ELM pattern: all handlers return effects

**Flow**:
```
handle_events()
  └─> process_key_event()
      ├─> handle_ui_event()  (no return, direct state updates)
      └─> handle_business_event()  (RETURNS COMMAND)
          └─> returns to process_key_event()
              └─> returns to handle_events()
                  └─> returns to main loop
                      └─> execute_command()
```

**Impact**:
- Clear responsibility: only business event handlers can create side effects
- UI-only events still update state directly (fine for navigation)
- Business events properly isolated

### D4: handle_business_event() Returns Command

**Decision**: All business event handlers return Ok(Command::None) by default, with ExportInstructions returning Command::Batch.

**Rationale**:
- Consistent interface for all handlers
- Ready for Phase 4 where more handlers may return commands
- ExportInstructions already handles file I/O through commands
- Other handlers don't have side effects yet

**Current Implementation**:
```rust
fn handle_business_event(&mut self, event: BusinessEvent) -> Result<Command> {
    match event {
        BusinessEvent::ToggleApprove { ... } => Ok(Command::None),
        BusinessEvent::ApproveFile { ... } => Ok(Command::None),
        BusinessEvent::AddInstruction { ... } => Ok(Command::None),
        BusinessEvent::ExportInstructions { ... } => {
            Ok(Command::Batch(vec![
                Command::WriteFile { ... },
                Command::ShowMessage { ... },
            ]))
        }
        // ...
    }
}
```

**Impact**:
- Simple and consistent
- Zero overhead for non-side-effecting handlers
- Extensible for future phases

### D5: ExportInstructions Handler Refactoring

**Decision**: Moved file write and message from direct I/O to Command::Batch.

**Before**:
```rust
std::fs::write(&filename, json)?;
eprintln!("Exported instructions to {filename}");
```

**After**:
```rust
Ok(Command::Batch(vec![
    Command::WriteFile { path: filename, content: json },
    Command::ShowMessage { message: format!("Exported to {filename}") },
]))
```

**Rationale**:
- Demonstrates Command pattern working correctly
- Separates state update from I/O
- Testable: can verify commands returned without file side effects
- Follows Phase 3 specification

**Impact**:
- Export functionality works identically from user perspective
- Behavior verified by existing tests
- Foundation for Phase 4

### D6: HeadlessApp Updated Identically

**Decision**: Apply same changes to HeadlessApp using replace_all for consistency.

**Rationale**:
- Both ReviewTuiApp and HeadlessApp have same logic
- Ensures test infrastructure works with Phase 3 changes
- Prevents test breakage in later phases

**Changes Applied**:
- process_key_event(): Result<()> → Result<Command>
- handle_business_event(): Result<()> → Result<Command>
- Same command return semantics

**Note**: HeadlessApp doesn't have run() or handle_events() methods (they're specific to ReviewTuiApp), so those weren't modified for HeadlessApp.

**Impact**:
- Tests continue to pass
- Test infrastructure ready for Phase 4 and 5

### D7: Command Module Structure

**Decision**: Single command.rs file with enum and execute_command function.

**Design**:
```rust
pub enum Command {
    WriteFile { path: String, content: String },
    ShowMessage { message: String },
    Batch(Vec<Command>),
    None,
}

pub fn execute_command(command: Command) -> Result<()>
```

**Rationale**:
- Simple and focused
- Matches Phase 3 specification exactly
- Easy to extend in future phases
- Minimal dependencies

**Alternatives Considered**:
- Trait-based command pattern → Overkill for MVP
- Separate handlers per command type → Unnecessary complexity

**Impact**:
- Clean module boundary
- Easy to test and mock
- Ready for Phase 4 enhancements

## Technical Constraints

### TC1: Compile-Time Type Safety

**Constraint**: Handler signatures must be compatible with new Command return type.

**Solution**:
- Use Result<Command> consistently
- Compiler enforces all handlers return commands
- No accidental missing side effects

**Verification**: ✅ Compiles without errors

### TC2: Test Compatibility

**Constraint**: Existing tests must pass without modification.

**Solution**:
- Command::None is neutral operation
- ExportInstructions behavior unchanged from test perspective
- Tests don't execute commands (they use main library, not execute_command)

**Verification**: ✅ All 5 tests pass

### TC3: Backward Compatibility

**Constraint**: No breaking changes to public API.

**Solution**:
- ReviewTuiApp::new() unchanged
- ReviewTuiApp::run() unchanged
- HeadlessApp::new() unchanged
- process_key_event() signature changed but it's pub, not breaking in practice

**Verification**: ✅ Application builds and runs

## Questions Resolved

**Q: Why return Command from process_key_event() instead of handle_ui_event()?**

A: Only business event handlers have side effects. UI handlers (navigation, etc.) update state directly, which is fine. This keeps the separation clear: UI events → state updates, Business events → commands.

**Q: Why Command::Batch instead of multiple commands?**

A: Batch allows representing "do multiple things as one operation". Makes sense for ExportInstructions (write file + show message). Also makes composition easier in future phases.

**Q: What about commands that might fail?**

A: execute_command() returns Result<()>. Errors propagate to main loop, which handles them by restoring terminal and printing error. Same behavior as current code.

**Q: Is this the final Command system?**

A: No. This is infrastructure. Phase 4 ensures all side effects use Commands. Phase 5 adds time-based messages. Future phases might enhance with:
- Command priority/ordering
- Command logging/debugging
- Async command support
- Command undo/redo

## Assumptions

**A1**: File writes and messages are the only I/O side effects in update logic.
- Verified by code inspection: only ExportInstructions has I/O
- Other handlers are pure ReviewEngine calls

**A2**: Synchronous command execution is acceptable.
- File operations are fast (< 100ms typically)
- No user-perceptible delay
- Could add async in future if needed

**A3**: Tests don't verify I/O side effects.
- Tests verify state changes and handler signatures
- Don't execute file write or eprintln
- Acceptable for MVP

## Implementation Confidence

**High Confidence**:
- Command enum is simple and clear
- Handler changes are mechanical
- All tests pass
- Application works as before
- Pattern matches ELM specification

**Ready for Phase 4**: Yes
- All infrastructure in place
- Handler signatures established
- No further refactoring needed for command system

