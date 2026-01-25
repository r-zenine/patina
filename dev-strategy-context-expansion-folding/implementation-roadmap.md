# Implementation Roadmap: Context Expansion and Folding

**Strategy**: Core-then-Integrate
**Scope**: Context Expansion + TUI Validation

---

## Phase 1: Core Algorithm Implementation

**Objective**: Build context expansion algorithm as pure function

### Tasks

1. **Create context expansion helper function**
   - Location: `diffviz-core/src/reviewable_diff.rs` (add before line 288)
   - Function signature:
     ```rust
     fn build_context_tree_from_change<'source>(
         change: &ASTChange<'source>,
         parser: &dyn LanguageParser,
     ) -> ChangeWithContext<'source>
     ```
   - Algorithm steps:
     1. Get primary node and its semantic kind
     2. Get context boundaries from parser
     3. Walk up AST to find first matching boundary
     4. Build rich ContextNode tree recursively
     5. Assign relevance scores based on relationship to change

2. **Implement boundary detection**
   - Walk parent chain using `node.parent()`
   - Compare semantic kinds against boundary list
   - Stop at first match or root
   - Handle edge case: no matching boundary found → use primary node

3. **Implement recursive context tree builder**
   - Function signature:
     ```rust
     fn build_context_tree_recursive<'source>(
         node: NodeRef<'source>,
         change_node: &NodeRef<'source>,
         parser: &dyn LanguageParser,
         depth: usize,
     ) -> ContextNode<'source>
     ```
   - Determine relevance based on node relationship to change:
     - On change path: Check ancestry to change_node
     - Siblings: Use `classify_leaf_relevance()` default
   - Iterate children using TreeSitter cursor pattern
   - Recursively build child ContextNodes
   - Stop at reasonable depth (e.g., 10 levels) to prevent infinite recursion

4. **Implement relevance scoring logic**
   - Helper function:
     ```rust
     fn is_on_change_path<'source>(
         node: NodeRef<'source>,
         change_node: &NodeRef<'source>,
     ) -> bool
     ```
   - Algorithm: Walk up from change_node, check if any ancestor matches node
   - Nodes on path: ESSENTIAL
   - Siblings: Use parser's `classify_leaf_relevance()`

### Acceptance Criteria

- Function compiles without warnings
- Handles all ASTChange variants (Addition, Deletion, ContentChange, etc.)
- Returns ChangeWithContext with rich ContextNode tree
- No panics on edge cases (root node, missing parent, etc.)

---

## Phase 2: Integration with Pipeline

**Objective**: Replace trivial logic in `expand_changes_to_reviewable_diffs()`

### Tasks

1. **Replace single-node ContextNode creation**
   - Location: `diffviz-core/src/reviewable_diff.rs:302-305`
   - Old code:
     ```rust
     let change_with_context = ChangeWithContext {
         original_changes: vec![change.clone()],
         context_boundary: *change.primary_node(),
         context_tree: ContextNode::new(*change.primary_node(), ESSENTIAL),
     };
     ```
   - New code:
     ```rust
     let change_with_context = build_context_tree_from_change(change, parser);
     ```

2. **Verify existing conversion logic still works**
   - `convert_context_node_to_diff_node()` should handle rich trees
   - Check: Relevance override for nodes with changes (lines 138-142)
   - Check: Recursive child processing works with deep trees

3. **Preserve multi-change merging behavior**
   - Current code maps each change individually
   - Merging happens later in ReviewableDiff construction
   - Verify: No regression in boundary merging tests

### Acceptance Criteria

- All existing tests pass
- No compiler warnings
- `expand_changes_to_reviewable_diffs()` returns ReviewableDiffs with rich context
- Multi-change scenarios still merge correctly

---

## Phase 3: Unit and Integration Testing

**Objective**: Verify context expansion correctness

### Tasks

1. **Create integration test file**
   - Location: `diffviz-core/tests/context_expansion_tests.rs`
   - Test cases:
     - Simple function with 1-line change
     - Function with multiple changes
     - Nested structures (impl blocks in modules)
     - Different change types (Content, Structural, Reorder)

2. **Test boundary detection**
   - Verify correct boundary selection for each change type
   - Test edge cases:
     - Change at root level (no parent)
     - Change type with no matching boundary
     - Multiple matching boundaries (should pick first)

3. **Test relevance scoring**
   - Verify ESSENTIAL for changed nodes
   - Verify IMPORTANT for direct parents
   - Verify BACKGROUND for imports
   - Verify NOISE for comments
   - Test: Siblings of changed nodes get correct default relevance

4. **Test ContextNode tree structure**
   - Verify children are populated
   - Verify recursive structure is correct
   - Verify depth doesn't exceed reasonable bounds

5. **Run full test suite**
   - Command: `cargo test --package diffviz-core`
   - Fix any regressions
   - Ensure zero warnings

### Acceptance Criteria

- All new tests pass
- All existing tests still pass
- Test coverage includes boundary detection, relevance scoring, tree structure
- No compiler or clippy warnings

---

## Phase 4: Fixture Enhancement

**Objective**: Create realistic test data to exercise folding

### Tasks

1. **Enhance rust_trait_impl.json**
   - Expand to 50+ lines
   - Add:
     - Import statements at top
     - Multiple trait methods
     - Comments and docstrings
     - Helper functions within impl block
   - Ensure 1-2 actual changes amid much context
   - Expected folding: Imports (BACKGROUND), comments (NOISE), unchanged methods (BACKGROUND)

