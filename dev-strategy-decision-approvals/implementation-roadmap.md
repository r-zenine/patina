# Implementation Roadmap: Decision Approval System

## Strategy: Core-then-Integrate

Build pure business logic first, then integrate with review engine, finally add TUI.

---

## Phase 1: Core Domain Logic (diffviz-review)

**Goal**: Create decision approval entities and core approval tracking logic without any UI or engine integration.

### Task 1.1: Create DecisionApproval Entity

**File**: `diffviz-review/src/entities/decision.rs`

**What to build**:
- Add `DecisionApproval` struct mirroring the `Approval` pattern
- Fields: decision_number (u32), approved (bool), approved_by (String), approval_timestamp (String)
- Add `DecisionApprovals` collection struct with HashMap<u32, DecisionApproval>
- Implement methods: approve(), unapprove(), is_approved(), get_approval()
- Add Serialize/Deserialize derives for persistence

**Success criteria**:
- DecisionApproval compiles without warnings
- Serialization round-trip test passes
- approve()/unapprove()/is_approved() unit tests pass

**Dependencies**: None

### Task 1.2: Add Decision Approval Queries

**File**: `diffviz-review/src/entities/decision.rs`

**What to build**:
- Add helper methods to DecisionApprovals:
  - `total_approved() -> usize` - count approved decisions
  - `approval_percentage(total_decisions: usize) -> f32` - calculate approval %
- Mirror the pattern from ReviewApprovals (approval.rs:62-75)

**Success criteria**:
- Query methods return correct counts
- Percentage calculation handles zero division correctly
- Unit tests validate counting logic

**Dependencies**: Task 1.1

### Task 1.3: Update Module Exports

**File**: `diffviz-review/src/entities/mod.rs`

**What to build**:
- Export DecisionApproval and DecisionApprovals publicly
- Ensure they're available for use in state and engine layers

**Success criteria**:
- No compilation errors
- Types accessible from other modules

**Dependencies**: Task 1.1

### Task 1.4: Write Core Entity Tests

**File**: `diffviz-review/src/entities/decision.rs` (test module)

**What to build**:
- Test decision approval lifecycle (approve → is_approved → unapprove)
- Test serialization/deserialization
- Test approval counting and percentage calculation
- Test edge cases (empty collection, approve same decision twice)

**Success criteria**:
- All tests pass
- Coverage includes happy path and edge cases

**Dependencies**: Tasks 1.1, 1.2

---

## Phase 2: Integration with Review Engine (diffviz-review)

**Goal**: Wire decision approvals into ReviewState and ReviewEngine, implement cascading logic.

### Task 2.1: Extend ReviewState

**File**: `diffviz-review/src/state/mod.rs`

**What to build**:
- Add `decision_approvals: DecisionApprovals` field to ReviewState struct
- Initialize in ReviewState::new()
- Add query methods:
  - `is_decision_approved(&self, decision_number: u32) -> bool`
  - `decision_approval_progress(&self, decision_number: u32) -> (usize, usize)` - returns (approved_chunks, total_chunks)
- Add mutation methods with builder pattern:
  - `approve_decision(&mut self, decision_number: u32, approved_by: String) -> &mut Self`
  - `unapprove_decision(&mut self, decision_number: u32) -> &mut Self`

**Success criteria**:
- ReviewState compiles without warnings
- New methods follow existing builder pattern (&mut Self return)
- Query methods correctly access decision_approvals collection

**Dependencies**: Phase 1 complete

### Task 2.2: Implement Decision Approval Progress Calculation

**File**: `diffviz-review/src/state/mod.rs`

**What to build**:
- Implement `decision_approval_progress()` logic:
  1. Get all chunks for the decision via decision_index
  2. Count how many are approved
  3. Return (approved_count, total_count)
- Handle edge case: decision not found returns (0, 0)

**Success criteria**:
- Progress calculation correctly counts approved chunks
- Returns accurate totals
- Unit test validates counting at various approval states

**Dependencies**: Task 2.1

### Task 2.3: Add ReviewEngine Decision Approval Methods

**File**: `diffviz-review/src/engines/review_engine.rs`

**What to build**:
- Add public method: `approve_decision(&mut self, decision_number: u32, author: String) -> Result<()>`
  - Approve the decision itself via state.approve_decision()
  - Get all chunks for this decision from decision_index
  - Call approve() on each chunk (cascading)
  - Invalidate renderable cache for affected chunks
- Add public method: `reject_decision(&mut self, decision_number: u32) -> Result<()>`
  - Unapprove the decision via state.unapprove_decision()
  - Unapprove all chunks for this decision (symmetrical cascade)
  - Invalidate renderable cache
- Add query method: `is_decision_approved(&self, decision_number: u32) -> bool`

