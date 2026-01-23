# Code Context: Decision Approval System

## Relevant Code Locations

### Core Entities

**diffviz-review/src/entities/approval.rs**
Current chunk-level approval implementation. This pattern should be mirrored for decision approvals.

```
Lines 10-17: Approval struct definition
Lines 20-23: ReviewApprovals collection with HashMap
Lines 32-45: approve() method - adds approval to HashMap
Lines 47-49: unapprove() method - removes from HashMap
Lines 51-55: is_approved() query
Lines 62-66: total_approved() count
Lines 68-75: approval_percentage() calculation
```

**diffviz-review/src/entities/decision.rs**
Decision entity that needs approval capability added.

```
Lines 49-57: Decision struct - contains number, title, summary, code_impacts
Lines 60-69: ReviewDecisions collection structure
Lines 79-83: add_decision() method
Lines 86-123: build_index_from_review_state() - builds decision_index mapping
Lines 138-148: get_decisions_for_diff() - reverse lookup from chunk to decisions
Lines 150-155: all_decisions() - returns decisions sorted by number
```

### State Management

**diffviz-review/src/state/mod.rs**
Central state container that will need to hold decision approvals.

```
Lines containing ReviewState struct: Contains approvals, instructions, decisions collections
Lines containing is_approved(): Query method for chunk approval - pattern for decision approval query
Lines containing approval_progress(): Calculation method - pattern for decision progress calculation
Lines containing approve()/unapprove(): Mutation methods with &mut Self return
```

### Review Engine

**diffviz-review/src/engines/review_engine.rs**
Orchestrator that will implement cascading approval logic.

```
Lines containing ReviewEngine struct: Main orchestrator with state and diff_provider
Lines containing approve() method: Chunk approval - pattern for decision approval
Lines containing reject() method: Chunk rejection - pattern for decision rejection
Lines containing approve_all_in_file(): Bulk approval pattern - similar to approve_all_in_decision
Lines with cache invalidation: Pattern for invalidating cache after approval changes
```

### TUI Application Layer

**diffviz-review-tui/src/app.rs**
Main application that handles business events.

```
Lines containing BusinessEvent enum: Needs new variants for decision approval
Lines containing handle_business_event(): Needs cases for decision approval events
Lines containing ToggleApprove handling: Pattern for implementing ToggleApproveDecision
Lines containing ApproveFile handling: Pattern for implementing decision approval
```

**diffviz-review-tui/src/events/input.rs**
Event definitions for UI interactions.

```
Lines containing UiEvent enum: May need new event for decision approval
Lines containing business.rs reference: Event conversion pipeline
```

**diffviz-review-tui/src/decision_navigation.rs**
Navigation tree that will display approval state.

```
Lines containing DecisionNavigationTree: Tree structure that needs approval indicators
Lines containing TreePath: Contains depth field for routing (0=decision, 1=file, 2=chunk)
Lines containing get_current_reviewable_id(): Pattern for get_current_decision_number()
```

**diffviz-review-tui/src/ui/components/diff_view.rs**
Renders diff view with approval indicators.

```
Lines containing render() with depth routing: Pattern for depth-based approval display
Lines containing Icons::APPROVED/PENDING: Visual indicators to reuse
Lines containing is_approved() queries: Pattern for querying decision approval
```

**diffviz-review-tui/src/ui/components/decision_tree.rs**
Renders decision tree - needs approval indicators and progress counts.

```
Lines containing tree rendering: Where to add approval icons
Lines containing decision number display: Where to add progress count "(X/Y)"
```

## Key Patterns to Follow

### Entity Pattern
```rust
// From approval.rs - mirror this for DecisionApproval
pub struct Approval {
    pub reviewable_id: ReviewableDiffId,  // Change to decision_number: u32
    pub approved: bool,
    pub approved_by: String,
    pub approval_timestamp: String,
}

pub struct ReviewApprovals {
    pub approvals: HashMap<ReviewableDiffId, Approval>, // Change to HashMap<u32, DecisionApproval>
}
```

### Query Pattern
```rust
// From state/mod.rs - pattern for decision approval queries
pub fn is_approved(&self, reviewable_id: &ReviewableDiffId) -> bool {
    self.approvals.is_approved(reviewable_id)
}

// New pattern for decisions:
// pub fn is_decision_approved(&self, decision_number: u32) -> bool
```

### Mutation Pattern
```rust
// From state/mod.rs - builder pattern with &mut Self return
pub fn approve(&mut self, reviewable_id: ReviewableDiffId, approved_by: String) -> &mut Self {
    let timestamp = chrono::Utc::now().to_rfc3339();
    self.approvals.approve(reviewable_id, approved_by, timestamp);
    self
}
```

### Cascading Logic Pattern (New)
```rust
// In ReviewEngine - pseudo-code for cascading
// pub fn approve_decision(&mut self, decision_number: u32, author: String) {
//     // 1. Approve the decision itself
//     self.state.decision_approvals.approve(decision_number, author, timestamp);
//
//     // 2. Get all chunks for this decision
//     let chunks = self.get_chunks_for_decision(decision_number);
//
//     // 3. Approve each chunk
//     for chunk_id in chunks {
//         self.approve(chunk_id, author.clone(), None);
//     }
//
//     // 4. Invalidate cache
//     self.invalidate_cache_for_decision(decision_number);
// }
```

