# Phase 6 Human Output Format Design

## Overview

The `--human` flag transforms JSON debug output into readable text format optimized for manual developer review. Output uses ASCII tree visualization with ANSI colors for visual hierarchy and emphasis.

## Output Structure

### 1. Header Section

```
┌──────────────────────────────────────────┐
│ DiffViz Debug Analysis - Human Format    │
└──────────────────────────────────────────┘

File:     src/main.rs
Language: rust
From:     HEAD~1
To:       working-tree
```

**Styling:**
- Box borders: dim white (`\x1b[2m`)
- Section title: bold cyan (`\x1b[1;36m`)
- Labels (File, Language, From, To): bold (`\x1b[1m`)
- Values: plain text

### 2. Metadata Section

```
Metadata
  File Size:      2,048 bytes
  Lines Changed:  15 added, 8 deleted
  Analysis Time:  127 ms
  Phases:         7 (all analyzed)
```

**Styling:**
- Section header: bold cyan
- Labels: dim white
- Numeric values: plain text

### 3. Phase Summaries (One per Phase)

```
Phase 1: AST Outline
  Status: ✓ Complete
  Nodes:  42 total
    - Functions: 5
    - Structs:   2
    - Statements: 35

Phase 2: Semantic Pairs
  Status: ✓ Complete
  Matched:  18
  Added:    7
  Deleted:  3
```

**Styling:**
- Phase header: bold yellow (`\x1b[1;33m`)
- Status icon: green check (✓) for complete
- Counters: plain text

### 4. Phase 4/5/6/7 Tree Visualization

For phases with DiffNode hierarchy (4, 6, 7), show full tree:

```
Phase 4: DiffNode Hierarchy
  Status: ✓ Complete
  Tree:
    ├─ Function (modified) [Essential relevance] ESSENTIAL
    │  ├─ Parameter (added) [Important relevance] IMPORTANT
    │  └─ Statement (modified) [Important relevance] IMPORTANT
    └─ Struct (unchanged) [Background relevance] BACKGROUND
       └─ Field (unchanged) [Noise relevance] NOISE
```

**Tree Structure:**
- Prefix: `├─` for non-last sibling, `└─` for last sibling
- Continuation: `│ ` for open parent, `  ` for closed
- Node info: `[kind] (change_status) [relevance_label] {explanation}`

**Styling:**
- Tree characters: bright white (`\x1b[1;37m`)
- Node kind: plain text
- Change status: yellow (`\x1b[33m`)
- Relevance label in brackets: color-coded:
  - Essential: bright green (`\x1b[1;32m`)
  - Important: bright yellow (`\x1b[1;33m`)
  - Background: bright black/dim (`\x1b[2m`)
  - Noise: red (`\x1b[31m`)
- Explanation (if --explain-folding): plain text after relevance label

### 5. Footer Section

```
────────────────────────────────────────────
Debug output complete. 7 phases analyzed.
Total nodes in hierarchy: 12
Filtering: none (full file)
```

**Styling:**
- Separator: dim white
- Summary: plain text

## Implementation Details

### Node Rendering Function

```
fn format_tree_node(
    node: &SerializableDiffNode,
    depth: usize,
    is_last: bool,
    explain_folding: bool,
) -> String {
    // Construct prefix based on depth and is_last
    // Format: kind (change_status) [relevance] {explanation}
    // Apply colors using ANSI codes
}
```

### Color Constants

- Bold cyan (headers): `\x1b[1;36m`
- Bold yellow (phase nums): `\x1b[1;33m`
- Bright green (Essential): `\x1b[1;32m`
- Bright yellow (Important): `\x1b[1;33m`
- Dim (Background): `\x1b[2m`
- Red (Noise): `\x1b[31m`
- Reset: `\x1b[0m`

## Output Examples

### Minimal Tree (no explanations)
```
Phase 4: DiffNode Hierarchy
  └─ Function (modified) [Essential]
     ├─ Parameter (added) [Important]
     └─ Body (modified) [Essential]
```

### Detailed Tree (with --explain-folding)
```
Phase 4: DiffNode Hierarchy
  └─ Function (modified) [Essential] Essential relevance: modified function Function
     ├─ Parameter (added) [Important] Important relevance: added parameter SignatureComponent
     └─ Body (modified) [Essential] Essential relevance: modified statement Block
```

## Constraints & Assumptions

1. **ANSI Colors**: Output assumes terminal supports ANSI color codes. Non-terminal piping may show escape codes as visible text.
2. **Node Depth**: Tree nesting up to 10+ levels is acceptable for most real diffs. Beyond 15, consider truncation.
3. **Line Length**: Target ≤ 120 chars per line for readability. Long node types may wrap or abbreviate.
4. **Phase 1 Exception**: Phase 1 (AST outline) shown as count summary only, not full tree (too verbose).
5. **Phases 2/3/5 Exception**: These phases shown as table/list summaries, not trees (no hierarchy).

## Read Next

Implementation in dev-contribute will:
- Create `format_human_output()` function as alternative to JSON output
- Implement tree traversal and color application via ANSI codes
- Integrate --human flag into DebugCommand execute() flow
- Test output format and colors (visual inspection)
