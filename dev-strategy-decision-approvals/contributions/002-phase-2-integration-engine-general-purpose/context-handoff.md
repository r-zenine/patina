# Context Handoff - Phase 2: Integration with Review Engine

## What I Built & Why

Built the orchestration layer that wires decision approvals into the review engine, implementing cascading logic that connects decision-level approvals to chunk-level approvals. This is the critical piece that makes the approval system work end-to-end.

**Why Phase 2 focused on orchestration:**
- Phase 1 created pure entities (DecisionApproval, DecisionApprovals) that are data-only
- Phase 2 adds business logic: how decisions and chunks interact
- Phase 3 will add UI (TUI layer just renders this business logic)
- This layering ensures changes to approval logic stay in the engine, not scattered across TUI

## Architecture Overview

```
ReviewState (state/mod.rs)
├── reviewable_diffs: BTreeMap<ReviewableDiffId, ReviewableDiff>
├── approvals: ReviewApprovals (chunk approvals)
├── instructions: ReviewInstructions
├── decisions: ReviewDecisions
│   └── decision_index: HashMap<ReviewableDiffId, Vec<u32>>
└── decision_approvals: DecisionApprovals (ADDED IN PHASE 2)
    └── approvals: HashMap<u32, DecisionApproval>

ReviewEngine (engines/review_engine.rs)
├── state: ReviewState
├── renderable_cache: HashMap<ReviewableDiffId, String>
└── Cascading Methods (ADDED IN PHASE 2):
    ├── approve_decision(number) -> cascades to chunks
    ├── reject_decision(number) -> cascades rejection
    ├── Modified approve() -> checks for reverse cascade
    └── Modified reject() -> checks for reverse cascade
```

## Critical Integration Points

### 1. State-Level Query Methods

**Location**: `diffviz-review/src/state/mod.rs`

```rust
pub fn is_decision_approved(&self, decision_number: u32) -> bool {
    self.decision_approvals.is_approved(decision_number)
}

pub fn decision_approval_progress(&self, decision_number: u32) -> (usize, usize) {
    // Find all chunks for this decision via decision_index
    // Count how many are approved via approvals
}
```

These are called by:
- ReviewEngine when cascading
- TUI when rendering progress
- Tests validating behavior

### 2. State-Level Mutation Methods

**Location**: `diffviz-review/src/state/mod.rs`

```rust
pub fn approve_decision(&mut self, decision_number: u32, reviewer: String) -> &mut Self {
    let timestamp = chrono::Utc::now().format(...).to_string();
    self.decision_approvals.approve(decision_number, reviewer, timestamp);
    self  // Builder pattern
}

pub fn unapprove_decision(&mut self, decision_number: u32) -> &mut Self {
    self.decision_approvals.unapprove(decision_number);
    self
}
```

These are called by ReviewEngine when executing cascading operations.

### 3. Engine-Level Orchestration

**Location**: `diffviz-review/src/engines/review_engine.rs`

```rust
pub fn approve_decision(
    &mut self,
    decision_number: u32,
    reviewer: String,
    on_result: OperationCallback,
) -> Result<()> {
    // 1. Approve the decision
    self.state.approve_decision(decision_number, reviewer.clone());

    // 2. Get all chunks for this decision
    let chunks = self.get_chunks_for_decision(decision_number);

    // 3. Approve each chunk (cascading)
    for chunk_id in chunks {
        self.state.approve(chunk_id.clone(), reviewer.clone());
        self.renderable_cache.remove(&chunk_id);
    }

    // 4. Callback for UI feedback
    if let Some(callback) = on_result {
        callback(true, Some(format!("Decision #{} approved with {} chunks", decision_number, chunks.len())));
    }
    Ok(())
}
```

### 4. Reverse Cascade Logic

**Location**: `diffviz-review/src/engines/review_engine.rs` in `approve()` and `reject()` methods

Modified `approve()` to check after approving a chunk:
```rust
// After approving a chunk, check if this auto-completes any decisions
let decisions_for_chunk = self.state.decisions.decision_index.get(&reviewable_id).cloned().unwrap_or_default();

for decision_num in decisions_for_chunk {
    let (approved, total) = self.state.decision_approval_progress(decision_num);

    // If ALL chunks for this decision are now approved, auto-approve the decision
    if total > 0 && approved == total && !self.state.is_decision_approved(decision_num) {
        self.state.approve_decision(decision_num, reviewer.clone());
    }
}
```

## Routes Explored & Discarded

### ❌ Store Approved Chunks in DecisionApproval

**Why Considered**: Could pre-calculate which chunks are approved per decision

**Why Discarded**:
- Redundant data: decision_index already maps chunks to decisions
- Would need cache invalidation on every chunk approval
- YAGNI: on-demand calculation is simple and correct
- Violates "entities are pure data structures" principle

**Evidence**: Context document explicitly states "calculate on-demand, don't cache"

### ❌ Cascading in State Instead of Engine

**Why Considered**: Could hide cascading in state.approve_decision()