2. **Enhance typescript_react_component.json**
   - Expand to 50+ lines
   - Add:
     - Multiple import statements
     - Type definitions
     - JSX with comments
     - Helper functions
     - Props documentation
   - Ensure 1-2 actual changes
   - Expected folding: Imports (BACKGROUND), type definitions (BACKGROUND), comments (NOISE)

3. **Verify enhanced fixtures load correctly**
   - Test: `cargo test --package diffviz-review`
   - Verify MockDiffProvider loads enhanced JSON
   - Check: Expected line stats are updated

### Acceptance Criteria

- Both enhanced fixtures are 50+ lines
- Fixtures include realistic imports, comments, structure
- Fixtures have 1-2 changes amid much context
- MockDiffProvider loads fixtures without errors
- Line stats in JSON match actual additions/deletions

---

## Phase 5: TUI Validation

**Objective**: Confirm folding works visually in interactive TUI

### Tasks

1. **Build TUI with enhanced fixtures**
   - Command: `cargo build --package diffviz-review-tui`
   - Verify builds without warnings

2. **Run TUI interactively**
   - Command: `cargo run --package diffviz-review-tui`
   - Navigate to enhanced fixture diffs

3. **Test folding toggle**
   - Keybinding: Space+t+c (toggle context folding)
   - Verify:
     - BACKGROUND and NOISE lines hide when folding enabled
     - ESSENTIAL lines always visible
     - Changed lines never hidden
     - Folded content shows "..." or similar indicator

4. **Visual inspection**
   - Check: Function signatures always visible
   - Check: Changed lines always visible
   - Check: Imports can be hidden
   - Check: Comments can be hidden
   - Check: Unfolding shows all content again

5. **Test with both enhanced fixtures**
   - rust_trait_impl.json: Verify trait methods fold correctly
   - typescript_react_component.json: Verify JSX and imports fold correctly

### Acceptance Criteria

- TUI launches without errors
- Folding toggle (Space+t+c) works
- BACKGROUND/NOISE lines hide correctly
- ESSENTIAL lines always visible
- Changed lines never hidden
- Visual confirmation with both Rust and TypeScript fixtures

---

## Phase 6: Cleanup and Documentation

**Objective**: Polish implementation and ensure quality

### Tasks

1. **Run full workspace checks**
   - `cargo fmt --all` - Format code
   - `cargo clippy --workspace` - Fix all warnings
   - `cargo check --workspace` - Verify compilation
   - `cargo test --workspace` - Run all tests

2. **Code review checklist**
   - No defensive programming (fail fast)
   - No string/regex operations (TreeSitter only)
   - All edge cases handled or explicitly panicked
   - Function signatures documented
   - Complex logic has comments

3. **Update improvement tracking**
   - Mark Task 0 complete in `improvements/001_context_expansion_folding.md`
   - Document any deviations from original plan
   - Note any discovered issues for future work

4. **Verify zero warnings rule**
   - Check: No compiler warnings
   - Check: No clippy warnings
   - Fix any new warnings introduced

### Acceptance Criteria

- Zero compiler warnings
- Zero clippy warnings
- All tests pass
- Code follows project conventions
- Improvement document updated

---

## Success Metrics

### Functional Metrics

1. **Context Expansion Works**
   - ContextNode trees have children (not single nodes)
   - Context boundaries correctly identified
   - Relevance scores varied (not all ESSENTIAL)

2. **Folding Behavior Correct**
   - Lines with relevance >= BACKGROUND can fold
   - Changed lines never fold
   - ESSENTIAL lines never fold
   - 60-80% of lines foldable in 50-line function with 1 change

3. **No Regressions**
   - All existing tests pass
   - Multi-change merging still works
   - Performance not degraded

### Quality Metrics

1. **Zero Warnings**: All compiler and clippy warnings resolved
2. **Test Coverage**: New functionality has integration tests
3. **Visual Validation**: TUI confirms folding works end-to-end

---

## Implementation Sequence

```
Phase 1: Core Algorithm
  ↓
Phase 2: Integration
  ↓
Phase 3: Testing
  ↓
Phase 4: Fixtures
  ↓
Phase 5: TUI Validation
  ↓
Phase 6: Cleanup
```

Each phase builds on the previous. Do not proceed to next phase until current phase acceptance criteria are met.

---

## Estimated Complexity

**Phase 1 (Core Algorithm)**: Medium - Complex tree traversal logic
**Phase 2 (Integration)**: Low - Simple replacement
**Phase 3 (Testing)**: Medium - Comprehensive test coverage
**Phase 4 (Fixtures)**: Low - JSON editing
**Phase 5 (TUI Validation)**: Low - Manual testing
**Phase 6 (Cleanup)**: Low - Cleanup and documentation

---

## Risk Mitigation

### Risk: TreeSitter node lifetime issues
**Mitigation**: Use NodeRef wrapper, leverage existing patterns from codebase

### Risk: Performance degradation from deep tree traversal
**Mitigation**: Add depth limit to recursive builder, profile if needed

### Risk: Regressions in multi-change merging
**Mitigation**: Run full test suite after Phase 2, fix before proceeding

### Risk: Fixtures don't exercise folding properly
**Mitigation**: Start with Rust (best understood), validate with TUI before TypeScript

---

## Next Steps

1. Begin Phase 1: Implement core context expansion algorithm
2. Write helper functions in `diffviz-core/src/reviewable_diff.rs`
3. Test boundary detection logic manually
4. Proceed to Phase 2 integration when algorithm compiles without warnings
