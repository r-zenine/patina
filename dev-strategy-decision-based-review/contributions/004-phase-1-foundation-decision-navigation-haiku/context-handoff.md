# Context Handoff - Phase 1 Foundation: Decision Navigation State

## What I Built & Why

Implemented **DecisionNavigationState** - the navigation layer that will drive the decision-first TUI. This is the bridge between the decision entity model (001-003) and the visual components that let users navigate decisions, see their details, and drill into affected code.

**The Core Problem**: We have decision entities indexed by ReviewableDiffId and hardcoded test data, but no way to navigate them in the TUI. Navigation state tracks "where is the user now" - which decision are they viewing, which file, which chunk. Without this, every keystroke would need to determine position by scanning all decisions.

**Why This Phase**: This is the Steel Thread Foundation phase - establishing navigation infrastructure before building visual components. The state machine is tested and working before the first TUI line renders. This lets components focus solely on drawing what the state tells them to draw.

## What's Ready to Hand Off

### Core Component Ready
- ✅ `DecisionNavigationState` struct fully implemented in `diffviz-review-tui/src/decision_navigation.rs`
- ✅ Integrated into `UiState` as `decision_nav` field
- ✅ 15 unit tests covering all navigation scenarios (100% pass rate)
- ✅ Zero regressions in existing test suite (132 tests still pass)
- ✅ Full workspace builds successfully

### Navigation Hierarchy Implemented
```
current_level: NavigationLevel
├── Decision (viewing list of all decisions)
├── File (viewing files affected by selected decision)
└── Chunk (viewing specific chunk in selected file)

Overlays:
└── show_decision_modal: bool (decision detail modal)
```

### Public API Ready for TUI Components
```rust
// Navigation changes
state.next_decision()              // Move to next decision in list
state.prev_decision()              // Move to previous decision
state.clamp_decision_index(max)    // Ensure valid index
state.select_decision(number)      // Select decision and show modal
state.close_decision_modal()       // Hide modal
state.drill_into_files()           // Enter file view for decision
state.back_to_decisions()          // Return to decision list

// Queries
state.at_decision_list()           // Are we at decision level?
state.at_file_view()               // Are we viewing files?
state.at_chunk_view()              // Are we viewing chunks?
state.current_decision()           // What decision is selected?

// Helper functions
get_files_for_decision(engine, num)          // List files affected by decision
get_chunks_for_file_in_decision(engine, num, path)  // List chunks in file for decision
```

### Data Structure Ready
```rust
pub struct DecisionNavigationState {
    pub current_level: NavigationLevel,           // Where in hierarchy
    pub selected_decision: Option<u32>,           // Which decision
    pub selected_file: Option<String>,            // Which file (in File level)
    pub selected_chunk: Option<ReviewableDiffId>, // Which chunk (in Chunk level)
    pub show_decision_modal: bool,                // Modal overlay visible?
    pub decision_list_index: usize,               // Scroll position in decision list
    pub file_list_index: usize,                   // Scroll position in file list
}
```

## Routes Explored & Discarded

### ❌ Unified navigation state with existing NavigationState
**Why Considered**: Reuse existing navigation infrastructure
**Why Discarded**: Decision-first hierarchy (Decision → File → Chunk) is fundamentally different from file-first hierarchy (FileHeader → DiffItem). Trying to unify would create complex conditional logic throughout TUI. Better to keep both and let TUI choose which to use.
**Evidence**: decision_log.md explains architectural rationale

### ❌ Caching decision-to-file mappings in navigation state
**Why Considered**: Faster queries, avoid repeated lookups
**Why Discarded**: ReviewEngine already has the data; caching would duplicate structure and require invalidation logic. Phase 1 is MVP with small decision counts (typically 3-10). O(n) lookup is fast enough. Can optimize if profiling shows issue.
**Evidence**: Iteration-based queries in get_files_for_decision()

### ❌ Circular list navigation (wrap around at boundaries)
**Why Considered**: Smoother UX, matches some TUI patterns
**Why Discarded**: MVP simplicity. Matches vim/less convention of stopping at boundaries. Users can press 'g' to jump to top if needed. Can be added later if UX testing shows need.
**Evidence**: move_up/move_down use saturating_sub, not modulo

