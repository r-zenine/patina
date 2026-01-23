# Context Handoff - Callback Handling Decision Revisit

## What I Built & Why

Created a comprehensive analysis of callback handling approaches that align with ELM architecture used in diffviz-review-tui. The goal is to replace the callback-based feedback mechanism with approaches that respect the functional, event-driven nature of ELM.

## Architecture Research Conducted

### diffviz-review-tui ELM Implementation

Thoroughly analyzed the TUI codebase to understand its event flow:

1. **Event Flow** (`src/events/input.rs` → `src/events/business.rs` → `src/app.rs`)
   - Keyboard input maps to `UiEvent` variants via context-aware handlers
   - `UiEvent` converts to `BusinessEvent` via `ui_event_to_business_event()`
   - `BusinessEvent` triggers ReviewEngine operations through `handle_business_event()`
   - Components re-render on next cycle seeing updated state

2. **Component Model** (e.g., `src/components/diff_view.rs`, `src/components/file_list.rs`)
   - Components are **pure functions**: `fn render(f: &mut Frame, area: Rect, ui_state: &UiState, engine: &ReviewEngine)`
   - They **receive immutable references** to state
   - They **render based on current state**, not event callbacks
   - They have **no event emitters**, no callbacks passed in
   - Visual feedback comes automatically when state updates on next render

3. **State Management** (`src/ui_state.rs`)
   - `UiState` holds presentation state (focus, navigation, scroll, input mode)
   - Methods mutate state directly: `toggle_focus()`, `navigate_prev()`, `enter_input_mode()`
   - ReviewEngine holds business state and operations

4. **Feedback Patterns** (existing approval feedback)
   - Status bar shows progress: "3/5 approved" updated automatically
   - File list color-codes based on completion
   - Decision tree shows counts
   - All feedback driven by state queries, not callbacks

### Why Callbacks Don't Fit

Current callback approach in contribution 002:
```rust
pub fn approve_decision(
    &mut self,
    decision_number: u32,
    author: String,
    callback: Option<Box<dyn FnOnce(bool, Option<String>) + Send>>,
) -> Result<()>
```

Issues with ELM architecture:
- **Components can't emit events**: In ELM, handlers call methods and read updated state
- **No callback infrastructure**: Components receive immutable references, not callbacks
- **Breaks pure function model**: Callbacks introduce side effects that aren't state-driven
- **Inconsistent with existing patterns**: All other operations use state queries for feedback
- **Complicates testing**: Callback verification is harder than state assertion

## Routes Explored & Discarded

### ❌ Keep Callbacks, Add Message Queue
- **Why Considered**: Could work alongside existing patterns
- **Why Discarded**: Callbacks fundamentally violate ELM principle of pure event flow
- **Evidence**: diffviz-review-tui has zero callback infrastructure; events are only input source

### ❌ Silent Operations (No Feedback)
- **Why Considered**: Simple, matches pure Result<()> pattern
- **Why Discarded**: User experience suffers; decision cascades affect multiple chunks but no indication of what happened
- **Evidence**: Status bar already shows operation feedback; cascades should too