### Reverse Cascade Pattern (New)
```rust
// In ReviewEngine.approve() - pseudo-code for reverse cascade
// pub fn approve(&mut self, reviewable_id: ReviewableDiffId, author: String, comment: Option<String>) {
//     // 1. Approve the chunk (existing logic)
//     self.state.approve(reviewable_id.clone(), author.clone());
//
//     // 2. Check if this chunk belongs to any decisions
//     let decisions = self.state.decisions.get_decisions_for_diff(&reviewable_id);
//
//     // 3. For each decision, check if all chunks are now approved
//     for decision in decisions {
//         let all_chunks = self.get_chunks_for_decision(decision.number);
//         let all_approved = all_chunks.iter().all(|id| self.state.is_approved(id));
//
//         if all_approved {
//             self.state.decision_approvals.approve(decision.number, author.clone(), timestamp);
//         }
//     }
// }
```

### Progress Calculation Pattern (New)
```rust
// In ReviewState or ReviewEngine - pseudo-code
// pub fn decision_approval_progress(&self, decision_number: u32) -> (usize, usize) {
//     let chunks = self.get_chunks_for_decision(decision_number);
//     let approved_count = chunks.iter().filter(|id| self.is_approved(id)).count();
//     (approved_count, chunks.len())
// }
```

### TUI Event Pattern
```rust
// From app.rs - pattern for handling approval events
// BusinessEvent::ToggleApprove(reviewable_id) => {
//     let is_approved = self.review_engine.state().is_approved(&reviewable_id);
//     if is_approved {
//         self.review_engine.reject(&reviewable_id, None)?;
//     } else {
//         self.review_engine.approve(&reviewable_id, self.review_engine.author(), None)?;
//     }
// }

// New pattern for decisions:
// BusinessEvent::ToggleApproveDecision(decision_number) => {
//     let is_approved = self.review_engine.state().is_decision_approved(decision_number);
//     if is_approved {
//         self.review_engine.reject_decision(decision_number)?;
//     } else {
//         self.review_engine.approve_decision(decision_number, self.review_engine.author())?;
//     }
// }
```

## Integration Points

### ReviewState Extension
Add decision_approvals field to ReviewState struct alongside existing approvals, instructions, decisions.

### ReviewEngine Methods
Add new public methods:
- `approve_decision(decision_number, author)`
- `reject_decision(decision_number)`
- `is_decision_approved(decision_number) -> bool`
- `decision_approval_progress(decision_number) -> (usize, usize)`

### DecisionNavigationTree Integration
Add method to get current decision number:
- `get_current_decision_number() -> Option<u32>` (similar to get_current_reviewable_id)

### BusinessEvent Extension
Add new event variants:
- `ToggleApproveDecision(u32)` for decision-level toggle

### Keybinding Integration
Extend leader key menu (Space + a):
- Existing: `a` for chunk, `f` for file
- New: `d` for decision (only shown at depth 0)

## Files to Modify

### Phase 1: Core (diffviz-review)
1. `diffviz-review/src/entities/decision.rs` - Add DecisionApproval entity and DecisionApprovals collection
2. `diffviz-review/src/state/mod.rs` - Add decision_approvals field and query methods
3. `diffviz-review/src/entities/mod.rs` - Export new DecisionApproval types

### Phase 2: Integration (diffviz-review)
4. `diffviz-review/src/engines/review_engine.rs` - Add decision approval methods and cascading logic
5. `diffviz-review/src/engines/review_engine.rs` - Update chunk approve() for reverse cascade
6. `diffviz-review/src/lib.rs` - Export decision approval types if needed

### Phase 3: TUI (diffviz-review-tui - use skill)
7. `diffviz-review-tui/src/events/business.rs` - Add BusinessEvent::ToggleApproveDecision
8. `diffviz-review-tui/src/events/input.rs` - Add UiEvent if needed (or reuse existing)
9. `diffviz-review-tui/src/app.rs` - Handle ToggleApproveDecision event
10. `diffviz-review-tui/src/decision_navigation.rs` - Add get_current_decision_number()
11. `diffviz-review-tui/src/ui/components/diff_view.rs` - Show decision approval at depth 0
12. `diffviz-review-tui/src/ui/components/decision_tree.rs` - Add approval icons and progress counts
13. `diffviz-review-tui/src/ui/components/which_key.rs` - Add 'd' option to approval menu

## Testing Requirements

### Unit Tests (diffviz-review)
- `diffviz-review/src/entities/decision.rs` - Test DecisionApproval serialization
- `diffviz-review/src/state/mod.rs` - Test decision approval queries
- `diffviz-review/src/engines/review_engine.rs` - Test cascading and reverse cascade logic

### Integration Tests (diffviz-review)
- Create fixtures with decisions and chunks
- Test approval cascade: decision → all chunks
- Test reverse cascade: all chunks → decision
- Test partial approval: some chunks → decision stays unapproved
- Test unapproval cascade: decision → all chunks

### TUI Tests (diffviz-review-tui)
- Test keybinding: Space+a+d when at depth 0
- Test visual indicators: approval icons at depth 0
- Test progress display: "(X/Y)" format
- Use InputTestHarness for interaction testing
- Use RenderTestHarness for visual validation