### ❌ Implicit state machine (no NavigationLevel enum)
**Why Considered**: Fewer types, simpler struct
**Why Discarded**: Harder to reason about. Enum makes state explicit and prevents invalid combinations (e.g., selected_chunk without being at ChunkLevel). Tests are clearer. UX errors impossible.
**Evidence**: Enum approach enables guard conditions like drill_into_files()

### ❌ Modal as separate navigation level
**Why Considered**: Consistent with main view levels
**Why Discarded**: Modal is an overlay, not a destination. Users drill from modal to files, not navigate to "modal level". Simpler state machine with modal as boolean flag. Matches common modal UI patterns.
**Evidence**: show_decision_modal is boolean, not a NavigationLevel variant

## Architecture Decisions Made

### 1. **Separate Navigation State Initialization Pattern**
```rust
pub fn new() -> Self {
    Self {
        current_level: NavigationLevel::Decision,
        selected_decision: None,
        // ... all fields initialized
    }
}
```
**Rationale**: Clean, explicit initialization. All fields always have defined value (no Option for position). Prevents undefined state bugs.
**Future Implication**: Building selection_map in components should check state.at_decision_list() before rendering

### 2. **Index Clamping Responsibility Split**
- Navigation state provides `clamp_decision_index(max)` method
- Caller (event handler or component) calls this after querying decision count
- Navigation state itself doesn't know decision count

**Rationale**: State doesn't know data. Event handler queries engine, asks state to validate index.
**Future Implication**: Event handler must call clamp after any action that might invalidate index (delete decision, etc.)

