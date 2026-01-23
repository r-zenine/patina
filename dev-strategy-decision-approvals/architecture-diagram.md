# Architecture Diagram: Decision Approval System

## System Architecture (After Implementation)

```
┌─────────────────────────────────────────────────────────────────┐
│                       diffviz-review-tui                        │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ UI Layer                                                    │ │
│  │                                                             │ │
│  │  [Decision Tree]        [Diff View (Depth-Routed)]         │ │
│  │   ✓ Decision #1 (5/5)    Depth 0: [✓] Decision #1 (5/5)   │ │
│  │   ○ Decision #2 (2/5)    Depth 1: File view               │ │
│  │   ○ Decision #3 (0/5)    Depth 2: [✓] Chunk view          │ │
│  │                                                             │ │
│  │  [Which-Key Menu]                                          │ │
│  │   Space+a → a: Approve chunk                              │ │
│  │            f: Approve file                                │ │
│  │            d: Approve decision (NEW)                      │ │
│  └────────────────────────────────────────────────────────────┘ │
│                               ▲                                 │
│                               │ Query approval state            │
│                               │ Render UI                       │
│  ┌────────────────────────────┴────────────────────────────────┐ │
│  │ Event Handling                                              │ │
│  │                                                             │ │
│  │  UiEvent::ToggleApprove                                    │ │
│  │         │                                                   │ │
│  │         ▼                                                   │ │
│  │  ui_event_to_business_event()                             │ │
│  │    ├─ Depth 0 → BusinessEvent::ToggleApproveDecision      │ │
│  │    ├─ Depth 1 → BusinessEvent::ApproveFile                │ │
│  │    └─ Depth 2 → BusinessEvent::ToggleApprove              │ │
│  │         │                                                   │ │
│  │         ▼                                                   │ │
│  │  handle_business_event()                                   │ │
│  │    └─ Calls ReviewEngine methods                          │ │
│  └─────────────────────────────────────────────────────────────┘ │
└────────────────────────┬────────────────────────────────────────┘
                         │ Call approve_decision()
                         │ Call is_decision_approved()
                         │ Call decision_approval_progress()
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                        diffviz-review                           │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ ReviewEngine (Orchestration Layer)                         │ │
│  │                                                             │ │
│  │  approve_decision(number, author)                          │ │
│  │    ├─ state.approve_decision(number, author)              │ │
│  │    ├─ For each chunk in decision:                         │ │
│  │    │    └─ approve(chunk_id, author) [CASCADE]            │ │
│  │    └─ invalidate_cache_for_decision()                     │ │
│  │                                                             │ │
│  │  approve(chunk_id, author) [MODIFIED]                      │ │
│  │    ├─ state.approve(chunk_id, author)                     │ │
│  │    ├─ Get decisions for this chunk                        │ │
│  │    ├─ For each decision:                                  │ │
│  │    │    ├─ Check if all chunks approved                   │ │
│  │    │    └─ If yes: approve_decision() [REVERSE CASCADE]   │ │
│  │    └─ invalidate_cache_for_chunk()                        │ │
│  │                                                             │ │
│  │  is_decision_approved(number) -> bool                      │ │
│  │  decision_approval_progress(number) -> (usize, usize)      │ │
│  └────────────────────────────────────────────────────────────┘ │
│                               │                                 │
│                               │ Mutate state                    │
│                               ▼                                 │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ ReviewState (State Container)                              │ │
│  │                                                             │ │
│  │  ┌──────────────────┐  ┌──────────────────────────────┐   │ │
│  │  │ approvals        │  │ decision_approvals (NEW)     │   │ │
│  │  │ (ReviewApprovals)│  │ (DecisionApprovals)          │   │ │
│  │  │                  │  │                              │   │ │
│  │  │ HashMap<         │  │ HashMap<                     │   │ │
│  │  │  ReviewableDiffId│  │  u32,                        │   │ │
│  │  │  Approval>       │  │  DecisionApproval>           │   │ │
│  │  └──────────────────┘  └──────────────────────────────┘   │ │
│  │                                                             │ │
│  │  ┌──────────────────┐  ┌──────────────────────────────┐   │ │
│  │  │ decisions        │  │ reviewable_diffs             │   │ │
│  │  │ (ReviewDecisions)│  │ BTreeMap<ReviewableDiffId,   │   │ │
│  │  │                  │  │          ReviewableDiff>     │   │ │
│  │  │ decision_index:  │  │                              │   │ │
│  │  │ HashMap<         │  │                              │   │ │
│  │  │  ReviewableDiffId│  │                              │   │ │
│  │  │  Vec<u32>>       │  │                              │   │ │
│  │  └──────────────────┘  └──────────────────────────────┘   │ │
│  │                                                             │ │
│  │  Query methods:                                            │ │
│  │    - is_approved(chunk_id) -> bool                        │ │
│  │    - is_decision_approved(number) -> bool (NEW)           │ │
│  │    - decision_approval_progress(number) -> (usize, usize) │ │
│  │                                                             │ │
│  │  Mutation methods:                                         │ │
│  │    - approve(chunk_id, author) -> &mut Self               │ │
│  │    - approve_decision(number, author) -> &mut Self (NEW)  │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                 │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ Entities                                                    │ │
│  │                                                             │ │
│  │  ┌────────────────┐       ┌────────────────────────────┐  │ │
│  │  │ Approval       │       │ DecisionApproval (NEW)     │  │ │
│  │  ├────────────────┤       ├────────────────────────────┤  │ │
│  │  │ reviewable_id  │       │ decision_number: u32       │  │ │
│  │  │ approved       │       │ approved: bool             │  │ │
│  │  │ approved_by    │       │ approved_by: String        │  │ │
│  │  │ timestamp      │       │ approval_timestamp: String │  │ │
│  │  └────────────────┘       └────────────────────────────┘  │ │
│  │                                                             │ │
│  │  ┌────────────────────────────────────────────────────┐   │ │
│  │  │ Decision                                            │   │ │
│  │  ├────────────────────────────────────────────────────┤   │ │
│  │  │ number: u32                                         │   │ │
│  │  │ title: String                                       │   │ │
│  │  │ summary: String                                     │   │ │
│  │  │ code_impacts: Vec<CodeImpact>                       │   │ │
│  │  │   ├─ file: String                                   │   │ │
│  │  │   ├─ line_ranges: Vec<DecisionLineRange>            │   │ │
│  │  │   └─ (maps to ReviewableDiffIds via decision_index) │   │ │
│  │  └────────────────────────────────────────────────────┘   │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## Cascading Approval Flow

### Forward Cascade (Decision → Chunks)

```
User: Space+a+d at Decision #1
    │
    ▼
