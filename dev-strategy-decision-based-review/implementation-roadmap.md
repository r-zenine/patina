# Implementation Roadmap for Decision-Based Review - Steel Thread Strategy

## Steel Thread Overview

**Target User Journey**: Reviewer opens contribution → sees decision list → selects decision → views rationale + code impacts → reviews code with decision context → switches between decision/file views

**Steel Thread Approach**: Build minimal decision view in TUI with hardcoded data first, then load from files, then generate mappings in dev-contribute.

---

## Phase 1: Decision-First Navigation with Hardcoded Data

**What Works After This Phase**: TUI has decision-based navigation as the primary interface. Users navigate: Decision List → Decision Detail Modal → File View → Chunk Detail. All code (mapped and unmapped) is accessible through decision-based navigation.

**Intentional Shortcuts**:
- Hardcoded decision data in test binary
- No JSON loading (Phase 2)
- Simplified modal for decision detail

**Architecture Decision**: Decision-first navigation replaces file-first navigation. Decisions are the primary navigation concept, not just metadata badges.

### ✅ Design Step: Decision Entity Model (COMPLETED)

**Status**: Implemented with full test coverage

**What Was Delivered**:
- `Decision`, `CodeImpact`, `DecisionLineRange`, `ChangeType`, `Confidence` types
- `ReviewDecisions` collection with overlap-based indexing
- Complete unit test suite covering all overlap scenarios

**Files Modified**:
- `diffviz-review/src/entities/decision.rs` (483 lines, 13 tests)
- `diffviz-review/src/entities/mod.rs` (exports)
- `diffviz-review/src/state/mod.rs` (ReviewState integration)

### ✅ Design Step: ReviewEngine Integration (COMPLETED)

**Status**: Implemented with complete API surface

**What Was Delivered**:
- `set_decisions_with_index()` - automatically builds overlap index
- `get_decisions_for_diff()` - query decisions by ReviewableDiffId
- `get_decision()` - query by number
- `get_all_decisions()` - list all decisions

**Files Modified**:
- `diffviz-review/src/engines/review_engine.rs`

### ✅ Implementation Step: Hardcoded Test Data (COMPLETED)

**Status**: Test binary has 3 decisions with realistic code impacts

**What Was Delivered**:
- Decision 1: Refactor authentication (affects lib.rs, auth.rs)
- Decision 2: Error handling improvements (overlaps with Decision 1)
- Decision 3: Logging infrastructure (no-code decision)

**Files Modified**:
- `diffviz-review-tui/src/main.rs`

---

### Implementation Step: Synthetic Decision 0 for Unmapped Diffs

**Objective**: Ensure all ReviewableDiffs are accessible through decision navigation

**Tasks**:
- Add `create_unmapped_decision()` method to ReviewDecisions
- Identify diffs with no decision mapping after index build
- Create synthetic Decision 0: "Unmapped Changes" with those diffs
- Add to decisions collection automatically

**Files to Modify**:
- `diffviz-review/src/entities/decision.rs`

**Test Strategy**:
- Unit test with mixed mapped/unmapped diffs
- Verify Decision 0 created only when unmapped diffs exist
- Verify Decision 0 contains exactly the unmapped diffs

---

### Implementation Step: DecisionNavigationState

**Objective**: Create navigation state for decision-first hierarchy

**Tasks**:
- Create `DecisionNavigationState` struct (selected_decision, selected_file, selected_chunk)
- Implement navigation methods (next_decision, prev_decision, select_decision, etc.)
- Add to UiState as primary navigation state
- Track modal state for decision detail view

**Files to Modify**:
- `diffviz-review-tui/src/navigation.rs` or new `decision_navigation.rs`
- `diffviz-review-tui/src/state.rs`

**Navigation Hierarchy**:
```
DecisionNavigationState {
  current_level: DecisionLevel | FileLevel | ChunkLevel
  selected_decision: Option<u32>
  selected_file: Option<String>
  selected_chunk: Option<ReviewableDiffId>
  show_decision_modal: bool
}
```

