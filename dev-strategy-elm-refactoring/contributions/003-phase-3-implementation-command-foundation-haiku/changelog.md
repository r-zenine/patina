# Changelog: Phase 3 - Command Foundation (V2 Fix Part 1)

**Phase**: 3 of 5
**Violation Being Fixed**: V2 - Side Effects in Update Logic (Part 1 - Infrastructure)
**Date**: 2026-01-22
**Status**: ✅ Complete and Verified

## What Was Accomplished

Successfully implemented the Command system infrastructure that separates side effects from business logic. Created the foundation for making update logic pure and testable, with ExportInstructions handler already converted to return Commands.

## Files Created

**src/command.rs** (New file):
```rust
pub enum Command {
    WriteFile { path: String, content: String },
    ShowMessage { message: String },
    Batch(Vec<Command>),
    None,
}

pub fn execute_command(command: Command) -> Result<()> {
    match command {
        Command::WriteFile { path, content } => { std::fs::write(...) }
        Command::ShowMessage { message } => { eprintln!(...) }
        Command::Batch(commands) => { execute each }
        Command::None => { no-op }
    }
}
```

## Files Modified

**src/lib.rs** (1 line added):
- Added `pub mod command;` to export the command module

**src/app.rs** (Multiple signature and logic changes):

1. **Imports** (line ~19):
   - Added: `use crate::command::{execute_command, Command};`

2. **ReviewTuiApp::run()** method (lines ~74-91):
   - Changed loop pattern to:
     - Render UI
     - Get Command from handle_events()
     - Execute Command with execute_command()
     - Check should_quit flag
   - Result: Side effects now happen after state updates, not during

3. **ReviewTuiApp::handle_events()** signature (lines ~96-113):
   - Changed: `fn handle_events(&mut self) -> Result<bool>`
   - To: `fn handle_events(&mut self) -> Result<Command>`
   - Returns command from process_key_event or Command::None
   - No longer checks should_quit internally

4. **ReviewTuiApp::process_key_event()** signature (lines ~115-139):
   - Changed: `pub fn process_key_event(&mut self, key: KeyEvent) -> Result<()>`
   - To: `pub fn process_key_event(&mut self, key: KeyEvent) -> Result<Command>`
   - Captures command from handle_business_event
   - Returns command to caller
   - Returns Command::None for UI-only events

5. **ReviewTuiApp::handle_business_event()** signature and implementation (lines ~413-473):
   - Changed: `fn handle_business_event(&mut self, event: BusinessEvent) -> Result<()>`
   - To: `fn handle_business_event(&mut self, event: BusinessEvent) -> Result<Command>`

   **Handler implementations**:
   - `ToggleApprove`: Returns `Ok(Command::None)`
   - `ApproveFile`: Returns `Ok(Command::None)`
   - `AddInstruction`: Returns `Ok(Command::None)`
   - `ExportInstructions`: Returns `Ok(Command::Batch(vec![...]))`
     - File write command: `Command::WriteFile { path, content: json }`
     - Message command: `Command::ShowMessage { message: "Exported..." }`
   - Unimplemented handlers: Return `Ok(Command::None)`

6. **HeadlessApp** (lines ~534-895):
   - Updated with identical changes to ReviewTuiApp
   - `process_key_event()`: Updated to return `Result<Command>`
   - `handle_business_event()`: Updated to return `Result<Command>`
   - Test infrastructure ready for Phase 4

## Verification

**Compilation**: ✅ Passes
```
cargo check --package diffviz-review-tui
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.20s
```

**Tests**: ✅ All pass
```
test result: ok. 5 passed; 0 failed; 0 ignored

Tests passing:
- test_empty_tree_flatten
- test_navigate_empty_tree
- test_render_decision_detail_modal
- test_render_empty_decision_list
- test_tree_path_depth
```

**No Breaking Changes**: ✅
- Application behavior unchanged
- All existing tests pass
- Only internal structure improvements

## Impact Analysis

### Code Quality Improvements

**Separation of Concerns**:
- Update logic now returns descriptions of side effects (Commands)
- Side effects execution isolated to execute_command()
- Testability improved: update logic becomes pure functions

**Composability**:
- Commands can be batched with Command::Batch
- Easy to add new side effects without changing architecture
- Handler logic remains clean and focused

### Performance Impact
- **Zero impact**: No new allocations in critical path
- Commands are simple enums with string payloads
- Batch overhead negligible

### No Runtime Changes
- Application behavior identical to before
- File export still works same way
- Message display still uses eprintln
- All state updates still occur correctly

## Design Rationale

### Why Commands?

In ELM architecture, the update function is pure:
```
update : Model -> Msg -> (Model, Cmd Msg)
```

It returns both the new state AND commands describing effects. This allows:
1. **Testing**: Test update logic without executing side effects
2. **Determinism**: Same input always produces same output
3. **Composability**: Commands can be transformed, batched, filtered

Our Rust implementation follows the same pattern with Command enum.

### Why This Separation?

Before Phase 3:
```rust
// Side effects mixed with business logic
fn handle_business_event(&mut self, event: BusinessEvent) -> Result<()> {
    // ... ReviewEngine operations ...
    std::fs::write(&filename, json)?;  // Hidden side effect
    eprintln!("...");                   // Hidden side effect
}
```

After Phase 3:
```rust
// Side effects explicit and deferred
fn handle_business_event(&mut self, event: BusinessEvent) -> Result<Command> {
    // ... ReviewEngine operations ...
    Ok(Command::Batch(vec![
        Command::WriteFile { ... },
        Command::ShowMessage { ... },
    ]))
}

// Main loop handles execution
let command = handle_events()?;
execute_command(command)?;  // All side effects happen here
```

Benefits:
- Update logic is testable without I/O
- Side effects are explicit and auditable
- Easy to add command logging, filtering, batching
- Aligns with functional programming patterns

## Summary

Phase 3 successfully establishes the Command infrastructure needed to separate side effects from update logic. The main loop now:
1. Renders current state
2. Gets user input and returns Command
3. Executes Command (side effects)
4. Repeats

This is a critical ELM pattern that makes update logic pure and testable. All handlers are ready for Phase 4, where we'll ensure all side effects go through Commands.

## Known Limitations

**ReviewEngine Mutations**: Not wrapped in Commands
- Per architecture decision, ReviewEngine operations happen directly
- This is an acceptable pragmatic compromise
- Pure ELM would command these too, but it provides no benefit in Rust

**Command Execution**: Synchronous
- File writes block the event loop
- Acceptable for MVP: operations are fast
- Could add async/background execution in future if needed

## Next Phase

**Phase 4**: Convert Side Effects to Commands
- Handler signatures already support returning Commands ✅
- ExportInstructions already returns Commands ✅
- Other handlers return Command::None ✅
- Ready to proceed immediately

