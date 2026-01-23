# Technical Decisions - Phase 2: Integration with Review Engine

## 2026-01-23T16:00:00Z - Location of Decision Approvals Field

**Category**: State Organization

**Decision**: Add `decision_approvals: DecisionApprovals` directly to ReviewState struct alongside existing `approvals`, `instructions`, `decisions`

**Rationale**:
- Parallel structure to ReviewApprovals which tracks chunk approvals
- Single source of truth: ReviewState aggregates all review data
- Enables simple query methods at state level
- Consistent with existing pattern (approvals is separate collection, not merged into decisions)
- ReviewEngine can orchestrate between decisions and approvals at layer that sees both

**Impact**:
- ReviewState now has 5 major collections (reviewable_diffs, approvals, instructions, decisions, decision_approvals)
- Must be initialized in both constructors
- Simplifies ReviewEngine implementation (can call state methods directly)

**Evidence**: Context document specifies decision approvals are separate from decisions; implementation roadmap shows separate DecisionApprovals collection

---

## 2026-01-23T16:05:00Z - Progress Calculation Method Location

**Category**: Design Pattern

**Decision**: Implement `decision_approval_progress()` in ReviewState, not ReviewEngine

**Rationale**:
- Pure query method that doesn't mutate state, fits state layer responsibility
- ReviewEngine will call it when cascading, avoiding duplication
- TUI layer will call it for rendering progress indicators
- Consistent pattern: state provides queries, engine provides orchestration
- Query can use internal fields (decision_index, approvals) without exposing them

**Impact**:
- ReviewState becomes a more sophisticated query engine
- ReviewEngine remains focused on orchestration
- Single calculation logic prevents divergence

**Implementation**:
```rust
pub fn decision_approval_progress(&self, decision_number: u32) -> (usize, usize) {
    // Find all chunks for this decision via decision_index
    // Count how many are approved via approvals collection
}
```

---

## 2026-01-23T16:10:00Z - Reverse Cascade Trigger Conditions

**Category**: Business Logic

**Decision**: Reverse cascade only triggers when ALL chunks for a decision are approved/unapproved

**Rationale**:
- User expectation: decision only auto-completes when truly complete
- Prevents spurious state changes during partial reviews
- Clear, predictable behavior: if 10/10 chunks approved → decision auto-approves
- If any chunk is unapproved, decision unapproves (by symmetry)
- Aligns with user requirements from context document

**Implementation Strategy**:
- After `approve(chunk)`: check if approved count == total chunks for each decision
- After `reject(chunk)`: check if decision is approved AND approved count < total chunks

**Edge Cases Handled**:
- Decision with 0 chunks (e.g., unmapped decision) - won't auto-approve
- Decision with 1 chunk - approves/unapproves cleanly
- Multi-chunk decisions - works correctly

---

## 2026-01-23T16:15:00Z - Borrow Checker Strategy for Reverse Cascade

**Category**: Implementation Detail

**Decision**: Collect decision IDs before checking approval progress to avoid multiple borrows

**Rationale**:
- Rust borrow checker forbids holding reference to decision_index while calling mutable methods
- Solution: collect Vec<u32> of affected decisions FIRST
- Then iterate and call state methods (which may mutate approvals/decision_approvals)
- No performance impact: typically 1-3 decisions per chunk

**Pattern**:
```rust
let decisions_for_chunk = self.state.decisions.decision_index
    .get(&reviewable_id)
    .cloned()
    .unwrap_or_default();

for decision_num in decisions_for_chunk {
    // Now safe to call mutable methods
    self.state.approve_decision(decision_num, ...);
}
```

---

## 2026-01-23T16:20:00Z - Helper Method Visibility

**Category**: API Design

**Decision**: `get_chunks_for_decision()` is private helper in ReviewEngine

**Rationale**:
- Used only internally by `approve_decision()` and `reject_decision()`
- Encapsulates implementation detail
- TUI doesn't need direct access (can use query methods instead)
- Simpler public API surface

**Pattern**:
```rust
fn get_chunks_for_decision(&self, decision_number: u32) -> Vec<ReviewableDiffId> {
    // Private helper
}

pub fn approve_decision(&mut self, decision_number: u32, ...) -> Result<()> {
    let chunks = self.get_chunks_for_decision(decision_number);
    // ...
}
```

---

## 2026-01-23T16:25:00Z - Test Strategy for Cascading

**Category**: Testing

**Decision**: Comprehensive integration tests covering all cascading scenarios

