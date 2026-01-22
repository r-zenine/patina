# Decision Log: ELM Architecture Refactoring

## Strategy Selection

**Chosen Strategy**: Steel Thread with Built-in Refactor Steps

**Rationale**:
- Need to maintain working application throughout refactoring
- Multiple violations with dependencies between them
- Want to validate each change before proceeding
- Built-in refactor prevents accumulating technical debt

**Alternative Considered**: TDD
- Rejected because we're refactoring existing code, not adding features
- No new behavior to test-drive
- Existing tests should continue passing

## Key Architectural Decisions

### D1: Command System Scope

**Decision**: Implement Command system for I/O side effects only (file writes, notifications), NOT for ReviewEngine operations.

**Rationale**:
- V2 (side effects in update) is the critical violation needing Commands
- V6 (ReviewEngine mutations) is noted as "acceptable pragmatic compromise"
- ReviewEngine operations are synchronous and don't benefit from Command abstraction in Rust
- Keeps refactoring focused and manageable

**Alternatives Considered**:
- Full Command system including ReviewEngine → Too complex, diminishing returns
- No Command system → Doesn't fix V2, keeps side effects in update logic

**Impact**: Commands will be limited to:
```rust
pub enum Command {
    WriteFile { path: String, content: String },
    ShowMessage { message: String },
    None,
}
```

### D2: View Function Signatures

**Decision**: Change all view functions to use `&UiState` instead of `&mut UiState`.

**Rationale**:
- Easy fix with high architectural value
- Views never actually mutate state in current code
- Makes immutability contract explicit
- No behavioral changes needed

**Verification Strategy**:
- Compile-time check ensures no mutations
- If compilation succeeds, refactoring is correct

### D3: State Encapsulation Strategy

**Decision**: Add dedicated methods to UiState for all nested state operations, make DecisionNavigationTree fields private where possible.

**Rationale**:
- Fixes V4 (direct field access)
- Provides single update path for all state changes
- Enables future refactoring of internal representation
- Common Rust pattern (builder-like interface)

**Scope**:
- Add methods: `navigate_to_first_in_tree()`, `navigate_to_last_in_tree()`, `close_modal_if_open()`
- Keep `decision_tree.navigate_next()`, `navigate_prev()` as-is (already encapsulated)
- Don't make fields private yet (breaks too much, defer to future refactoring)

### D4: Time-Based Behavior

**Decision**: Model leader timeout as a `UiEvent::LeaderTimeout` message.

**Rationale**:
- Fixes V3 (time-based side effects)
- Consistent with message-based architecture
- Easy to implement and test

**Implementation**:
```rust
// In handle_events():
if ui_state.leader_active && ui_state.is_leader_timed_out() {
    self.handle_ui_event(&UiEvent::LeaderTimeout)?;
}
```

### D5: Tree Building Logic

**Decision**: Defer moving tree building logic to business layer (V5) to a separate follow-up refactoring.

**Rationale**:
- Not critical to ELM compliance
- Requires changes to diffviz-review crate
- Can be done independently after core ELM fixes
- Focus current work on architectural pattern violations

**Future Work**: Track as separate issue/task.

### D6: HeadlessApp Duplication

**Decision**: Keep duplicated logic in HeadlessApp for now, document as technical debt.

**Rationale**:
- HeadlessApp is behind test-harness feature flag
- Extracting shared logic requires careful trait/module design
- Risk of breaking test infrastructure
- Not blocking ELM compliance

**Future Work**: Extract shared update logic to trait or module.

## Technical Constraints

### TC1: Error Handling in Command Execution

**Constraint**: Command execution can fail (file I/O errors).

**Decision**: `execute_command()` returns `Result<()>`, errors propagate to main loop.

**Rationale**:
- Preserves current error handling behavior
- Failures still visible to user (via terminal restoration and error message)
- Could add error commands later if needed

### TC2: Testing Without Breaking Changes

**Constraint**: Must maintain existing test infrastructure.

**Decision**: Update HeadlessApp alongside ReviewTuiApp with same changes.

**Implementation**:
- Apply same Command system to HeadlessApp
- Update test assertions if needed (unlikely)
- Verify tests still pass at each phase

### TC3: Incremental Compilation

**Constraint**: Want to verify changes compile at each step.

**Decision**: Each phase produces compilable, runnable code.

**Verification**:
```bash
cargo check --package diffviz-review-tui
cargo test --package diffviz-review-tui
```

## Open Questions

None - all decisions made based on audit requirements and codebase constraints.

## Assumptions

**A1**: View functions don't actually mutate UiState in current code.
- Verified by inspection
- Will be confirmed by compilation after signature changes

**A2**: Command execution can be synchronous.
- File writes are fast enough not to block UI
- No need for async/background execution

**A3**: Existing tests cover critical behavior.
- Tests should continue passing after refactoring
- Manual testing will catch any UI regressions

**A4**: ReviewEngine mutations are acceptable in update logic.
- Audit explicitly notes this as "pragmatic compromise"
- Not addressing V6 in this refactoring
