# Context Handoff - Phase 2: Refactor Callbacks to Results

## What I Built & Why

Refactored `approve_decision()` and `reject_decision()` methods in ReviewEngine to return `CascadeResult` instead of using callbacks. This aligns the cascade API with ELM architecture principles while maintaining clean architectural separation.

## Problem Solved

**Original Issue** (from contribution 003 decision revisit):
- Contribution 002 used callbacks for cascade operations
- Callbacks don't fit ELM architecture (which uses pure functions returning data)
- diffviz-review-tui has zero callbacks; events flow through messages, not callbacks
- Need to align cascade API with TUI's functional programming style

**Solution Implemented**:
- Created `CascadeResult` enum describing what cascade operation affected
- Methods return `Result<CascadeResult>` instead of taking callback parameter
- TUI receives explicit information about cascade scope (decision # and chunks affected)
- Clean separation: ReviewEngine returns facts, TUI decides what to display

## Routes Explored & Decisions Made

### Why CascadeResult (not BusinessEvent)?

**Considered**: Return `BusinessEvent` variants instead of custom enum
- **Pro**: Reuse existing TUI event type
- **Con**: Couples diffviz-review (domain layer) to diffviz-review-tui (presentation layer)
- **Con**: Violates clean architecture (review shouldn't depend on UI)
- **Decision**: Create domain-neutral `CascadeResult` type
- **Rationale**: TUI layer can convert to BusinessEvent if needed, but review stays independent

### Why Enum (not Struct)?

**Considered**: Single `CascadeResult { decision: u32, chunks: usize, was_approved: bool }`
- **Pro**: Simpler data structure
- **Con**: Less type-safe (could construct invalid combinations)
- **Con**: NoChunksAffected edge case less obvious
- **Decision**: Three-variant enum (DecisionApproved, DecisionUnapproved, NoChunksAffected)
- **Rationale**: Enum exhaustiveness forces TUI to handle all cases; more explicit about outcomes

### Why New Entity File?

**Considered**: Define in review_engine.rs or state/mod.rs
- **Con**: Couples cascade result to engine implementation
- **Con**: Bloats root module if in mod.rs
- **Decision**: Separate file `entities/cascade_result.rs`
- **Rationale**: CascadeResult is a domain entity like Approval/Instruction/Decision; belongs with other entities

### Why Leave Existing Methods Unchanged?

**Considered**: Refactor `approve()`, `reject()`, `add_instruction()` too
- **Pro**: Complete consistency across all methods
- **Con**: Larger scope, more integration points to test
- **Con**: Higher risk of breaking changes
- **Decision**: Focus only on cascade methods; leave others for future contribution
- **Rationale**: Cascade methods are new (from contribution 002), lower risk to refactor; demonstrates pattern clearly

## Code Changes Made

### New File: `diffviz-review/src/entities/cascade_result.rs`

```rust
pub enum CascadeResult {
    DecisionApproved {
        decision_number: u32,
        chunks_affected: usize,
    },
    DecisionUnapproved {
        decision_number: u32,
        chunks_affected: usize,
    },
    NoChunksAffected {
        decision_number: u32,
    },
}

impl CascadeResult {
    pub fn decision_number(&self) -> u32 { /* ... */ }
    pub fn description(&self) -> String { /* ... */ }
    pub fn affected_any(&self) -> bool { /* ... */ }
}
```

**Helper Methods**:
- `description()`: Returns "Decision #1 and 3 chunks approved" format (handles pluralization)
- `decision_number()`: Extract decision number from any variant
- `affected_any()`: True unless NoChunksAffected (useful for conditional logic)

### Modified: `diffviz-review/src/engines/review_engine.rs`

**Before**:
```rust
pub fn approve_decision(
    &mut self,
    decision_number: u32,
    reviewer: String,
    on_result: OperationCallback,
) -> Result<()> {
    // ... logic ...
    if let Some(callback) = on_result {
        callback(true, Some(format!("Decision #{} and all {} affected chunks approved", ...)));
    }
    Ok(())
}
```

**After**:
```rust
pub fn approve_decision(
    &mut self,
    decision_number: u32,
    reviewer: String,
) -> Result<CascadeResult> {
    // ... logic ...
    let result = if chunks.is_empty() {
        CascadeResult::NoChunksAffected { decision_number }
    } else {
        CascadeResult::DecisionApproved {
            decision_number,
            chunks_affected: chunks.len(),
        }
    };
    Ok(result)
}
```

Similar change for `reject_decision()`.

### Updated Tests

**Before**:
```rust
let result = engine.approve_decision(1, "reviewer".to_string(), None);
assert!(result.is_ok());
// No way to verify what happened
```

**After**:
```rust
let result = engine.approve_decision(1, "reviewer".to_string());
assert!(result.is_ok());

let cascade_result = result.unwrap();
match cascade_result {
    CascadeResult::DecisionApproved { decision_number, chunks_affected } => {
        assert_eq!(decision_number, 1);
        assert_eq!(chunks_affected, 3);
    }
    _ => panic!("Expected DecisionApproved"),
}
```

Now tests validate actual cascading behavior, not just success status.

## Key Architectural Insights

1. **ELM Doesn't Have Callbacks**: TUI's pure functions return values, handlers process them
2. **Domain Layer Independence**: diffviz-review shouldn't know about TUI types
3. **Type Safety Through Enums**: Exhaustiveness checking forces correct handling
4. **Conversion at Boundaries**: TUI converts CascadeResult to its own event types
5. **Immutable Results**: Operations return facts, no side-effecting callbacks

## For Next Contributor (TUI Integration)

### When Adding TUI Event Handlers

**Pattern to follow**:
```rust
// In diffviz-review-tui event handlers
BusinessEvent::ApproveDec ision { decision_number } => {
    let cascade_result = self.review_engine.approve_decision(decision_number, author)?;
    match cascade_result {
        CascadeResult::DecisionApproved { chunks_affected, .. } => {
            self.ui_state.show_message(cascade_result.description());
        }
        CascadeResult::NoChunksAffected { .. } => {
            // Handle edge case
        }
        // ...
    }
}
```

### Key Points for Integration

1. **Cascade Result is already fully typed** - No guessing what it contains
2. **Description method ready** - Can display directly or use for logging
3. **No callback mocking needed** - Tests just assert on returned result
4. **Edge cases explicit** - `NoChunksAffected` forces you to handle it
5. **Both operations done** - approve_decision and reject_decision both use results

## Assumptions Made

1. **ELM architecture is desired** - diffviz-review-tui uses it; cascade methods should too
2. **TUI will handle CascadeResult** - Next phase owns the event handler implementation
3. **One variant per operation outcome** - Atomic success/failure (no partial cascades in MVP)
4. **Chunks_affected count useful** - TUI wants to know scope of cascade
5. **No serialization needed now** - CascadeResult doesn't need Serialize (in-memory only)

## Known Limitations

1. **Existing methods still use callbacks** - `approve()`, `reject()`, `add_instruction()` unchanged
   - Out of scope for this contribution
   - Clear pattern established for future refactoring

2. **English-only descriptions** - `description()` hardcoded English text
   - Compatible with future i18n if needed
   - Not blocking for MVP

3. **TUI integration not shown** - This contribution doesn't show how TUI uses CascadeResult
   - Next phase (TUI integration) will implement handlers
   - Contract is clear from method signatures

4. **No cascade history** - Operation results not persisted
   - Current design allows easy addition in future
   - Acceptable for MVP

## Test Results

✅ All 148 tests passing (4 decision tests + 144 existing)
✅ Zero clippy warnings
✅ Code properly formatted with rustfmt
✅ Full test coverage for CascadeResult enum

## Summary

Successfully refactored cascade methods to use result types instead of callbacks. This aligns with ELM architecture principles while maintaining clean separation between domain (diffviz-review) and presentation (diffviz-review-tui) layers. The implementation is type-safe, well-tested, and ready for TUI integration in the next phase.

Ready for: Phase 3 TUI Integration (when next contributor adds UI event handlers and keybindings)