**Success criteria**:
- Methods compile and follow existing approve() pattern
- Cascading logic correctly iterates all chunks
- Cache invalidation happens for all affected chunks

**Dependencies**: Task 2.1

### Task 2.4: Implement Reverse Cascade (Chunks → Decision)

**File**: `diffviz-review/src/engines/review_engine.rs`

**What to build**:
- Modify existing `approve()` method to trigger reverse cascade:
  1. After approving chunk, get decisions that affect this chunk via get_decisions_for_diff()
  2. For each decision, check if ALL chunks are now approved
  3. If yes, auto-approve the decision via state.approve_decision()
- Modify existing `reject()` method:
  1. After rejecting chunk, get decisions for this chunk
  2. For each decision, if it was approved, unapprove it (since not all chunks approved anymore)

**Success criteria**:
- Approving last chunk in a decision auto-approves the decision
- Rejecting any chunk in an approved decision unapproves the decision
- Integration test validates reverse cascade behavior

**Dependencies**: Task 2.3

### Task 2.5: Add Helper Method for Getting Chunks

**File**: `diffviz-review/src/engines/review_engine.rs`

**What to build**:
- Add private helper: `get_chunks_for_decision(&self, decision_number: u32) -> Vec<ReviewableDiffId>`
  - Query decision_index to find all ReviewableDiffIds mapped to this decision
  - Return as vector
- Reuse in both approve_decision() and reverse cascade logic

**Success criteria**:
- Helper correctly retrieves all chunks for a decision
- Used consistently across cascading operations

**Dependencies**: Task 2.3

### Task 2.6: Write Integration Tests

**File**: `diffviz-review/tests/decision_approval_integration.rs` (new file)

**What to build**:
- Test fixtures with mock ReviewState containing decisions and chunks
- Test cascading approval: approve decision → all chunks approved
- Test cascading rejection: reject decision → all chunks unapproved
- Test reverse cascade: approve all chunks → decision auto-approved
- Test partial approval: approve some chunks → decision not approved
- Test progress calculation: various approval states → correct (X, Y) counts

**Success criteria**:
- All integration tests pass
- Coverage includes all cascading scenarios
- Tests use realistic ReviewState fixtures

**Dependencies**: Tasks 2.1-2.5

---

## Phase 3: TUI Integration (diffviz-review-tui)

**Goal**: Add UI for decision approval with keybindings, events, and visual indicators.

**Note**: Use the `diffviz-review-tui-contribution` skill for this phase as specified by user.

### Task 3.1: Add Decision Approval Business Events

**File**: `diffviz-review-tui/src/events/business.rs`

**What to build**:
- Add `ToggleApproveDecision(u32)` variant to BusinessEvent enum
- Update event conversion in `ui_event_to_business_event()`:
  - When UiEvent::ToggleApprove occurs at depth 0 (decision level)
  - Extract current decision number from navigation state
  - Return BusinessEvent::ToggleApproveDecision(decision_number)

**Success criteria**:
- Event compiles and integrates into existing enum
- Conversion logic correctly identifies depth 0 context
- Event carries decision number

**Dependencies**: Phase 2 complete

### Task 3.2: Handle Decision Approval in App

**File**: `diffviz-review-tui/src/app.rs`

**What to build**:
- Add match arm in `handle_business_event()` for `BusinessEvent::ToggleApproveDecision(decision_number)`
- Implementation:
  1. Query current approval state via review_engine.is_decision_approved(decision_number)
  2. If approved, call review_engine.reject_decision(decision_number)
  3. If not approved, call review_engine.approve_decision(decision_number, author)
  4. Return Command::None
- Mirror the pattern from ToggleApprove handler

**Success criteria**:
- Handler correctly toggles decision approval state
- No panics or errors during event handling
- State updates propagate to UI

**Dependencies**: Task 3.1

### Task 3.3: Add Navigation Helper for Current Decision

**File**: `diffviz-review-tui/src/decision_navigation.rs`

**What to build**:
- Add method to DecisionNavigationTree: `get_current_decision_number(&self) -> Option<u32>`
  - If current TreePath is at depth 0, extract decision number from path
  - Otherwise return None
- Mirror the pattern from get_current_reviewable_id()

**Success criteria**:
- Returns Some(number) when at depth 0
- Returns None at depth 1 or 2
- Used by event conversion logic

**Dependencies**: None (Phase 2 complete)

### Task 3.4: Update Diff View for Decision Approval Display

**File**: `diffviz-review-tui/src/ui/components/diff_view.rs`

