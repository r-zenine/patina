# Context Document: Decision Approval System

## Behavioral Specification

Extend the existing approval system to support approvals at the decision level, not just individual chunks. Currently, reviewers can only approve individual ReviewableDiffs (chunks). This feature adds the ability to approve entire architectural decisions, which automatically cascades to all chunks mapped to that decision.

### Current Behavior
- Approvals exist only at the ReviewableDiff (chunk) level
- Decisions are purely organizational/navigational constructs
- No way to approve all chunks related to a decision in one action
- Decision tree shows no approval indicators

### Desired Behavior
- **Decision-level approval**: Approve/reject entire decisions
- **Cascading approvals**: Approving a decision approves all its chunks
- **Symmetrical unapproval**: Unapproving a decision unapproves all its chunks
- **Reverse cascade**: When all chunks in a decision are approved, the decision auto-marks as approved
- **Progress indication**: Show "3/5 chunks approved" for partially-approved decisions in the TUI

## Architecture Summary

### Current System Architecture

**Three-Layer Separation:**
1. **diffviz-review** (domain layer): Contains approval and decision entities
2. **diffviz-review** (engine layer): ReviewEngine orchestrates approval operations
3. **diffviz-review-tui**: TUI displays approval state and handles user interactions

**Key Architectural Patterns:**
- **Entity-Centric Design**: Approval and Decision are separate entities indexed by ReviewableDiffId
- **Orthogonal Concerns**: Decisions (navigation) vs Approvals (actions) are currently completely separate
- **State Query Pattern**: All UI components query approval state through `review_engine.state().is_approved(&id)`
- **No Caching**: UI never caches approval state, always queries ReviewEngine

**Current Approval Flow:**
```
User presses Space+a+a → UiEvent::ToggleApprove
→ Extracts current reviewable_id from UiState
→ BusinessEvent::ToggleApprove(reviewable_id)
→ Queries ReviewEngine.state().is_approved(&id)
→ Calls ReviewEngine.approve() or ReviewEngine.reject()
→ Updates ReviewState.approvals (HashMap<ReviewableDiffId, Approval>)
→ UI re-renders with updated approval icons
```

**Current Decision Structure:**
- `Decision`: Contains number, title, summary, code_impacts
- `ReviewDecisions`: Collection with decision_index (ReviewableDiffId → Vec<u32>)
- Index built via `build_index_from_review_state()` - maps chunks to decisions
- Decision 0: Synthetic decision for unmapped chunks
- Decisions currently have NO approval state

**Dependency Rules:**
- diffviz-review is self-contained (depends only on diffviz-core)
- diffviz-review-tui depends on diffviz-review
- ReviewEngine owns ReviewState which contains both approvals and decisions

### Relevant Files

**diffviz-review/src/entities/approval.rs** (diffviz-review:5-76)
- `Approval` struct with reviewable_id, approved, approved_by, approval_timestamp
- `ReviewApprovals` collection with HashMap indexing
- Methods: approve(), unapprove(), is_approved(), total_approved(), approval_percentage()

**diffviz-review/src/entities/decision.rs** (diffviz-review:1-723)
- `Decision` struct with number, title, summary, code_impacts
- `ReviewDecisions` collection with decision_index for reverse lookup
- Methods: add_decision(), build_index_from_review_state(), get_decisions_for_diff(), create_unmapped_decision()
- NO approval-related functionality currently

**diffviz-review/src/state/mod.rs** (diffviz-review state)
- `ReviewState` contains: reviewable_diffs, approvals, instructions, decisions
- Query methods: is_approved(), approval_progress()
- Mutation methods return &mut Self for chaining

**diffviz-review/src/engines/review_engine.rs** (diffviz-review engine)
- Main orchestrator with ReviewState
- approve(), reject(), approve_all_in_file() methods for chunk-level operations
- Cache invalidation on state changes

