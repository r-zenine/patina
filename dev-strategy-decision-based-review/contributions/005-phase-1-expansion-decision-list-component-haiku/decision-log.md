# Decisions Made - Phase 1 Expansion: Decision List Component

## Decision 1: Conditional Rendering by NavigationLevel

**Category**: Architecture
**Decision**: Render decision_list when at NavigationLevel::Decision, render file_list when at File/Chunk levels
**Rationale**:
- NavigationLevel enum already tracks current view position
- Cleaner than maintaining separate "view mode" flag
- Mirrors existing architectural pattern of state driving rendering
**Impact**:
- Main draw() function has conditional logic based on navigation level
- Easy to add new view types in future (just add NavigationLevel variant)
- Components remain simple presenters

**Alternatives Considered**:
- Add `current_view: ViewMode` enum to UiState - rejected because NavigationLevel already fills this role
- Use separate boolean flag (showing_decisions: bool) - rejected because violates DRY principle
- Always render both and hide one - rejected because wasteful and harder to manage focus

## Decision 2: Modal as Overlay (Not NavigationLevel Variant)

**Category**: Architecture
**Decision**: Keep decision detail modal as overlay with `show_decision_modal: bool` flag, not as NavigationLevel::Modal variant
**Rationale**:
- Modal is overlay "on top of" current view, not a destination you navigate to
- Aligns with existing modal pattern in codebase (input_modal, help_overlay)
- Simpler state machine: modal can appear at any level (Future: from File view, Chunk view)
- Less coupling between components
**Impact**:
- Modal renders last, ensuring it's always visible
- Escape key closes modal but keeps you at same level
- Could add modal from any view without restructuring navigation

**Alternatives Considered**:
- NavigationLevel::Modal variant - rejected as it's not a destination, just an overlay
- Modal state in separate modal manager struct - rejected as over-engineering for MVP

## Decision 3: Using Ratatui List Widget for Decision List

**Category**: Technical
**Decision**: Use Ratatui's List widget with styled ListItems for decision list rendering
**Rationale**:
- List widget handles scrolling and rendering efficiently
- ListItem with Span styling gives fine control over appearance
- Consistent with file_list.rs which uses same pattern
- Ratatui List is battle-tested and performant
**Impact**:
- Clean separation between data and rendering
- Easy to extend with additional styling or metadata
- Could reuse same component in other contexts

**Alternatives Considered**:
- Custom table rendering - rejected as List widget is simpler for single-column data
- Paragraph widget with manual highlighting - rejected as harder to maintain selection state

## Decision 4: Index Clamping After Navigation

**Category**: Safety/Robustness
**Decision**: Clamp decision_list_index after every next_decision/prev_decision call
**Rationale**:
- Prevents UI crashes from invalid indices
- Handles edge cases: empty list, single decision, boundary conditions
- DecisionNavigationState.clamp_decision_index() designed for exactly this
- Future-proofs against dynamic decision addition/removal
**Impact**:
- No panics from out-of-bounds indices
- Safe to navigate even if decision count is 0 (clamps to 0)
- Slightly more code but eliminates entire class of bugs

**Alternatives Considered**:
- Let UI code handle bounds checking - rejected as error-prone and repeated
- Use modulo for wrap-around - rejected as per vim convention, UI stops at boundaries
- No clamping needed - rejected as doesn't handle edge cases

## Decision 5: Keyboard Bindings Follow Vim/Less Conventions

**Category**: UX
**Decision**: Map navigation keys to vim/less conventions:
- Up/Down arrows: navigate decision list
- Enter: open decision detail
- Right arrow: drill into files for selected decision
- Left arrow: go back to decisions
- Escape: close modal and exit input modes
**Rationale**:
- Matches expectations of CLI/TUI users (vim, less, tmux, etc.)
- Reduces learning curve for developer audience
- Consistent with existing file_list navigation
**Impact**:
- Users can navigate without mouse
- Keyboard-only workflows fully supported
- Aligns with broader DiffViz UX patterns

**Alternatives Considered**:
- hjkl keys like vim - rejected as conflicts with input mode, slower to type
- Mouse support - rejected as adds complexity and not needed for CLI use
- Configurable bindings - rejected as scope creep for MVP

## Decision 6: Color Coding for Confidence Levels

**Category**: UX
**Decision**: Show confidence levels in modal with color coding:
- High: Green
- Medium: Yellow
- Low: Red
**Rationale**:
- Users instantly understand reliability of mapping
- Green/yellow/red is universal convention for status
- Reduces need to read confidence text
- Matches warning system conventions
**Impact**:
- Better scanning of decision impacts
- Users focus on high-confidence impacts first
- Color helps distinguish decision rationale from confidence indicator

