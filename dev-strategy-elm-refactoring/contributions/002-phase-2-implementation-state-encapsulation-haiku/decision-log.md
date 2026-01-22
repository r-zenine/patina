# Decision Log: Phase 2 - Encapsulate State Mutations

## Implementation Decisions

### D1: Method Scope - Which Operations to Encapsulate

**Decision**: Encapsulate only the four specific operations identified in the roadmap:
- `navigate_to_first_in_tree()`
- `navigate_to_last_in_tree()`
- `is_modal_open()`
- `close_modal_if_open()`

**Rationale**:
- The roadmap specifically identified these four direct field accesses in app.rs
- They represent the clearest violations of V4 (direct field access)
- Other operations like `navigate_next()`, `toggle_expansion()` are already encapsulated
- `open_decision_modal()` is already a method on DecisionNavigationTree (no redundant wrapper needed)

**Impact**:
- Minimal, focused changes
- No over-engineering or premature abstraction
- Easier to review and maintain

### D2: Navigation Method Implementation - Embedded Logic or Separate

**Decision**: Embed the tree flattening and navigation logic directly in UiState methods

**Current Implementation**:
```rust
pub fn navigate_to_first_in_tree(&mut self) {
    let flattened = self.decision_tree.flatten();
    if let Some(first) = flattened.first() {
        self.decision_tree.selected_path = first.path.clone();
        self.reset_scroll();
    }
}
```

**Rationale**:
- Encapsulates the full operation: flatten → select → reset
- Previously this logic was duplicated in NavigateToTop handler
- Now it's a single, reusable operation
- The method "owns" the complete operation from event to state update

**Alternative Considered**:
- Extract flattening logic to DecisionNavigationTree method
- Rejected: Adds complexity without benefit for Phase 2 focus

### D3: Modal State Access - Getter vs Direct Access

**Decision**: Add both `is_modal_open()` getter and `close_modal_if_open()` combined operation

**Rationale**:
- `is_modal_open()` enables checks through UiState interface
- `close_modal_if_open()` guards the close operation with clear intent
- Prevents leaking DecisionNavigationTree field names (`show_decision_modal`)
- Maintains clean boundary between layers

**Why Not Add Setters?**
- Setting modal state is not a direct field access violation in current code
- Opening modal is already called as `decision_tree.open_decision_modal()`
- No need for redundant wrapper methods
- Focus on fixing identified violations, not unnecessary abstraction

### D4: Method Mutability - All Methods Take `&mut self`

**Decision**: All four new methods are `&mut self` (mutable)

**Rationale**:
- They all modify UiState (selection, modal state, scroll)
- Rust compiler enforces mutation contract
- Caller knows operation modifies state

**Exception**: `is_modal_open() -> bool` is `&self` (immutable getter)

### D5: Duplication Between ReviewTuiApp and HeadlessApp

**Decision**: Apply same changes to both ReviewTuiApp and HeadlessApp in app.rs

**Why Both Modified**:
- Both have identical handle_ui_event implementations
- Both need same encapsulation fixes
- HeadlessApp is test infrastructure behind feature flag
- Not breaking tests by updating both

**Not Extracting Shared Logic**:
- Dev-strategy notes this as known technical debt
- Extracting would require careful trait/module design
- Not blocking ELM compliance
- Deferred to separate refactoring task

## Technical Constraints

### TC1: Avoiding Over-Engineering

**Constraint**: Don't abstract more than necessary for V4 fix

**Decision**: Only add the four methods identified in roadmap

**Implementation**:
- No generic state accessor patterns
- No builder methods for complex operations
- No private field enforcement (yet)
- Focus on encapsulation, not redesign

### TC2: Maintaining Test Compatibility

**Constraint**: Changes must not break existing tests

**Decision**: All changes are additive (new methods) or refactoring (same behavior)

**Verification**:
- All 5 existing tests pass
- No test code modifications needed
- Behavior is identical

## Open Questions

**Q: Why not make DecisionNavigationTree fields private?**
- A: That's not part of V4 fix. V4 is about direct field access in app.rs event handlers. Full encapsulation would require changes to DecisionNavigationTree and its usage throughout the crate. Deferred to future work.

**Q: Should we add more methods for other tree operations?**
- A: No. The roadmap identified only these four. Other operations are either already encapsulated (`navigate_next`, `toggle_expansion`) or already use methods (`open_decision_modal`, `close_decision_modal`). Adding more would be gold-plating.

**Q: Why is `close_modal_if_open()` a combined operation?**
- A: It's more ergonomic than requiring callers to check then call. The pattern "close if open" is clear and common. It also prevents potential bugs from checking and then the state changing between operations.

## Assumptions

**A1**: The four methods in the roadmap represent all V4 violations in app.rs
- Verified by inspection of app.rs event handlers
- No other direct field access to `decision_tree` fields found
- All accesses use proper methods or through new encapsulation methods

**A2**: Phase 1 changes (immutable view signatures) don't affect Phase 2 implementation
- Phase 2 changes are orthogonal to view function signatures
- No conflicts or dependencies
- Both phases can coexist without issues

**A3**: HeadlessApp needs same changes as ReviewTuiApp
- Both implementations are identical in structure
- Both are behind feature gate (test harness)
- Updating both maintains consistency

## Deferred Decisions

**Why DecisionNavigationTree Methods Aren't Wrapped**:
- `open_decision_modal()` and `close_decision_modal()` are already proper methods
- Wrapping them would add redundancy
- They're used correctly in ShowDecisionModal handler
- The issue is checking `show_decision_modal` (field access), not the methods

**Why Tree Building Stays in UI Layer**:
- V5 violation (business logic in UI layer)
- Different from V4 (direct field access)
- Requires changes to diffviz-review crate
- Deferred to separate Phase 5 work

## Implementation Quality

**Code Organization**:
- Methods added at end of UiState impl block (lines 417-439)
- Clear doc comments for each method
- Consistent with existing method style

**No Regressions**:
- All tests pass without modification
- Behavior unchanged
- Zero breaking changes

**Maintainability**:
- Intent-expressing method names
- Clear separation of concerns
- Easy to extend in future phases

