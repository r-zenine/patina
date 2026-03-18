# Debug Subcommand - Design Specification

## Command Structure

```bash
diffviz debug \
  --file <path>              # File to analyze (required)
  --from <ref>               # Git ref for old code (default: HEAD)
  --to <ref>                 # Git ref for new code (default: working_tree)
  --line-range <start>-<end> # Optional: filter ReviewableDiffs by boundary overlap
  --phase <1,2,3,4,5,6,7>   # Optional: show only specific phases
  --explain-folding          # Optional: show WHY each node got its relevance
  --export-fixture <path>    # Optional: save minimal ReviewFixture JSON
  --human                    # Optional: human-readable output (default: JSON)
```

## Processing Flow

1. Validate inputs (file exists, git refs valid)
2. Run full ReviewEngineBuilder pipeline (git → semantic analysis → review → rendering)
3. Filter ReviewableDiffs to those overlapping `--line-range` (if provided)
4. Extract output from each phase:
   - Phase 1: Tree-sitter AST (structure outline)
   - Phase 2: SemanticTree (nodes + metadata)
   - Phase 3: SemanticPairs (matched/added/deleted)
   - Phase 4: ReviewableDiffs (boundaries + counts)
   - Phase 5: DiffNode hierarchy (tree + relevance scores)
   - Phase 6: RenderableDiff (Myers diff applied)
   - Phase 7: Final output (ready for TUI)
5. Apply `--explain-folding`: add reasoning for each DiffNode's relevance
6. Output as JSON (or human-readable with `--human`)
7. If `--export-fixture`: save ReviewFixture (old_code, new_code, file_path, language)

## JSON Output Structure

```json
{
  "file_path": "src/main.rs",
  "language": "rust",
  "query": { "from": "HEAD", "to": "working_tree" },
  "line_range_filter": { "start": 50, "end": 100 },
  "phases": {
    "2_semantic_tree": { "nodes": [...] },
    "3_semantic_pairs": { "matched": N, "pairs": [...] },
    "4_reviewable_diffs": { "count": N, "diffs": [...] },
    "5_diff_node_hierarchy": { "explanation": "...", "nodes": [...] },
    "6_renderable_diff": { "lines": [...] }
  },
  "metadata": { "analysis_duration_ms": N, "diffs_after_filtering": N }
}
```

## Implementation Notes

- Reuse ReviewEngineBuilder (orchestrates phases 1-5)
- After build, walk state.reviewable_diffs and filter by line_range overlap
- For `--explain-folding`: inspect each DiffNode's semantic_kind/unit_type; explain relevance assignment
- Use serde for JSON serialization (may need custom serializers for domain types)
- New file: `diffviz-cli/src/commands/debug.rs`
- Update: `diffviz-cli/src/main.rs` to register command
