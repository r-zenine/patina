# Changelog: Phase 1 - Entity Structure Updates

## Summary
Completed Phase 1 of the two-track implementation: Removed `ChangeType` and `Confidence` enums, replaced `Decision.summary` with `Decision.rationale`, and updated all test fixtures in diffviz-review crate.

## Changes Made

### Core Entity Updates (diffviz-review/src/entities/decision.rs)
- **Removed:** `ChangeType` enum (Addition | Modification | Deletion) - lines 12-28
- **Removed:** `Confidence` enum (High | Medium | Low) - lines 22-28
- **Updated:** `CodeImpact` struct - removed `change_type` and `confidence` fields
- **Updated:** `Decision` struct - removed `summary: String`, added `rationale: Option<String>` with serde defaults
- **Updated:** `create_unmapped_decision()` method - removed change_type/confidence from CodeImpact construction, updated Decision initialization to use rationale
- **Updated:** Test helper `create_test_decision()` - updated field construction
- **Removed:** Test functions for ChangeType and Confidence serialization (no longer needed)

### Test Fixture Updates (diffviz-review/src/entities/decision.rs)
Updated 15+ test functions to use new schema:
- `test_review_decisions_all_decisions()`
- `test_build_index_exact_overlap()`
- `test_build_index_partial_overlap()`
- `test_build_index_no_overlap()`
- `test_build_index_different_file_no_match()`
- `test_build_index_multiple_decisions()`
- `test_build_index_nested_range()`
- `test_create_unmapped_decision_with_unmapped_diffs()`
- `test_create_unmapped_decision_with_no_unmapped_diffs()`
- `test_create_unmapped_decision_preserves_existing_decisions()`
- All tests now use `rationale: None` or `rationale: Some("...")`
- All tests now omit `change_type` and `confidence` from CodeImpact construction

### Review Engine Test Updates (diffviz-review/src/engines/review_engine.rs)
Updated 2 test helper functions:
- `create_engine_with_decision_and_chunks()` - removed ChangeType/Confidence imports, updated Decision construction
- `test_multiple_decisions_independent()` - removed imports, updated both decision1 and decision2 construction

### Export Updates
- **diffviz-review/src/entities/mod.rs** - Removed ChangeType and Confidence from public re-exports
- **diffviz-review/src/lib.rs** - Removed ChangeType and Confidence from public re-exports

## Verification
- ✅ `cargo check --package diffviz-review` - Compiles successfully
- ✅ `cargo test --package diffviz-review` - All 140 tests pass
- ✅ `cargo clippy --package diffviz-review` - Zero warnings
- ✅ `cargo fmt --package diffviz-review` - Code formatted

## Test Results
```
test result: ok. 140 passed; 0 failed; 0 ignored; 0 measured
```

## Files Modified
1. diffviz-review/src/entities/decision.rs (main entity definitions + tests)
2. diffviz-review/src/entities/mod.rs (re-exports)
3. diffviz-review/src/lib.rs (public API re-exports)
4. diffviz-review/src/engines/review_engine.rs (test helpers)

## Breaking Changes
- Public API: `ChangeType` enum removed
- Public API: `Confidence` enum removed
- Serialization: Old YAML with `change_type` and `confidence` fields won't deserialize
- Serialization: Old YAML with `summary` field should use `rationale` instead

## Next Steps
Phase 2 will update the TUI rendering layer (diffviz-review-tui) to work with the simplified Decision structure:
- Update decision_details_panel.rs to render optional rationale
- Remove change_type and confidence rendering logic
- Update code impact display to show only file, line_ranges, reasoning
