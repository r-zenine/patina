# Additional Decisions - Phase 1 Foundation: Decision Navigation

## Navigation State Separation
**Category**: Architecture
**Decision**: Create separate `DecisionNavigationState` instead of extending existing `NavigationState`
**Rationale**:
- Existing NavigationState is file-first hierarchy (FileHeader → DiffItem)
- Decision-first hierarchy is fundamentally different (Decision → File → Chunk)
- Separate implementation allows incremental TUI migration without breaking existing code
- Clear separation of concerns: two navigation patterns coexist until file-first is deprecated
**Impact**:
- UiState now has both navigation_state and decision_nav fields
- TUI can toggle between patterns during transition
- Future: Remove old NavigationState once decision-first is default

## Navigation Levels as Enum
**Category**: Data Structure
**Decision**: Use `NavigationLevel` enum (Decision, File, Chunk) instead of tracking state implicitly
**Rationale**:
- Explicit state machine easier to reason about
- Prevents invalid state combinations (e.g., selected_chunk without being at Chunk level)
- Clearer code: `if state.at_chunk_view()` reads better than complex conditionals
**Alternatives Considered**:
- Implicit tracking via Option<selected_decision> only → Less clear, harder to debug
- Variant enum with nested state → More complex, overkill for MVP
**Impact**:
- Predictable state transitions
- Easy to add UI conditional rendering based on current_level
- Test coverage validates all level transitions

## Modal as Overlay, Not State Level
**Category**: UX Architecture
**Decision**: Decision detail modal is an overlay that doesn't change navigation level
**Rationale**:
- Modal appears "on top" of current view (typically Decision level)
- Users drill from modal to FileLevel, not navigate to "Modal" level
- Simpler state machine: modal is just a flag, not a navigation level
- Matches common modal UX patterns
**Alternatives Considered**:
- Modal as separate level → Would complicate state machine, makes back-navigation ambiguous
- Modal automatically navigates to File level → Breaks user expectation of overlay behavior
**Impact**:
- `show_decision_modal` is boolean flag, not navigation state
- Users can see modal then drill into files without "leaving" the modal
- Clear back-navigation: File → Decision with modal still open if desired

## Separate Index Tracking
**Category**: Navigation Implementation
**Decision**: Track `decision_list_index` and `file_list_index` separately, not unified position
**Rationale**:
- Decision list has different length than file list within a decision
- When returning from File level to Decision level, need to restore decision position
- When switching decisions, need fresh file list navigation starting at 0
- Unified index would require complex mapping logic
**Alternatives Considered**:
- Single `current_index` with level-based interpretation → Error-prone
- Stack-based position tracking → Overly complex for MVP
**Impact**:
- Each level maintains independent scroll/selection position
- No index range conflicts
- Natural behavior: drilling down resets file index, backing up restores decision index

## Guard Condition: Drill Requires Decision
**Category**: State Safety
**Decision**: `drill_into_files()` requires `selected_decision.is_some()`, returns silently if not
**Rationale**:
- Prevents invalid state: can't show files for non-existent decision
- Fail-fast approach: invalid user action results in no state change
- Silent failure acceptable for MVP: TUI can disable drill button when no decision selected
- Alternative of returning Result would require error handling at TUI level
**Alternatives Considered**:
- Return Result<()> → Requires error handling in event loop
- Panic on invalid state → Too harsh for user input handling
- Auto-select first decision → Surprises users who just navigate
**Impact**:
- TUI keybindings must check `selected_decision.is_some()` before allowing drill
- Or TUI disables drill keybinding when no decision selected
- Simple and safe: impossible to reach invalid state

## Helper Functions: Iteration-Based Lookup
**Category**: Implementation Strategy
**Decision**: Query functions (get_files_for_decision, get_chunks_for_file_in_decision) use iteration rather than caching
**Rationale**:
- ReviewEngine already has queries; avoid duplicating index structure
- Phase 1 is MVP: premature optimization wrong (likely < 100 decisions)
- Changes to ReviewState automatically reflected in queries
- Cached approach would require invalidation logic
**Alternatives Considered**:
- Cache decision → file and file → chunk mappings → More infrastructure, harder to maintain
- Pass pre-computed map from ReviewEngine → Couples navigation to engine architecture
**Impact**:
- Query functions O(n) per call, but n is small (typically 3-10 decisions)
- No stale data issues
- Can optimize to cached index if performance becomes issue later

## No Circular Navigation (Yet)
**Category**: UX Feature
**Decision**: List navigation doesn't wrap: moving down at end stays at end (no circular wrap)
**Rationale**:
- MVP simplicity: easier to implement and test
- Matches common TUI pattern (vim, less, etc.)
- Users can press 'g' for top if needed (vim pattern)
- Circular navigation can be added later if desired
**Alternatives Considered**:
- Circular: prev at start goes to last, next at end goes to first → More complex
- Explicit feedback: return whether moved or not → Requires UI to show feedback
**Impact**:
- `next_decision()` checks `< selection_map.len() - 1` before incrementing
- `prev_decision()` uses `saturating_sub(1)`
- TUI can show "at end" / "at start" messages if desired

## State Reset: Complete vs. Partial
**Category**: State Management
**Decision**: `reset()` clears all navigation state, returns to Decision level with no selection
**Rationale**:
- New review session should start fresh
- Prevents stale selections when loading new contribution
- Simpler than partial reset logic
- Test fixture setup can initialize specific states as needed
**Alternatives Considered**:
- Partial reset: keep level but clear selections → Confusing when loading new data
- Preserve last position → Breaks when review data changes
**Impact**:
- When loading new review: call `decision_nav.reset()`
- TUI starts back at decision list every time
- Explicit: no hidden preserved state to surprise users

## Test Coverage: Behavior Over Implementation
**Category**: Testing Philosophy
**Decision**: Test public API and state transitions, not internal field access
**Rationale**:
- Tests validate user-facing behavior
- Implementation can change without breaking tests
- Clear test names describe user intent (e.g., "drill_into_files")
- Easier to read: tests show usage patterns
**Alternatives Considered**:
- White-box testing of all field values → Brittle to refactoring
- Integration tests only → Too slow for iterative development
**Impact**:
- Tests use public methods: at_decision_list(), select_decision(), drill_into_files()
- Some internal state checked only when necessary (e.g., verify current_level changed)
- Tests double as documentation: "how do I use this?"