### ❌ External Event Queue in ReviewEngine
- **Why Considered**: Could emit events at operation completion
- **Why Discarded**: ReviewEngine is domain layer; shouldn't know about event queue
- **Evidence**: Violates clean architecture (domain shouldn't depend on UI patterns)
- **Better**: Return events/summaries, let TUI decide what to do

## Chosen Analysis Approach

Rather than recommending a single solution, gathered **four distinct options** that all:
- ✅ Eliminate callbacks completely
- ✅ Align with ELM functional principles
- ✅ Enable TUI feedback without breaking architecture
- ✅ Work with existing approval/decision patterns

Each option trades off complexity vs. richness of feedback:

**Option A: State-Based (Minimal)**
- Pure ELM: just Result<()>
- TUI infers changes from state re-render
- Simplest, but no explicit feedback

**Option B: Return Summary (Moderate)**
- Return structured data about operation
- TUI shows explicit feedback from return value
- Clean separation of concerns

**Option C: Event Emission (Excellent)**
- Return BusinessEvent variants
- Natural for ELM: events describe what happened
- Integrates perfectly with existing event handlers

**Option D: Detailed Result (Rich)**
- Return detailed operation info
- Handles partial failures and warnings
- Most information available to TUI

## Key Architectural Insights

1. **ELM doesn't have callbacks** - Operations return values (data or events), state updates happen through handlers, components re-render seeing new state

2. **TUI feedback is state-driven** - Status bar, file list colors, decision tree counts all come from `review_engine.state()` queries, not callbacks

3. **Business events are already defined** - Can extend `BusinessEvent` enum to represent cascade results

4. **Existing pattern to follow** - `BusinessEvent::ToggleApprove { reviewable_id }` shows the model: simple enum variant with context

5. **Decision tree is selection context** - Similar to how `ui_event_to_business_event()` extracts current selection, cascade operations can be driven by selected decision

## Assumptions Made

1. **TUI is the primary consumer** - Focus on feedback patterns that work for TUI, not generic callback compatibility
2. **Cascade succeeds or fails atomically** - No need to handle partial failures in MVP (keeps complexity down)
3. **Operations are synchronous** - No async callbacks needed; TUI event loop waits for operation completion
4. **Feedback timing isn't critical** - Showing feedback on next render cycle (few ms) is acceptable UX
5. **Decision-approval is UI-initiated** - Only TUI calls cascade methods; no internal cascades from reviewable logic

## For Next Contributor (Decision Selection & Implementation)

### Before You Start Implementation

1. **Review all four options** in decision-log.md carefully
2. **Understand your constraints**:
   - How much feedback detail does TUI need?
   - Is transaction semantics important?
   - Should partial failures be possible?
3. **Propose to user** which option best fits the project's goals
4. **Get explicit approval** before starting code changes

### If Option B (Return Summary) is Chosen

- Add `OperationSummary` struct in `diffviz-review/src/entities/`
- Update signatures: `approve_decision(...) -> Result<OperationSummary>`
- Add TUI state field: `last_operation: Option<OperationSummary>`
- Update status bar to display operation summary
- Clear summary after N frames or on next user input

### If Option C (Event Emission) is Chosen

- Add to `BusinessEvent` enum:
  ```rust
  DecisionApproved { decision_number: u32, chunks_affected: usize }
  DecisionUnapproved { decision_number: u32, chunks_affected: usize }
  ```
- Update signatures: `approve_decision(...) -> Result<Vec<BusinessEvent>>`
- TUI processes returned events through existing event handler
- Fits naturally with `ui_event_to_business_event()` pattern

### Testing Approach

Whichever option is chosen, tests should verify:
1. ✅ Correct return type from cascade methods
2. ✅ No callback invocations (callbacks are completely gone)
3. ✅ Operation result accurately reflects what changed
4. ✅ Integration with ReviewEngine tests still passes

## Known Limitations

1. **MVP scope**: Current analysis assumes atomic success/failure; partial failures deferred
2. **No performance analysis**: Decision with 100+ chunks would still work, but feedback could be batched in future
3. **TUI integration not shown**: Actual TUI changes to handle chosen approach not yet implemented
4. **No migration strategy**: Existing tests using callbacks will need updating once approach is chosen

## Questions for User Input

1. **Feedback richness**: How much detail should TUI show about cascade operations?
   - "Decision approved" (minimal) vs.
   - "Decision #1 and 3 chunks approved" (moderate) vs.
   - Full detail including chunk IDs and any warnings (rich)

2. **Consistency**: Should cascade operations behave like existing approve/reject operations?
   - Currently they take optional callback → would become return-based
   - Should consistency extend to all ReviewEngine operations? (future consideration)

3. **Error handling**: Should partial failures be possible or are operations atomic?
   - Affects complexity and return type design

4. **Implementation timeline**: Should this be fixed before or after TUI integration?
   - Could do cascading logic first with callbacks, then refactor
   - Or fix architecture now while code is fresh in contribution 002

## Recommendation

**Propose Option C (Event Emission)** to user for the following reasons:

1. **Perfect ELM fit**: BusinessEvent already exists; just extend it
2. **Reuses patterns**: `ui_event_to_business_event()` already converts events
3. **TUI integration natural**: Event handlers already process BusinessEvent
4. **Future-proof**: Events can drive other effects (exports, logging, metrics)
5. **Moderate complexity**: Not as simple as pure Result<()>, but much simpler than detailed results

But get user feedback on all four options first - architectural preferences vary by project.