---

### Implementation Step: Decision List Component

**Objective**: Primary TUI view showing all decisions

**Tasks**:
- Create `decision_list.rs` component
- Display: decision number, title, code impact count
- Highlight selected decision
- Keyboard: up/down (navigate), Enter (open modal), right arrow (drill into files)

**Files to Modify**:
- `diffviz-review-tui/src/ui/components/decision_list.rs` (new)
- `diffviz-review-tui/src/ui/components/mod.rs`
- `diffviz-review-tui/src/ui/mod.rs` (wire as primary view)

**Visual Design**:
```
╭─ Decisions ─────────────────────────────────╮
│ ► 1. Refactor authentication module     [2] │
│   2. Improve error handling              [2] │
│   3. Add structured logging              [0] │
│   0. Unmapped Changes                    [1] │
╰─────────────────────────────────────────────╯
```

---

### Implementation Step: Decision Detail Modal

**Objective**: Show decision context when user presses Enter on decision

**Tasks**:
- Create `decision_detail_modal.rs` component
- Display: title, summary, decision_log_line reference
- List all code_impacts with file, line ranges, confidence, reasoning
- Keyboard: Esc (close), Enter on impact (navigate to file)

**Files to Modify**:
- `diffviz-review-tui/src/ui/components/decision_detail_modal.rs` (new)
- `diffviz-review-tui/src/ui/mod.rs` (render modal when active)

**Visual Design**:
```
╭─ Decision 1: Refactor authentication module ─────╮
│ Extract authentication logic into separate,      │
│ testable module                                  │
│                                                  │
│ Code Impacts:                                    │
│ ► src/lib.rs (lines 1-50) [HIGH]                │
│   Main library module imports new auth module   │
│   src/auth.rs (lines 1-100) [HIGH]              │
│   New authentication module with functions      │
│                                                  │
│ [Esc: Close] [Enter: View File]                 │
╰──────────────────────────────────────────────────╯
```

---

### Implementation Step: File View Filtered by Decision

**Objective**: Show only chunks relevant to selected decision

**Tasks**:
- Update FileListComponent to accept decision filter
- Query ReviewEngine for diffs matching selected decision
- Display file with chunks belonging to decision
- Maintain existing chunk detail view

**Files to Modify**:
- `diffviz-review-tui/src/ui/components/file_list.rs`
- `diffviz-review-tui/src/ui/mod.rs`

**Behavior**:
- When decision selected, file view shows only chunks from that decision's code_impacts
- File list grouped by file path
- Existing chunk rendering unchanged

---

### Implementation Step: Wire Navigation Flow

**Objective**: Connect decision → modal → file → chunk navigation

**Tasks**:
- Handle input events for decision list (up/down/Enter/right)
- Handle modal events (Esc/Enter on impact)
- Transition between DecisionLevel → FileLevel → ChunkLevel
- Update event handlers in input.rs

**Files to Modify**:
- `diffviz-review-tui/src/events/input.rs`
- `diffviz-review-tui/src/state.rs` (state transitions)

**Navigation Bindings**:
- **Decision List**: up/down (navigate), Enter (modal), right (drill to files)
- **Decision Modal**: up/down (select impact), Enter (go to file), Esc (close)
- **File View**: up/down (navigate chunks), Enter (chunk detail), left (back to decisions)
- **Chunk Detail**: existing bindings, left (back to file list)

---

**Success Criteria**:
- TUI launches with decision list as primary view
- User can navigate decisions with keyboard
- Decision modal shows context and code impacts
- Drilling into decision shows filtered file/chunk view
- Unmapped diffs accessible through Decision 0
- All existing chunk-level actions work (comment, approve)

---

## Phase 2: Decision Navigation and JSON Loading

**Building On**: Phase 1 decision display with hardcoded data

