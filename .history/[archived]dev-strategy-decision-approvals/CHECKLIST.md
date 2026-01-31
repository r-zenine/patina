# Implementation Checklist: Decision Approval System

Use this checklist to track progress through the implementation phases.

## Phase 1: Core Domain Logic (diffviz-review)

### Task 1.1: Create DecisionApproval Entity
- [ ] Add DecisionApproval struct to `diffviz-review/src/entities/decision.rs`
  - [ ] Fields: decision_number (u32), approved (bool), approved_by (String), approval_timestamp (String)
  - [ ] Add Serialize/Deserialize derives
- [ ] Add DecisionApprovals collection struct
  - [ ] HashMap<u32, DecisionApproval> field
  - [ ] new() constructor
- [ ] Implement methods:
  - [ ] approve(decision_number, approved_by, timestamp)
  - [ ] unapprove(decision_number)
  - [ ] is_approved(decision_number) -> bool
  - [ ] get_approval(decision_number) -> Option<&DecisionApproval>

### Task 1.2: Add Decision Approval Queries
- [ ] Add to DecisionApprovals:
  - [ ] total_approved() -> usize
  - [ ] approval_percentage(total_decisions: usize) -> f32

### Task 1.3: Update Module Exports
- [ ] Export DecisionApproval from `diffviz-review/src/entities/mod.rs`
- [ ] Export DecisionApprovals from `diffviz-review/src/entities/mod.rs`

### Task 1.4: Write Core Entity Tests
- [ ] Test approval lifecycle (approve → is_approved → unapprove)
- [ ] Test serialization/deserialization round-trip
- [ ] Test approval counting (total_approved)
- [ ] Test percentage calculation
- [ ] Test edge cases (empty collection, approve twice)
- [ ] All tests pass

### Phase 1 Verification
- [ ] `cargo build --package diffviz-review` succeeds
- [ ] `cargo test --package diffviz-review` passes
- [ ] `cargo clippy --package diffviz-review` shows zero warnings
- [ ] `cargo fmt --all` applied

---

## Phase 2: Integration with Review Engine (diffviz-review)

### Task 2.1: Extend ReviewState
- [ ] Add decision_approvals field to ReviewState in `diffviz-review/src/state/mod.rs`
- [ ] Initialize in ReviewState::new()
- [ ] Add query methods:
  - [ ] is_decision_approved(&self, decision_number: u32) -> bool
  - [ ] decision_approval_progress(&self, decision_number: u32) -> (usize, usize)
- [ ] Add mutation methods (return &mut Self):
  - [ ] approve_decision(&mut self, decision_number: u32, approved_by: String) -> &mut Self
  - [ ] unapprove_decision(&mut self, decision_number: u32) -> &mut Self

### Task 2.2: Implement Decision Approval Progress
- [ ] Implement decision_approval_progress() logic
  - [ ] Get all chunks for decision via decision_index
  - [ ] Count approved chunks
  - [ ] Return (approved_count, total_count)
- [ ] Handle decision not found (return (0, 0))
- [ ] Write unit test for progress calculation

### Task 2.3: Add ReviewEngine Decision Approval Methods
- [ ] Add to `diffviz-review/src/engines/review_engine.rs`:
  - [ ] approve_decision(&mut self, decision_number: u32, author: String) -> Result<()>
  - [ ] reject_decision(&mut self, decision_number: u32) -> Result<()>
  - [ ] is_decision_approved(&self, decision_number: u32) -> bool
- [ ] Implement cascading in approve_decision():
  - [ ] Approve decision via state.approve_decision()
  - [ ] Get chunks via get_chunks_for_decision()
  - [ ] Loop: approve each chunk
  - [ ] Invalidate renderable cache
- [ ] Implement cascading in reject_decision():
  - [ ] Unapprove decision via state.unapprove_decision()
  - [ ] Get chunks via get_chunks_for_decision()
  - [ ] Loop: reject each chunk
  - [ ] Invalidate renderable cache

### Task 2.4: Implement Reverse Cascade
- [ ] Modify approve() method in review_engine.rs:
  - [ ] After approving chunk, get decisions via get_decisions_for_diff()
  - [ ] For each decision, check if all chunks approved
  - [ ] If yes, auto-approve decision
- [ ] Modify reject() method:
  - [ ] After rejecting chunk, get decisions for chunk
  - [ ] If decision was approved, unapprove it

### Task 2.5: Add Helper Method
- [ ] Add private helper to ReviewEngine:
  - [ ] get_chunks_for_decision(&self, decision_number: u32) -> Vec<ReviewableDiffId>
  - [ ] Query decision_index to find all chunks mapped to decision
- [ ] Use in approve_decision(), reject_decision(), and reverse cascade

### Task 2.6: Write Integration Tests
- [ ] Create `diffviz-review/tests/decision_approval_integration.rs`
- [ ] Test cascading approval: approve decision → all chunks approved
- [ ] Test cascading rejection: reject decision → all chunks unapproved
- [ ] Test reverse cascade: approve all chunks → decision auto-approved
- [ ] Test partial approval: approve some chunks → decision not approved
- [ ] Test progress calculation at various states
- [ ] All integration tests pass

### Phase 2 Verification
- [ ] `cargo build --package diffviz-review` succeeds
- [ ] `cargo test --package diffviz-review` passes (unit + integration)
- [ ] `cargo clippy --package diffviz-review` shows zero warnings
- [ ] `cargo fmt --all` applied
- [ ] Cascading logic works correctly
- [ ] Reverse cascade triggers as expected

---

