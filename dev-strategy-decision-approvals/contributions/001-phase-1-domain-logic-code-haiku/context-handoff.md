# Context Handoff - Phase 1: Core Domain Logic

## What I Built & Why

Created the `DecisionApproval` entity and `DecisionApprovals` collection in `diffviz-review/src/entities/decision.rs`, following the exact pattern established by the existing `Approval`/`ReviewApprovals` system.

**Why Phase 1 was minimal and focused:**
- Core-then-Integrate strategy requires building testable domain logic with zero external dependencies first
- DecisionApproval is pure data structure with no git, TUI, or orchestration complexity
- This foundation enables Phase 2 (ReviewEngine integration) and Phase 3 (TUI) to build cleanly on top

## Routes Explored & Discarded

### ❌ Complex State Machine for Decisions
**Why Considered**: Could track cascading state automatically, prevent invalid transitions
**Why Discarded**:
- Over-engineering for a simple boolean approval state
- Cascading logic belongs in ReviewEngine (orchestrator), not entities
- YAGNI principle: don't add complexity entities don't need
- Would violate "entities are pure data structures" architectural pattern

**Evidence**: Onboarding and context document explicitly state "cascading logic in ReviewEngine, not entities"

### ❌ Store Approval with Decision Number Range Mapping
**Why Considered**: Could store which chunks were approved per decision
**Why Discarded**:
- Redundant: decision_index already maps chunks to decisions
- ReviewEngine can query decision_index + chunk approvals to calculate progress
- YAGNI: calculate on-demand, don't store derived state
- Violates Single Responsibility: Decision is about architecture, approval tracking is separate

### ❌ Cache Approval Progress in DecisionApprovals
**Why Considered**: Could store total_approved/total_chunks to speed up UI queries
**Why Discarded**:
- Would require invalidation on every chunk approval
- Design rule states: "calculate on-demand, don't cache"
- YAGNI: progress calculation is O(chunks_per_decision) which is small
- Cache invalidation is harder than query calculation

### ✅ Chosen Approach: Mirror Approval Pattern
- Simple HashMap<u32, DecisionApproval> collection
- Standard methods: approve/unapprove/is_approved/get_approval
- Utility methods: total_approved/approval_percentage
- Pure data structures with no orchestration logic
- Comprehensive unit test coverage for behavior validation

## Architecture Decisions Made

### 1. Primary Key: decision_number (u32)

Decisions are identified by number, not code location. DecisionApprovals stores state per decision:
```rust
pub approvals: HashMap<u32, DecisionApproval>
```

This differs from ReviewApprovals which uses ReviewableDiffId:
```rust
pub approvals: HashMap<ReviewableDiffId, Approval>
```

**Justification**: Approvals track approval state per decision (a navigation construct), not per code change location.

### 2. Separation from ReviewDecisions

DecisionApprovals is separate from ReviewDecisions (which stores Decision data and the decision_index):
- ReviewDecisions: "What are the architectural decisions and which code do they affect?"
- DecisionApprovals: "Which decisions have been approved?"

Both will live in ReviewState, but remain independent. ReviewEngine will orchestrate between them.

### 3. Approve/Unapprove Semantics

- `approve()` always succeeds, overwrites previous approval if exists
- `unapprove()` removes approval completely (no record it was ever approved)
- `is_approved()` returns false if not in collection OR if collected but not approved

**Why not track approval history?**: Out of scope for MVP, can be added later if needed.

### 4. Calculate Progress On-Demand

Methods provided for Phase 2 integration:
- `total_approved()` - count approved decisions
- `approval_percentage(total)` - calculate percentage

Progress per decision (approved_chunks/total_chunks) will be calculated by ReviewEngine using:
- `decision_index` from ReviewDecisions (which chunks affect this decision)
- `approvals` from ReviewApprovals (which chunks are approved)

No cached progress in DecisionApprovals.

### 5. Serialization Support

Both DecisionApproval and DecisionApprovals have `#[derive(Serialize, Deserialize)]`:
- Required for ReviewState persistence
- ReviewEngine exports/imports JSON with approval state
- Same format as existing Approval types

## For Next Contributor (Phase 2: Integration)

### Critical Integration Points

1. **Add to ReviewState** (diffviz-review/src/state/mod.rs):
   ```rust
   pub decision_approvals: DecisionApprovals,
   ```
   - Initialize in `ReviewState::new()`
   - Add query methods: `is_decision_approved()`, `decision_approval_progress()`
   - Add mutation methods: `approve_decision()`, `unapprove_decision()`

