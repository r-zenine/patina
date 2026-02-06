# Code Changes: D3 Revisited - SourceProvider Integration

## Summary

Refactored `create_reviewable_diff_from_range()` to use `FullSourceProvider` trait instead of raw `&str` parameters. Created new `FullSourceProvider` trait extending `SourceProvider` to enable full source extraction while maintaining AST-first architecture.

## File: diffviz-core/src/ast_diff/source.rs

### Addition: FullSourceProvider Trait

```rust
/// Extended source provider that allows access to full source content
/// Used for operations that require the complete file text (e.g., parsing, semantic tree building)
/// This trait bridges the gap between the AST-first SourceProvider and parsing requirements
pub trait FullSourceProvider: SourceProvider {
    /// Get the complete source code
    fn full_source(&self) -> &str;
}
```

### Addition: FullSourceProvider Implementation for SourceCode

```rust
impl FullSourceProvider for SourceCode {
    fn full_source(&self) -> &str {
        &self.content
    }
}
```

---

## File: diffviz-core/src/decision_based_diff.rs

### Update: Module Imports

**Before:**
```rust
use crate::ast_diff::{
    ASTChangeType, BACKGROUND, ESSENTIAL, IMPORTANT, OwnedNodeData, SourceProvider,
};
```

**After:**
```rust
use crate::ast_diff::{
    ASTChangeType, BACKGROUND, ESSENTIAL, IMPORTANT, FullSourceProvider, OwnedNodeData,
    SourceProvider,
};
```

### Update: Function Signature

**Before:**
```rust
pub fn create_reviewable_diff_from_range(
    _file_path: &str,
    start_line: usize,
    end_line: usize,
    old_source: Option<&str>,
    new_source: &str,
    language: ProgrammingLanguage,
    parser: &dyn LanguageParser,
) -> Result<ReviewableDiff, DecisionDiffError>
```

**After:**
```rust
pub fn create_reviewable_diff_from_range(
    _file_path: &str,
    start_line: usize,
    end_line: usize,
    old_source: Option<&dyn FullSourceProvider>,
    new_source: &dyn FullSourceProvider,
    language: ProgrammingLanguage,
    parser: &dyn LanguageParser,
) -> Result<ReviewableDiff, DecisionDiffError>
```

### Update: Implementation - Extract Source from Provider

**Before:**
```rust
// Parse new file
let new_ast = parser
    .try_parse(new_source)
    .map_err(|e| DecisionDiffError::ParseError(format!("Failed to parse new file: {e}")))?;

let new_tree = parser
    .build_semantic_tree(&new_ast, new_source)
    .map_err(|e| {
        DecisionDiffError::SemanticError(format!("Failed to build new semantic tree: {e}"))
    })?;
```

**After:**
```rust
// Extract full source content from providers
let new_source_str = new_source.full_source();

// Parse new file
let new_ast = parser
    .try_parse(new_source_str)
    .map_err(|e| DecisionDiffError::ParseError(format!("Failed to parse new file: {e}")))?;

let new_tree = parser
    .build_semantic_tree(&new_ast, new_source_str)
    .map_err(|e| {
        DecisionDiffError::SemanticError(format!("Failed to build new semantic tree: {e}"))
    })?;
```

### Update: Implementation - Handle Old Source Provider

**Before:**
```rust
let old_node_data = if let Some(old_source_str) = old_source {
    let old_ast = parser
        .try_parse(old_source_str)
        .map_err(|e| DecisionDiffError::ParseError(format!("Failed to parse old file: {e}")))?;

    let old_tree = parser
        .build_semantic_tree(&old_ast, old_source_str)
        .map_err(|e| {
            DecisionDiffError::SemanticError(format!("Failed to build old semantic tree: {e}"))
        })?;
    // ...
}
```

**After:**
```rust
let old_node_data = if let Some(old_source_provider) = old_source {
    let old_source_str = old_source_provider.full_source();
    let old_ast = parser
        .try_parse(old_source_str)
        .map_err(|e| DecisionDiffError::ParseError(format!("Failed to parse old file: {e}")))?;

    let old_tree = parser
        .build_semantic_tree(&old_ast, old_source_str)
        .map_err(|e| {
            DecisionDiffError::SemanticError(format!("Failed to build old semantic tree: {e}"))
        })?;
    // ...
}
```

### Update: Implementation - Use Provider Clone

**Before:**
```rust
// Create SourceCode providers for the ReviewableDiff
let new_source_provider = crate::ast_diff::SourceCode::new(new_source);
let old_source_provider = old_source
    .map(crate::ast_diff::SourceCode::new)
    .unwrap_or_else(|| crate::ast_diff::SourceCode::new(new_source));

// Build context and construct ReviewableDiff
let context = DiffBuildContext {
    new_unit: Some(new_unit),
    old_node_data,
    classification,
    parser,
    start_time,
};

Ok(build_reviewable_diff_from_unit_with_data(
    context,
    language,
    Box::new(old_source_provider),
    Box::new(new_source_provider),
))
```

**After:**
```rust
// Build context and construct ReviewableDiff
let context = DiffBuildContext {
    new_unit: Some(new_unit),
    old_node_data,
    classification,
    parser,
    start_time,
};

Ok(build_reviewable_diff_from_unit_with_data(
    context,
    language,
    old_source.map(|p| p.clone_box()).unwrap_or_else(|| new_source.clone_box()),
    new_source.clone_box(),
))
```

---

## File: diffviz-core/src/ast_diff/mod.rs

### Update: Public Exports

**Before:**
```rust
pub use source::{LineRange, SourceCode, SourceProvider};
```

**After:**
```rust
pub use source::{FullSourceProvider, LineRange, SourceCode, SourceProvider};
```

---

## Test Results

✅ **Compilation**: All crates compile without warnings
✅ **Tests**: All 100+ fixture tests pass
✅ **Clippy**: Zero warnings in workspace
✅ **Formatting**: Code formatted with cargo fmt

## Breaking Changes

None. This is an internal API change in Phase 1. Callers outside the decision-based pipeline are not affected. Phase 2.1 integration will use this new interface.

## Migration Path for Phase 2.1

When integrating in ReviewEngineBuilder:

```rust
// Get source from DiffProvider
let new_source_str = diff_provider.get_source_code(&file_path, &git_ref)?;

// Create FullSourceProvider (SourceCode implements it)
let new_provider = SourceCode::new(new_source_str);

// Call refactored function
let diff = create_reviewable_diff_from_range(
    &file_path,
    start_line,
    end_line,
    old_provider.as_ref(),
    &new_provider,
    language,
    parser,
)?;
```