**diffviz-review-tui/src/app.rs** (TUI application)
- Handles BusinessEvent::ToggleApprove and BusinessEvent::ApproveFile
- Calls ReviewEngine approval methods

**diffviz-review-tui/src/decision_navigation.rs** (TUI navigation)
- DecisionNavigationTree manages navigation state
- TreePath with depth: 0 (decision), 1 (file), 2 (chunk)

**diffviz-review-tui/src/ui/components/diff_view.rs** (TUI display)
- Shows approval icons (✓ or ○) based on chunk approval state
- Depth-routed display: renders differently based on TreePath.depth()

## Design Constraints

### User Requirements
1. **Cascade approval**: Approving decision → approves all chunks
2. **Symmetrical unapproval**: Unapproving decision → unapproves all chunks
3. **Reverse cascade**: All chunks approved → decision auto-approved
4. **Progress indication**: Show "X/Y chunks approved" in TUI

### Technical Constraints
1. **ZERO WARNINGS RULE**: No compiler or clippy warnings allowed
2. **Clean Architecture**: diffviz-review must not depend on infrastructure layers
3. **Fail Fast**: No fallbacks or defensive programming in diffviz-core
4. **Entity Pattern**: Follow existing Approval/Decision entity patterns
5. **State Query Pattern**: UI must query state, never cache
6. **Builder Pattern**: Mutation methods return &mut Self
7. **Test Coverage**: Update test suite to prevent regressions

### Architectural Constraints
1. **No backwards compatibility**: Don't preserve legacy patterns
2. **Minimal abstraction**: Only abstract when needed, not for hypothetical futures
3. **Existing pattern reuse**: Follow ReviewApprovals collection pattern for DecisionApprovals
4. **Serialization**: Must support serde for persistence

## Implementation Strategy

**Core-then-Integrate** approach:

**Phase 1: Core Domain Logic**
- Add decision-level approval entity and collection to diffviz-review
- Implement approval state tracking for decisions
- Build cascading approval logic (decision → chunks, chunks → decision)
- Write comprehensive unit tests

**Phase 2: Integration with Review Engine**
- Add ReviewEngine methods for decision approval operations
- Integrate decision approvals into ReviewState
- Update approval query methods to handle decision-level queries
- Wire cascading logic into approval/rejection operations

**Phase 3: TUI Integration** (use diffviz-review-tui-contribution skill)
- Add BusinessEvents for decision approval
- Add keybindings for decision approval actions
- Update visual components to show decision approval state and progress
- Update decision tree to display approval indicators
- Add tests using TUI test harness

## Key Implementation Decisions

### Decision Approval Storage
Store decision approvals separately from chunk approvals:
- New entity: `DecisionApproval` (similar to `Approval`)
- New collection: `DecisionApprovals` (HashMap<u32, DecisionApproval>)
- Rationale: Separation of concerns, follows existing pattern, enables independent queries

### Cascading Logic Location
Implement cascading in ReviewEngine, not in entities:
- Entities remain pure data structures
- ReviewEngine orchestrates cross-entity operations
- Rationale: Entities shouldn't know about each other, engine owns business logic

### Reverse Cascade Trigger
Trigger reverse cascade (chunks → decision) during chunk approval:
- When approve() is called on a chunk, check if all sibling chunks are approved
- If yes, auto-approve the decision
- Rationale: Immediate feedback, no separate reconciliation step needed

### Progress Calculation
Calculate progress on-demand, don't store:
- Method: `decision_approval_progress(decision_number) -> (approved, total)`
- Iterate decision's chunks and count approved ones
- Rationale: YAGNI - no need for cached progress when calculation is cheap

### TUI Visual Indicators
Show approval state at all depth levels:
- Depth 0 (decision): Show approval icon + progress count "(3/5)"
- Depth 1 (file): Show existing file-level approval
- Depth 2 (chunk): Show existing chunk-level approval
- Rationale: Consistent visual feedback at every navigation level