**Alternatives Considered**:
- Text only (CONFIDENCE: HIGH) - rejected as slower to scan
- Numeric scores (1-3 or 1-100) - rejected as arbitrary, color is more intuitive
- Icons (✓/◐/✗) - rejected as takes more screen space

## Decision 7: DecisionListComponent Stays Simple and Pure

**Category**: Architecture
**Decision**: Component receives data, renders, but never modifies navigation state
**Rationale**:
- State changes handled in app.rs event handler
- Component is pure presenter - easy to test and reuse
- Single responsibility: render decisions, don't manage navigation
- Prevents bugs from components modifying wrong state
**Impact**:
- Components are stateless and testable
- All state changes traceable to one place (app.rs)
- Could reuse component in other contexts (CLI, other TUI layouts)

**Alternatives Considered**:
- Component calls navigation methods directly - rejected as creates hidden state changes
- Component returns action events to handler - rejected as already have UiEvent system
- Bidirectional data binding - rejected as too complex for CLI application

## Decision 8: No Scroll Support in Modal (MVP)

**Category**: MVP Scope
**Decision**: Don't implement scrolling in decision detail modal for Phase 1
**Rationale**:
- Typical decision has 2-5 code impacts
- MVP target terminal size is 80+ columns, 24+ rows
- Modal reserves ~70% of screen space
- Scrolling adds complexity to Ratatui usage
- Can add in Phase 2 if user feedback requires it
**Impact**:
- Simpler code, easier to maintain
- Works for current test data and expected data sizes
- Clear path to add if needed (use Paragraph::scroll or ScrollState)

**Alternatives Considered**:
- Scrollable Paragraph widget - rejected as defers UX decision to user
- Truncate impacts - rejected as loses information
- Two-column display - rejected as doesn't fit typical impact count

## Decision 9: Modal Width 75% of Screen, Height 70%

**Category**: UX
**Decision**: Use centered_popup() with 75% width, 70% height for decision detail modal
**Rationale**:
- Leaves visible border of underlying content (context)
- Provides enough space for typical decision info + impacts
- Centered positioning draws attention
- Matches conventions from other TUI applications
- Existing centered_popup() utility already supports these dimensions
**Impact**:
- Modal is prominent but not overwhelming
- Can still see decision list/file list behind modal
- User maintains context awareness

**Alternatives Considered**:
- Full screen modal - rejected as too aggressive, hides context
- 50% width/height - rejected as too small for content
- Left/right aligned - rejected as less intuitive than centered

## Decision 10: Exiting Modal vs Exiting Input Mode

**Category**: UX
**Decision**: Escape closes decision modal if open, otherwise exits input mode or quits TUI
**Rationale**:
- Single key (Escape) for "go back" is intuitive
- Modal is logical scope for escape (it's open)
- Doesn't interfere with existing input mode behavior
- Users learn: Escape = undo last action
**Impact**:
- Consistent UX: Escape always "backs out" from current context
- No conflicts between modal and input mode
- Future: could extend to "navigate back" context

**Alternatives Considered**:
- Separate key (q) for modal close - rejected as more keys to learn
- Modal doesn't support escape - rejected as breaks convention
- Always exit input mode regardless - rejected as loses modal UX

---

## Implementation Decisions & Trade-offs

### 1. Using styled Spans instead of Tables for Layout
**Decision**: Use Vec<Span> for decision list items instead of Table widget
**Trade-off**: Simpler code vs. less alignment flexibility
**Why**: Decision list is simple (number, title, count). Don't need table's power.

### 2. No Separate Modal Component State
**Decision**: Modal data computed from selected_decision + ReviewEngine, not stored in state
**Trade-off**: O(1) lookups vs. potential to cache
**Why**: Decisions don't change during review, so no need to cache

### 3. Rendering Modal After Status Bar
**Decision**: Modal renders last so it appears on top
**Trade-off**: Rendering order dependency vs. guaranteed visibility
**Why**: Ratatui renders in order - last component appears on top

---

## Future Decisions Deferred to Phase 2

1. **File View Filtering by Decision**: Currently file_list shows all files. Phase 2 will filter based on selected_decision when at File level.

2. **Modal Scrolling**: If decision has >10 impacts, add scrolling support using ScrollState.

3. **Decision Keyboard Shortcuts**: Could add 'd1', 'd2' shortcuts to jump to specific decisions.

4. **Persistent Navigation Position**: Could save/restore user's position in decision list across sessions.

5. **Full Navigation Flow**: Complete end-to-end: Decision → Modal → File → Chunk with seamless back-navigation.

---

## Decision Rationale Summary

This contribution prioritizes:
- **Simplicity**: Use existing widgets and patterns, don't over-engineer
- **Safety**: Clamp indices, validate state transitions
- **Consistency**: Follow vim/less conventions, match existing codebase patterns
- **Testability**: Components are pure, state changes are traceable
- **Incrementality**: Modal and list rendering working, navigation flow completed in Phase 2
