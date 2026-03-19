# Phase 6 Design: Human Output Format - Handoff

## What Was Designed

Interactive design session determined human-readable output format for `--human` flag:

- **Target audience**: Manual developer review (optimized for readability)
- **Tree visualization**: Full hierarchy with ASCII box-drawing (├──, └──)
- **Colors**: ANSI codes with semantic color-coding for relevance levels
- **Detail level**: Complete tree for phases 4/6/7; summary counts for others
- **Explanations**: Include --explain-folding text inline when flag passed

## Key Design Decisions

1. **Phases 4, 6, 7 as trees** — Show full DiffNode hierarchy with colors
2. **Relevance color scheme** — Green (Essential), Yellow (Important), Dim (Background), Red (Noise)
3. **Node format** — `[kind] (change_status) [relevance_label]` + optional explanation
4. **ANSI colors for emphasis** — Headers bold cyan, phase numbers yellow, relevance color-coded
5. **Header + phases + footer** — Clear sections with metadata, summaries, and completion status

## Implementer Notes

Implementation should:
- Create `format_human_output()` function that mirrors JSON serialization phases
- Use tree traversal with depth tracking for proper prefix formatting (├──, └──, │)
- Apply ANSI color codes via constants for consistency
- Integrate --human flag into DebugCommand.execute() as alternative to JSON output
- Test visual output (colors render correctly in terminal)

No changes needed to Phase 1-5 implementations. Phase 6 is self-contained.

## Next Steps for Implementation

1. Create format_human_output() entry point
2. Implement phase-by-phase formatters (summary or tree based)
3. Implement tree_format_node() with prefix/color logic
4. Wire --human flag into execute() flow
5. Test with real diffs to verify colors and tree structure