2. **Add to ReviewEngine** (diffviz-review/src/engines/review_engine.rs):
   - `approve_decision(&mut self, number: u32, author: String) -> Result<()>`
     - Call `state.approve_decision()`
     - Get chunks via `decision_index.get(&decision_number)`
     - Loop: call `approve()` on each chunk
     - Invalidate cache
   - `reject_decision(&mut self, number: u32) -> Result<()>`
     - Call `state.unapprove_decision()`
     - Loop: call `reject()` on each chunk
     - Invalidate cache

3. **Reverse Cascade Logic**:
   - Modify existing `approve()` method in ReviewEngine
   - After approving chunk, check if ALL chunks for each decision are now approved
   - If yes, auto-approve those decisions via `state.approve_decision()`
   - Do similar in `reject()`: if decision was approved and any chunk is rejected, unapprove decision

4. **Progress Calculation** (ReviewState):
   - `decision_approval_progress(decision_number) -> (approved, total)`
   - Use `decision_index.get(decision_number)` to find chunks
   - Count how many are approved via `approvals.is_approved()`

### Test Strategy for Phase 2

Unit tests should validate:
- Cascading approval: approve decision → all chunks approved
- Cascading rejection: reject decision → all chunks unapproved
- Reverse cascade: approve all chunks → decision auto-approved
- Partial approval: approve some chunks → decision not auto-approved
- Progress calculation: correct (X, Y) at various states

See: `diffviz-review/tests/decision_approval_integration.rs` (to be created)

### Assumptions I Made

1. **Decision numbers are stable**: u32 won't change for a decision during review
2. **Decisions always exist before approvals**: ReviewEngine creates decisions before allowing approval
3. **All chunks are mappable to decisions**: At least Decision 0 exists for unmapped code
4. **Author string is provided**: ReviewEngine always has author context when approving

### Known Limitations

1. **No approval history**: Only current approval state is stored (can add history in future if needed)
2. **No conflict tracking**: No warning if decision approval conflicts with chunk approvals (Phase 2 should handle gracefully)
3. **No audit trail**: Timestamps only record current approval (could enhance with timestamp history)

## For Future Implementation Phases

### Phase 2 Notes
- Entity is ready, just needs ReviewState/ReviewEngine wiring
- Cascading logic is straightforward: iterate decision_index results and call chunk methods
- No new entity needed for Phase 2, just orchestration methods

### Phase 3 (TUI) Notes
- DecisionApprovals is already serializable (Serde)
- ReviewEngine will have is_decision_approved() and decision_approval_progress()
- TUI can directly query these methods
- Use existing approval icons (✓/○) for visual consistency
- No new data structure needed, just UI events and rendering

### Performance Considerations
- Cascading: O(chunks_per_decision) per decision approval
- Reverse cascade: O(decisions_affecting_chunk) which is typically 1-3
- Progress: O(chunks_per_decision) calculated on-demand
- All operations acceptable for typical decision/chunk counts

## Test Results

```
running 137 tests
...
test entities::decision::tests::test_decision_approval_lifecycle ... ok
test entities::decision::tests::test_decision_approval_percentage ... ok
test entities::decision::tests::test_decision_approval_serialization ... ok
test entities::decision::tests::test_decision_approval_multiple_approvals ... ok
test entities::decision::tests::test_decision_approval_edge_cases ... ok
...
test result: ok. 137 passed; 0 failed; 0 ignored
```

All tests passing, zero warnings, code formatted.

## Code Quality

- ✅ Clippy: Clean (zero warnings)
- ✅ Formatter: Applied and verified
- ✅ Tests: 137/137 passing
- ✅ Coverage: All methods tested
- ✅ Serialization: Validated round-trip
- ✅ Edge cases: Handled (non-existent, duplicates, etc.)

## Files Modified

1. **diffviz-review/src/entities/decision.rs**
   - Added DecisionApproval struct (~10 lines)
   - Added DecisionApprovals collection (~60 lines)
   - Added implementation with all methods (~50 lines)
   - Added 5 comprehensive tests (~100 lines)

2. **diffviz-review/src/entities/mod.rs**
   - Added exports for DecisionApproval and DecisionApprovals (1 line)

Total: ~220 lines added, all passing tests.

## Ready For

✅ Phase 2: ReviewState and ReviewEngine integration
✅ Cascading approval logic implementation
✅ Reverse cascade on chunk approval
✅ Progress calculation
