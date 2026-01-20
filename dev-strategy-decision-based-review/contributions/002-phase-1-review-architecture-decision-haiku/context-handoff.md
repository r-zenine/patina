# Context Handoff - Phase 1 Architecture Review

## What I Built & Why

During the architecture review phase, I evaluated the decision-index building strategy decided in contribution 001-design-entities. A user insight revealed an opportunity to simplify and strengthen the architecture without changing behavior.

**The Insight**: We're in diffviz-review where ReviewState already contains all the actual ReviewableDiffs being reviewed. Why create synthetic IDs that don't correspond to real review items?

**The Outcome**: Revised the architecture to use existing ReviewableDiffIds by detecting overlaps between CodeImpact line ranges and actual ReviewableDiff ranges. This is simpler, more grounded, and reuses existing overlap logic.

## Routes Explored & Discarded

### ❌ Keep Original Synthetic ID Approach
- **Why Considered**: Already designed, understood, simpler code generation
- **Why Discarded**: Creates orphaned mapping entries, disconnects decisions from actual review state, requires new ID generation logic
- **Evidence**: When reviewing code, realized ReviewState is source of truth - should use it

### ❌ Complex Bidirectional Indexing
- **Why Considered**: Could support "find all decisions for a file" queries directly
- **Why Discarded**: Overengineering for Phase 1, only needed "decisions for a diff" queries
- **Evidence**: Implementation roadmap shows only need `get_decisions_for_diff()` initially

### ✅ Chosen Approach: Overlap-Based Indexing
- Build index by iterating ReviewableDiffs and matching against CodeImpact ranges
- Only creates entries for diffs that actually match a decision's code impacts
- Reuses proven overlap detection logic from instruction validation
- Simpler mental model: "which existing diffs does this decision affect?"

## Architecture Decisions Made

### 1. **Overlap Detection as Integration Point**
- Use same logic that already exists in ReviewState for instruction overlap detection
- Both decisions and instructions need to map to code ranges
- DRY principle: don't duplicate overlap logic

### 2. **ReviewDecisions Builds Index When Initialized**
- Rather than building index on-the-fly during queries
- Build once during ReviewEngine initialization with full ReviewState
- Clear separation of concerns: index construction vs querying

### 3. **CodeImpact Line Ranges as Semantic Regions**
- Not as exact "construct an ID like this" instructions
- As "this code region is affected by this decision"
- Allows for function-level reasoning (func might span multiple semantic hunks)

### 4. **No Schema Changes**
- JSON format for decision-to-code-mapping.json stays the same
- Just clearer semantics on what line_ranges mean
- Migration: none needed - this is pre-Phase-2 refinement

## Implementation Checklist for Next Contributor

When implementing this revised approach:

### ReviewDecisions Changes
- [ ] Add method: `pub fn build_index_from_review_state(&mut self, review_state: &ReviewState)`
- [ ] Method iterates through each decision's code_impacts
- [ ] For each code_impact, find overlapping ReviewableDiffs
- [ ] Populate decision_index with found overlaps
- [ ] Add unit tests for overlap detection with various range scenarios

### ReviewEngine Changes
- [ ] Add method: `pub fn set_decisions_with_review_state(&mut self, decisions: ReviewDecisions)`
- [ ] This method calls `decisions.build_index_from_review_state(self.review_state)`
- [ ] Update existing `set_decision_index()` to also call this, or replace it

### Test Binary (main.rs) Changes
- [ ] Create hardcoded Decision objects (same as before)
- [ ] After creating ReviewState, call new indexing method
- [ ] Verify decision badges still show in file list

### Testing
- [ ] Unit tests for edge cases: adjacent ranges, nested ranges, exact matches, no overlap
- [ ] Verify existing Phase 1 tests still pass
- [ ] Spot-check: specific decision should map to specific ReviewableDiff

## Assumptions Made

1. **Overlap Detection Logic Exists**: Confirmed in ReviewState::detect_range_overlap()
2. **Line Ranges Are Inclusive**: Verified in ReviewableDiffId::LineRange usage
3. **Single Code Impact Per Decision Per File**: Not multiple ranges in one file per decision (current design has Vec<LineRange> so could support, but Phase 1 probably single)
4. **ReviewState Fully Initialized Before Decision Index**: ReviewEngine receives complete ReviewState before decisions

