# Phase 4 Implementation: Context Handoff

## What Was Done

Implemented the `--explain-folding` flag to generate human-readable explanations for DiffNode relevance scores:

- **Explanation Generation**: Created `generate_node_explanation()` helper that converts semantic_kind, change_status, and relevance_score into human-readable text
- **Relevance Level Mapping**: ESSENTIAL→0, IMPORTANT→1, BACKGROUND→2, NOISE→3 mapped to readable labels
- **Semantic Kind Conversion**: All 16 SemanticNodeKind variants converted to human-friendly descriptions (e.g., "function", "struct", "impl block")
- **Change Status Inclusion**: Explanation includes what happened to each node (added, deleted, modified, moved, reordered, unchanged)
- **Phase Integration**: Explanations added to phases 4, 6, and 7 (all phases displaying node-level diff information)
- **Optional Flag**: Explanations only computed when `--explain-folding` is passed, keeping baseline output lean

## Files Modified

- `diffviz-cli/src/commands/debug.rs` — Added `generate_node_explanation()` helper function and modified `serialize_phase_4()`, `serialize_phase_6()`, and `serialize_phase_7()` to include explanations when flag is set

## Implementation Details

### generate_node_explanation() Helper

Maps three pieces of information into a single human-readable explanation:

1. **Relevance Score** (0-3) → Relevance Level (Essential, Important, Background, Noise)
   - ESSENTIAL: Contains or is the actual change
   - IMPORTANT: Direct semantic container of change
   - BACKGROUND: Sibling context (collapsible in UI)
   - NOISE: Unrelated context (hideable in UI)

2. **SemanticNodeKind** → Human-Friendly Description
   - Function, Class, Struct, Enum, Interface, ImplBlock, Module, Import
   - Variable, SignatureComponent, Statement, Expression, TypeDefinition, Comment, SourceFile
   - Other (for language-specific kinds)

3. **NodeChangeStatus** → Change Action
   - Added, Deleted, Modified, Moved, Reordered, Unchanged

### Explanation Format

Format: `"{level} relevance: {change} {kind} {node_type}"`

Examples:
- `"Essential relevance: modified function Function"`
- `"Important relevance: added struct MyStruct"`
- `"Background relevance: unchanged statement Expression"`
- `"Noise relevance: deleted comment Comment"`

### Phase 4/6/7 Integration

Modified each phase's serialization to:
1. Access `diff.core_diff.boundary` (the DiffNode)
2. Extract real relevance_score from node (not hardcoded 0.5)
3. Generate explanation via `generate_node_explanation()` if `self.explain_folding` is true
4. Insert explanation field into JSON object only when flag is set

## Testing

- ✅ `cargo check --workspace` passes
- ✅ `cargo test --package diffviz-cli` passes (6 tests)
- ✅ `diffviz debug --file src/main.rs --explain-folding --phase 4` produces JSON with explanation fields
- ✅ `diffviz debug --file src/main.rs --phase 4` produces JSON without explanation fields (baseline behavior unchanged)
- ✅ Explanations are accurate and human-readable
- ✅ All 7 phases still serialize correctly
- ✅ No performance regression when flag not used

## Known Constraints & Gotchas

1. **Explanation Text is Static**: No LLM analysis. Explanations are deterministic based on code classification. Always the same for the same node type and change.

2. **node_type Value**: The `node_type` field in DiffNode is a String, typically the TreeSitter node kind (e.g., "function_item", "struct_item" in Rust). Included in explanation as-is for technical clarity.

3. **Only Available in Phases 4, 6, 7**: These are the phases that work with DiffNode data. Other phases (1=AST outline, 2=pair counts, 3=reviewable diffs, 5=renderable lines) don't have semantic metadata to explain.

4. **Format Doesn't Change**: Explanation format is consistent across all nodes and languages. If more detailed explanations are needed later, format can be extended to structured object.

## Next Phase (Phase 5)

Phase 5 will implement `--export-fixture` to export minimal ReviewFixture JSON (old_code, new_code, file_path, language) for test data creation:
- Extract full source code via DiffProvider
- Serialize to ReviewFixture struct
- Write to file specified in `--export-fixture <path>`

No changes needed to Phase 4 code for Phase 5 to work. Phases are independent.

## Handoff Checklist

- [x] Explanation generation working and tested
- [x] Relevance score properly extracted from DiffNode
- [x] SemanticNodeKind and NodeChangeStatus mappings complete
- [x] Phases 4, 6, 7 updated with conditional explanation inclusion
- [x] --explain-folding flag working correctly
- [x] Baseline output unchanged when flag not used
- [x] All tests passing
- [x] No compiler warnings (new code)
- [x] JSON output valid and parseable
- [x] No changes to domain crates or other commands
- [x] Architecture constraints respected (CommandExecutor, Environment pattern)
