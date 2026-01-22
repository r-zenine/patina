# Dev Strategy: ELM Architecture Refactoring

**Status**: Ready for Implementation
**Created**: 2026-01-22
**Crate**: diffviz-review-tui
**Strategy**: Steel Thread with Refactor Steps

## Quick Start

This strategy refactors `diffviz-review-tui` to comply with pure ELM (Elm Architecture) patterns by fixing identified violations while maintaining a working application at every step.

**Read these files in order**:
1. **context.md** - What we're building and why
2. **code_context.md** - Current code structure and violations
3. **decisions.md** - Key architectural decisions
4. **roadmap.md** - Step-by-step implementation plan

## What's Being Fixed

### Critical Violations (Blocking ELM Compliance)
- ✅ **V1**: View functions accept mutable state → Change to immutable references
- ✅ **V2**: Side effects in update logic → Introduce Command system
- ✅ **V6**: Direct ReviewEngine mutations → Accepted as pragmatic compromise

### Moderate Violations (Important but not blocking)
- ✅ **V3**: Time-based side effects → Model as messages
- ✅ **V4**: Direct field access → Encapsulate through methods
- ⏸️ **V5**: Business logic in UI layer → Deferred to future work

## Implementation Phases

**Phase 1**: Pure View Functions (V1)
- Change all view signatures to use `&UiState`
- Compiler-enforced immutability

**Phase 2**: Encapsulate State (V4)
- Add UiState methods for nested state operations
- Eliminate direct field access

**Phase 3**: Command Foundation (V2 Part 1)
- Create Command enum and execution infrastructure
- Wire into main loop

**Phase 4**: Convert Side Effects (V2 Part 2)
- Refactor handlers to return Commands
- Isolate I/O to command execution

**Phase 5**: Time as Messages (V3)
- Add LeaderTimeout event
- Handle timeout through event system

## Key Design Decisions

**Command Scope**: I/O operations only (file writes, notifications)
- NOT wrapping ReviewEngine operations (pragmatic compromise)

**State Encapsulation**: Add methods, but keep fields public for now
- Defer full encapsulation to future refactoring
- Focus on ELM pattern compliance first

**Tree Building**: Deferred to separate task
- Requires cross-crate changes
- Not critical to ELM compliance

## Success Criteria

After completing all phases:
- ✅ All view functions use immutable state references
- ✅ Update logic returns Commands instead of executing side effects
- ✅ Time-based behavior modeled as messages
- ✅ State updates go through dedicated methods
- ✅ Existing tests pass
- ✅ Application behavior unchanged from user perspective

## Verification Commands

```bash
# Compile check after each phase
cargo check --package diffviz-review-tui

# Run application
cargo run --bin review-tui

# Run tests
cargo test --package diffviz-review-tui

# Full workspace build
cargo build --workspace
```

## File Changes Summary

**New Files**:
- `src/command.rs` - Command enum and execution logic

**Modified Files**:
- `src/app.rs` - Main loop, event handling, command execution
- `src/state.rs` - Encapsulation methods
- `src/ui/mod.rs` - View signature
- `src/ui/components/*.rs` - All component signatures
- `src/events/input.rs` - LeaderTimeout event
- `src/lib.rs` - Module exports

**Files Analyzed** (no changes):
- `src/events/business.rs` - Already well-structured
- `src/decision_navigation.rs` - Deferred improvements

## Dependencies

**Crates**:
- `ratatui` - Terminal UI rendering
- `crossterm` - Terminal control and events
- `diffviz-review` - Business logic (ReviewEngine)

**Features**:
- `test-harness` - Enables HeadlessApp for testing

## References

**Audit Document**: `diffviz-review-tui/ARCHITECTURE_AUDIT.md`
**Onboarding**: `diffviz-review-tui/onboarding.md`
**Workspace Root**: `/Users/ryad/workspace/patina`

## Next Steps

1. Review all strategy documents
2. Start with Phase 1 (Pure View Functions)
3. Execute phases sequentially
4. Verify application works after each phase
5. Run full test suite after Phase 5

## Notes

**HeadlessApp Duplication**: Test infrastructure has duplicated update logic from ReviewTuiApp. This is known technical debt but doesn't block ELM compliance. Both need same refactoring applied.

**Tree Building**: Violation V5 (business logic in UI layer) is deferred because it requires changes to the diffviz-review crate and can be addressed independently.

**ReviewEngine Mutations**: Violation V6 (direct mutations in update logic) is accepted as a pragmatic compromise for Rust implementation. Pure ELM would use Commands for all side effects, but synchronous ReviewEngine operations don't benefit from this abstraction.