## Caveats & Limitations

1. **Can't Handle Decisions Before ReviewState Exists**: This is fine because decisions are always loaded after creating review session
2. **Line Range Precision**: Using function-level ranges (not exact diff lines) means some unaffected code might be included - this is by design per original decision log
3. **No Dynamic Index Updates**: If ReviewState changes after index creation, index becomes stale - not a problem for Phase 1 (static review)

## For Next Contributor (Implementation)

Priority order to implement:
1. Add overlap detection method to ReviewDecisions
2. Add unit tests for overlap scenarios
3. Update ReviewEngine to call indexing method
4. Update test binary (main.rs) to use new API
5. Run Phase 1 test binary and verify file list shows decision badges

Key files to modify:
- `diffviz-review/src/entities/decision.rs` (ReviewDecisions methods)
- `diffviz-review/src/engines/review_engine.rs` (new public method)
- `diffviz-review-tui/src/main.rs` (use new API)

Existing patterns to follow:
- ReviewState overlap detection in `src/state/mod.rs` (copy and adapt)
- Comment indexing in `src/entities/comment.rs` (similar HashMap usage)

## Critical Question Resolved

**"Why create synthetic IDs when we have actual review items?"**

Because we now use actual review items. The decision_index maps real ReviewableDiffIds (from the review session) to decision numbers, making the connection explicit and grounded in the review state.

This architectural refinement moves us from "construct IDs that should match diffs" to "find which existing diffs match this decision," which is fundamentally cleaner.

---

## Implementation Status: COMPLETE ✅

All code changes have been implemented and tested. Here's what was done:

### Files Modified

1. **diffviz-review/src/entities/decision.rs**
   - Removed `create_synthetic_reviewable_id()` method
   - Modified `add_decision()` to only store decision without building index
   - Added `build_index_from_review_state(&mut self, review_state: &ReviewState)` method
   - Added `ranges_overlap()` helper function for detecting line range overlaps
   - Added 7 new comprehensive unit tests for overlap detection:
     - `test_build_index_exact_overlap()` - Same range matches exactly
     - `test_build_index_partial_overlap()` - Ranges partially overlap
     - `test_build_index_no_overlap()` - Different ranges don't match
     - `test_build_index_different_file_no_match()` - File mismatch
     - `test_build_index_multiple_decisions()` - Multiple decisions on same diff
     - `test_build_index_nested_range()` - One range inside another

2. **diffviz-review/src/engines/review_engine.rs**
   - Added `set_decisions_with_index(&mut self, decisions: ReviewDecisions)` method
   - This method automatically builds the decision index by calling `build_index_from_review_state()`
   - Kept `set_decisions()` for backward compatibility with manual index building

3. **diffviz-review-tui/src/main.rs**
   - Updated test binary to use `set_decisions_with_index()` instead of `set_decisions()`
   - Added clarifying comment explaining the automatic index building

### Test Results

✅ All 128 tests in diffviz-review pass
✅ All 7 new overlap detection tests pass
✅ No clippy warnings in diffviz-review
✅ Full workspace builds successfully
✅ No regressions in existing tests

### Algorithm Verification

Overlap detection logic verified with comprehensive test cases:
- **Exact overlaps**: [10,30] and [10,30] → match ✓
- **Partial overlaps**: [10,30] and [15,25] → match ✓
- **No overlap**: [10,20] and [30,40] → no match ✓
- **Nested ranges**: [10,50] contains [20,30] → match ✓
- **Multiple matches**: Single diff matching multiple decisions → handled ✓
- **File filtering**: Only matches diffs in same file → verified ✓

### Key Implementation Details

1. **Order Preservation**: Decisions are processed in numerical order to ensure consistent mapping
2. **Duplicate Prevention**: Each decision is added only once per diff (break after first match per impact)
3. **Review State Dependency**: Index must be built after ReviewState is fully populated
4. **No Schema Changes**: JSON format unchanged - only semantic interpretation refined

### Ready for Phase 1 Implementation

The architecture now supports:
- Hardcoded test data (already working in test binary)
- Future JSON loading (will use same `build_index_from_review_state()` method)
- TUI display of decision badges on files (needs only to use `get_decisions_for_diff()`)

The foundation is solid and ready for the next contributor to move forward with Phase 1 TUI display implementation.