**Why Discarded**:
- State layer should be pure data + queries
- Cascading is orchestration, belongs in engine
- State doesn't have access to chunk IDs (needs engine to find them)
- Violates separation of concerns
- Cache invalidation only possible in engine (has access to renderable_cache)

### ❌ Auto-Approve When ANY Chunk is Approved

**Why Considered**: Could be more permissive

**Why Discarded**:
- Doesn't match user expectation (decision not fully approved until all chunks are)
- Creates confusing state (decision "partially approved")
- TUI would need to show partial progress anyway
- User requirements explicitly state "all chunks must be approved"

**Evidence**: Context document specifies "auto-complete when all chunks approved"

### ✅ Chosen Approach: Clean Orchestration

- Engine orchestrates cascading
- State provides queries and basic mutations
- Decision_index enables efficient chunk lookup
- Reverse cascade implemented in modify methods (approve/reject)
- Comprehensive tests validate all scenarios

## For Next Contributor (Phase 3: TUI)

### What's Ready for TUI Layer

All business logic is implemented and tested. TUI only needs to:

1. **Add UI events**: Create `ToggleApproveDecision(u32)` business event
2. **Add UI handler**: Call `engine.approve_decision()` or `engine.reject_decision()`
3. **Add UI queries**: Call `engine.is_decision_approved()` and `engine.decision_approval_progress()`
4. **Add keybinding**: Space+a+d when at decision level
5. **Add visual rendering**: Show approval icon + progress "(X/Y)"

**Methods available from ReviewEngine**:
```rust
pub fn approve_decision(&mut self, decision_number: u32, reviewer: String, on_result: OperationCallback) -> Result<()>
pub fn reject_decision(&mut self, decision_number: u32, on_result: OperationCallback) -> Result<()>
pub fn is_decision_approved(&self, decision_number: u32) -> bool
pub fn decision_approval_progress(&self, decision_number: u32) -> (usize, usize)
```

All engine methods follow existing patterns from chunk approval layer.

### Using the diffviz-review-tui-contribution Skill

When implementing Phase 3:

```
Use the skill to:
- Add ToggleApproveDecision event to BusinessEvent enum
- Handle it in app.rs with engine method calls
- Add decision_approval_progress() query calls for rendering
- Add keybinding in which_key.rs
- Write TUI tests validating keyboard interaction and rendering
```

The skill knows the TUI patterns and will guide consistent implementation.

### Test Strategy for Phase 3

From TUI perspective, you should test:
1. **Keyboard interaction**: Navigate to decision, press Space+a+d, decision approves
2. **Visual updates**: Icon changes, progress count updates
3. **Cascading visible**: Approving decision shows all chunks as approved in tree
4. **Progress accuracy**: Count matches actual chunk approvals

All underlying logic is tested in Phase 2 (integration tests verify cascading works).

## Performance Considerations

- **Cascading**: O(chunks_per_decision) when approving decision
- **Reverse cascade**: O(1) per chunk, checks 1-3 decisions typically
- **Progress calculation**: O(chunks_per_decision) on-demand
- **All acceptable** for typical 10-50 decision, 100-500 chunk review sessions

## Test Results Summary

```
Compilation: ✅ Zero warnings
Clippy: ✅ Clean
Tests: ✅ 144 passed (7 new decision approval tests)

New Tests:
- test_decision_approval_cascades_to_chunks
- test_decision_rejection_cascades_to_chunks
- test_reverse_cascade_all_chunks_approved
- test_reverse_cascade_reject_one_chunk_unapproves_decision
- test_decision_progress_partial_approval
- test_multiple_decisions_independent
- test_decision_progress_zero_chunks
```

All tests comprehensive and cover:
- Happy paths
- Edge cases
- Partial states
- Multi-decision scenarios
- Zero-chunk scenarios

## Files Modified

**diffviz-review/src/state/mod.rs**:
- Added `decision_approvals: DecisionApprovals` field
- Added `is_decision_approved()` query method
- Added `decision_approval_progress()` query method
- Added `approve_decision()` mutation method
- Added `unapprove_decision()` mutation method
- Updated `new()` and `with_review_data()` constructors
- Total: ~80 lines added

**diffviz-review/src/engines/review_engine.rs**:
- Added `get_chunks_for_decision()` private helper
- Added `approve_decision()` public method
- Added `reject_decision()` public method
- Added `is_decision_approved()` public method
- Added `decision_approval_progress()` public method
- Modified `approve()` method for reverse cascade
- Modified `reject()` method for reverse cascade
- Added 7 comprehensive integration tests
- Total: ~370 lines added

## Code Quality

- ✅ All tests passing
- ✅ Zero compiler warnings
- ✅ Zero clippy warnings
- ✅ Formatted with rustfmt
- ✅ Follows existing architectural patterns
- ✅ No backwards compatibility concerns (additive change)

## Ready For

✅ Phase 3: TUI Integration (use diffviz-review-tui-contribution skill)
✅ Decision approval feature is feature-complete from business logic perspective
✅ All building blocks in place for TUI to build upon
