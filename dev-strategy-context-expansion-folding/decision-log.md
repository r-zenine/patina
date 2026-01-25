# Decision Log

## Implementation Strategy

**Decision**: Use **Core-then-Integrate** approach
**Rationale**:
- Context expansion is a complex algorithm requiring focused development
- Clear interface boundaries (LanguageParser trait provides all hooks)
- Can test algorithm in isolation before integration
- Aligns with clean architecture principles (pure function for core logic)

**Alternatives Considered**:
- TDD: Would work but harder to test without real TreeSitter nodes
- Steel Thread: Phases would be too granular for this focused task

---

## Fixture Enhancement Strategy

**Decision**: **Enhance existing fixtures** rather than create new ones
**Rationale**:
- User explicitly selected this option
- Maintains consistency with existing test infrastructure
- Easier to verify behavior changes in familiar fixtures
- Reduces total fixture count

**Approach**:
- Expand 2-3 existing fixtures to 50+ lines
- Add realistic imports, comments, docstrings
- Focus on: `rust_trait_impl.json`, `typescript_react_component.json`

---

## Implementation Scope

**Decision**: **Context Expansion + TUI Validation** for first deliverable
**Rationale**:
- User explicitly selected this scope
- Ensures end-to-end functionality verification
- Balances focused implementation with practical validation
- Folding infrastructure already works - just needs proper input

**Excluded from Scope**:
- Creating brand new fixture infrastructure
- Performance optimization (unless critical)
- Edge case handling beyond core languages

---

## Technical Decisions

### TreeSitter Navigation Approach

**Decision**: Single upward walk to find boundary, then recursive downward build
**Rationale**:
- O(h) complexity where h = tree height (typically small)
- Avoids redundant traversals
- Natural fit with TreeSitter API (parent() navigation)

### Relevance Classification Strategy

**Decision**: Two-phase approach
1. Default relevance based on node semantic kind
2. Override to ESSENTIAL for nodes on change path

**Rationale**:
- Leverages existing `classify_leaf_relevance()` infrastructure
- Simple and predictable
- Aligns with existing relevance scoring philosophy

### Context Merging Strategy

**Decision**: Keep existing boundary merging logic unchanged
**Rationale**:
- Current implementation in `expand_changes_to_reviewable_diffs()` already handles multiple changes
- Don't break working functionality
- Focus only on enriching ContextNode trees

---

## Architecture Decisions

### Location of Context Expansion Logic

**Decision**: Implement as private helper function in `reviewable_diff.rs`
**Rationale**:
- Single responsibility (only used by expand_changes_to_reviewable_diffs)
- No need for public API yet
- Can extract to separate module later if needed

**Function signature**:
```rust
fn build_context_tree_from_change<'source>(
    change: &ASTChange<'source>,
    parser: &dyn LanguageParser,
) -> ChangeWithContext<'source>
```

### Testing Strategy

**Decision**: Unit tests + integration tests + TUI validation
**Rationale**:
- Unit tests: Verify algorithm correctness in isolation
- Integration tests: Verify pipeline (ASTChange → ReviewableDiff → RenderableDiff)
- TUI validation: Confirm visual folding works end-to-end

**Test files**:
- New: `diffviz-core/tests/context_expansion_tests.rs`
- Modify: Enhanced fixtures in `diffviz-review/tests/fixtures/`
- Visual: `diffviz-review-tui` manual testing

---

## Deferred Decisions

### Performance Optimization
**Status**: Deferred
**Rationale**: Implement correct behavior first, optimize only if profiling shows issues

### Advanced Relevance Heuristics
**Status**: Deferred
**Rationale**: Start with simple semantic kind-based classification, add sophistication if needed

### Multi-Language Testing
**Status**: Partial - focus on Rust and TypeScript first
**Rationale**: Prove algorithm works for 2 languages, extend to others later
