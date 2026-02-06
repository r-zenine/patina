# Context Handoff: Phase 2.1 Integration - Decision-Based ReviewEngineBuilder

## Implementation Status

✅ **Phase 2.1 Complete**: Decision-based ReviewEngineBuilder integration is fully functional and tested.

## What Was Built

### New Public Method: `ReviewEngineBuilder::build_from_decisions()`

**Location**: `diffviz-review/src/review_engine_builder.rs` (lines ~97-170)

**Signature**:
```rust
pub fn build_from_decisions(
    self,
    decisions: Vec<Decision>,
    query: DiffQuery,
) -> Result<ReviewEngine, DiffVizError>
```

**Workflow**:
```
1. For each Decision:
   - For each CodeImpact:
     - For each DecisionLineRange:
       a. Fetch old/new source from DiffProvider
       b. Get language parser for file
       c. Wrap source in SourceCode (FullSourceProvider)
       d. Call create_reviewable_diff_from_range() from core
       e. Extract line range using existing helper
       f. Create review-layer ReviewableDiff with unique ID
       g. Add to collection

2. Create ReviewEngine with all ReviewableDiffs
3. Call set_decisions_with_index() to populate decision_index
4. Return initialized engine
```

**Key Features**:
- Handles language detection and parser selection
- Wraps sources appropriately for core API (FullSourceProvider)
- Clear error handling with proper propagation
- Logs warnings for unsupported files (continues processing)
- Unique ReviewableDiffId format: `{file_path}#d{decision_number}:{start_line}-{end_line}`
  - Handles multiple ranges per decision per file without collisions

## How It Integrates with Existing Code

### Core Dependency (Phase 1)
- Uses `create_reviewable_diff_from_range()` from `diffviz-core::decision_based_diff`
- Wraps source with `SourceCode` which implements `FullSourceProvider`
- Converts core `ReviewableDiff` to review-layer `ReviewableDiff`

### DiffProvider Integration
- Calls `get_source_code(file_path, git_ref)` for old and new versions
- Uses `query.from` and `query.to` fields (not `query.old_ref`/`query.new_ref`)
- Handles both successful and optional old source (gracefully handles missing old version)

### Decision Index Integration
- Calls existing `engine.set_decisions_with_index(decisions)` method
- Populates reverse index: ReviewableDiffId → Vec<decision_numbers>
- Enables UI to show "this diff is part of Decision N"

### LanguageParser Integration
- Reuses `get_language_parser_for_file()` helper
- Returns both parser and language enum
- Language enum passed to core `create_reviewable_diff_from_range()`

### LineRange Extraction
- Reuses `extract_line_range_from_core_diff()` helper with proper error handling
- Requires both old and new source providers
- Returns `Option<LineRange>` (must handle None case)

## Testing & Validation

### Unit Tests
- All 148+ diffviz-review tests pass
- All 102+ diffviz-core tests pass
- No new test failures introduced

### Code Quality
- ✅ Zero compiler warnings
- ✅ Zero clippy warnings
- ✅ Code formatting compliant (cargo fmt)
- ✅ Backward compatible (existing `build()` method unchanged)

## For Phase 2.2 Contributors: Decision Loading

The current implementation accepts `Vec<Decision>` as input. The test harness in `diffviz-review-tui/src/main.rs` still uses hardcoded `create_hardcoded_decisions()`.

**Phase 2.2 would involve**:
1. Creating a decision loader abstraction (trait or module)
2. Support for loading decisions from:
   - JSON files
   - Decision log markdown
   - LLM extraction from PR description
3. Updating test harness to use `build_from_decisions()` instead of git-based pipeline
4. Validating that decisions from various sources work correctly

**Current hardcoded approach** (in `diffviz-review-tui/src/main.rs`):
```rust
// Phase 2.1: Still using this for testing
let decisions = create_hardcoded_decisions();
review_engine.set_decisions_with_index(decisions);
```

**Phase 2.2 should change to**:
```rust
// Phase 2.2: Load from file/JSON/LLM
let decisions = decision_loader.load_decisions()?;
let engine = builder.build_from_decisions(decisions, query)?;
// Decision index is automatically built
```

