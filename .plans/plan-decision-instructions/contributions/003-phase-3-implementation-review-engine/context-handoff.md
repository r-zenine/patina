# Phase 3 Context Handoff: ReviewEngine Decision Instruction Operations

## What Was Done

**Commit**: `2768848` - "feat: add ReviewEngine decision instruction operations (Phase 3)"

**Deliverables Completed**:
- ✅ Added `InvalidDecision { decision_number: u32 }` and `InstructionNotFound { instruction_id: String }` to `ReviewError` in `diffviz-review/src/errors.rs`
- ✅ `add_decision_instruction(decision_number, content, author) -> Result<()>` implemented with decision-existence guard
- ✅ `remove_decision_instruction(instruction_id) -> Result<()>` implemented, returns Err if ID not found
- ✅ `get_decision_instructions(decision_number) -> Option<Vec<&Instruction>>` implemented
- ✅ 10 new tests covering: success paths, invalid decision, multiple instructions, remove success/not-found, returns-all, missing decision, add-and-remove, state persistence, independence from code instructions
- ✅ All 161 tests pass (up from 151 after Phase 2)
- ✅ Zero clippy warnings in `diffviz-review`
- ✅ `cargo fmt` applied

## Architecture Overview

Three new public methods on `ReviewEngine`:

```
ReviewEngine
├── add_decision_instruction(u32, String, String) -> Result<()>
│   ├── validates decision exists (InvalidDecision error if not)
│   └── creates Instruction with UUID + timestamp, delegates to state.decision_instructions
├── remove_decision_instruction(&str) -> Result<()>
│   └── delegates to DecisionInstructions::remove_instruction_by_id; Err on miss
└── get_decision_instructions(u32) -> Option<Vec<&Instruction>>
    └── read-only view, returns None for unknown decision numbers
```

Two new error variants in `ReviewError`:
- `InvalidDecision { decision_number: u32 }` — decision doesn't exist in ReviewState
- `InstructionNotFound { instruction_id: String }` — instruction ID not present in any decision

## Key Decisions

- **Decision validation is strict**: `add_decision_instruction` checks `state.decisions.decisions.contains_key(&decision_number)` before mutating. No silent no-op; callers get a clear error.
- **No cache invalidation needed**: Decision instructions don't affect `renderable_cache` (which is keyed by `ReviewableDiffId`). Only diff-level operations invalidate the cache.
- **Instruction construction mirrors `add_instruction`**: Same UUID/timestamp pattern, same `InstructionStatus::Active` default. No new dependencies.

## What Phase 4 Should Focus On

### Phase 4: Integration - Export and Reporting

**Objective**: Add JSON export/import for decision instructions.

**What to Add**:
- `ExportedDecisionInstruction` struct (parallel to `ExportedInstruction`)
- `ExportedDecisionInstructions` container struct with `_meta` and `decision_instructions` fields
- `DecisionExportScope` enum: `SingleDecision(u32)` | `All`
- `export_decision_instructions_json(scope: DecisionExportScope) -> Result<String>`
- `import_decision_instructions_json(json: &str) -> Result<ImportSummary>`

**Key constraints for Phase 4**:
- `ExportMetadata`, `ExportFieldDescriptions`, `ImportSummary` structs already exist — reuse them
- `export_decision_instructions_json` must iterate `state.decision_instructions.instructions` (the `HashMap<u32, Vec<Instruction>>`)
- `import_decision_instructions_json` must validate that each `decision_number` in the JSON exists in `state.decisions.decisions` before adding; skip with error message (don't fail hard) for missing decisions — matching `import_instructions_json` behavior
- The exported JSON should include `decision_number: u32` per instruction (not a `ReviewableDiffId`)
- `serde_json` is already a dependency in `diffviz-review/Cargo.toml`

**Files to Read Before Starting Phase 4**:
1. `diffviz-review/src/engines/review_engine.rs` — `export_instructions_json` and `import_instructions_json` methods as reference implementations (~lines 509–671)
2. The `ExportedInstructions`, `ExportMetadata`, `ImportSummary` struct definitions (lines 34–130)

**Expected Test Results**:
- 8+ new export/import tests
- Round-trip test: export → import → verify count and content
- All 161 existing tests still pass
- Zero clippy warnings

## Testing Evidence

**Phase 3 Test Results**:
```
cargo test --package diffviz-review
Result: 161 passed (up from 151 in Phase 2)
```

**New Tests**:
- `test_add_decision_instruction_success` ✅
- `test_add_decision_instruction_invalid_decision` ✅
- `test_add_decision_instruction_multiple_to_same_decision` ✅
- `test_remove_decision_instruction_success` ✅
- `test_remove_decision_instruction_not_found` ✅
- `test_get_decision_instructions_returns_all` ✅
- `test_get_decision_instructions_for_missing_decision` ✅
- `test_add_and_remove_decision_instructions` ✅
- `test_decision_instructions_persist_through_state` ✅
- `test_decision_instructions_independent_from_reviewable_instructions` ✅

## Files Modified

- `diffviz-review/src/errors.rs` — **Modified** (+6 lines: 2 new error variants)
- `diffviz-review/src/engines/review_engine.rs` — **Modified** (+196 lines: 3 methods + 10 tests + helper)

---

**Ready for Phase 4** ✅

ReviewEngine now has full CRUD for decision instructions. Phase 4 can add export/import.
