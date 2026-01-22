# Decision Log: Phase 1 Implementation

## Decisions Made During Implementation

### D1: Minimal Changes Approach

**Decision**: Change only the strictly necessary function signatures, update call sites, no refactoring of internal logic.

**Rationale**:
- Steel Thread strategy requires minimal changes per phase
- Reducing risk of unintended side effects
- Views don't actually mutate state, so no internal logic changes needed
- Compiler will catch any violations

**Outcome**: Changed signatures and one call site. No changes to function bodies.

### D2: Consistency with Existing Code

**Decision**: Noticed some components already used `&UiState`. Kept consistent with those rather than changing them.

**Rationale**:
- Components like `decision_tree.rs` were already correct
- Indicates previous developer was aware of ELM principle
- Confirms our approach is right direction

**Outcome**: Only changed the 2 components that violated the pattern.

### D3: Helper Functions

**Decision**: Also changed the `render_diff_content` helper function in `diff_view.rs` to use `&UiState`.

**Rationale**:
- Helper functions are part of view layer logic
- Should follow same immutability principle
- Consistency throughout module

**Outcome**: Added signature change for `render_diff_content` in addition to main `render` function.

## Questions Resolved

**Q: Do views actually mutate state?**
- A: No. The audit noted views never actually mutate state. This change makes that explicit in types.

**Q: Will this break anything?**
- A: No. Tested with `cargo check` and `cargo test`. All pass without modification.

**Q: Should we change all function parameters?**
- A: No. Only the UiState parameter. Frame and other parameters remain mutable as needed by ratatui.

## Technical Notes

### Compiler as Verification

The Rust compiler enforces these signature changes at compile time:
- If a view function tried to mutate UiState, it would fail to compile
- If a call site tried to pass `&mut self.ui_state`, it would fail to compile
- This provides strong verification that the change works correctly

### No Internal Changes Needed

No changes to function bodies because:
- Views already only read from state
- No mutations were happening
- Change is purely signature/contract

### Call Site Change

Only one call site needed updating:
- `ReviewTuiApp::render()` in `src/app.rs`
- Changed from passing `&mut self.ui_state` to `&self.ui_state`
- HeadlessApp doesn't directly call draw (test infrastructure difference)

## Integration with Other Phases

**Phase 1 as Foundation**:
- Phase 2 will add UiState methods (compatible with immutable references)
- Phase 3 will introduce Command system (doesn't affect view layer)
- Phase 4 will refactor business event handlers (doesn't affect view layer)
- Phase 5 will add timeout as message (doesn't affect view layer)

Pure view signatures are compatible with all subsequent phases.

## Lessons Learned

1. **ELM principles were partially there**: Several components already followed the pattern
2. **Small changes can clarify architecture**: Just changing signatures made the design intent clearer
3. **Rust compiler enforces contracts**: Type system provides strong verification
4. **Minimal is better**: Steel Thread approach kept this phase focused and low-risk