## For Phase 3 Contributors: TUI Validation

The TUI test harness can be updated to use the new pipeline:

**Current flow** (git-based):
```
ReviewEngineBuilder::new() → build(query) → ReviewEngine
                              ↓
                         semantic pairing
                              ↓
                         ReviewableDiffs
```

**New flow** (decision-based):
```
Decisions → ReviewEngineBuilder::new() → build_from_decisions(decisions, query) → ReviewEngine
                                         ↓
                                    decision_based_diff
                                         ↓
                                    ReviewableDiffs
```

**Test validation checklist** (from Phase 3 roadmap):
- [ ] Decision tree navigation works
- [ ] File expansion shows chunks under each file
- [ ] Chunk selection displays actual diff content
- [ ] Approval workflows function correctly
- [ ] Calculator.rs, api.ts, Greeting.tsx, fetcher.py, client.rs, reader.rs all show code
- [ ] Diff rendering (added/deleted/modified lines) looks correct
- [ ] Status bar counts are accurate

## Architecture Insights

### Decision-to-Diff Relationship: Created vs. Indexed

**Phase 1 approach** (now removed):
1. Git finds all changed files
2. Semantic pairing creates ReviewableDiffs
3. Decisions overlap-matched to ReviewableDiffs (post-hoc)
4. Problem: Some diffs might not map to decisions → create Decision 0

**Phase 2 approach** (new):
1. Decisions are input
2. For each decision impact (file + range)
3. Create ReviewableDiff directly from that range
4. Build index as relationships are created
5. Benefit: All diffs belong to decisions by construction

### No More "Unmapped Decisions"

The `create_unmapped_decision()` method is no longer needed in decision-based pipeline because:
- All ReviewableDiffs are created from decision impacts
- By definition, each diff maps to at least one decision
- Decision index is populated at creation time

However, `create_unmapped_decision()` remains in `ReviewDecisions` for potential future use or if git-driven pipeline is re-enabled.

## Code Organization

### Imports Added
```rust
use crate::entities::decision::{Decision, ReviewDecisions};
use diffviz_core::decision_based_diff::create_reviewable_diff_from_range;
```

### New Method Location
- File: `diffviz-review/src/review_engine_builder.rs`
- Position: After existing `build()` method
- Visibility: Public
- Dependencies: Minimal and well-integrated

### No New Files
- Integrated into existing ReviewEngineBuilder
- No new modules or abstractions introduced
- Pure integration work

## Known Limitations & Future Work

1. **No Partial Success Handling**
   - If one decision impact fails, entire operation fails
   - Future: Could implement partial success (skip failed impacts)

2. **Decision Index Always Built**
   - Currently always calls `set_decisions_with_index()`
   - Future: Could make this optional for performance

3. **ID Format Includes Line Range**
   - ReviewableDiffId format: `{file_path}#d{decision_number}:{start_line}-{end_line}`
   - Ensures uniqueness when a decision has multiple ranges for the same file
   - Examples: `src/auth.rs#d1:10-20`, `src/auth.rs#d1:50-60` are distinct

4. **Silent Skipping of Unsupported Files**
   - Logged as warning but doesn't fail
   - Could be stricter if needed

## Verification Commands

```bash
# Full compilation check
cargo check --workspace

# Run all tests
cargo test --workspace

# Check code formatting
cargo fmt --all -- --check

# Check clippy warnings
cargo clippy --workspace

# Run specific test suite
cargo test --package diffviz-review
cargo test --package diffviz-core decision_based
```

All commands should pass with zero warnings.

## Next Phase Context

When Phase 2.2 or 3 work begins, you'll have:
- ✅ Fully functional decision-based diff pipeline
- ✅ Integrated with ReviewEngineBuilder
- ✅ All tests passing
- ✅ No compiler/clippy warnings
- ✅ Clear integration points documented

The pipeline is ready for:
1. Decision source loading (Phase 2.2)
2. TUI end-to-end testing (Phase 3)
3. Old semantic pairing removal (Phase 4)
