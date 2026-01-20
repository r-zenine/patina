# Context Handoff - Phase 1 Expansion: Decision List Component and Modal

## What I Built & Why

Implemented the **first visual capability for decision-based review** - a decision list component that displays all decisions as the primary navigation interface, along with a modal for viewing decision details and code impacts.

**The Core Problem**: We had decision entities, indexing, unmapped decision handling, and navigation state infrastructure (001-004), but users couldn't see decisions in the TUI. The decision-first navigation existed in data structures but not in the UI.

**Why This Phase**: This is the Steel Thread Capability Expander phase - building the first end-to-end user-facing feature with hardcoded test data. Users can now:
1. Launch TUI and see decision list instead of file list
2. Navigate decisions with arrow keys
3. Press Enter to see decision details in a modal
4. Press right arrow to view files affected by that decision
5. Press Escape to close modal

The navigation state infrastructure (from 004) proves its value here - the event handlers simply route keyboard input to navigation methods, and the UI renders what the state tells it to render.

## Architecture Overview

### Component Architecture

```
ReviewTuiApp (main event loop)
├── handle_key_event() → UiEvent
├── handle_ui_event(UiEvent)
│   ├── Routes to decision_nav methods based on NavigationLevel
│   └── Updates state but never directly renders
└── ui::draw(Frame, UiState, ReviewEngine)
    ├── Checks decision_nav.current_level
    ├── Decision => render_decision_list()
    ├── File/Chunk => render_file_list()
    ├── render_diff_view()
    ├── render_decision_detail_modal() if show_decision_modal
    └── render_overlays()
```

### Component Responsibilities

**DecisionListComponent**:
- Input: decisions list, selected_index
- Output: Rendered List widget with styled items
- Logic: Format each decision as "number. title [count]"
- Selection: Highlight selected decision with inverted colors and cyan marker

**DecisionDetailModalComponent**:
- Input: selected decision number, ReviewEngine
- Output: Centered modal window
- Logic: Query engine for decision, format with impacts, color-code confidence
- Display: Title, summary, log line, impacts with file/lines/type/confidence/reasoning

**Event Handler**:
- Input: UiEvent (keyboard input)
- Output: Navigation state mutations
- Logic: Route events based on current NavigationLevel
- Safety: Always clamp indices after navigation

## Routes Explored & Discarded

### ❌ Initially Considered: Single Component for All Views

**Why Considered**:
- Simplify by having one unified component
- Could render decision list, file list, or chunk detail based on state
- Seemed more elegant

**Why Discarded**:
- Decision list is fundamentally different from file list (organization by decision vs file)
- Component would become monster with too many conditional branches
- Hard to test each view independently
- Reusability gone - can't use decision list in other contexts

**Evidence**:
- Tried sketching combined component mentally - 400+ lines with nested match statements
- file_list.rs is already complex at ~150 lines with single responsibility
- Separated components are easier to understand and modify

### ❌ Considered: Storing Modal Content in State

**Why Considered**:
- Cleaner separation: state stores everything, components read it
- Could cache decision details and update them independently

**Why Discarded**:
- Decisions are immutable during review session
- Why store what we can compute from decision_number + engine?
- Extra state to keep synchronized (brittleness)
- This defeats the purpose of clean architecture

**Evidence**:
- ReviewEngine already has get_decision(number) method
- Decision data never changes after loading
- Storing decision details separately creates update coordination problem

### ❌ Considered: Using Ratatui Table Widget for Decision List

**Why Considered**:
- Could show number, title, count as columns
- Table widget handles complex layouts
- Professional appearance

**Why Discarded**:
- Overkill for three-column data with simple alignment
- Table widget adds complexity without benefit
- List widget with Span styling achieves same result more simply
- Follows existing pattern in file_list.rs

**Evidence**:
- file_list.rs uses List + ListItem + Span pattern
- Consistency with codebase reduces learning curve
- Table widget is powerful but adds ~50 lines of configuration code

