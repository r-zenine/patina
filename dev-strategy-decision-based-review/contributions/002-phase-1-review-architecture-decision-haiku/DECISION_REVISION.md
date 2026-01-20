# Architectural Decision Revision: ReviewableDiffId Mapping Strategy

## Original Decision (001 - Design Entities)
**Synthetic ReviewableDiffId Creation**
- Create ReviewableDiffIds synthetically from CodeImpact data (file_path + line_ranges)
- Build a decision_index mapping these synthetic IDs to decision numbers
- Pros: Simple generation logic, predictable ID structure
- Cons: Divorces mapping from actual review state, may miss precision issues

## The Challenge: User's Insight
When implementing Phase 1, a critical observation emerged:
- We're in diffviz-review crate where ReviewState already contains `reviewable_diffs: BTreeMap<ReviewableDiffId, ReviewableDiff>`
- These are the **actual** diffs being reviewed in the session
- Why create synthetic IDs that don't correspond to what's actually being reviewed?

## Revised Decision
**Use Existing ReviewableDiffIds Instead of Synthetic Ones**

### New Approach
1. Decisions contain `CodeImpact` with (file_path, line_ranges, reasoning)
2. When building decision index, iterate through `ReviewState.reviewable_diffs`
3. For each ReviewableDiff, check if it overlaps with any CodeImpact line ranges
4. If overlap detected, map the **actual ReviewableDiffId** to that decision
5. Result: `decision_index: HashMap<ReviewableDiffId, Vec<u32>>` contains only existing diffs

### Algorithm (Overlap Detection)
```
For each Decision in decisions:
  For each CodeImpact in decision.code_impacts:
    For each ReviewableDiff in review_state.reviewable_diffs:
      If ReviewableDiff.file_path == CodeImpact.file_path:
        If ReviewableDiff.line_range overlaps CodeImpact.line_ranges:
          Map ReviewableDiff.id -> decision.number
```

### Line Range Overlap Logic
Two ranges [A_start, A_end] and [B_start, B_end] overlap if:
```
A_start <= B_end && B_start <= A_end
```

## Impact Analysis

### What Changes
- `decision_index` construction: from synthetic ID generation → overlap matching
- ReviewDecisions methods: same API, cleaner internals
- CodeImpact semantics: line_ranges now interpreted as "code region to match" rather than "exact ID"

### What Stays the Same
- Decision and CodeImpact entity structures (no schema changes)
- ReviewState.decisions integration (stored same way)
- ReviewEngine API surface (same query methods)
- JSON schema for decision-to-code-mapping.json (semantics clarified)
- TUI display logic (works with either approach)

### Why This Is Better
1. **Grounded in Reality**: Decisions map only to code that exists in the review session
2. **Precision**: If a function is split across multiple semantic hunks, we capture all of them
3. **No Orphaned Data**: No synthetic IDs with no corresponding ReviewableDiff
4. **Simpler Semantics**: CodeImpact line_ranges = "which code does this decision affect" (not "construct an ID like this")
5. **Composable**: Easy to add future features (e.g., "which decisions affect this file?")

## Implementation Plan

### Phase 1 Changes Needed
1. **ReviewDecisions refactor** (minimal)
   - Remove synthetic ID generation from constructor
   - Add `build_index_from_review_state()` method
   - Method iterates through diffs and overlaps CodeImpact ranges

2. **ReviewEngine update** (minimal)
   - Add method: `set_decisions_with_review_state(decisions: ReviewDecisions)`
   - Internally calls `decisions.build_index_from_review_state(self.review_state)`

3. **Test binary (main.rs)** update (minimal)
   - After creating hardcoded decisions, call the new indexing method

### No Breaking Changes
- Existing test suite continues to pass
- ReviewableDiff structure unchanged
- ReviewState structure unchanged
- Code already exists for overlap detection (used in instruction validation)

## Risk Assessment

### Low Risk
- Reusing existing overlap logic (already proven in instruction validation)
- Only internal to ReviewDecisions - no public API changes
- Hardcoded test data makes validation easy

### Testing Strategy
- New unit tests in ReviewDecisions for overlap matching
- Existing Phase 1 tests validate the display still works
- Can validate decision index is built correctly by checking specific ReviewableDiffIds

## Decision Rationale

This revision comes from understanding the actual usage context:
- The review session already has all the code being reviewed
- Decisions should reference real review items, not reconstructed IDs
- The simpler mental model: "Which existing diffs does this decision affect?" vs "Create IDs that might match diffs"
- Aligns with the "fail-fast" principle in CLAUDE.md - don't synthesize data when reality exists

This is an **architectural refinement**, not a requirement change. The visible behavior remains the same, but the internals are cleaner and more grounded.
