# Context Handoff: D3 Revisited - SourceProvider Integration

## Rationale for Revision

The original Decision D3 proposed using raw `&str` for source code parameters in `create_reviewable_diff_from_range()`. This approach was reconsidered because:

1. **Existing Pattern Violation**: The codebase already uses `SourceProvider` as the abstraction for file content access
2. **Testability**: Using `SourceProvider` enables cleaner mock implementations for unit tests
3. **Future Extensibility**: A trait-based approach allows for caching, lazy loading, or other enhancements without changing caller code
4. **Consistency**: Keeps the decision-based pipeline aligned with other parts of the system

## Implementation Details

### New Abstraction: `FullSourceProvider`

The solution introduces `FullSourceProvider` trait (extends `SourceProvider`) to bridge a critical gap:

- **SourceProvider** is intentionally minimal (node-based access only) - enforces AST-first design
- **FullSourceProvider** adds `full_source() -> &str` method for parsing/tree-building needs
- **SourceCode** implements both traits

This design maintains the architectural constraint (no string-based analysis) while enabling operations that require the complete file text.

### Function Signature Change

```rust
// Before
fn create_reviewable_diff_from_range(
    old_source: Option<&str>,
    new_source: &str,
    ...
) -> Result<ReviewableDiff, DecisionDiffError>

// After
fn create_reviewable_diff_from_range(
    old_source: Option<&dyn FullSourceProvider>,
    new_source: &dyn FullSourceProvider,
    ...
) -> Result<ReviewableDiff, DecisionDiffError>
```

### Internal Implementation Pattern

1. Extract full source: `let source_str = provider.full_source();`
2. Parse and build semantic tree using the string
3. Store SourceProvider in ReviewableDiff via `provider.clone_box()`

This pattern cleanly separates concerns:
- Parsing needs raw strings
- ReviewableDiff uses lazy-loading SourceProvider abstraction
- No string-based analysis in semantic code

## Key Files Changed

1. **diffviz-core/src/ast_diff/source.rs**
   - New `FullSourceProvider` trait
   - Implementation for `SourceCode`

2. **diffviz-core/src/decision_based_diff.rs**
   - Function signature updated to use `&dyn FullSourceProvider`
   - Implementation uses `full_source()` to extract content for parsing
   - Returns provider instances for ReviewableDiff

3. **diffviz-core/src/ast_diff/mod.rs**
   - Exported new `FullSourceProvider` trait

## For Next Contributors (Phase 2.1 Integration)

When wiring into ReviewEngineBuilder:

1. Get source string from DiffProvider: `let source = diff_provider.get_source_code(...)?`
2. Create SourceCode: `let provider = SourceCode::new(source)`
3. Call function: `create_reviewable_diff_from_range(..., &provider, ...)`

Example:
```rust
let new_source = diff_provider.get_source_code(&file_path, &git_ref)?;
let new_provider = SourceCode::new(new_source);
let diff = create_reviewable_diff_from_range(..., &new_provider, ...)?;
```

## Testing & Validation

- ✅ All 100+ fixture tests pass
- ✅ Zero compiler warnings
- ✅ Zero clippy warnings
- ✅ No breaking changes to public APIs outside this module

## Backward Compatibility

This is an internal API change in Phase 1. The decision-based pipeline is not yet integrated into production, so no migration concerns exist.

## Architecture Preservation

This refactoring carefully preserves the AST-first design principle:
- `SourceProvider` remains minimal (no string operations)
- String-based operations limited to parsing setup
- `FullSourceProvider` is an extension, not a replacement
- All semantic analysis continues using SourceProvider's node-based interface
