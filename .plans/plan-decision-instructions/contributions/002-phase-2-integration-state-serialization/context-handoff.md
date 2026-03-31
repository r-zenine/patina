# Phase 2 Context Handoff: DecisionInstructions in ReviewState

## What Was Done

**Commit**: `26fba67` - "feat: integrate DecisionInstructions into ReviewState (Phase 2)"

**Deliverables Completed**:
- ✅ Added `pub decision_instructions: DecisionInstructions` field to `ReviewState` struct
- ✅ Updated `ReviewState::new()` to initialize `decision_instructions: DecisionInstructions::new()`
- ✅ Updated `ReviewState::with_review_data()` with new `decision_instructions` parameter
- ✅ Added `#[allow(clippy::too_many_arguments)]` to suppress now-8-arg function warning
- ✅ 4 new serialization/integration tests added
- ✅ All 151 tests pass: `cargo test --package diffviz-review`
- ✅ No clippy warnings in diffviz-review
- ✅ Code formatted with `cargo fmt`

## Architecture Overview

`ReviewState` now carries `decision_instructions: DecisionInstructions` alongside its existing peers:

```
ReviewState
├── reviewable_diffs: BTreeMap<ReviewableDiffId, ReviewableDiff>
├── approvals: ReviewApprovals
├── instructions: ReviewInstructions
├── decisions: ReviewDecisions
├── decision_approvals: DecisionApprovals
├── decision_instructions: DecisionInstructions  ← NEW (Phase 2)
└── journey: ReviewJourney
```

The field is always initialized (never `Option<>`). `ReviewState::new()` uses `DecisionInstructions::new()`. `ReviewState::with_review_data()` accepts it as a parameter.

## Key Decisions

- **Required field, not Option**: Empty `DecisionInstructions` is valid state; no reason for Option.
- **clippy::too_many_arguments**: Suppressed with `#[allow]` on `with_review_data`. Creating a struct just for this function would be premature abstraction.

## What Phase 3 Should Focus On

### Phase 3: Integration - ReviewEngine Operations

**Objective**: Add ReviewEngine methods to manage decision instructions.

**What to Modify**:
- `diffviz-review/src/engines/review_engine.rs`: Add three methods:
  1. `add_decision_instruction(decision_number: u32, content: String, author: String) -> Result<()>`
  2. `remove_decision_instruction(instruction_id: &str) -> Result<()>`
  3. `get_decision_instructions(decision_number: u32) -> Option<Vec<&Instruction>>`

**Key constraints for Phase 3**:
- `add_decision_instruction` must validate that `decision_number` exists in `self.state.decisions.decisions` before adding; return an appropriate `ReviewError` if not found
- `remove_decision_instruction` must return `Err(ReviewError::InstructionNotFound { ... })` if the ID doesn't exist
- Need to check existing error variants: look for `ReviewError::InvalidDecision` and `ReviewError::InstructionNotFound` in `diffviz-review/src/errors.rs` — add them if they don't exist
- The `Instruction` struct requires: id (UUID via `uuid::Uuid::new_v4()`), author (caller-provided), timestamp (`chrono::Utc::now().format(...)`), content (caller-provided), status (always `InstructionStatus::Active` for new instructions)
- Check if `uuid` and `chrono` are already dependencies in `diffviz-review/Cargo.toml` — they likely are since `add_instruction` in `review_engine.rs` already uses them

**Files to Read Before Starting Phase 3**:
1. `diffviz-review/src/engines/review_engine.rs` — find `add_instruction()` method as reference implementation (~line 248)
2. `diffviz-review/src/errors.rs` — check existing `ReviewError` variants
3. `diffviz-review/Cargo.toml` — verify uuid and chrono are present

**Expected Test Results**:
- 10+ new engine tests
- All 151 existing tests still pass
- Zero clippy warnings

## Testing Evidence

**Phase 2 Test Results**:
```
cargo test --package diffviz-review
Result: 151 passed (up from 147 in Phase 1)
```

**New Tests**:
- `test_decision_instructions_field_initializes_empty` ✅
- `test_decision_instructions_survives_state_clone` ✅
- `test_decision_instructions_accessible_through_state` ✅
- `test_review_state_with_review_data_includes_decision_instructions` ✅

## Files Modified

- `diffviz-review/src/state/mod.rs` — **Modified** (+97 lines: field, constructor updates, 4 tests)

---

**Ready for Phase 3** ✅

ReviewState now carries DecisionInstructions. Phase 3 can add ReviewEngine methods to manage it.