**What to build**:
- In render() method, add depth 0 branch for decision display
- Query decision approval state via review_engine.state().is_decision_approved(decision_number)
- Query progress via review_engine.state().decision_approval_progress(decision_number)
- Display format: "[✓] Decision #X: Title (3/5)" or "[○] Decision #X: Title (0/5)"
- Use existing Icons::APPROVED and Icons::PENDING

**Success criteria**:
- Approval icon shows correctly based on decision state
- Progress count displays accurate numbers
- Visual format is consistent with existing chunk display

**Dependencies**: Task 3.3

### Task 3.5: Add Approval Indicators to Decision Tree

**File**: `diffviz-review-tui/src/ui/components/decision_tree.rs`

**What to build**:
- Modify tree rendering to show approval state for decisions
- For each decision node:
  - Query is_decision_approved(decision.number)
  - Query decision_approval_progress(decision.number) for (approved, total)
  - Render: "✓ Decision #1 (5/5)" or "○ Decision #2 (2/5)"
- Use theme colors for approved vs pending

**Success criteria**:
- Decision tree shows approval icons
- Progress counts appear next to each decision
- Visual hierarchy remains clear

**Dependencies**: Task 3.4

### Task 3.6: Add Keybinding for Decision Approval

**File**: `diffviz-review-tui/src/ui/components/which_key.rs`

**What to build**:
- Extend leader key menu (Space + a) to include decision approval option
- Add entry: `d` - "Toggle approve decision" (only shown when depth == 0)
- Wire to UiEvent::ToggleApprove (context determines it's for decision)

**Success criteria**:
- Keybinding appears in menu when at depth 0
- Pressing 'd' triggers decision approval toggle
- Menu hides 'd' option when at depth 1 or 2

**Dependencies**: Tasks 3.1, 3.2

### Task 3.7: Write TUI Tests

**File**: `diffviz-review-tui/tests/decision_approval_tests.rs` (new file)

**What to build**:
- Use InputTestHarness to test keyboard interactions:
  - Navigate to decision (depth 0)
  - Press Space+a+d
  - Assert decision is approved
  - Press Space+a+d again
  - Assert decision is unapproved
- Use RenderTestHarness to test visual output:
  - Assert approval icon changes
  - Assert progress count updates
- Test cascading via chunk approval sequence:
  - Approve all chunks individually
  - Assert decision auto-approves
  - Assert progress shows (N/N)

**Success criteria**:
- All TUI tests pass
- Keyboard sequences work as expected
- Visual assertions validate correct rendering

**Dependencies**: Tasks 3.1-3.6

---

## Phase 4: Final Polish

### Task 4.1: Update Documentation

**Files**: READMEs, onboarding.md files

**What to build**:
- Update diffviz-review/onboarding.md to document DecisionApproval
- Update diffviz-review-tui/onboarding.md to document decision approval UX
- Add examples of cascading behavior
- Document keybindings

**Success criteria**:
- Documentation is clear and complete
- Examples demonstrate key features

**Dependencies**: Phase 3 complete

### Task 4.2: Run Full Test Suite

**Command**: `cargo test --workspace`

**What to verify**:
- All unit tests pass
- All integration tests pass
- All TUI tests pass
- No clippy warnings
- No compiler warnings

**Success criteria**:
- ZERO warnings or errors
- Test coverage is comprehensive

**Dependencies**: All phases complete

### Task 4.3: Format and Lint

**Commands**:
- `cargo fmt --all`
- `cargo clippy --workspace`

**What to verify**:
- Code follows project style
- No clippy warnings remain

**Success criteria**:
- Clean clippy output
- Consistent formatting

**Dependencies**: All phases complete

---

## Rollout Strategy

Since this feature extends existing functionality without breaking changes:
1. Merge Phase 1 and 2 together (core + integration)
2. Merge Phase 3 separately (TUI) after core is stable
3. No feature flags needed (additive change)
4. No migration required (new data structures)

## Risk Mitigation

**Risk**: Cascading logic causes performance issues with large decision trees
**Mitigation**: Decision approval iterates chunks once, chunk approval checks N decisions where N is typically 1-3. Both O(chunks) which is acceptable.

**Risk**: Reverse cascade triggers unexpectedly
**Mitigation**: Comprehensive integration tests cover edge cases. Only triggers when ALL chunks approved.

**Risk**: Visual clutter in decision tree
**Mitigation**: Progress counts are concise "(X/Y)" format. Approval icons reuse existing visual vocabulary.

## Success Metrics

**Functional**:
- Can approve entire decision with one keystroke
- All chunks cascade correctly
- Reverse cascade auto-completes decisions
- Progress indicators show accurate counts

**Technical**:
- Zero compiler warnings
- Zero clippy warnings
- All tests pass
- Clean integration with existing patterns

**UX**:
- Keybindings are intuitive and consistent
- Visual feedback is immediate and clear
- Navigation remains fluid