### 3. **Modal as Computed Property, Not State**
Decision detail modal content comes from:
- `selected_decision` (which decision's data to show)
- `show_decision_modal` (whether to render)

**Rationale**: Modal data is always computable from decision number + ReviewEngine
**Future Implication**: Modal component can be stateless: takes `show` flag and `decision_number`, queries engine for content

### 4. **Guard Conditions for Invalid Transitions**
```rust
pub fn drill_into_files(&mut self) {
    if self.selected_decision.is_some() {
        self.current_level = NavigationLevel::File;
        // ... transition
    }
    // Silent fail if no decision selected
}
```
**Rationale**: TUI can't accidentally reach invalid state. Simpler than Result<()> error handling.
**Future Implication**: TUI keybindings should check preconditions before calling state methods, or check if action took effect after

### 5. **Separate Index Fields Per Level**
- `decision_list_index: usize` - position when at Decision level
- `file_list_index: usize` - position when at File level

**Rationale**: Each level has different list length. Separate tracking preserves position when drilling down and returning.
**Future Implication**: Components render based on correct index field: `if state.at_decision_list() { use decision_list_index } else { use file_list_index }`

## For Next Contributor (Decision List Component)

### Your Task
Build the first visual component: **DecisionListComponent** that renders the decision list and handles selection highlighting.

### What You Inherit
✅ Navigation state fully implemented and tested
✅ ReviewEngine can query all decisions
✅ state.decision_list_index and state.decision_list_max track position
✅ UiState.decision_nav accessible from app state
✅ Keyboard input event handling already exists, just needs to wire to decision_nav methods

### Implementation Path
1. **Create DecisionListComponent**:
   - Accept current state (decisions list, selected index)
   - Render each decision as a line with number and title
   - Highlight selected decision with marker or color
   - Show code impact count: `decision.code_impacts.len()`

2. **Visual Design** (from implementation-roadmap.md):
   ```
   ╭─ Decisions ─────────────────────────────╮
   │ ► 1. Refactor authentication     [2] │
   │   2. Improve error handling       [2] │
   │   3. Add structured logging       [0] │
   │   0. Unmapped Changes             [1] │
   ╰──────────────────────────────────────────╯
   ```

3. **Component Signature**:
   ```rust
   pub fn draw_decision_list(
       f: &mut Frame,
       area: Rect,
       decisions: &[&Decision],
       selected_index: usize,
   )
   ```

4. **Integration Point**:
   - In main draw() function, check `ui_state.decision_nav.at_decision_list()`
   - If true, render decision_list component
   - Pass `review_engine.get_all_decisions()`, `ui_state.decision_nav.decision_list_index`

### Key APIs You'll Need
```rust
review_engine.get_all_decisions()    // Get all decisions (in order)
review_engine.get_decision(number)   // Get specific decision details
decision.code_impacts.len()          // Show number of affected diffs
decision.title                        // Display decision title
decision.number                       // Display decision number (0 for unmapped)
```

### Keyboard Integration You'll Need
- Up arrow: `decision_nav.prev_decision()`
- Down arrow: `decision_nav.next_decision()`
- Enter: `decision_nav.select_decision(decision_number)` to open modal
- Right arrow: `decision_nav.drill_into_files()` to see files
- Escape: Exit application (existing handling)

### Common Pitfalls to Avoid
❌ Don't hardcode decision count - use `decisions.len()`
❌ Don't assume Decision 0 always exists - check with `get_decision(0)`
❌ Don't assume decisions are contiguous (1,2,3) - use actual decision numbers
❌ Don't render unmapped Decision 0 with special styling - treat same as other decisions (order it specially if needed)
❌ Don't update navigation state in render function - only in event handlers

### Tests You Can Assume Pass
- All DecisionNavigationState tests (15/15 passing)
- All ReviewEngine decision queries (working)
- All Decision entity tests (from contribution 001)
- All overlap indexing tests (from contribution 002)
- All unmapped decision tests (from contribution 003)

### Performance Notes
- get_all_decisions() is O(1) - already cached
- Drawing 10 decisions is instant - no optimization needed yet
- File count lookup is O(n) per decision if rendering multiple - acceptable for MVP

## Critical Insights

### Navigation State as Thin Layer
DecisionNavigationState is purposefully simple - it tracks position and validity, nothing more. It doesn't:
- Know about ReviewEngine or Decision data
- Cache query results
- Validate against actual decision count
- Enforce business rules

This is a feature, not a limitation. It makes state predictable and testable. Components and event handlers handle the business logic.

### Tests as Documentation
The 15 unit tests serve as spec and examples:
- `test_drill_into_files()` shows: select decision → call drill_into_files → check level changed
- `test_drill_into_files_requires_decision()` shows: must have selected_decision to drill
- `test_back_to_decisions()` shows: resets file selection and clears file_list_index

Read them to understand expected behavior before building components.

### Future: TUI State Machine
This navigation state will eventually become the complete TUI state machine. Eventually:
- Original NavigationState deprecated
- DecisionNavigationState as primary `ui_state.navigation`
- Modal overlay pattern generalized
- Input handler routes all keys through state methods

For now, both coexist. Choose DecisionNavigationState for new features.

## Assumptions Made

1. **DecisionCount Stable**: Decision count doesn't change during session (not dynamic)
2. **ReviewState Immutable During Review**: ReviewableDiffs not added/removed during navigation
3. **Decision Numbers Unique**: No two decisions with same number
4. **Decisions Ordered**: get_all_decisions() returns decisions in number order (0, 1, 2, ...)
5. **No Circular Lists Yet**: Navigation stops at boundaries (can add wrap-around later)

## Caveats & Limitations

1. **O(n) Queries**: get_chunks_for_file_in_decision() scans all diffs - acceptable for MVP
   - Fix: Cache in ReviewEngine if profiling shows issue

2. **No Position Persistence**: Navigation position lost when loading new review
   - Expected: Users want fresh start with new review
   - Fix: Could save/restore position if needed later

3. **No Undo**: State changes aren't reversible
   - Acceptable: Navigation changes are lightweight, not data changes
   - Fix: Could implement undo if needed, probably not worth it

4. **Modal Appears Instantly**: No fade-in animation
   - Expected for TUI
   - Fix: Ratatui can do animations if desired

5. **No Hotkey Customization**: Key bindings are hardcoded in event handler
   - Acceptable for MVP
   - Fix: Could load from config if needed

## Implementation Status: COMPLETE ✅

All functionality implemented, tested, and validated:
- ✅ DecisionNavigationState struct fully defined
- ✅ 15 unit tests cover all navigation scenarios (100% pass rate)
- ✅ Integrated into UiState with default initialization
- ✅ Helper functions for querying decisions and files
- ✅ Zero clippy warnings, full workspace builds
- ✅ Zero regressions in existing test suite

**Ready for Phase 1 TUI implementation with Decision List Component as next step.**

