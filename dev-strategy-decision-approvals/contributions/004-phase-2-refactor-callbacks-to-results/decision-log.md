# Technical Decisions - Phase 2 Refactor: Callbacks to Results

## 2026-01-23 - Callback Handling Approach Decision

**Category**: API Design - Architecture Alignment

**Decision**: Replace callbacks with `CascadeResult` enum for `approve_decision()` and `reject_decision()` methods

**Selected from**: Contribution 003 analyzed four options (A: State-Based, B: Summary Return, C: Event Emission, D: Detailed Result)

**Rationale**:
- Eliminates callbacks completely (ELM principle #1: no side effect functions)
- Returns immutable data describing what happened (ELM principle #2: operations return facts)
- Maintains clean architecture: diffviz-review doesn't know about TUI or BusinessEvent
- TUI can interpret CascadeResult as needed (event, status message, visual update)
- Type-safe: compiler enforces handling all cascade outcomes

**Impact**:
- `approve_decision()`: `(decision: u32, reviewer: String, callback) -> Result<()>` becomes `(decision: u32, reviewer: String) -> Result<CascadeResult>`
- `reject_decision()`: `(decision: u32, callback) -> Result<()>` becomes `(decision: u32) -> Result<CascadeResult>`
- Simpler signatures, no callback boilerplate
- Tests now validate returned CascadeResult instead of mocking callbacks

**Evidence**: Analyzed diffviz-review-tui source code showing pure ELM implementation with zero callbacks; operations return values, TUI processes them

---

## 2026-01-23 - CascadeResult Enum Design

**Category**: Entity Design

**Decision**: Three-variant enum:
```rust
pub enum CascadeResult {
    DecisionApproved { decision_number: u32, chunks_affected: usize },
    DecisionUnapproved { decision_number: u32, chunks_affected: usize },
    NoChunksAffected { decision_number: u32 },
}
```

**Rationale**:
- `DecisionApproved`: Typical forward cascade (decision → chunks)
- `DecisionUnapproved`: Rejection cascade (opposite direction)
- `NoChunksAffected`: Edge case when decision has no mapped chunks
- Always includes `chunks_affected` count for UI feedback granularity
- Enum exhaustiveness forces TUI to handle all cases

**Alternative Rejected**:
- Single `CascadeResult { decision: u32, chunks: usize, was_approved: bool }` struct
  - Less type-safe (could construct invalid combinations)
  - `NoChunksAffected` edge case less obvious
  - Enum variants are more explicit about what happened

**Impact**:
- Compiler ensures TUI handles all cascade outcomes
- `match` statements forced to be exhaustive
- Helper methods: `decision_number()`, `description()`, `affected_any()`

---

## 2026-01-23 - Helper Method Location

**Category**: Design Pattern

**Decision**: Implement helper methods on `CascadeResult` enum:
- `decision_number() -> u32` - Extract decision involved
- `description() -> String` - Human-readable summary ("Decision #1 and 3 chunks approved")
- `affected_any() -> bool` - True unless NoChunksAffected variant

**Rationale**:
- Methods on entity are more discoverable than free functions
- TUI can call `result.description()` for status bar immediately
- `affected_any()` provides clear semantics for conditional logic
- Encapsulates formatting logic (chunk singular/plural handling)

**Implementation**:
- `description()` returns localized-ready string (no i18n yet, but compatible)
- Handles chunk pluralization: "1 chunk" vs "3 chunks"
- Format: "Decision #X and Y chunk(s) approved/unapproved"

---

## 2026-01-23 - Entity Location Decision

**Category**: Code Organization

**Decision**: Place `CascadeResult` in new file `diffviz-review/src/entities/cascade_result.rs`

**Rationale**:
- CascadeResult is a domain entity (describes review operations)
- Belongs with other entities: Approval, Instruction, Decision
- Not infrastructure or engine-specific
- Reuses existing entities module export pattern
- Enables future serialization/persistence if needed

**Alternative Rejected**:
- Define in review_engine.rs: Couples entity to engine implementation
- Define in mod.rs: Would bloat the module's root file
- Define in separate ops.rs: Unnecessary abstraction layer

**Pattern Consistency**:
- Follows existing pattern: `approval.rs`, `decision.rs`, `instruction.rs`
- Exported via `entities/mod.rs`
- Well-scoped with proper test utilities

---

## 2026-01-23 - Backward Compatibility Decision

**Category**: Refactoring Strategy

**Decision**: Leave existing `approve()`, `reject()`, `add_instruction()` methods unchanged with their callback parameters

**Rationale**:
- Focuses this contribution on cascade methods only (narrow scope)
- Cascade methods are new in contribution 002, lower risk to refactor
- Existing methods have more integration points (would require broader testing)
- `OperationCallback` type alias kept for existing methods
- Future work can standardize all methods

**Future Work**:
- Contribution 005 (or later): Refactor remaining methods to use results
- Would be mechanical: same pattern as cascade methods
- Current contribution demonstrates the pattern clearly

**Impact**:
- ReviewEngine has mixed callback/result patterns temporarily
- Each method type is consistent internally
- Migration path is clear for next refactoring

---

## 2026-01-23 - Test Strategy for CascadeResult

**Category**: Testing

**Decision**: Comprehensive test coverage:
1. Unit tests on CascadeResult (variants, helpers, formatting)
2. Integration tests validating cascade operations return correct results
3. Edge case: NoChunksAffected variant when decision has zero chunks

**Tests Added**:
- `cascade_result::tests::test_approved_result_description()`
- `cascade_result::tests::test_approved_result_single_chunk()`
- `cascade_result::tests::test_unapproved_result_description()`
- `cascade_result::tests::test_no_chunks_affected_description()`
- Updated integration tests to validate returned CascadeResult

**Coverage**:
- All three enum variants tested
- Singular/plural handling verified
- Integration tests check both result correctness and state changes

---

## Implementation Strategy

**Step 1**: Create CascadeResult entity with helper methods
**Step 2**: Update approve_decision() to return CascadeResult
**Step 3**: Update reject_decision() to return CascadeResult
**Step 4**: Remove callback parameters from both methods
**Step 5**: Update all 4 integration tests to validate results
**Step 6**: Run full test suite (all 148 tests pass)
**Step 7**: Zero clippy warnings, code formatted

**Result**: All steps completed successfully ✅

---

## Known Limitations & Future Work

1. **Existing Callback Methods**: `approve()`, `reject()`, `add_instruction()` still use callbacks
   - Not part of this contribution's scope
   - Can be refactored independently following same pattern
   - Current cascade methods demonstrate the approach

2. **Localization**: `description()` returns hardcoded English strings
   - Future: Could accept locale parameter or use i18n system
   - Current format is compatible with standard localization patterns

3. **TUI Integration Not Complete**: CascadeResult is returned but TUI doesn't handle it yet
   - Next phase (Phase 3) will add UI event handlers
   - This phase establishes the domain contract

4. **No Serialization**: CascadeResult not derived Serialize/Deserialize
   - Operations happen in-memory during review session
   - May add in future if cascade history is persisted
   - Current design allows easy addition when needed

---

## Architecture Principles Upheld

✅ **Domain Independence**: diffviz-review knows nothing about TUI or BusinessEvent
✅ **Type Safety**: Enum exhaustiveness forces handling all cases
✅ **Immutability**: Operations return data (not invoke effects)
✅ **Separation of Concerns**: Result describes what happened, TUI decides what to do
✅ **Testability**: No callbacks to mock; results are easy to assert on

## Consensus Achieved

Decision to use `CascadeResult` (not BusinessEvent) aligns with:
- ELM architecture principles (used by diffviz-review-tui)
- Clean architecture (domain layer independent of UI)
- Type safety (enum vs generic struct)
- Future extensibility (can add variants as needed)
