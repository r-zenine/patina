# Phase 4 Context Handoff: Export and Reporting for Decision Instructions

## What Was Done

**Commit**: `b9a5bae` - "feat: add export/import for decision instructions (Phase 4)"

**Deliverables Completed**:
- ✅ `DecisionExportScope` enum: `SingleDecision(u32)` | `All`
- ✅ `ExportedDecisionInstruction` struct (decision_number, content, author, timestamp, status)
- ✅ `ExportedDecisionInstructions` container struct with `_meta` (ExportMetadata) and `decision_instructions` fields
- ✅ `export_decision_instructions_json(scope: DecisionExportScope) -> Result<String>` implemented
- ✅ `import_decision_instructions_json(json: &str) -> Result<ImportSummary>` implemented
- ✅ 8 new tests covering: export all, single decision, empty, JSON structure, import success, invalid decision, round-trip, malformed JSON
- ✅ All 169 tests pass (up from 161 after Phase 3)
- ✅ Zero clippy warnings in `diffviz-review`
- ✅ `cargo fmt` applied

## Architecture Overview

Two new public methods on `ReviewEngine`:

```
ReviewEngine
├── export_decision_instructions_json(DecisionExportScope) -> Result<String>
│   ├── filters state.decision_instructions.instructions (HashMap<u32, Vec<Instruction>>)
│   ├── scope: All iterates all entries; SingleDecision(n) filters by key
│   └── wraps in ExportedDecisionInstructions with ExportMetadata (no query_formats/git_usage_examples — N/A for decision-level)
└── import_decision_instructions_json(&str) -> Result<ImportSummary>
    ├── deserializes ExportedDecisionInstructions from JSON
    ├── validates each decision_number exists in state.decisions.decisions
    ├── skips missing decisions with error message in ImportSummary.errors (no hard fail)
    └── adds valid instructions to state.decision_instructions
```

Three new public types:
- `DecisionExportScope` — enum for scoping exports
- `ExportedDecisionInstruction` — per-instruction JSON record
- `ExportedDecisionInstructions` — top-level export container

## Key Decisions

- **Reuse existing metadata types**: `ExportMetadata`, `ExportFieldDescriptions`, `ImportSummary` reused from file-level export. No new structs for shared concepts.
- **field_descriptions sets N/A for file/query/line_range fields**: These are file-level concepts; decision instructions don't have them. Setting "N/A - decision level annotations" makes the format self-documenting for agents.
- **Skip-don't-fail on invalid decision import**: Mirrors `import_instructions_json` behavior. Callers inspect `ImportSummary.errors` for skipped items.

## Feature Complete

All four phases of the Decision Instructions roadmap are now done:

| Phase | Description | Status |
|-------|-------------|--------|
| 1 | DecisionInstructions entity + CRUD | ✅ Done (commit 26fba67) |
| 2 | ReviewState integration + serialization | ✅ Done (commit 26fba67) |
| 3 | ReviewEngine CRUD operations | ✅ Done (commit 2768848) |
| 4 | Export/import JSON | ✅ Done (commit b9a5bae) |

**Total tests**: 169 (16 entity + 4 state + 10 engine + 8 export/import + existing)

## Testing Evidence

**Phase 4 Test Results**:
```
cargo test --package diffviz-review
Result: 169 passed (up from 161 in Phase 3)
```

**New Tests**:
- `test_export_all_decision_instructions` ✅
- `test_export_single_decision` ✅
- `test_export_empty_decision_instructions` ✅
- `test_export_decision_json_structure` ✅
- `test_import_decision_instructions_success` ✅
- `test_import_decision_instructions_invalid_decision` ✅
- `test_export_import_round_trip` ✅
- `test_import_malformed_json_returns_error` ✅

## Files Modified

- `diffviz-review/src/engines/review_engine.rs` — **Modified** (+323 lines: 3 types + 2 methods + 8 tests + 1 test helper)
