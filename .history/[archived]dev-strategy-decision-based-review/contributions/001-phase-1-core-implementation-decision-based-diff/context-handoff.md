# Context Handoff: Phase 1 → Phase 2

## What Was Built
A complete, tested core module (`decision_based_diff.rs`) that converts decision-specified line ranges into ReviewableDiffs. This is the foundation of the entire decision-based pipeline.

### Key Public API
```rust
pub fn create_reviewable_diff_from_range(
    file_path: &str,
    start_line: usize,
    end_line: usize,
    old_source: Option<&str>,
    new_source: &str,
    language: ProgrammingLanguage,
    parser: &dyn LanguageParser,
) -> Result<ReviewableDiff, DecisionDiffError>
```

## Integration Points for Phase 2

### In ReviewEngineBuilder
Location: `diffviz-review/src/review_engine_builder.rs`

**Current Flow** (to be replaced):
```
for file in changed_files {
    old/new source → parse → semantic trees →
    build_semantic_pairs() → semantic_pairs_to_reviewable_diffs()
}
Then: Decisions → build_index_from_review_state() → map to existing diffs
```

**New Flow** (Phase 2):
```
for decision in decisions {
    for impact in decision.code_impacts {
        old/new source via DiffProvider →
        create_reviewable_diff_from_range() → ReviewableDiff
        (association is implicit from creation)
    }
}
```

### DiffProvider Integration
The `DiffProvider` trait (diffviz-git crate) provides:
- `get_old_source(file_path)` → Option<String>
- `get_new_source(file_path)` → String

ReviewEngineBuilder will:
1. Fetch sources using DiffProvider
2. Pass them as strings to `create_reviewable_diff_from_range()`
3. Collect ReviewableDiffs directly (no post-hoc mapping needed)

### Key Classes to Modify

**ReviewEngineBuilder**:
- Add method: `create_decision_based_reviewable_diffs(decisions: &[Decision])`
- Remove: `create_semantic_reviewable_diffs()` (in Phase 4)
- Remove: `create_semantic_reviewable_diffs_for_added_file()` (in Phase 4)
- Keep: File discovery infrastructure (needed for other paths initially)

**ReviewState/ReviewDecisions**:
- Build decision index at creation time instead of post-hoc
- Remove: `build_index_from_review_state()` (in Phase 4)
- New responsibility: Associate ReviewableDiffIds with source decisions during creation

## What Worked Well

1. **Lifetime Strategy**: The OwnedNodeData approach cleanly solved the tree lifetime problem
2. **Reusing Patterns**: Building on `reviewable_diff_from_semantic.rs` patterns ensured consistency
3. **Clean Errors**: Structured error types provide clear failure modes
4. **Zero Warnings**: Implementation produces no compiler/clippy warnings

## What to Watch For in Phase 2

### Common Integration Mistakes
1. **Forgetting to fetch sources**: Ensure DiffProvider is called before passing strings
2. **Line range off-by-one**: Test with 1-based line numbers (standard in most editors)
3. **Assuming both sources exist**: Handle `old_source = None` for new files correctly
4. **Language mismatch**: Ensure parser language matches the file being processed

### Testing Approach for Phase 2
1. **Mock DiffProvider**: Create test harness that returns fixture content
2. **Known inputs**: Use existing calculator.rs, api.ts fixtures from tests/
3. **Verify structure**: Check that ReviewableDiffs have correct DiffNode trees
4. **TUI validation**: Use Phase 3 test harness to validate end-to-end

## File Locations to Reference

### Fixtures for Testing
- `diffviz-core/tests/fixtures/` - Real code samples in all languages
- Useful: calculator.rs (Rust), api.ts (TypeScript), Greeting.tsx (React), etc.

### Similar Integration Code
- `diffviz-core/src/reviewable_diff_from_semantic.rs` - Pattern for building ReviewableDiffs
- `diffviz-review/src/entities/decision.rs` - Decision/CodeImpact structure
- `diffviz-git/src/providers/diff_provider.rs` - How to fetch sources

### Architecture References
- `CLAUDE.md` - Project constraints and rules
- `diffviz-core/src/lib.rs` - Public API exports

## Expected Outcomes for Phase 2

When complete, Phase 2 should enable:
- ✅ Decisions drive ReviewableDiff creation (no git discovery first)
- ✅ No post-hoc overlap detection needed
- ✅ Clear 1:N relationship (decision → diffs)
- ✅ Simpler ReviewEngineBuilder logic
- ✅ TUI still shows same diffs, now organized by decision

## Quick Reference: What to Change

```rust
// In ReviewEngineBuilder::build()

// OLD:
let reviewable_diffs = self.create_semantic_reviewable_diffs()?;
decisions.build_index_from_review_state(&review_state);

// NEW:
let reviewable_diffs = self.create_decision_based_reviewable_diffs(&decisions)?;
// Index is built implicitly during creation
```

The new method should:
1. Iterate decisions
2. Iterate code_impacts
3. Fetch old/new sources via DiffProvider
4. Call `create_reviewable_diff_from_range()`
5. Collect into Vec<ReviewableDiff>
6. Pass to review pipeline

## Phase 2 Success Criteria
- [ ] All 43 diffviz-core tests still pass
- [ ] ReviewEngineBuilder compiles without warnings
- [ ] TUI test harness runs without errors
- [ ] Decision tree in TUI shows correct structure
- [ ] No references to unmapped decisions
