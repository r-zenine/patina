# Phase 6: Mock Provider Integration - Implementation Plan

## Objective

Integrate context expansion into MockDiffProvider so that TUI's mock fixtures produce properly relevance-scored ReviewableDiff instances, enabling visual folding in the TUI.

## Problem Statement

**Current State**: MockDiffProvider loads fixtures but bypasses context expansion
- Fixtures load successfully (rust_trait_impl.json, typescript_react_component.json)
- ReviewableDiff created without calling `expand_changes_to_reviewable_diffs()`
- All lines get ESSENTIAL relevance score
- TUI folding toggle has no effect (nothing to fold)

**Desired State**: MockDiffProvider integrates context expansion
- Fixtures load with rich ContextNode trees
- Relevance scores properly assigned (ESSENTIAL, IMPORTANT, BACKGROUND, NOISE)
- TUI folding hides ~40-50% of lines in Rust fixture
- TUI folding hides ~20-25% of lines in TypeScript fixture

## Root Cause Analysis

### Where Context Expansion Should Happen

**Current Flow** (BROKEN):
```
Fixture JSON
    ↓
MockDiffProvider.from_review_fixtures()
    ↓
ReviewableDiff (created without context expansion) ← PROBLEM HERE
    ↓
TUI RenderableDiff (all lines ESSENTIAL)
    ↓
Folding toggle has no effect (nothing to fold)
```

**Desired Flow** (FIXED):
```
Fixture JSON
    ↓
MockDiffProvider parses old_code/new_code
    ↓
Parse trees → AST diff → ASTChanges
    ↓
expand_changes_to_reviewable_diffs() ← INSERT CONTEXT EXPANSION HERE
    ↓
ReviewableDiff (with rich ContextNode trees)
    ↓
TUI RenderableDiff (varied relevance scores)
    ↓
Folding toggle works! (BACKGROUND/NOISE lines hide)
```

### Files to Modify

1. **diffviz-review/src/providers/mock_provider.rs**
   - Change how it creates ReviewableDiff from fixtures
   - Integrate context expansion pipeline

2. **diffviz-review-tui/src/main.rs** (maybe)
   - May need to pass language parser to mock provider

## Implementation Tasks

### Task 1: Understand Current Mock Provider Flow
**Objective**: Trace how fixtures become ReviewableDiff

**Steps**:
1. Read MockDiffProvider.from_review_fixtures() implementation
2. Find where ReviewableDiff instances are created
3. Check what data is passed to ReviewableDiff construction
4. Identify if ASTDiff is computed at all

**Acceptance**:
- Clear understanding of where context expansion should be inserted

### Task 2: Integrate Diff Computation
**Objective**: Compute AST diffs for fixture old_code/new_code

**Steps**:
1. Get or create language parser for fixture
2. Parse old_code and new_code to AST trees
3. Compute diff using existing diffviz-core algorithms
4. Extract ASTChanges from diff

**Acceptance**:
- ASTChanges computed for each fixture
- Verified with logging/testing

### Task 3: Apply Context Expansion
**Objective**: Call expand_changes_to_reviewable_diffs() on computed changes

**Steps**:
1. Call diffviz-core's expand_changes_to_reviewable_diffs()
2. Pass LanguageParser for fixture's language
3. Collect resulting ReviewableDiff instances
4. Store in mock provider

**Acceptance**:
- ReviewableDiff has rich ContextNode trees
- Relevance scores vary (not all ESSENTIAL)

### Task 4: Test Integration
**Objective**: Verify folding works in TUI

**Steps**:
1. Run TUI: `cargo run --package diffviz-review-tui`
2. Toggle folding: Space+t+c
3. Verify lines actually hide/show
4. Check visual indicator shows count

**Acceptance**:
- Visual output changes when toggling
- Rust fixture: ~40-50% of lines fold
- TypeScript fixture: ~20-25% of lines fold
- Changes never hidden

## Expected Outcome

After Phase 6:
- ✅ Mock provider produces relevance-scored ReviewableDiff
- ✅ TUI folding visually works
- ✅ Enhanced fixtures exercise context expansion
- ✅ Phase 5 validation complete
- ✅ Ready for Phase 7 (Cleanup & Documentation)

## Complexity Assessment

**Complexity**: Medium
- Requires understanding mock provider architecture
- Needs integration with existing diffviz-core APIs
- Testing requires TUI interaction

**Risk**: Low
- No breaking changes to existing APIs
- Mock provider is test-only code
- Core algorithm already proven

## Blockers

None identified. All necessary code exists (Phases 1-4).

## Success Criteria

All criteria must be met:
1. ✅ Code compiles without warnings
2. ✅ TUI launches with modified mock provider
3. ✅ Enhanced fixtures load successfully
4. ✅ Folding toggle (Space+t+c) changes visual output
5. ✅ Appropriate lines fold (BACKGROUND/NOISE hidden, ESSENTIAL visible)
6. ✅ "… N hidden context lines …" indicator appears
7. ✅ Changes never fold
8. ✅ All existing tests still pass
