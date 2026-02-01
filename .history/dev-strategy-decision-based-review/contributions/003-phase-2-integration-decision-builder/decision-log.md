# Decision Log: Phase 2.1 Integration - Decision-Based ReviewEngineBuilder

## D1: Method Placement and Public API

**Decision**: Add `build_from_decisions()` as a new public method on ReviewEngineBuilder alongside the existing `build()` method.

**Rationale**:
- Preserves backward compatibility (existing code using `build()` continues unchanged)
- Allows both pipelines to coexist during Phase 3 TUI validation
- Clear separation: git-driven pipeline vs. decision-driven pipeline
- Phase 4 can remove the old pipeline when confident in the new approach

**Trade-offs**:
- ReviewEngineBuilder has two distinct responsibilities (git-based and decision-based building)
- Could be separated into two builders, but that's premature abstraction at this stage

**Alternative Considered**:
- Replace `build()` entirely: Too risky during Phase 2, would break existing git pipeline before decision pipeline is fully validated

## D2: Error Handling Strategy

**Decision**: Use existing `DiffVizError` variants (`Git`, `ProcessingFailed`, `InvalidOperation`) rather than creating new ones.

**Rationale**:
- Aligns with codebase conventions
- `ProcessingFailed` accurately describes parse/semantic errors from core
- `Git` for DiffProvider failures is consistent with existing patterns
- `InvalidOperation` for unsupported files/missing parsers

**Consequence**:
- Errors lack semantic specificity (no `DecisionIntegration` variant)
- Future Phase 4 cleanup could add more specific error types if needed
- Currently adequate for Phase 2 requirements

## D3: Source Provider Wrapping

**Decision**: Create new `SourceCode` instances for both old and new sources, wrapping them as `FullSourceProvider` for the decision_based_diff API.

**Rationale**:
- `FullSourceProvider` is required by `create_reviewable_diff_from_range()`
- `SourceCode` already implements both `SourceProvider` and `FullSourceProvider`
- Clean separation: parsing needs full source, ReviewableDiff uses lazy node-based access
- Maintains architectural constraint (no string-based semantic analysis)

**Implementation Details**:
```rust
let new_provider = Box::new(SourceCode::new(new_source_str))
    as Box<dyn FullSourceProvider>;
let old_provider = old_source_str.map(|src|
    Box::new(SourceCode::new(src)) as Box<dyn FullSourceProvider>
);
```

**Note**: The core module handles extraction of full source from providers internally.

## D4: Line Range Extraction and Error Handling

**Decision**: Use existing `extract_line_range_from_core_diff()` with proper error propagation when it returns `None`.

**Rationale**:
- Reuses existing logic (DRY principle)
- Consistent with how the git-driven pipeline extracts line ranges
- Returns `Option<LineRange>` because some diffs may not have extractable boundaries
- `.ok_or_else()` with `ProcessingFailed` error is appropriate fallback

**Trade-off**:
- Silently skipping diffs with no line range extraction could hide issues
- Future: Consider logging or returning more specific errors

## D5: Decision-to-Diff ID Mapping

**REVISED DECISION**: Use format `{file_path}#d{decision_number}:{start_line}-{end_line}` for ReviewableDiffId.

**Rationale**:
- Handles multiple line ranges in a single CodeImpact for the same file
- Uniqueness guaranteed: decision number + file + exact line range
- Distinguishable from git-driven diffs (clear decision-based prefix)
- Decision number visible for debugging
- Line range explicitly encoded for unambiguous reference

**Why the Revision**:
- Original format `{file_path}#d{decision_number}` would collide if a decision's CodeImpact included multiple ranges for the same file (e.g., lines 10-20 AND lines 50-60)
- Example collision: Both ranges would create ID `src/auth.rs#d1`
- Line range inclusion eliminates this ambiguity

**Examples**:
- `src/auth.rs#d1:10-20` - Decision 1's first range
- `src/auth.rs#d1:50-60` - Decision 1's second range in same file
- `src/utils.rs#d2:5-15` - Decision 2's impact on different file

**Implementation**:
```rust
let reviewable_id = ReviewableDiffId::new(
    query.clone(),
    format!(
        "{}#d{}:{}-{}",
        file_path,
        decision.number,
        range.start,
        range.end
    ),
    line_range,
);
```

**Trade-offs**:
- Longer ID strings (minor performance impact)
- More verbose but more explicit and debuggable
- Line ranges are already in `ReviewableDiffId` via `line_range` field, so this is redundant but clarifying

## D6: Loop Structure for Code Impacts

**Decision**: Iterate decisions → code_impacts → line_ranges in that order.

**Rationale**:
- Natural mapping to the `Decision` → `CodeImpact` → `DecisionLineRange` hierarchy
- Each combination produces exactly one ReviewableDiff
- Allows errors on one range to not block others (continue on error could be added)
- Clear responsibility: for each range, create one diff

**Consequence**:
- If a decision has 3 file impacts with 2 ranges each, creates 6 ReviewableDiffs
- Current implementation will fail fast if any range fails
- Future: Could implement partial success (skip failed ranges, continue with others)

## D7: Decision Index Building

**Decision**: Call `engine.set_decisions_with_index()` after creating the engine with ReviewableDiffs.

**Rationale**:
- Existing method already handles the overlap detection logic
- Populates `decision_index` automatically (ReviewableDiffId → decision numbers)
- Eliminates manual overlap checking in this method
- Reuses tested code from Phase 1

**Architecture Benefit**:
- Decision-to-diff relationship established at creation time (D-driven)
- Index building now works post-creation (can enhance with additional logic later)
- No need for `create_unmapped_decision()` in decision-driven pipeline
  (all diffs are created from decisions by definition)

## D8: Skipping Unsupported Files

**Decision**: Log warning and continue when encountering unsupported file types in decision impacts.

**Rationale**:
- Decisions might reference architecture files that don't have parsers (e.g., JSON configs)
- Silently skipping prevents confusing "incomplete review" scenarios
- User can see which decision impacts were skipped
- Non-fatal: other impacts from same decision still processed

**Alternative**:
- Fail fast on unsupported file: Too strict, decision might have mixed impacts
- Skip silently: User wouldn't know why some impacts are missing

## D9: Language Detection

**Decision**: Use existing `get_language_parser_for_file()` which returns both parser and language.

**Rationale**:
- Single source of truth for language detection
- Returns both parser and `ProgrammingLanguage` enum
- Consistent with git-driven pipeline
- Reuses tested logic

**Usage**:
```rust
let (parser, language) = get_language_parser_for_file(file_path)?;
// Use both for `create_reviewable_diff_from_range()`
```

## Summary of Phase 2.1 Design

The new `build_from_decisions()` method:
1. Takes decisions as primary input (not git changes)
2. For each decision → code impact → line range:
   - Fetches old/new source via DiffProvider
   - Creates providers wrapping source code
   - Calls decision_based_diff core API
   - Wraps result in review-layer ReviewableDiff
3. Builds decision index automatically
4. Returns fully initialized ReviewEngine ready for review

This design establishes the decision-driven pipeline while preserving the existing git-driven pipeline for parallel validation in Phase 3.
