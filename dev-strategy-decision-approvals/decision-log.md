# Decision Log: Decision Approval System

## User Requirements

### Q: Should approving a decision automatically approve all chunks within that decision?
**Decision**: Yes - approval cascades to all chunks (Recommended)

When you approve Decision #1, all chunks mapped to that decision get automatically approved. Mirrors the organizational structure.

### Q: Should unapproving a decision unapprove all its chunks?
**Decision**: Yes - symmetrical behavior (Recommended)

Unapproving a decision removes approval from all its chunks. Keeps approval state consistent with the cascade model.

### Q: Should approving all chunks in a decision automatically mark the decision as approved?
**Decision**: Yes - auto-complete decision (Recommended)

When all chunks are approved, the decision automatically becomes approved. Provides visual completion feedback.

### Q: How should partially-approved decisions be visually indicated in the TUI?
**Decision**: Progress indicator (e.g., "3/5 chunks") (Recommended)

Show how many chunks are approved out of total chunks for each decision in the tree view.

## Technical Decisions

### Decision Storage Model
**Decision**: Separate DecisionApproval entity, independent from Approval

**Rationale**:
- Follows existing pattern (Approval for chunks, DecisionApproval for decisions)
- Clean separation of concerns
- Enables independent queries and serialization
- HashMap<u32, DecisionApproval> indexed by decision number

**Alternatives Considered**:
- Store decision approval as a flag in Decision entity: Rejected because Decision is domain data, approval is review state
- Extend Approval entity to handle both: Rejected because ReviewableDiffId doesn't map to decisions

### Cascading Logic Location
**Decision**: Implement in ReviewEngine, not entities

**Rationale**:
- ReviewEngine owns business logic and orchestration
- Entities remain pure data structures (current pattern)
- Engine has access to both decision and approval collections
- Enables transaction-like behavior across entities

**Alternatives Considered**:
- Cascading in DecisionApprovals collection: Rejected because collections shouldn't know about other collections
- Cascading in ReviewState: Rejected because State is data container, not orchestrator

### Reverse Cascade Trigger Point
**Decision**: Trigger during chunk approval in ReviewEngine

**Rationale**:
- Immediate feedback when last chunk is approved
- No separate reconciliation step needed
- Simple implementation: check sibling chunks during approve()
- Follows existing approval flow pattern

**Alternatives Considered**:
- Periodic reconciliation: Rejected due to complexity and delayed feedback
- Explicit "check decision status" command: Rejected due to poor UX

### Progress Calculation Strategy
**Decision**: Calculate on-demand, don't cache

**Rationale**:
- YAGNI principle: calculation is O(n) where n = chunks per decision (typically small)
- No cache invalidation complexity
- Always accurate, no stale data possible
- Simple implementation

**Alternatives Considered**:
- Cache progress in DecisionApproval: Rejected due to cache invalidation complexity
- Store approved count: Rejected because it can drift from actual state

### TUI Keybinding Strategy
**Decision**: Add decision-level approval to existing Space+a menu

**Rationale**:
- Consistent with existing approval UX (Space+a+a for chunk, Space+a+f for file)
- Natural extension: Space+a+d for decision
- Leverages existing leader key infrastructure
- Depth-aware: only show when decision is selected (depth 0)

**Alternatives Considered**:
- Separate keybinding (e.g., D): Rejected because it breaks consistency with current UX
- Auto-detect based on selection: Rejected because Space+a is already context-aware

### Visual Indicator Strategy
**Decision**: Icon + progress count at depth 0

**Rationale**:
- Consistent with existing approval icons (✓/○)
- Progress count "(3/5)" provides at-a-glance status
- No new visual vocabulary needed
- Works within current theme system

**Alternatives Considered**:
- Color coding: Rejected because monochrome terminals may not support
- Progress bar: Rejected due to limited horizontal space in tree view
- Partial icon (◐): Rejected because progress count is more informative

## Implementation Strategy Decisions

### Development Approach
**Decision**: Core-then-Integrate strategy

**Phases**:
1. Core domain logic (entities, approval tracking, cascading)
2. Integration (ReviewEngine methods, ReviewState)
3. TUI (events, keybindings, visual components)

**Rationale**:
- User explicitly requested this approach
- Enables testing business logic before UI complexity
- Clean boundary between domain and presentation
- Follows project's clean architecture principles

### Testing Strategy
**Decision**: Unit tests for core, integration tests for engine, test harness for TUI

**Rationale**:
- Core logic is pure and easily testable
- ReviewEngine integration requires ReviewState setup
- TUI test harness enables keyboard interaction testing
- Follows existing testing patterns in codebase

**Coverage Requirements**:
- Test cascading: decision approval → all chunks approved
- Test reverse cascade: all chunks → decision approved
- Test partial approval: some chunks → decision remains unapproved
- Test unapproval: decision unapproval → all chunks unapproved
- Test progress calculation: correct counts at various approval states