ReviewEngine.approve_decision(1, "author")
    │
    ├─ 1. Approve decision itself
    │      state.approve_decision(1, "author")
    │         └─ decision_approvals.approve(1, "author", timestamp)
    │
    ├─ 2. Get all chunks for Decision #1
    │      chunks = get_chunks_for_decision(1)
    │         └─ Query decision_index: ReviewableDiffId → Vec<u32>
    │         └─ Filter: where Vec contains 1
    │         └─ Returns: [chunk_A, chunk_B, chunk_C]
    │
    ├─ 3. Approve each chunk (CASCADE)
    │      for chunk in chunks:
    │         approve(chunk, "author")
    │            └─ state.approve(chunk, "author")
    │
    └─ 4. Invalidate cache
           invalidate_cache_for_decision(1)

Result: Decision #1 ✓, Chunk A ✓, Chunk B ✓, Chunk C ✓
```

### Reverse Cascade (Chunks → Decision)

```
User: Space+a+a at last unapproved Chunk C
    │
    ▼
ReviewEngine.approve(chunk_C, "author")
    │
    ├─ 1. Approve the chunk
    │      state.approve(chunk_C, "author")
    │
    ├─ 2. Get decisions for this chunk
    │      decisions = state.decisions.get_decisions_for_diff(chunk_C)
    │         └─ Query decision_index[chunk_C]
    │         └─ Returns: [Decision #1]
    │
    └─ 3. For each decision, check if all chunks approved
           for decision in decisions:
              all_chunks = get_chunks_for_decision(decision.number)
                 └─ Returns: [chunk_A, chunk_B, chunk_C]
              all_approved = all_chunks.iter().all(|id| state.is_approved(id))
                 └─ chunk_A: ✓, chunk_B: ✓, chunk_C: ✓ → TRUE
              if all_approved:
                 state.approve_decision(decision.number, "author")
                    └─ decision_approvals.approve(1, "author", timestamp)

Result: Chunk C ✓ → Decision #1 auto-approved ✓ (REVERSE CASCADE)
```

## Data Structures

### ReviewState Structure

```rust
pub struct ReviewState {
    // Existing fields
    pub reviewable_diffs: BTreeMap<ReviewableDiffId, ReviewableDiff>,
    pub approvals: ReviewApprovals,              // Chunk-level approvals
    pub instructions: ReviewInstructions,
    pub decisions: ReviewDecisions,
    pub journey: ReviewJourney,

    // NEW field
    pub decision_approvals: DecisionApprovals,   // Decision-level approvals
}
```

### Indexing Relationships

```
Decision #1
    ├─ decision_approvals[1] → DecisionApproval { approved: true, ... }
    └─ decision_index maps to chunks:
            ├─ ReviewableDiffId("working:file.rs:L10-20")
            │      └─ approvals["working:file.rs:L10-20"] → Approval { approved: true }
            ├─ ReviewableDiffId("working:file.rs:L30-40")
            │      └─ approvals["working:file.rs:L30-40"] → Approval { approved: true }
            └─ ReviewableDiffId("working:file.rs:L50-60")
                   └─ approvals["working:file.rs:L50-60"] → Approval { approved: true }
```

### Progress Calculation

```rust
pub fn decision_approval_progress(&self, decision_number: u32) -> (usize, usize) {
    // 1. Get all chunks for this decision
    let chunks = get_chunks_for_decision(decision_number);
    //    e.g., [chunk_A, chunk_B, chunk_C]

    // 2. Count approved chunks
    let approved_count = chunks
        .iter()
        .filter(|id| self.is_approved(id))
        .count();
    //    e.g., chunk_A ✓, chunk_B ✓, chunk_C ○ → approved_count = 2

    // 3. Return (approved, total)
    (approved_count, chunks.len())
    //    e.g., (2, 3)
}
```

## TUI Display Flow

### Depth-Routed Display

```
TreePath.depth() determines what to show:

Depth 0 (Decision level):
    [✓] Decision #1: Refactor auth module (5/5)
           ▲                                  ▲
           │                                  │
           │                                  └─ decision_approval_progress(1)
           └─ is_decision_approved(1)

Depth 1 (File level):
    file: src/auth.rs
    [Shows file-level approval - existing behavior]

Depth 2 (Chunk level):
    [✓] working:src/auth.rs:L10-20
           ▲
           └─ is_approved(reviewable_id)
```

### Visual Components

```
Decision Tree:                 Diff View (Depth 0):
├─ ✓ Decision #1 (5/5)        ┌────────────────────────────┐
├─ ◐ Decision #2 (2/5)  ←──── │ [✓] Decision #1            │
│   ├─ src/file1.rs           │                            │
│   └─ src/file2.rs           │ Title: Refactor auth       │
└─ ○ Decision #3 (0/5)        │ Summary: Extract auth...   │
                               │                            │
                               │ Progress: 5/5 chunks ✓     │
                               │                            │
                               │ Key: Space+a+d to toggle   │
                               └────────────────────────────┘
```

## Event Flow Diagram

```
Keyboard Input: Space+a+d
    │
    ▼
[Input Handler]
    │
    ▼
UiEvent::ToggleApprove
    │
    ▼
ui_event_to_business_event()
    │
    ├─ Extract TreePath from UiState
    ├─ Check depth: depth == 0 (decision level)
    ├─ Get current decision number: navigation.get_current_decision_number()
    │
    ▼
BusinessEvent::ToggleApproveDecision(1)
    │
    ▼
handle_business_event()
    │
    ├─ Query: is_decision_approved(1)?
    │     └─ Returns: false
    │
    ├─ Call: review_engine.approve_decision(1, "author")
    │     ├─ Approves decision
    │     └─ Cascades to all chunks
    │
    └─ Return: Command::None
           │
           ▼
[Main Loop Re-renders]
    │
    ▼
[UI Components Query State]
    ├─ DecisionTree: is_decision_approved(1) → ✓
    ├─ DecisionTree: decision_approval_progress(1) → (5, 5)
    └─ DiffView: Shows [✓] Decision #1 (5/5)
```

## Implementation Phases Breakdown

```
Phase 1: Core
├─ entities/decision.rs
│   ├─ DecisionApproval struct
│   ├─ DecisionApprovals collection
│   └─ Unit tests
├─ state/mod.rs
│   ├─ Add decision_approvals field
│   ├─ Query methods (is_decision_approved, progress)
│   └─ Mutation methods (approve_decision, unapprove_decision)
└─ Tests: Entity serialization, query logic

Phase 2: Integration
├─ engines/review_engine.rs
│   ├─ approve_decision() method
│   ├─ reject_decision() method
│   ├─ Modify approve() for reverse cascade
│   └─ Helper: get_chunks_for_decision()
└─ Tests: Cascading scenarios, reverse cascade

Phase 3: TUI (USE SKILL)
├─ events/business.rs
│   └─ Add BusinessEvent::ToggleApproveDecision
├─ app.rs
│   └─ Handle ToggleApproveDecision event
├─ decision_navigation.rs
│   └─ Add get_current_decision_number()
├─ ui/components/diff_view.rs
│   └─ Depth 0 rendering with approval state
├─ ui/components/decision_tree.rs
│   └─ Approval icons + progress counts
├─ ui/components/which_key.rs
│   └─ Add 'd' option to Space+a menu
└─ Tests: Input sequences, visual rendering
```
