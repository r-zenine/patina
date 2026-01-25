# Decision Log - Phase 3: Unit and Integration Testing

## Test Organization Decision

**Decision**: Create single integration test file rather than separate unit and integration test directories.

**Rationale**:
- Context expansion is tested holistically through the parser interface
- Boundary detection, relevance scoring, and tree building are interdependent
- Single file keeps related tests together
- Follows existing test structure in diffviz-core

**Location**: `diffviz-core/tests/context_expansion_tests.rs`

## Test Scope Decision

**Decision**: Focus on parser-level tests rather than lower-level internal functions.

**Rationale**:
- Internal helper functions (find_context_boundary, build_context_tree_recursive, etc.) are private
- Tests can validate behavior through public ReviewableDiff pipeline
- Parser interface is more stable than internal implementation
- Validates integration with LanguageParser trait

## Test Strategy Decision

**Decision**: Use realistic Rust code snippets and parser-based validation.

**Rationale**:
- Real code structures exercise boundary detection more thoroughly
- Parser already tested for basic functionality - focus on context expansion integration
- Avoids mocking complexities while maintaining test clarity

## Coverage Decisions

### Decision 1: Boundary Detection Scope
**Covered**: Content, Structural, Rename, Reorder change types
**Rationale**: Roadmap specifies different change types need testing

### Decision 2: Relevance Scoring Scope
**Covered**: All SemanticNodeKind variants through parser classification
**Rationale**: Ensures all node types get appropriate scores

### Decision 3: Edge Cases
**Covered**:
- Nested structures (impl blocks in modules)
- Root-level changes (no parent)
- Tree depth limits
**Not Covered**:
- Error conditions (invalid syntax, etc.)
**Rationale**: Tests validate correct behavior path, not error handling

## Implementation Decisions

### Decision 1: Helper Functions
Created helper functions for test utilities:
- `count_nodes_at_relevance()` - Count nodes by relevance score
- `max_tree_depth()` - Calculate tree depth for validation

**Rationale**: Reusable utilities that clarify test intent

### Decision 2: Source Provider Mock
Created `SimpleSource` struct implementing SourceProvider.

**Rationale**:
- Needed for line range calculations in tests
- Minimal implementation focused on test needs
- Consistent with other test infrastructure

## Deferred Decisions

### Deferred: Performance Testing
**Rationale**: Focus on correctness first, profile later if needed

**Future Consideration**: Add performance benchmarks if context expansion shows slowdown on large files

### Deferred: Language-Specific Tests
**Rationale**: Start with Rust, TypeScript later

**Future Consideration**: Add Python, Go tests after proving Rust and TypeScript work

### Deferred: Stress Testing
**Rationale**: Current test suite covers normal cases

**Future Consideration**: Test with pathological ASTs if issues discovered in production

## Test Quality Decisions

### Decision 1: Assertion Clarity
All assertions include descriptive messages for test failures.

**Example**:
```rust
assert_eq!(essential_count, 1, "Should have 1 ESSENTIAL node");
```

**Rationale**: Debugging test failures easier with clear messages

### Decision 2: Test Independence
Each test is independent and can run in any order.

**Rationale**: Allows parallel test execution, isolates failures

### Decision 3: No Mocking of LanguageParser
Use real RustParser instead of mocks.

**Rationale**: Validates real integration, simpler than mocks, existing parser is stable