**Tests Added**:
1. `test_decision_approval_cascades_to_chunks` - Forward cascade
2. `test_decision_rejection_cascades_to_chunks` - Reverse cascade
3. `test_reverse_cascade_all_chunks_approved` - Auto-approve when all chunks approved
4. `test_reverse_cascade_reject_one_chunk_unapproves_decision` - Auto-unapprove when any chunk rejected
5. `test_decision_progress_partial_approval` - Progress tracking during partial approval
6. `test_multiple_decisions_independent` - Multiple decisions don't interfere
7. `test_decision_progress_zero_chunks` - Edge case: no chunks

**Test Fixtures**:
- Created `create_engine_with_decision_and_chunks()` helper
- Sets up 3 chunks in same file, 1 decision affecting all
- Enables rapid test iteration without code duplication

**Coverage**:
- Happy path: approve/reject cascade works
- Reverse cascade: auto-triggers at right condition
- Partial states: progress calculated correctly
- Multiple decisions: remain independent
- Edge cases: handled gracefully

---

## 2026-01-23T16:30:00Z - Callback Handling in Cascade Methods

**Category**: API Design

**Decision**: `approve_decision()` and `reject_decision()` include callback for UI feedback

**Rationale**:
- Consistent with existing `approve()` and `reject()` signatures
- Allows UI to show "approved X chunks" feedback
- TUI can chain callbacks or ignore (pass None)
- Type signature: `Option<Box<dyn FnOnce(bool, Option<String>) + Send>>`

**Example Usage**:
```rust
engine.approve_decision(1, "reviewer".to_string(), Some(Box::new(|success, msg| {
    println!("{:?}: {}", success, msg);
})))?;
// Prints: "true: Decision #1 and all 3 affected chunks approved"
```

---

## 2026-01-23T16:35:00Z - Constructor Signature for with_review_data

**Category**: API Design

**Decision**: Add `decision_approvals` parameter to `ReviewState::with_review_data()`

**Rationale**:
- Maintains symmetry with `new()` constructor
- Enables loading previously saved review sessions with decision approval state
- No calls exist to this method yet, so signature is non-breaking
- Required for persistence/serialization support

**Signature**:
```rust
pub fn with_review_data(
    reviewable_diffs: Vec<ReviewableDiff>,
    author: String,
    journey: ReviewJourney,
    approvals: ReviewApprovals,
    instructions: ReviewInstructions,
    decisions: ReviewDecisions,
    decision_approvals: DecisionApprovals,  // NEW
) -> Self
```

---

## 2026-01-23T16:40:00Z - Ordering of Cascading Operations

**Category**: Business Logic

**Decision**: In `approve_decision()`: approve decision FIRST, then chunks (allows state consistency checks)

**Rationale**:
- Decision approved before cascade starts
- If cascade partially fails, decision is in consistent state
- Easier to debug: clear sequencing
- No functional difference (all operations succeed or fail atomically in tests)

**Pattern**:
```rust
pub fn approve_decision(...) -> Result<()> {
    // 1. Approve the decision
    self.state.approve_decision(decision_number, ...);

    // 2. Get chunks
    let chunks = self.get_chunks_for_decision(decision_number);

    // 3. Approve each chunk
    for chunk_id in chunks {
        self.state.approve(chunk_id, ...);
        self.renderable_cache.remove(&chunk_id);
    }
}
```

---

## 2026-01-23T16:45:00Z - Clippy Warning Fix

**Category**: Code Quality

**Decision**: Use `.cloned()` instead of `.map(|x| x.clone())`

**Rationale**:
- Idiomatic Rust
- Clippy explicitly recommends this
- More concise
- Same performance (compiler optimizes identical)

**Pattern**:
```rust
// Before:
.get(&id).map(|nums| nums.clone()).unwrap_or_default()

// After:
.get(&id).cloned().unwrap_or_default()
```

---

## Implementation Assumptions

1. **Decision index is always built** - Assumes `build_index_from_review_state()` called before approval operations
2. **All chunks are indexed** - Assumes every ReviewableDiff is mapped to at least Decision 0 (unmapped decision)
3. **No concurrent access** - Assumes single-threaded use during review session
4. **Author context available** - ReviewEngine always has author when approving decisions

---

## Known Limitations & Future Work

1. **No approval history** - Only current state stored (could add history in future)
2. **No conflict detection** - No warning if decision approval conflicts with chunk approvals (Phase 2 handles gracefully)
3. **No transaction support** - Partial failures aren't rolled back (acceptable for MVP)
4. **No performance optimization** - Decision progress recalculated on each query (acceptable for typical decision/chunk counts)
