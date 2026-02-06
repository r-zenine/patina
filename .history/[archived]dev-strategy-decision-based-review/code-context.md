# Code Context: Decision-Based Review Pipeline

## Files to Modify

### diffviz-core/src/semantic_ast.rs
- `build_semantic_pairs()` (line 932) — Current pairing algorithm to be replaced. Greedy name matching, O(n^2) worst case.
- `build_semantic_pairs_with_coverage()` (line 1061) — Coverage-tracking variant, also to be removed.
- `can_pair_with()` (line 685) — Name+type matching logic. The lightweight name lookup in the new module will reuse this concept but simplified to a single-unit lookup.
- `SemanticPair` enum (line 420) — Matched/Addition/Deletion. Will be replaced by a simpler change classification.
- `SemanticNode` (line 129) — Universal semantic construct. Still needed for tree-sitter semantic analysis.
- `SemanticTree` (line 60) — Root container. Still needed for parsing.
- `mark_node_and_children_as_used()` (line 1024) — Parent-child dedup. Goes away with pairing removal.
- `should_mark_children_as_used()` (line 1045) — Full-file module special case. Goes away with pairing removal.

### diffviz-core/src/reviewable_diff_from_semantic.rs
- `semantic_pairs_to_reviewable_diffs()` (line 21) — Current bridge from SemanticPairs to ReviewableDiffs. Will be replaced by the new module's direct ReviewableDiff creation.
- `create_matched_diff()` — Handles modified pairs. Logic for building DiffNode trees with context is reusable.
- `create_addition_diff()` / `create_deletion_diff()` — Handles pure adds/deletes. Pattern is reusable.
- `build_child_nodes_with_context()` — Relevance assignment (ESSENTIAL/IMPORTANT/BACKGROUND). Reusable as-is.
- `should_create_diff_for_pair()` (line 60) — Filter for full-file module pairs. Goes away.

### diffviz-core/src/reviewable_diff.rs
- `ReviewableDiff` struct — Target output type. No changes needed, the new module produces these.
- `DiffNode` hierarchy — Context tree structure. No changes needed.
- `NodeChangeStatus` enum — Unchanged/Added/Deleted/Modified. No changes needed.

### diffviz-core/src/renderable_diff/mod.rs
- `RenderableDiff::from(&ReviewableDiff)` (line 375) — Downstream consumer. No changes needed, it consumes ReviewableDiffs regardless of how they were created.
- `myers_diff.rs` — Already used for line-level diffing in RenderableDiff. No changes.

### diffviz-core/src/parsers/*.rs
- `LanguageParser::try_parse()` — Still needed for tree-sitter parsing.
- `LanguageParser::build_semantic_tree()` — Still needed for semantic tree construction.
- `LanguageParser::compare_semantic_units()` — Only used by `build_semantic_pairs()`. Candidate for removal in cleanup.

### diffviz-core/src/semantic_unit_partitioner.rs
- Already marked DEPRECATED. Candidate for removal in cleanup phase.

### diffviz-review/src/review_engine_builder.rs
- `build()` method — Orchestrates the full pipeline. Phase 2 rewires this to use decisions as input.
- `create_semantic_reviewable_diffs()` — Current per-file pipeline (parse → semantic trees → pairs → diffs). Will be replaced.
- `extract_line_range_from_core_diff()` — LineRange extraction. Still needed.

### diffviz-review/src/entities/decision.rs
- `Decision` / `CodeImpact` / `DecisionLineRange` — Input types for the new pipeline. No changes to structure.
- `build_index_from_review_state()` — Current reverse-index builder. May no longer be needed since decisions now drive diff creation directly.
- `create_unmapped_decision()` — Goes away (user chose to ignore unmapped code).

### diffviz-review-tui/src/main.rs
- `create_test_review_engine()` — Test harness setup. Will need updating to use decision-driven pipeline.
- `create_hardcoded_decisions()` — Already provides decisions. Becomes the primary input.

## New Files

### diffviz-core/src/decision_based_diff.rs (NEW)
New module implementing:
- Semantic unit lookup by line range (find unit covering a given range)
- Semantic unit lookup by name in another tree (lightweight matching)
- Change classification (addition/deletion/modification)
- Direct ReviewableDiff construction from expanded ranges
