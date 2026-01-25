# Decision Log - Phase 1 Implementation

## Technical Decisions

### Decision 1: NodeRef Wrapper Handling
**Context**: NodeRef is a wrapper around tree_sitter::Node, requiring access to the inner `node` field for TreeSitter operations.

**Decision**: Access `node.node` to call TreeSitter methods (parent(), walk(), children()).

**Rationale**:
- NodeRef design intentionally wraps TreeSitter Node
- This is the established pattern in the codebase
- Maintains encapsulation while allowing necessary operations

**Code Pattern**:
```rust
// Access inner node for TreeSitter operations
let mut cursor = node.node.walk();
for child in node.node.children(&mut cursor) {
    let child_ref = NodeRef::new(child);
    // Process child_ref...
}
```

### Decision 2: Node Equality Comparison
**Context**: Need to determine if two NodeRef instances refer to the same AST node.

**Decision**: Compare by position (start_byte, end_byte) and kind.

**Rationale**:
- TreeSitter nodes don't have stable identifiers
- Position + kind uniquely identifies a node in a tree
- Lightweight comparison without deep traversal

**Implementation**:
```rust
fn nodes_equal<'source>(a: &NodeRef<'source>, b: &NodeRef<'source>) -> bool {
    a.start_byte() == b.start_byte()
        && a.end_byte() == b.end_byte()
        && a.kind() == b.kind()
}
```

### Decision 3: Recursion Depth Limit
**Context**: Recursive tree building could theoretically recurse infinitely.

**Decision**: Set MAX_DEPTH = 10 levels.

**Rationale**:
- Real-world code rarely nests beyond 5-6 levels
- 10 levels provides safety margin
- Prevents stack overflow on malformed or pathological ASTs
- Performance: Stops at reasonable depth even for very deep trees

### Decision 4: Boundary Selection Algorithm
**Context**: Multiple possible approaches for finding context boundaries.

**Decision**: Single upward walk using parser's get_context_boundaries() priority list.

**Rationale**:
- Leverages existing language-specific knowledge in parsers
- O(h) complexity where h = tree height
- First match wins (priority ordering matters)
- Fallback to primary node if no boundary found

### Decision 5: Relevance Assignment Strategy
**Context**: How to assign relevance scores to context nodes.

**Decision**: Two-phase approach:
1. Nodes on change path → ESSENTIAL
2. Other nodes → Use parser's classify_leaf_relevance()

**Rationale**:
- Simple and predictable
- Leverages existing infrastructure
- Ensures changed nodes and their ancestors are always visible
- Language-specific defaults for other nodes

## Implementation Insights

### Insight 1: TreeSitter Cursor Pattern
**Observation**: Must use a cursor for iterating children in TreeSitter.

**Pattern**:
```rust
let mut cursor = node.node.walk();
for child in node.node.children(&mut cursor) {
    // Process child
}
```

**Learning**: Cannot iterate children directly - cursor is required for TreeSitter's internal state management.

### Insight 2: LanguageParser Trait Provides All Needed Hooks
**Observation**: The LanguageParser trait already has all methods needed for context expansion.

**Methods Used**:
- `classify_node_kind()` - Semantic classification
- `get_context_boundaries()` - Boundary selection
- `classify_leaf_relevance()` - Default relevance scores

**Learning**: The existing abstractions were well-designed for this feature - no new trait methods needed.

### Insight 3: Pre-existing Test Failure
**Observation**: C++ enum semantic pairing test was already failing before our changes.

**Verification**: Stashed changes, ran test, confirmed failure, restored changes.

**Action**: Documented as pre-existing issue, not a regression from our implementation.

## Deferred Decisions

### Deferred: Performance Optimization
**Rationale**: Implement correct behavior first. Current O(h) algorithm is efficient enough.

**Future Consideration**: Profile if performance issues arise with very deep ASTs.

### Deferred: Advanced Relevance Heuristics
**Rationale**: Start with simple semantic kind-based classification.

**Future Consideration**: Could add distance-based scoring or domain-specific rules if needed.

### Deferred: Boundary Merging
**Rationale**: Existing merge logic in ReviewableDiff construction should handle this.

**Future Consideration**: Verify in Phase 2 integration testing.
