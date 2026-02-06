# Decision Log: Phase 1 Implementation

## D1: Lifetime Management Strategy
**Decision**: Use OwnedNodeData extraction rather than holding references across tree lifetimes

**Context**: SemanticTree and its nodes hold references tied to tree-sitter's lifetime. When processing both old and new files sequentially, we cannot hold references to old_tree after it's been processed, since the SemanticTree borrows from it.

**Rationale**:
- Extracting OwnedNodeData immediately after finding a unit preserves the essential information before the tree is dropped
- Allows clean separation: parse old → extract data → drop old tree → parse new → use both data and new tree
- Avoids complex lifetime annotations and self-referential struct problems
- Matches Rust's borrowing model naturally

**Alternative Rejected**: Holding both trees alive simultaneously would require restructuring SemanticTree to use owned data instead of references, which is beyond Phase 1 scope

## D2: Semantic Unit Matching Strategy
**Decision**: Simple name-based matching via linear scan instead of complex similarity comparison

**Rationale**:
- We already know which unit to find (same name, same type) from the new file's decision range
- O(n) is acceptable - semantic trees are typically small (<1000 units)
- Matches decision-based philosophy: decisions specify exact units, not approximations
- Reuses existing `SemanticNode::name_node` infrastructure

**Alternative Rejected**: Re-implementing `compare_semantic_units()` logic would recreate the complexity we're moving away from

## D3: API Design - String Input vs SourceProvider
**Decision**: Accept `&str` for source content rather than SourceProvider references

**Rationale**:
- DiffProvider in diffviz-review will fetch sources before calling this function
- Cleaner separation: content retrieval is diffviz-review's responsibility
- SourceCode can be constructed locally as needed
- Matches the pattern of "dumb" core functions that work with plain data

**Implication**: Phase 2 (ReviewEngineBuilder) becomes responsible for calling DiffProvider and passing strings

## D4: Classification Simplification
**Decision**: Classify based on old_node_data existence rather than tracking full old unit

**Rationale**:
- Addresses the lifetime issue cleanly: only store what's needed (OwnedNodeData), not the reference
- Three cases:
  - `old_node_data.is_some()` → Modification
  - `old_source.is_some() && !old_node_data.is_some()` → Addition (unit is new)
  - `old_source.is_none()` → Addition (file is new)
- Semantically correct: if we found the unit in old file, it's a modification

**Alternative Rejected**: Trying to hold old_unit reference leads to lifetime issues; trying to track all deletion cases requires holding old_tree

## D5: Error Types
**Decision**: Use `thiserror` crate for structured error types with context

**Rationale**:
- Provides automatic Display/Debug implementation
- Allows rich error information (e.g., InvalidRange includes start/end line)
- Consistent with project's error handling patterns
- Better error messages for debugging Phase 2 integration

## D6: Helper Function Organization
**Decision**: Private helper functions organized by purpose (lookup, classification, building)

**Rationale**:
- Keeps public API small and focused
- Allows refactoring internals without breaking external code
- Matches the roadmap structure (1.1, 1.2, 1.3, 1.4, 1.5)
- Easy to understand implementation flow

## D7: Context Expansion Strategy
**Decision**: Reuse patterns from `reviewable_diff_from_semantic.rs` exactly

**Rationale**:
- These patterns are battle-tested in the existing codebase
- Ensures consistent relevance assignment across all paths
- MAX_DEPTH=10 limit prevents pathological cases
- Parser-based classification ensures language-appropriate results

**Implication**: Both pipelines (old semantic pairing → new decision-based) will produce identical DiffNode structures for the same code

## D8: Test Strategy for Phase 1
**Decision**: Minimal unit tests for Phase 1, comprehensive tests in Phase 3

**Rationale**:
- Phase 1 is pure domain logic with clear inputs/outputs
- Real validation requires end-to-end testing with actual parsers and semantic trees
- Phase 3 includes TUI test harness which exercises the full pipeline
- Avoid over-testing internal helper functions that may be refactored

**Implication**: Focus testing effort on the public API in Phase 3 validation

## Architectural Consequences

These decisions establish the foundation for:
- **Phase 2**: Clean wiring into ReviewEngineBuilder where decisions drive diff creation
- **Phase 4**: Safe removal of semantic pairing code (no dependencies remain)
- **Phase 5**: Clean removal of orphaned types and methods

The key insight is that by moving from "discover changes then map to decisions" to "decisions specify ranges then build diffs", we:
1. Eliminate the fuzzy overlap-detection logic
2. Make source of truth explicit (decisions)
3. Enable easier testing (known inputs → known outputs)
4. Reduce implementation complexity
