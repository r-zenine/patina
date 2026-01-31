# Decision Log - Phase 3: Visual Components (Tasks 3.2-3.6)

## Decisions Made During Implementation

### Decision: How to Display Approval Status in Decision Details Panel

**Challenge**: Need to show approval status at depth 0 when decision is selected.

**Options Considered**:
1. Show only approval icon (✓/○)
2. Show icon + progress count "(X/Y)"
3. Show full status line "(X/Y chunks approved)"

**Decision**: Show icon + progress count "(X/Y)"

**Rationale**:
- Concise: Fits well in title line without taking much space
- Informative: Shows both approval state and completion progress
- Consistent: Matches pattern used in decision tree
- Readable: "(3/5)" is immediately understood

### Decision: Where to Place Approval Indicators in Decision Tree

**Challenge**: Decision tree already shows decision title, code impact count, expansion icon, and selection indicator. Where to add approval?

**Options Considered**:
1. After expansion icon: `►  ✓  1. Title [2]`
2. Before title: `►  ✓ 1. Title [2]`
3. Before impact count: `►  1. Title ✓ [2]`
4. Replace title with approval status

**Decision**: Place after expansion icon, before title (Option 2)

**Rationale**:
- Consistent with UI conventions (status indicators before content)
- Logical flow: selection > expand > approval > content
- Easy to scan: approval status visible immediately
- No layout conflicts: doesn't interfere with impact count
- Visual balance: approval icon + title + progress + impact count

### Decision: How to Color-Code Approval Status in Tree

**Challenge**: Need to visually distinguish approved vs pending decisions.

**Options Considered**:
1. Green for approved, red for pending
2. Green for approved, gray for pending
3. Bold for approved, normal for pending
4. Different icons (✓ vs ✗)

**Decision**: Green for approved, gray for pending (with different icons ✓/○)

**Rationale**:
- Accessible: Works in both color and monochrome terminals
- Consistent: Uses existing Icons::APPROVED and Icons::NOT_APPROVED
- Subtle: Gray is less aggressive than red for "not approved"
- Immediate recognition: Green = done, Gray = pending

### Decision: Keybinding for Decision Approval

**Challenge**: Space+a menu already has "a" and "f" for chunk and file. Need keybinding for decision.

**Options Considered**:
1. Reuse Space+a with context detection (already done in contribution 005)
2. New keybinding Space+a+d
3. New keybinding Space+d
4. Make it configurable

**Decision**: Use Space+a+d (context-aware, depth-based)

**Rationale**:
- Already implemented in contribution 005 (event system)
- Consistent with existing pattern: Space+a+a (chunk), Space+a+f (file)
- Natural extension: Space+a+d (decision)
- Only shows when available (depth 0)
- No new cognitive load: same menu, different options based on context

### Decision: Context-Aware Keybinding Display Strategy

**Challenge**: Should "d" option always appear in Actions menu, or only at depth 0?

**Options Considered**:
1. Always show "d", but make it greyed out when not available
2. Conditionally show "d" only at depth 0
3. Show separate menu for decisions
4. Use different leader key for decisions

**Decision**: Conditionally show "d" only at depth 0

**Rationale**:
- Less clutter: Menu only shows relevant options for current context
- Intuitive: Pressing Space+a+d at depth 1 makes no sense, so don't show it
- Vim-style: Vim doesn't show commands that are invalid in current mode
- Simple: No greyed-out/disabled state logic needed
- Reduces cognitive load: Fewer options = clearer menu

### Decision: How to Calculate and Display Progress

**Challenge**: Need to show "3/5" chunks approved for each decision.

**Options Considered**:
1. Store cached progress in ReviewEngine (performance optimization)
2. Calculate on-demand during every render
3. Calculate once per frame and cache in UI state
4. Display just the count, not the total

**Decision**: Calculate on-demand using ReviewEngine methods

**Rationale**:
- YAGNI principle: No need to optimize until there's a problem
- Simple: One method call per decision per render
- Accurate: Always reflects actual approval state
- No state synchronization: Can't go stale
- Performance acceptable: Decision tree typically has <100 decisions