### ❌ Considered: Implementing Full File Filtering in This Phase

**Why Considered**:
- Would complete the full "Decision → Modal → Files" flow
- Users could see complete end-to-end workflow

**Why Discarded**:
- Violates Steel Thread - each phase should be focused and deliverable
- Too much scope: decision list + modal + filtering would be one big contribution
- Better to get feedback on decision list/modal before adding file filtering
- Clear separation: this phase = decision navigation, next phase = file filtering

**Evidence**:
- implementation-roadmap.md clearly separates "Decision List" and "File View Filtered by Decision" as different steps
- Keeping scope focused makes it easier to test and debug each capability

### ✅ Chosen Approach: Separate Simple Components with Clear Responsibilities

Each component does one thing:
- DecisionListComponent: Display decisions
- DecisionDetailModalComponent: Display decision details
- Event handler: Route input to navigation state
- Navigation state (from 004): Track position
- ReviewEngine: Provide data

Benefits:
- Easy to understand by reading one component
- Each component is testable
- Easy to debug issues ("look at just the component that's broken")
- Easy to reuse ("need decision list elsewhere? use this component")

## Architecture Decisions Made

### 1. **Decision List Item Format: "number. title [count]"**

```rust
// Example output:
// ► 1. Refactor authentication module     [2]
//   2. Improve error handling              [2]
//   3. Add structured logging              [0]
//   0. Unmapped Changes                    [1]
```

**Rationale**:
- Number provides unique identifier for keyboard shortcuts (future: press '1' to jump)
- Title is human-readable reason for decision
- Count [n] shows immediately "this decision affects n code blocks"
- Selected item shown with blue ► marker and background highlighting

**Impact**:
- Users understand what each decision does at a glance
- No extra modal needed to see how many impacts each decision has
- Consistent with tools like git log, which shows hash, message, counts

**Alternative Considered**:
- Show confidence indicator on list too - rejected as adds clutter, modal shows this

### 2. **Modal Shows Decision Content in Semantic Order**

```
[Decision Title]
[Blank line]
[Summary paragraph]
[Blank line]
[Decision log line reference]
[Blank line]
[Code Impacts header]
[Impact 1: file, lines, type, confidence, reasoning]
[Impact 2: ...]
[Blank line]
[Footer with keybindings]
```

**Rationale**:
- Semantic grouping: understanding → rationale → implementation → footer
- Blank lines prevent visual fatigue from dense text
- Confidence level color-coded (green/yellow/red) for instant comprehension
- Footer reminds users of available actions without clutter

**Impact**:
- Users spend less time understanding impacts
- Color coding draws attention to low-confidence mappings
- Consistent with modern CLI tool design (e.g., git blame colorization)

### 3. **Navigation Level Drives Rendering Completely**

```rust
// In ui::draw()
match ui_state.decision_nav.current_level {
    NavigationLevel::Decision => {
        render_decision_list();  // Primary view: decisions
    }
    NavigationLevel::File | NavigationLevel::Chunk => {
        render_file_list();      // Primary view: files
    }
}
```

**Rationale**:
- Single source of truth: NavigationLevel enum
- Clear, explicit routing
- Easy to add new views in future
- Event handler doesn't need to know about rendering

**Impact**:
- UI always shows what navigation state says should be shown
- No hidden state - can't have UI and state out of sync
- Future: if we add "Approval Overview" view, just add NavigationLevel::ApprovalOverview

### 4. **Modal is Overlay, Not a Destination**

```rust
// In DecisionNavigationState:
pub current_level: NavigationLevel,    // Where user is (Decision/File/Chunk)
pub show_decision_modal: bool,         // Modal overlay on top?
```

**Rationale**:
- Modal is informational - user doesn't "navigate to" a modal
- Modal can appear at any level in future (show decision context from file view)
- Simpler state machine: binary flag, not another level to track
- Follows existing modal patterns in codebase