## Phase 3: TUI Integration (diffviz-review-tui)

**IMPORTANT: Use the `diffviz-review-tui-contribution` skill for this phase**

### Task 3.1: Add Decision Approval Business Events
- [ ] Add to `diffviz-review-tui/src/events/business.rs`:
  - [ ] ToggleApproveDecision(u32) variant to BusinessEvent enum
- [ ] Update ui_event_to_business_event():
  - [ ] When UiEvent::ToggleApprove at depth 0
  - [ ] Extract decision number from navigation state
  - [ ] Return BusinessEvent::ToggleApproveDecision(decision_number)

### Task 3.2: Handle Decision Approval in App
- [ ] Add match arm in `diffviz-review-tui/src/app.rs`:
  - [ ] BusinessEvent::ToggleApproveDecision(decision_number)
  - [ ] Query is_decision_approved()
  - [ ] If approved: call reject_decision()
  - [ ] If not approved: call approve_decision()
  - [ ] Return Command::None

### Task 3.3: Add Navigation Helper
- [ ] Add to `diffviz-review-tui/src/decision_navigation.rs`:
  - [ ] get_current_decision_number(&self) -> Option<u32>
  - [ ] Return Some(number) when at depth 0
  - [ ] Return None at depth 1 or 2

### Task 3.4: Update Diff View Display
- [ ] Modify `diffviz-review-tui/src/ui/components/diff_view.rs`:
  - [ ] Add depth 0 rendering branch
  - [ ] Query is_decision_approved()
  - [ ] Query decision_approval_progress()
  - [ ] Display format: "[✓] Decision #X: Title (A/T)"
  - [ ] Use Icons::APPROVED and Icons::PENDING

### Task 3.5: Add Approval Indicators to Decision Tree
- [ ] Modify `diffviz-review-tui/src/ui/components/decision_tree.rs`:
  - [ ] For each decision node, query approval state
  - [ ] Query approval progress
  - [ ] Render: "✓ Decision #1 (5/5)" or "○ Decision #2 (2/5)"

### Task 3.6: Add Keybinding
- [ ] Update `diffviz-review-tui/src/ui/components/which_key.rs`:
  - [ ] Extend Space+a menu with 'd' option
  - [ ] Label: "Toggle approve decision"
  - [ ] Only show when depth == 0

### Task 3.7: Write TUI Tests
- [ ] Create `diffviz-review-tui/tests/decision_approval_tests.rs`
- [ ] Use InputTestHarness:
  - [ ] Test navigation to decision + Space+a+d
  - [ ] Assert decision approved
  - [ ] Test toggle (Space+a+d again)
  - [ ] Assert decision unapproved
- [ ] Use RenderTestHarness:
  - [ ] Assert approval icon changes
  - [ ] Assert progress count updates
- [ ] Test cascading via chunk approvals:
  - [ ] Approve all chunks individually
  - [ ] Assert decision auto-approves
  - [ ] Assert progress shows (N/N)
- [ ] All TUI tests pass

### Phase 3 Verification
- [ ] `cargo build --package diffviz-review-tui` succeeds
- [ ] `cargo test --package diffviz-review-tui` passes
- [ ] `cargo clippy --package diffviz-review-tui` shows zero warnings
- [ ] `cargo fmt --all` applied
- [ ] Keybindings work as expected
- [ ] Visual indicators display correctly
- [ ] Manual testing: run TUI and verify UX

---

## Phase 4: Final Polish

### Task 4.1: Update Documentation
- [ ] Update `diffviz-review/onboarding.md`:
  - [ ] Document DecisionApproval entity
  - [ ] Document cascading behavior
  - [ ] Add usage examples
- [ ] Update `diffviz-review-tui/onboarding.md`:
  - [ ] Document decision approval keybindings
  - [ ] Document visual indicators
  - [ ] Explain cascading UX

### Task 4.2: Run Full Test Suite
- [ ] `cargo test --workspace` passes
- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] All TUI tests pass

### Task 4.3: Format and Lint
- [ ] `cargo fmt --all` applied to entire workspace
- [ ] `cargo clippy --workspace` shows zero warnings
- [ ] `cargo check --workspace` succeeds
- [ ] No compiler warnings anywhere

### Final Verification
- [ ] ZERO WARNINGS RULE satisfied
- [ ] All tests pass
- [ ] Documentation updated
- [ ] Code follows existing patterns
- [ ] Manual TUI testing completed
- [ ] Ready for review

---

## Success Criteria Summary

### Functional
- [x] Can approve entire decision with Space+a+d
- [x] Approving decision cascades to all chunks
- [x] Approving all chunks auto-completes decision (reverse cascade)
- [x] Unapproving decision cascades to chunks
- [x] Progress indicators show accurate counts

### Technical
- [x] Zero compiler warnings
- [x] Zero clippy warnings
- [x] All tests pass (unit + integration + TUI)
- [x] Follows existing architectural patterns
- [x] Clean integration with ReviewState and ReviewEngine

### UX
- [x] Keybindings are intuitive (Space+a+d)
- [x] Visual feedback is immediate and clear
- [x] Progress counts are accurate "(X/Y)"
- [x] Approval icons use existing theme (✓/○)
- [x] Navigation remains fluid

---

## Notes

- Read onboarding.md files before modifying any crate
- Follow existing patterns exactly (mirror Approval → DecisionApproval)
- Use diffviz-review-tui-contribution skill for Phase 3
- Test after each task completion
- Run `cargo fmt` and `cargo clippy` frequently
- No backwards compatibility needed (additive change)
