# Implementation Roadmap for Decision-Based Review - Steel Thread Strategy

## Steel Thread Overview

**Target User Journey**: Reviewer opens contribution → sees decision list → selects decision → views rationale + code impacts → reviews code with decision context → switches between decision/file views

**Steel Thread Approach**: Build minimal decision view in TUI with hardcoded data first, then load from files, then generate mappings in dev-contribute.

---

## Phase 1: Minimal Decision View with Hardcoded Data

**What Works After This Phase**: TUI shows hardcoded decisions alongside file list. User can see decision numbers but navigation is file-based only.

**Intentional Shortcuts**:
- Hardcoded decision data in test binary
- No decision navigation (display only)
- No JSON loading

### Design Step: Decision Entity Model

**Objective**: Agree on data structures for decisions in diffviz-review

**Design Questions to Resolve**:
- Decision entity structure (number, title, summary, code_impacts)
- CodeImpact structure (file, line_ranges, confidence, reasoning)
- DecisionIndex type: `HashMap<ReviewableDiffId, Vec<DecisionNumber>>`
- Where to store: new `src/entities/decision.rs` or extend ReviewState?

**Files to Analyze**:
- `diffviz-review/src/entities/comment.rs` - Entity pattern reference
- `diffviz-review/src/state/mod.rs` - ReviewState structure

**Output**: Agreed data structures documented in contribution decision-log

### Design Step: ReviewEngine Integration

**Objective**: Agree on how ReviewEngine exposes decision context

**Design Questions to Resolve**:
- How does ReviewEngine receive decision mapping? (constructor param vs setter)
- API for querying decisions by ReviewableDiffId
- API for querying code impacts by decision number

**Files to Analyze**:
- `diffviz-review/src/engines/review_engine.rs` - Current API surface

**Output**: Agreed API additions documented in contribution decision-log

### Design Step: TUI Decision Display

**Objective**: Agree on UiState changes for decision awareness

**Design Questions to Resolve**:
- How to display decision labels in file view (badge? prefix?)
- New view mode enum variant for decision view?
- DecisionNavigationState structure (parallel to NavigationState?)

**Files to Analyze**:
- `diffviz-review-tui/src/state.rs` - UiState structure
- `diffviz-review-tui/src/navigation.rs` - NavigationState pattern

**Output**: Agreed UI patterns documented in contribution decision-log

### Implementation Step: Decision Entities

**Objective**: Implement agreed decision data structures

**Tasks**:
- Create `diffviz-review/src/entities/decision.rs`
- Add Decision, CodeImpact, DecisionIndex types
- Add decision_index field to ReviewState
- Export from entities/mod.rs

**Files to Modify**:
- `diffviz-review/src/entities/decision.rs` (new)
- `diffviz-review/src/entities/mod.rs`
- `diffviz-review/src/state/mod.rs`

### Implementation Step: ReviewEngine Decision API

**Objective**: Add decision query methods to ReviewEngine

**Tasks**:
- Add method to set decision mapping
- Add method to get decisions for a ReviewableDiffId
- Add method to get code impacts for a decision number

**Files to Modify**:
- `diffviz-review/src/engines/review_engine.rs`

### Implementation Step: Hardcoded Test Data

**Objective**: Add hardcoded decision mapping to test binary

**Tasks**:
- Create sample Decision data matching mock fixtures
- Wire into ReviewEngine in main.rs
- Verify data accessible via API

**Files to Modify**:
- `diffviz-review-tui/src/main.rs`

### Implementation Step: Decision Labels in File View

**Objective**: Display decision context in existing file-based view

**Tasks**:
- Query decision_index when rendering file list
- Show decision numbers as labels/badges on files
- No navigation changes yet

**Files to Modify**:
- `diffviz-review-tui/src/ui/components/file_list.rs`

**Success Criteria**:
- TUI launches with mock data
- File list shows decision number badges
- Hardcoded decisions visible in display

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