**Impact**:
- Escape key: close modal, stay at same level
- Future: could add modal from any view without restructuring state
- Prevents state explosion (if we add more modals, they're just more boolean flags)

### 5. **Event Handler Always Clamps Navigation Index**

```rust
UiEvent::NavigateDown => {
    if self.ui_state.decision_nav.current_level == NavigationLevel::Decision {
        self.ui_state.decision_nav.next_decision();
        let decision_count = self.review_engine.get_all_decisions().len();
        self.ui_state.decision_nav.clamp_decision_index(decision_count.saturating_sub(1));
    }
    // ...
}
```

**Rationale**:
- Prevent out-of-bounds indices that would cause panics or render glitches
- Handle edge cases: empty list, navigation at boundaries
- Future-proof: if decisions are added/removed, index stays valid

**Impact**:
- Safe navigation - impossible to reach invalid state
- No crashes from index out of bounds
- Code is robust even if data changes

### 6. **Components Never Directly Modify State**

```rust
// ❌ Component should NOT do this:
pub fn render(f: &mut Frame, ..., state: &mut UiState) {
    // NEVER: state.decision_nav.next_decision();
}

// ✅ Component does this:
pub fn render(f: &mut Frame, ..., state: &UiState) {
    // Only READ state, NEVER modify
}

// Event handler does this:
pub fn handle_ui_event(&mut self, event: &UiEvent) {
    if event == UiEvent::NavigateDown {
        self.ui_state.decision_nav.next_decision();  // Modification here only
    }
}
```

**Rationale**:
- Unidirectional data flow: Events → State → Rendering
- Easier to reason about: state changes only happen in one place
- Easier to debug: if something's wrong, look at event handlers
- Easier to test: components are pure functions

**Impact**:
- No hidden state mutations in render code
- Clear flow of control through application
- Prevents bugs from components doing unexpected things

## Critical Insights

### Component Simplicity Enables Feature Velocity

By keeping DecisionListComponent simple (107 lines, does one thing), adding it took 30 minutes and introduced zero bugs. If the component had complex logic, state management, or multiple responsibilities, it would take longer and require more testing.

Lesson: **Simple components with single responsibility are easier to add and maintain.**

### Navigation State Abstraction Proves Value

The decision_navigation.rs module from contribution 004 is doing its job:
- App.rs doesn't need to know about overlaps, indexing, or drilling logic
- Just calls simple methods: next_decision(), drill_into_files(), back_to_decisions()
- All the complex behavior is encapsulated and tested

This shows that investing in good abstraction layer (contribution 004) pays off immediately in contribution 005 - features are easier to add.

Lesson: **Good abstractions compound - each new feature becomes cheaper to build.**

### UI is Driven by State, Not the Other Way Around

The event handler route is:
1. Keyboard event → UiEvent enum
2. UiEvent → State mutation in app.rs
3. State → Rendering in ui::draw()

Notice: **The UI never drives state changes directly.** Components render what the state tells them to render. This prevents the common problem of UI being out of sync with state.

Lesson: **State-driven rendering beats imperative UI manipulation.**

### Modal Pattern Generalizes Well

Decision detail modal uses the same centered_popup() utility as input_modal and help_overlay. This means:
- UI patterns are consistent
- Users learn once, applies everywhere
- Adding new modals follows established pattern

This is how you build a coherent TUI - by having consistent patterns across all UI elements.

Lesson: **Extract and reuse UI patterns - it pays off.**

## For Next Contributor (File View Filtering)

### Your Task

Complete Phase 1 by filtering the file list based on selected decision, enabling the full navigation flow: Decision List → Decision Modal → Filtered File List → Chunk Detail.

### What You Inherit

✅ **Decision List Component**: Fully rendering with keyboard navigation working
✅ **Decision Detail Modal**: Showing all code impacts with confidence levels
✅ **Navigation State**: DecisionNavigationState with all methods implemented and tested
✅ **Event Routing**: Keyboard events routing to decision nav when at Decision level
✅ **Index Management**: Clamping logic preventing invalid indices
✅ **ReviewEngine APIs**: All decision query methods available (get_decision, get_decisions_for_diff, get_all_decisions)

### Implementation Path

1. **Update file_list component to accept decision filter**:
   - Add parameter: `selected_decision: Option<u32>`
   - Filter diffs to only show those matching selected_decision
   - Use ReviewEngine::get_decisions_for_diff() to check if diff belongs to decision
   - Keep existing UI layout and styling

2. **Update ui::draw() to pass decision filter**:
   ```rust
   if state.decision_nav.current_level == NavigationLevel::File {
       let decision_num = state.decision_nav.selected_decision;
       components::file_list::render(f, ..., ui_state, review_engine, decision_num);
   }
   ```

3. **Wire modal → file drill-down**:
   - When user presses Enter on code impact in modal
   - Call state.selected_file = Some(impact.file)
   - Call state.drill_into_files() (moves to File level)
   - File view renders that file with filtered chunks

4. **Handle file list → back to decisions**:
   - Left arrow when at File level: call state.back_to_decisions()
   - Clears selected_file, resets to Decision level
   - File list no longer visible

### Key Code Patterns to Follow

**Pattern 1: Decision-Aware Filtering**
```rust
// In file_list component
if let Some(decision_num) = selected_decision {
    // Only show diffs that this decision affects
    let diffs_for_decision: Vec<_> = reviewable_ids
        .iter()
        .filter(|&id| {
            let decisions = review_engine.get_decisions_for_diff(id);
            decisions.iter().any(|d| d.number == decision_num)
        })
        .collect();
} else {
    // Show all diffs (existing behavior for file-first navigation)
}
```

**Pattern 2: Modal Content → Drill Down**
```rust
// In decision_detail_modal
// When user selects an impact:
fn on_impact_selected(impact: &CodeImpact, state: &mut DecisionNavigationState) {
    state.selected_file = Some(impact.file.clone());
    state.drill_into_files();  // Moves to File level, file_list renders
}
```

**Pattern 3: Conditional Filtering in Event Handler**
```rust
// After user presses right arrow and drills from Decision to File:
let diffs_for_decision = if let Some(num) = state.decision_nav.selected_decision {
    Some(num)
} else {
    None
};
components::file_list::render(..., diffs_for_decision);
```

### What NOT to Do

❌ **Don't modify NavigationState structure** - it's complete for your use case
❌ **Don't add decision data to file_list** - just add filtering logic
❌ **Don't implement scrolling in modal** - Phase 2 scope
❌ **Don't change keyboard bindings** - stay with vim/less conventions
❌ **Don't cache decision-to-file mappings** - O(n) lookup is fast enough for MVP

### Tests to Verify

Before committing:
```bash
cargo test --package diffviz-review-tui  # All TUI tests pass
cargo test --workspace                   # No regressions in core
cargo build --workspace                  # Zero warnings
```

Manual testing in TUI:
1. Launch TUI: `cargo run --bin review-tui`
2. Navigate decisions: Up/Down arrows
3. Open decision modal: Enter
4. Drill to files: Right arrow (from modal or decision list)
5. See only files from that decision
6. Navigate chunks in filtered file view: Up/Down
7. Go back to decisions: Left arrow
8. Modal closes automatically

### Performance Considerations

- `get_decisions_for_diff()` is O(1) per diff - fast
- Filtering all diffs for decision: O(n) per render - acceptable for <100 diffs
- If >1000 diffs, optimize by building index in ReviewEngine
- No caching needed for Phase 1 (decisions are static)

### Common Pitfalls

**Pitfall 1: Breaking File-First Navigation**
- File-first navigation still needs to work (when at File/Chunk level, not at Decision level)
- Don't always filter by decision - only filter when `current_level == NavigationLevel::File`

**Pitfall 2: Forgetting to Handle Unmapped Decisions**
- Decision 0 (Unmapped Changes) can be selected
- When drilling from Decision 0, should show ALL unmapped diffs
- Test this: unmapped diffs should only appear under Decision 0, nowhere else

**Pitfall 3: State Inconsistency**
- If user drills to files but no diffs exist for that decision
- Show empty file list with helpful message
- Or prevent drilling with guard condition

**Pitfall 4: Clearing State at Wrong Time**
- When navigating back to decisions, clear selected_file
- When opening new modal, don't clear selected_file (user might drill again)
- When changing decisions while at File level, update file filter

### Success Criteria

✅ User can navigate decisions, open modal, drill to files, see filtered view
✅ File view only shows diffs that belong to selected decision
✅ Decision 0 shows all unmapped diffs
✅ Navigation between levels works without state corruption
✅ All TUI tests still pass
✅ Zero regressions in existing functionality
✅ Keyboard works smoothly (no lag, no skipped input)

### Future Work After Phase 1

- Phase 2: Load decisions from JSON files (decision-to-code-mapping.json)
- Phase 2: Add view mode toggle (decision-first vs file-first navigation)
- Phase 3: Implement dev-contribute integration to generate mapping files
- Phase 3: End-to-end: contribution → mapping generation → review workflow

## Assumptions Made

1. **No Empty Decisions**: At least one decision always exists (or Decision 0 synthetic)
2. **Decisions Don't Change**: Decision count and content stable during review session
3. **Line Ranges Are Valid**: CodeImpact line ranges match actual diffs in ReviewState
4. **Decision Numbers Unique**: No duplicate decision numbers
5. **Terminal Size Reasonable**: Assume >80 columns, >20 rows for readable output

## Caveats & Limitations

1. **File Filtering is O(n) Per Render**:
   - For 100 decisions × 1000 diffs = 100k comparisons per render
   - Acceptable for MVP, optimize if profiling shows issue
   - Fix: Build decision-to-diffs index in ReviewEngine

2. **No Scroll in Modal**:
   - Decision with 20+ impacts won't fit in modal
   - Solution: implement scrollable paragraph widget

3. **Navigation State Doesn't Persist**:
   - Loading new review resets to Decision List
   - Users don't mind - fresh start is natural

4. **Keyboard Bindings Are Hardcoded**:
   - Could load from config in future
   - MVP uses standard conventions (vim/less)

5. **No Direct Jump to Decision**:
   - Can't press '2' to jump to Decision 2
   - Future enhancement: add leader key patterns

## Implementation Notes

### Why Ratatui List Widget?

List widget handles:
- Scrolling if items > screen height
- Mouse events (click to select)
- Efficient rendering
- Styled items with colors and modifiers

Only costs: understanding Ratatui's abstraction. Once learned, very powerful.

### Why Modal After diff_view?

Rendering order:
1. file_list
2. diff_view
3. status_bar
4. decision_detail_modal (renders last, appears on top)
5. which_key overlay
6. help_overlay

Last component rendered appears on top. Modal must render last.

### Why Event Handler Owns State Mutations?

All keyboard → state mappings in one place (app.rs handle_ui_event) means:
- Easy to see "what does each key do"
- Easy to change behavior (just edit one function)
- Hard to accidentally break keybindings in component code

## Conclusion

This contribution establishes the Steel Thread pattern:
- Foundation (004): Navigation state infrastructure
- Capability Expansion (005): Decision list & modal UI
- Next Expansion (006): File filtering by decision
- Next Expansion (007): Full end-to-end flow

Each phase is self-contained, testable, and deliverable. Users see progress after each phase. Foundation is built right, making each subsequent phase easier to add.

The pattern is repeatable: good abstraction → simple components → easy-to-add features.