### Decision: When to Pass Context to Submenu Functions

**Challenge**: `create_actions_submenu()` needs to know tree depth, but other menus don't.

**Options Considered**:
1. Pass full UiState to all submenu functions
2. Pass only depth parameter
3. Create context object
4. Pass UiState only to actions_submenu

**Decision**: Pass UiState only to create_actions_submenu()

**Rationale**:
- Minimal changes: Only modify what's needed
- Flexible: If other menus need context later, pattern is already established
- Type-safe: UiState provides full context, not just depth
- Future-proof: Can check other properties if needed (e.g., modal state)

### Decision: Approval Icon in Decision Details Panel vs Tree

**Challenge**: Icons and colors in decision details panel are different than tree.

**Options Considered**:
1. Use same icon/color in both places
2. Use different visual style in each (icon in tree, text in panel)
3. Use different colors based on context
4. Create separate icon set

**Decision**: Use same icon (✓/○) but different colors

**Rationale**:
- Consistency: Same symbol = same meaning everywhere
- Context-appropriate: Panel uses green/muted based on state
- Simple: No new icon set needed
- Semantic: Icon meaning is consistent

## Architectural Decisions Confirmed

### ELM Architecture Maintained

**Decision**: Keep all UI changes within pure view functions

**Reasoning**:
- Views remain pure: `&UiState` only, never `&mut`
- Event handling unchanged: Contribution 005 handles toggle
- Command pattern respected: No side effects in views
- Composability preserved: Views can be tested independently

### Depth-Routed Display Pattern

**Decision**: Continue using depth-based routing for display

**Reasoning**:
- Scalable: Easy to add new display types at each depth
- Maintainable: Clear separation of concerns
- Extensible: Future depth levels can add their own panels
- Proven pattern: Already working for existing displays

## Validation Decisions

### Testing Strategy for Phase 3

**Decision**: Defer TUI test harness tests to Task 3.7

**Reasoning**:
- Visual components are now ready to test
- Task 3.7 specifically covers test harness implementation
- Allows focused testing with proper test infrastructure
- Prevents coupling tests to implementation details

### Quality Assurance

**Decision**: Fix clippy warnings immediately

**Reasoning**:
- ZERO WARNINGS RULE is project requirement
- Format string inlining warnings are trivial but important
- Clean code baseline prevents technical debt
- Shows commitment to project standards

## Why These Decisions Were Right

1. **Context-aware keybindings**: Reduces menu clutter, more intuitive
2. **Conditional display**: Shows only relevant information
3. **On-demand calculation**: Keeps code simple and correct
4. **Consistent visual language**: Icon + color system already established
5. **Maintained ELM architecture**: Preserves testability and clarity

## Implementation Impact

**No Breaking Changes**:
- Existing keybindings work exactly as before
- Decision approval feature is purely additive
- No changes to ReviewEngine API
- All existing tests pass

**Additive Changes Only**:
- New visual elements in existing panels
- New keybinding option in existing menu
- New ReviewEngine queries (already implemented)
- Zero impact on chunk or file approval

## Lessons Learned

1. **Context-aware UI reduces complexity**: Showing only relevant options is better than showing all with some disabled

2. **On-demand calculations beat caching**: For small data sets, the simplicity and correctness of recalculation outweighs caching overhead

3. **Consistent visual language matters**: Using existing icons (✓/○) and colors is more important than perfect aesthetics

4. **Small function signatures are better**: Passing only what's needed (ui_state) is clearer than passing everything

## Future Considerations

1. **Approval feedback**: Could show "Decision and 3 chunks approved" message in status bar

2. **Cascading animation**: Could animate progress counter when bulk approval happens

3. **Undo capability**: Could add ability to undo approval cascades

4. **Approval timeline**: Could show when each chunk/decision was approved

5. **Keybinding customization**: Could allow users to rebind Space+a+d to custom key

These are all enhancements, not requirements. Current implementation is complete and correct.
