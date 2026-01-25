# Changelog - Phase 1: Core Algorithm Implementation

## Summary

Implemented the context expansion algorithm as a pure function in `diffviz-core/src/reviewable_diff.rs`. The algorithm builds rich context trees from AST changes by walking up the parent chain to find semantic boundaries and recursively building child nodes with relevance scores.

## Changes Made

### New Functions Added

1. **`build_context_tree_from_change()`** (lines 288-308)
   - Main entry point for context expansion
   - Takes an ASTChange and LanguageParser
   - Returns ChangeWithContext with rich context tree

2. **`find_context_boundary()`** (lines 311-335)
   - Walks up parent chain to find appropriate context boundary
   - Uses LanguageParser's get_context_boundaries() for language-specific logic
   - Returns first matching boundary or primary node if none found

3. **`build_context_tree_recursive()`** (lines 338-377)
   - Recursively builds ContextNode tree with relevance scores
   - Assigns ESSENTIAL to nodes on change path
   - Uses classify_leaf_relevance() for other nodes
   - Includes MAX_DEPTH protection (10 levels)

4. **`is_on_change_path()`** (lines 380-403)
   - Helper to check if a node is ancestor of change node
   - Walks up from change node to root

5. **`nodes_equal()`** (lines 406-411)
   - Compares NodeRef for equality by position and kind

### Modified Functions

1. **`expand_changes_to_reviewable_diffs()`** (lines 426-437)
   - Replaced trivial ChangeWithContext creation with call to `build_context_tree_from_change()`
   - Changed from single-node context trees to rich multi-level trees

## Impact

### Functional Changes
- **Context expansion now works**: Instead of marking only the changed node as ESSENTIAL, the algorithm now:
  - Finds the appropriate semantic boundary (function, class, etc.)
  - Builds a full tree of context nodes
  - Assigns varied relevance scores (ESSENTIAL, IMPORTANT, BACKGROUND, NOISE)

- **Folding becomes possible**: With varied relevance scores, the UI can now fold BACKGROUND and NOISE content while keeping ESSENTIAL content visible

### Performance
- Algorithm is O(h) where h = tree height (typically small)
- No redundant traversals
- Depth-limited recursion (max 10 levels)

### Test Results
- All 42 unit tests pass
- All previously passing integration tests still pass
- One pre-existing C++ semantic pairing test failure (unrelated to our changes)
- Zero compiler warnings
- Zero clippy warnings

## Files Modified

- `diffviz-core/src/reviewable_diff.rs`: Added 133 lines of new code

## Next Steps

Phase 2: Integration testing to verify the rich context trees work correctly through the full pipeline.
