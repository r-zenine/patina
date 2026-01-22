# Context Handoff: Phase 3 Complete

**Phase**: 3 of 5 ✅ Complete
**Next Phase**: Phase 4 - Convert Side Effects to Commands

## What Was Done

Implemented the Command system infrastructure that separates side effects from business logic. This foundation enables the update logic to be pure and testable while still supporting necessary I/O operations.

### Key Changes

**src/command.rs** - New file (Command enum and execute_command function):
- `Command` enum with variants: `WriteFile`, `ShowMessage`, `Batch`, `None`
- `execute_command()` function that executes commands and returns `Result<()>`

**src/lib.rs** - Export command module:
- Added `pub mod command;` to module exports

**src/app.rs** - Main loop and handler signatures:
- Updated `run()` main loop to:
  - Call `handle_events()` and get Command
  - Call `execute_command()` after handling events
  - Check `should_quit` for exit condition
- Changed `handle_events()` signature: `Result<bool>` → `Result<Command>`
  - Now returns command from process_key_event
  - Returns `Command::None` if no key event
- Changed `process_key_event()` signature: `Result<()>` → `Result<Command>`
  - Captures command from handle_business_event
  - Returns command to caller
  - Returns `Command::None` for UI-only events
- Changed `handle_business_event()` signature: `Result<()>` → `Result<Command>`
  - All handlers return `Ok(Command::None)` for most events
  - ExportInstructions returns `Ok(Command::Batch(vec![...]))` for file write and message

### Status: ✅ Verified
- Compiles without errors (only pre-existing warning in main.rs)
- All 5 existing tests pass
- No breaking changes
- Ready for Phase 4

## Architecture Checkpoint

**ELM Compliance Progress**:
- ✅ V1: View functions accept mutable state → FIXED (Phase 1)
- ✅ V4: Direct field access → FIXED (Phase 2)
- ⏳ V2: Side effects in update logic → INFRASTRUCTURE READY (Phase 3)
- ⏳ V3: Time-based side effects → PENDING (Phase 5)
- ⏸️ V5: Business logic in UI layer → DEFERRED
- ⏸️ V6: Direct ReviewEngine mutations → PRAGMATIC COMPROMISE

## Why This Matters

In ELM architecture, side effects are **Commands** that describe what should happen, not instructions for how to do it. The update logic returns Commands instead of executing side effects directly.

**Before Phase 3**:
```rust
// Side effect in update logic - hard to test
fn handle_business_event(&mut self, event: BusinessEvent) -> Result<()> {
    std::fs::write(&filename, json)?;  // Direct I/O
    eprintln!("Exported...");           // Direct I/O
}
```

**After Phase 3**:
```rust
// Update logic returns command describing the effect
fn handle_business_event(&mut self, event: BusinessEvent) -> Result<Command> {
    Ok(Command::Batch(vec![
        Command::WriteFile { path: filename, content: json },
        Command::ShowMessage { message: "Exported..." },
    ]))
}

// Main loop executes commands separately
let command = handle_events()?;
execute_command(command)?;  // Side effects isolated here
```

This separation makes the update logic:
- **Testable**: Returns commands without executing them
- **Composable**: Commands can be batched or modified
- **Observable**: All side effects are explicit

## For Phase 4 Implementation

### What Phase 4 Will Do

Convert all side-effecting handlers to return Commands:

1. ExportInstructions handler already done (returns Command::Batch with WriteFile and ShowMessage)
2. Other handlers return `Command::None` for now
3. This completes the V2 fix (side effects in update logic)

### Files Phase 4 Will Modify

- `src/app.rs` - handle_business_event is already set up for Phase 4
- HeadlessApp - already updated alongside ReviewTuiApp

### Key Points for Phase 4

**Handler Return Values** (already structured correctly):
- `ToggleApprove`: `Ok(Command::None)`
- `ApproveFile`: `Ok(Command::None)`
- `AddInstruction`: `Ok(Command::None)`
- `ExportInstructions`: Already returns `Ok(Command::Batch(...))`
- Unimplemented handlers: `Ok(Command::None)`

**No further signature changes needed** - Phase 3 established all needed infrastructure.

## Integration Notes

### How Phase 3 Fits with Previous Phases

- **Phase 1** (Pure View Functions): Views can't mutate state ✅
- **Phase 2** (State Encapsulation): State mutations through methods ✅
- **Phase 3** (Command Foundation): Side effects become explicit ✅
- **Phase 4** (Convert to Commands): Full V2 implementation
- **Phase 5** (Time as Messages): V3 implementation

### Testing Considerations

- Phase 3 infrastructure enables better testing in Phase 4
- Commands can be tested without executing them
- Update logic becomes pure functions (State, Event) → (State, Command)

### No New Technical Debt

- Clean separation: update logic vs side effects
- Simple command model: WriteFile, ShowMessage, Batch, None
- No performance impact: commands are simple enums

## Known Limitations (Intentional)

**Command System Scope** (as per architecture decision D1):
- I/O operations only (file writes, notifications)
- Does NOT wrap ReviewEngine operations (pragmatic compromise)
- ReviewEngine mutations happen directly in update logic (acceptable)

This keeps the refactoring focused and manageable while still achieving ELM compliance for side effects.

## Verification Commands for Phase 4

```bash
# After Phase 4, run:
cargo check --package diffviz-review-tui
cargo test --package diffviz-review-tui

# Manual verification:
cargo run --bin review-tui
# Press Space+e+a to export - file created and message shown
```

## Files Modified Summary

**New Files**:
- `src/command.rs` (43 lines)

**Modified Files**:
- `src/lib.rs` (1 line added)
- `src/app.rs`:
  - Import command module (1 line)
  - Updated run() loop (6 lines changed)
  - Updated handle_events signature and return (6 lines changed)
  - Updated process_key_event signature (23 lines changed, both ReviewTuiApp and HeadlessApp)
  - Updated handle_business_event signature (47 lines changed, both ReviewTuiApp and HeadlessApp)

**Total**: 43 lines added to command.rs, 1 line to lib.rs, 85 lines modified in app.rs
**Tests**: All 5 existing tests pass ✅
**Status**: ✅ Ready for Phase 4

## Next Steps

1. Review Phase 3 changes (minimal, focused on infrastructure)
2. Start Phase 4: All handlers now return Commands
3. ExportInstructions already returns proper commands
4. Other handlers already return `Command::None` as needed
5. Verify compilation and tests
6. Then move to Phase 5: Time as Messages

