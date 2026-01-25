The goal is to make sure context expansion and folding work properly

Tasks

  0. finish the implementation of context expansion that is not fully implemented ( see contenxt below )
  1. Understand current fixture state (Done - main.rs has 3 decisions)
  2. Create complex fixtures with all relevance levels
    - Current fixtures are too simple (just trait impl changes)
    - Need to create code that generates:
        - ESSENTIAL lines: actual changes (modified lines, added functions)
      - IMPORTANT lines: containing functions/variables with changes
      - BACKGROUND lines: module/import statements
      - NOISE lines: comments and surrounding statements
  3. Enrich fixture in main.rs
    - Replace or expand the 3 hardcoded decisions
    - Create realistic multi-file scenarios with imports, modules, comments
    - Ensure code impacts span lines where different relevance levels exist
  4. Add test fixtures to verify folding
    - Create integration tests that render diffs with folding enabled/disabled
    - Verify BACKGROUND and NOISE lines are hidden when not showing all context
    - Check that ESSENTIAL/IMPORTANT lines always show
    - Verify changed lines are never folded (even if BACKGROUND relevance)
  5. Test the folding in TUI
    - Use the test harness modes to verify visual rendering
    - Toggle context folding (Space+t+c) and verify lines appear/disappear correctly


+### How Context Expansion Should Work
 +When a single line changes in a long function:
 +1. **Context Expansion**: The change boundary expands to the nearest parent semantic unit (function, class, module)
 +2. **Relevance Classification**: Within that expanded scope:
 +   - Changed lines and their direct ancestors get marked ESSENTIAL
 +   - Sibling nodes (imports, comments, unchanged methods) retain natural relevance (BACKGROUND/NOISE)
 +3. **Folding**: Non-essential context (BACKGROUND/NOISE) can be hidden while keeping structural understanding
 +
 +**Example**: Change one line in a 50-line function
 +- Expanded boundary: entire function (50 lines)
 +- ESSENTIAL: the changed line + function signature + parent blocks
 +- BACKGROUND: import statements within function scope
 +- NOISE: comments, docstrings
 +- Foldable: ~30-40 lines of BACKGROUND/NOISE context

+**Expected Flow**:
 +1. AST change detected (e.g., single line modification)
 +2. `expand_changes_to_reviewable_diffs()` expands to parent semantic boundary
 +3. Build `ContextNode` tree with varied relevance:
 +   - Changed nodes: ESSENTIAL
 +   - Parent/ancestor nodes: ESSENTIAL (via `has_changes()` override)
 +   - Sibling nodes: Natural relevance (BACKGROUND for imports, NOISE for comments)
 +4. Convert to `ReviewableDiff` with proper `DiffNode` tree
 +5. Convert to `RenderableDiff` with line annotations
 +6. `should_fold()` hides BACKGROUND/NOISE lines
 +
 +**ACTUAL CURRENT STATE**:
 +The expansion step is **NOT IMPLEMENTED**.
 +
 +Location: `diffviz-core/src/reviewable_diff.rs:288-319`
 +
 +Current implementation:
 +```rust
 +pub fn expand_changes_to_reviewable_diffs() {
 +    // Creates simple ChangeWithContext for each change
 +    context_tree: ContextNode::new(*change.primary_node(), ESSENTIAL)
 +    // ^^ Just marks primary node as ESSENTIAL, no actual expansion!
 +}
 +```
 +
 +**Problem**:
 +- No parent boundary detection
 +- No context tree building
 +- No sibling classification
 +- Every line gets marked ESSENTIAL (relevance=0)
 +- **Result**: Nothing can be folded!
 +
 +### RelevanceScore Assignment Flow (BROKEN)
 +
 +Current broken flow:
 +1. ❌ **No Context Expansion**: Just primary changed node marked ESSENTIAL
 +2. ❌ **No Context Tree**: Children/siblings not classified
 +3. ✅ **Override for Changes**: `has_changes()` correctly upgrades to ESSENTIAL
 +4. ✅ **Line Annotation**: `collect_all_annotations()` works correctly
 +5. ❌ **Folding Fails**: All lines are ESSENTIAL, so `should_fold()` always returns false
