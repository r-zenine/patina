# Context Document: Decision-Based Review Pipeline

## Behavioral Spec

Replace the semantic pairing algorithm (`build_semantic_pairs()`) with a decision-range-based approach. Decisions already specify files and line ranges via `CodeImpact`. Instead of discovering changes through AST comparison, use those ranges as the source of truth:

1. Take a decision's CodeImpact (file path + line range)
2. Parse both old and new file versions with tree-sitter
3. Build semantic trees for both
4. Find the semantic unit covering the target range in the new file
5. Expand the range to cover complete semantic unit boundaries
6. Look up the same-named unit in the old file's semantic tree
7. Classify: addition (no old match), deletion (unit gone from new), modification (both exist)
8. Produce a ReviewableDiff with proper DiffNode tree and context
9. Feed into existing RenderableDiff pipeline (unchanged)

## Architecture Summary

### Current Pipeline (being replaced)
```
DiffProvider → changed files → for each file:
  old/new source → tree-sitter parse → semantic trees
  → build_semantic_pairs(old_tree, new_tree)     ← REMOVED
  → semantic_pairs_to_reviewable_diffs()          ← REMOVED
  → ReviewableDiffs
Then: Decisions → build_index_from_review_state() → maps to existing diffs
```

### New Pipeline
```
Decisions (CodeImpact: file + range) → for each impact:
  old/new source via DiffProvider → tree-sitter parse → semantic trees
  → find_semantic_unit_at_range(new_tree, range)   ← NEW
  → expand range to unit boundaries                 ← NEW
  → find_unit_by_name(old_tree, unit_name)          ← NEW (lightweight lookup)
  → classify change type                            ← NEW
  → build ReviewableDiff with DiffNode tree          ← NEW (reuses existing DiffNode/context patterns)
  → ReviewableDiff
Then: ReviewableDiffs feed directly into RenderableDiff pipeline (unchanged)
```

### What's Preserved
- Tree-sitter parsing (Phase 1)
- Semantic tree building (Phase 2)
- DiffNode hierarchy with relevance scores (ESSENTIAL/IMPORTANT/BACKGROUND)
- Context expansion via `build_child_nodes_with_context()`
- ReviewableDiff structure (unchanged)
- RenderableDiff pipeline including Myers diff (unchanged)
- TUI rendering and navigation (unchanged)

### What's Removed
- `build_semantic_pairs()` / `build_semantic_pairs_with_coverage()`
- `semantic_pairs_to_reviewable_diffs()` and `reviewable_diff_from_semantic.rs`
- `SemanticPair` enum (replaced by direct change classification)
- `SemanticSimilarity` and `CoverageStats` types
- Parent-child marking helpers (`mark_node_and_children_as_used`, `should_mark_children_as_used`)
- `create_unmapped_decision()` and Decision 0 concept
- `build_index_from_review_state()` (decisions drive creation, not post-hoc mapping)
- `semantic_unit_partitioner.rs` (already deprecated)
- `LanguageParser::compare_semantic_units()` (only used by pairing)

### Key Design: Lightweight Name Matching

The new approach does NOT pair all units. It performs a targeted lookup:

1. From the decision's CodeImpact, we know: file `src/auth.rs`, lines 10-30
2. Parse new file → semantic tree → find unit at lines 10-30 → e.g., `fn authenticate()`
3. Parse old file → semantic tree → scan for unit named `authenticate` with same type
4. If found: modification (diff old range vs new range)
5. If not found: addition

This is O(n) where n = number of units in the old file, versus O(n*m) for full pairing.

### Flow Inversion

The fundamental change is that decisions now **drive** ReviewableDiff creation instead of mapping to pre-existing diffs:

- **Before**: Git changes → ReviewableDiffs → Decisions map to them
- **After**: Decisions → ReviewableDiffs created from their CodeImpacts

This eliminates the `build_index_from_review_state()` step entirely. The relationship between decisions and ReviewableDiffs is established at creation time, not through post-hoc overlap detection.