**What Works After This Phase**: User can toggle between file view and decision view. Decision view lists decisions, selecting one shows its code impacts. JSON mapping loaded from file.

### Implementation Step: JSON Loader

**Objective**: Load decision-to-code-mapping.json from file

**Tasks**:
- Add serde derives to Decision/CodeImpact structs
- Create loader function in diffviz-review
- Add DecisionMapping struct matching JSON schema
- Parse and build DecisionIndex from loaded data

**Files to Modify**:
- `diffviz-review/src/entities/decision.rs`
- `diffviz-review/src/lib.rs` (export loader)

### Implementation Step: DecisionNavigationState

**Objective**: Add navigation state for decision view

**Tasks**:
- Create DecisionNavigationState (selected decision, expanded state)
- Add to UiState alongside existing NavigationState
- Add view mode toggle (File/Decision)

**Files to Modify**:
- `diffviz-review-tui/src/navigation.rs` or new file
- `diffviz-review-tui/src/state.rs`

### Implementation Step: Decision List Panel

**Objective**: Create UI component for decision list

**Tasks**:
- New decision_list.rs component
- Display decision number, title, code impact count
- Keyboard navigation (up/down/enter)

**Files to Modify**:
- `diffviz-review-tui/src/ui/components/decision_list.rs` (new)
- `diffviz-review-tui/src/ui/components/mod.rs`

### Implementation Step: Decision Detail View

**Objective**: Show decision details when selected

**Tasks**:
- Display decision summary and reasoning
- List all code impacts with file paths
- Allow jumping to specific code impact

**Files to Modify**:
- `diffviz-review-tui/src/ui/components/decision_view.rs` (new)

### Implementation Step: View Toggle

**Objective**: Allow switching between file and decision views

**Tasks**:
- Add keybinding for view toggle (e.g., Tab or 'd')
- Update draw function to render correct view
- Maintain selection state when switching

**Files to Modify**:
- `diffviz-review-tui/src/events/input.rs`
- `diffviz-review-tui/src/ui/mod.rs`

**Success Criteria**:
- JSON file loads successfully
- User can toggle between file/decision views
- Decision view shows list → detail → code navigation
- All Phase 1 functionality preserved

---

## Phase 3: dev-contribute Integration

**Building On**: Phases 1-2 with working TUI and JSON loading

**What Works After This Phase**: dev-contribute skill generates decision-to-code-mapping.json automatically. Full end-to-end: contribution → mapping → review.

### Implementation Step: Mapping Generator Module

**Objective**: Create mapping generation logic in dev-contribute

**Tasks**:
- Parse decision-log.md for decision numbers/titles
- Use git diff to identify changed files
- Map decisions to function-level code impacts
- Generate confidence scores based on context-handoff references

**Files to Modify**:
- `agent-skills/skills/dev-contribute/reference.md` (add Step 5.5)

### Implementation Step: Update Skill Templates

**Objective**: Add decision-to-code-mapping.json to mandatory artifacts

**Tasks**:
- Update SKILL.md to list 4 mandatory artifacts
- Create mapping template or generation instructions
- Update contribution validation checklist

**Files to Modify**:
- `agent-skills/skills/dev-contribute/SKILL.md`
- `agent-skills/skills/dev-contribute/reference.md`

### Implementation Step: End-to-End Validation

**Objective**: Verify full workflow works

**Tasks**:
- Create test contribution with dev-contribute
- Verify mapping file generated correctly
- Load in DiffViz TUI and validate display

**Success Criteria**:
- dev-contribute generates valid mapping file
- DiffViz loads and displays decision view
- Complete reviewer workflow functional

---

## Steel Thread Principles Applied

**Always Working**: Each phase maintains functional TUI with mock/test data
**Incremental Value**: Phase 1 = see decisions, Phase 2 = navigate decisions, Phase 3 = auto-generate
**User-Focused**: Every phase delivers observable improvement to review experience
**Minimal Viable Slices**: Hardcoded → file-loaded → auto-generated
