# Technical Decisions - Phase 3.1: Business Events

## 2026-01-23 - ToggleApproveDecision Event Variant

**Category**: API Design - Event System

**Decision**: Add `ToggleApproveDecision { decision_number: u32 }` as new variant to BusinessEvent enum

**Rationale**:
- Mirrors existing `ToggleApprove { reviewable_id }` pattern for consistency
- Carries decision number explicitly (type-safe)
- Compiler ensures exhaustiveness checking (must handle all variants)
- Distinguishes decision-level approval from chunk-level approval
- Allows future customization of behavior per event type

**Alternative Rejected**:
- Reuse `ToggleApprove` with context parameter: Would lose type safety, harder to debug
- Use separate `ApproveDecision` and `RejectDecision` events: Too verbose, `ToggleApprove` is idiomatic for toggles

**Impact**:
- All pattern matches on BusinessEvent must handle new variant
- Event conversion logic becomes more sophisticated (depth-aware)
- Type system guarantees handlers process both event types

---

## 2026-01-23 - Context-Aware Event Conversion

**Category**: Business Logic - Event Routing

**Decision**: `UiEvent::ToggleApprove` converts to different BusinessEvent variants based on navigation depth

**Rationale**:
- User presses same key (Space+a) whether at decision or chunk level
- Single keybinding is more intuitive than separate keys
- Navigation depth already tracked (`TreePath::depth()`)
- Conversion logic can inspect depth and route correctly

**Implementation Strategy**:
```rust
match ui_event {
    UiEvent::ToggleApprove => {
        if let Some(decision_number) = ui_state.current_decision_number() {
            // At depth 0 (decision level)
            Some(BusinessEvent::ToggleApproveDecision { decision_number })
        } else {
            // At depth 1 or 2 (file/chunk level)
            ui_state
                .current_reviewable_id()
                .map(|id| BusinessEvent::ToggleApprove { reviewable_id: id })
        }
    }
    // ...
}
```

**Advantages**:
- User doesn't need to learn different keybindings
- Same key behaves contextually (like vim navigation)
- No duplicate handlers in app.rs
- Backward compatible (chunk toggle still works)

**Alternative Rejected**:
- Separate keybindings (Space+a+d for decision, Space+a+a for chunk): More keys to remember
- Always check context in handler: Pushes complexity higher, less separation of concerns

---

## 2026-01-23 - UI State Helper Method

**Category**: API Design - State Encapsulation

**Decision**: Add `current_decision_number()` method to UiState that checks depth

**Rationale**:
- Encapsulates the logic: "return decision number ONLY if at depth 0"
- Single source of truth for this check
- Callers don't need to know about TreePath depths
- Matches pattern of `current_reviewable_id()` and `current_file_path()`

**Implementation**:
```rust
pub fn current_decision_number(&self) -> Option<u32> {
    if self.decision_tree.selected_path.depth() == 0 {
        self.decision_tree.selected_decision_number()
    } else {
        None
    }
}
```

**Why Not**:
- Inline the logic in conversion: More brittle, duplicated if needed elsewhere
- Add to DecisionNavigationTree: Violates separation (UI state concerns vs navigation tree)
- Make DecisionNavigationTree aware of depth: Already implemented correctly, don't add confusion

---

## 2026-01-23 - Event Handler Location and Pattern

**Category**: Architecture - Responsibility Distribution

**Decision**: Implement `ToggleApproveDecision` handler in `ReviewTuiApp::handle_business_event()`

**Rationale**:
- Consistent with other BusinessEvent handlers
- ReviewTuiApp has access to both ui_state and review_engine
- Centralized business logic orchestration
- Follows Command pattern: handler returns Command enum, not direct I/O

**Implementation Pattern** (mirrors `ToggleApprove`):
```rust
BusinessEvent::ToggleApproveDecision { decision_number } => {
    if self.review_engine.is_decision_approved(decision_number) {
        self.review_engine.reject_decision(decision_number)?;
    } else {
        self.review_engine.approve_decision(decision_number, author)?;
    }
    Ok(Command::None)
}
```

**Why This Pattern**:
- Query-then-decide: Check state first, then decide action
- Mirrors existing toggle pattern (elegant, predictable)
- Returns `Command::None`: No I/O side effects, TUI handles rendering
- Error propagation: `?` operator bubbles up errors naturally

**Alternative Rejected**:
- Always approve without checking: Wrong behavior (user expects toggle)
- Return feedback command: Premature (feedback mechanism not designed yet)
- Direct state mutation: Bypasses ReviewEngine business logic

---

## 2026-01-23 - Return Type for Decision Approval

**Category**: API Design

**Decision**: `approve_decision()` and `reject_decision()` return `Result<CascadeResult>`

**Rationale**:
- Established in Contribution 004
- Handler can ignore result (it's for future feedback)
- TUI can access result if feedback is added
- Clean separation: ReviewEngine returns facts, TUI interprets them

**Note**: Handler returns `Command::None` for now, but future contributions can:
- Convert `CascadeResult::description()` to status message
- Show cascading scope to user
- Display progress information

**Implementation**:
```rust
let result = self.review_engine.approve_decision(decision_number, author)?;
// result contains: DecisionApproved { decision_number, chunks_affected }
// Future: could use result for feedback
Ok(Command::None)
```

---

## Implementation Decisions Summary

| Decision | Chosen | Alternative | Rationale |
|----------|--------|-------------|-----------|
| Event variant | `ToggleApproveDecision` | Separate events | Consistency + type safety |
| Keybinding | Reuse Space+a | Separate key | Single key, contextual routing |
| Context detection | Depth check at conversion | At handler | Earlier, cleaner separation |
| Helper method | `current_decision_number()` | Inline logic | Encapsulation + reusability |
| Handler location | `handle_business_event()` | Separate | Consistent architecture |
| Return handling | Ignore `CascadeResult` | Use for feedback | Future extensibility |

---

## Verification Checklist

- [x] Event variant created and integrated
- [x] Compiler enforces exhaustiveness checking
- [x] Conversion logic checks depth correctly
- [x] Helper method encapsulates logic
- [x] Handler implemented and working
- [x] Compilation clean
- [x] All tests pass
- [x] Zero clippy warnings
- [x] Code formatted

## Future Extensibility

This design enables future enhancements:
1. **Task 3.4**: Use `cascade_result.description()` in feedback command
2. **Task 3.5**: Query `is_decision_approved()` for rendering
3. **Task 3.6**: Customize keybinding without changing event logic
4. **Future**: Add undo/redo of cascade operations
5. **Future**: Analytics on decision approval patterns

The event system is extensible without breaking existing code.
