# Development Strategy: Decision Approval System

## Overview

Extend DiffViz's approval system to support decision-level approvals with cascading behavior. Currently, approvals exist only at the chunk (ReviewableDiff) level. This feature adds the ability to approve entire architectural decisions, which automatically propagates to all associated chunks.

## What This Adds

**User Capabilities:**
- Approve/reject entire decisions with one action (Space+a+d)
- See decision approval progress in tree view (e.g., "Decision #1 (3/5)")
- Auto-complete decisions when all chunks are approved (reverse cascade)
- Symmetrical approval/unapproval behavior

**Technical Capabilities:**
- DecisionApproval entity mirroring existing Approval pattern
- Cascading approval logic (decision → chunks, chunks → decision)
- Progress calculation (approved/total chunks per decision)
- TUI integration with visual indicators

## Implementation Strategy

**Approach**: Core-then-Integrate

1. **Phase 1: Core Domain Logic** - Build DecisionApproval entity and collections in diffviz-review
2. **Phase 2: Integration** - Wire into ReviewEngine with cascading logic
3. **Phase 3: TUI** - Add events, keybindings, and visual components (use diffviz-review-tui-contribution skill)

## Key Documents

- **context-document.md** - Behavioral spec, architecture summary, design constraints
- **decision-log.md** - User requirements and technical decisions with rationale
- **code-context.md** - Relevant files, code patterns, integration points
- **implementation-roadmap.md** - Phased task breakdown with dependencies

## User Requirements (Confirmed)

1. ✓ Cascading approval: Approving decision → approves all chunks
2. ✓ Symmetrical unapproval: Unapproving decision → unapproves all chunks
3. ✓ Reverse cascade: All chunks approved → decision auto-approves
4. ✓ Progress indicator: Show "X/Y chunks" in TUI tree view

## Technical Decisions

- **Storage**: Separate DecisionApproval entity, HashMap<u32, DecisionApproval> indexed by decision number
- **Cascading**: Implemented in ReviewEngine, not entities (orchestration pattern)
- **Progress**: Calculate on-demand, don't cache (YAGNI principle)
- **Keybinding**: Space+a+d for decision approval (extends existing leader key menu)
- **Visual**: Icon + progress count at depth 0 (decision level)

## Development Phases

### Phase 1: Core (diffviz-review)
**Files**: entities/decision.rs, state/mod.rs, entities/mod.rs

**Tasks**:
- Create DecisionApproval and DecisionApprovals (mirror Approval pattern)
- Add decision_approvals to ReviewState
- Implement query methods (is_decision_approved, decision_approval_progress)
- Write unit tests

**Estimated Complexity**: Low - follows existing patterns exactly

### Phase 2: Integration (diffviz-review)
**Files**: engines/review_engine.rs, tests/

**Tasks**:
- Add approve_decision() and reject_decision() methods
- Implement cascading logic (decision → chunks)
- Implement reverse cascade (chunks → decision)
- Add helper get_chunks_for_decision()
- Write integration tests for cascading scenarios

**Estimated Complexity**: Medium - cascading logic requires care

### Phase 3: TUI (diffviz-review-tui) - USE SKILL
**Files**: events/business.rs, app.rs, decision_navigation.rs, ui/components/diff_view.rs, ui/components/decision_tree.rs, ui/components/which_key.rs

**Tasks**:
- Add ToggleApproveDecision business event
- Handle event in app.rs
- Add get_current_decision_number() navigation helper
- Update diff_view for depth 0 approval display
- Add approval indicators to decision tree
- Add Space+a+d keybinding
- Write TUI test harness tests

**Estimated Complexity**: Medium - TUI integration with visual components

**IMPORTANT**: Use the `diffviz-review-tui-contribution` skill for Phase 3

## Testing Strategy

**Unit Tests**: Entity serialization, approval logic, query methods
**Integration Tests**: Cascading scenarios, reverse cascade, progress calculation
**TUI Tests**: Keyboard interactions (InputTestHarness), visual rendering (RenderTestHarness)

## Success Criteria

**Functional**:
- ✓ One keystroke approves entire decision
- ✓ Approvals cascade to all chunks
- ✓ Auto-complete when all chunks approved
- ✓ Accurate progress indicators

**Technical**:
- ✓ Zero compiler/clippy warnings
- ✓ All tests pass
- ✓ Follows existing architectural patterns
- ✓ No backwards compatibility concerns

**UX**:
- ✓ Intuitive keybindings (Space+a+d)
- ✓ Clear visual feedback (icons + counts)
- ✓ Consistent with existing approval UX

## Quick Start

1. Read **context-document.md** for behavioral spec and architecture
2. Review **decision-log.md** for requirements and technical decisions
3. Check **code-context.md** for code patterns and file locations
4. Follow **implementation-roadmap.md** task-by-task

## Notes

- No backwards compatibility needed (additive change)
- No feature flags required
- No migration logic needed
- Follows YAGNI principle throughout
- Uses onboarding.md files as source of truth for architecture
