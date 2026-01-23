# Additional Decisions - Phase 1: Core Domain Logic

## 2026-01-23T14:30:00Z - Approval Record Structure

**Category**: Entity Design

**Decision**: Use `decision_number: u32` as primary key instead of ReviewableDiffId

**Rationale**:
- Decisions are identified by number (u32), not by code location
- DecisionApproval must track approval state per decision, not per code change
- Mirror the Approval pattern: Approval uses ReviewableDiffId, DecisionApproval uses u32
- Simpler, more efficient indexing via HashMap<u32, DecisionApproval>

**Impact**:
- DecisionApprovals collection indexed by decision number
- is_approved(decision_number) query method
- Methods accept/return u32 consistently

**Evidence**: Context document specifies "approval at the decision level" and implementation roadmap shows "HashMap<u32, DecisionApproval>"

---

## 2026-01-23T14:35:00Z - Collection Storage Pattern

**Category**: Architecture

**Decision**: Separate DecisionApprovals collection from ReviewDecisions

**Rationale**:
- Decisions and approvals are orthogonal concerns (navigation vs actions)
- Mirrors existing pattern: Approval is separate from decisions
- Enables independent serialization and state management
- Clear separation for future TUI integration
- ReviewDecisions focuses on code-to-decision mapping
- DecisionApprovals focuses on approval state tracking

**Impact**:
- New DecisionApprovals will be added to ReviewState separately (Phase 2)
- ReviewEngine will orchestrate cascading between the two
- Both collections indexed independently

**Evidence**: Onboarding and context document describe decisions and approvals as separate orthogonal concerns

---

## 2026-01-23T14:40:00Z - Approval Lifecycle Methods

**Category**: API Design

**Decision**: Implement approve()/unapprove() as mutating methods

**Rationale**:
- Mirror ReviewApprovals API exactly
- approve() inserts or overwrites, unapprove() removes
- Simple, predictable behavior
- No complex state transitions needed at entity level
- Cascading logic belongs in ReviewEngine, not entities

**Impact**:
- approve() with (decision_number, approved_by, timestamp)
- unapprove() with decision_number
- is_approved() returns bool
- Behavior: approve overwrites previous approval, unapprove removes completely

---

## 2026-01-23T14:45:00Z - Query Methods

**Category**: API Design

**Decision**: Add total_approved() and approval_percentage() query methods

**Rationale**:
- Needed for UI progress indicators
- Mirror ReviewApprovals implementation
- Calculate on-demand (no caching per YAGNI principle)
- approval_percentage guards against zero division

**Impact**:
- total_approved() counts approved decisions in collection
- approval_percentage(total_decisions) calculates percentage
- UI can query progress without caching

---

## 2026-01-23T14:50:00Z - Float Precision in Tests

**Category**: Testing

**Decision**: Use approximate comparison for floating-point percentage tests

**Rationale**:
- Floating-point arithmetic causes precision differences
- 3/5 * 100 = 59.999998 ≠ 60.0 in binary floating point
- Use (actual - expected).abs() < 0.01 for safe comparison

**Impact**:
- Tests use `assert!((result - expected).abs() < 0.01)`
- Ensures tests are robust and maintainable
- No impact on production code (approvals are bool/u32)

---

## 2026-01-23T14:55:00Z - Test Coverage Strategy

**Category**: Testing

**Decision**: Comprehensive unit tests covering lifecycle, serialization, and edge cases

**Rationale**:
- Phase 1 is pure domain logic, must be thoroughly tested
- Tests validate entity behavior before integration in Phase 2
- Tests serve as documentation of expected behavior
- Edge cases guide future integration work

**Tests Added**:
1. `test_decision_approval_lifecycle` - approve/is_approved/unapprove flow
2. `test_decision_approval_percentage` - percentage calculation accuracy
3. `test_decision_approval_serialization` - serde round-trip
4. `test_decision_approval_multiple_approvals` - managing multiple approvals
5. `test_decision_approval_edge_cases` - unapproving non-existent, double approve

**Impact**:
- 5 new tests all passing
- Clear examples for Phase 2 integration tests
- Edge cases documented for edge case handling in ReviewEngine

---

## 2026-01-23T15:00:00Z - Module Export Strategy

**Category**: Public API

**Decision**: Export DecisionApproval and DecisionApprovals from entities module

**Rationale**:
- Follow existing Approval export pattern from approval.rs
- Makes types available for ReviewState in Phase 2
- Required for TUI access in Phase 3
- Maintains consistent module organization

**Impact**:
- Added to `pub use` statement in entities/mod.rs
- Types immediately available in Phase 2 work
- No breaking changes (additive export)
