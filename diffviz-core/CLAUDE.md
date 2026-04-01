# diffviz-core

## Core Architecture

diffviz-core implements a **clean layered architecture** with strict separation of concerns:

```
diffviz-core/
├── ast_diff/            # Pure Tree-sitter AST diffing with Merkle trees
├── semantic_ast/        # Language-aware semantic analysis
├── reviewable_diff/     # Business logic for review preparation
├── renderable_diff/     # UI-ready diff presentation
├── parsers/            # Tree-sitter language implementations
└── semantic_unit_partitioner/ # Intelligent code unit extraction
```

### Key Capabilities

**Sophisticated Change Detection:**
- Merkle tree hashing for O(log n) structural comparison
- Multiple detection strategies: structural, positional, content
- Relevance scoring (ESSENTIAL, IMPORTANT, BACKGROUND, NOISE)
- Context preservation around changes

**Clean Domain Modeling:**
- `ReviewableDiff` - Business entity for review workflows
- `RenderableDiff` - UI-ready presentation layer
- `SemanticTree` - Language-aware AST representation
- `MerkleASTNode` - Efficient tree comparison

## Tree-sitter Language Support

Comprehensive parsing capabilities across **11 programming languages**:

### Core Languages (Full Semantic Analysis)
- **Rust** (`tree-sitter-rust:0.20`) - Ownership analysis, trait implementations, unsafe code
- **Python** (`tree-sitter-python:0.20`) - Import resolution, type hints, async/await patterns
- **Go** (`tree-sitter-go:0.20`) - Module/package analysis, goroutine safety, error handling
- **TypeScript** (`tree-sitter-typescript:0.20`) - Type system changes, React components, dependency tracking

### Additional Languages
- **Java** (`tree-sitter-java:0.20`) - Object-oriented analysis, inheritance patterns
- **C/C++** (`tree-sitter-c/cpp:0.20`) - Low-level parsing, pointer analysis
- **JavaScript** (`tree-sitter-javascript:0.20`) - ES6+ features, module analysis
- **JSON/CSS/TOML** (`tree-sitter-json/css/toml`) - Configuration and data formats

### Parser Architecture

All parsers use a **descriptor + generic builder** pattern:

- **`LanguageDescriptor` trait** (`src/parsers/descriptor.rs`) — language-specific static
  configuration: kind tables, trivial token lists, container body fields, metadata kind,
  and optional override hooks (`extract_visibility`, `collect_metadata`).
- **`GenericSemanticTreeBuilder<D: LanguageDescriptor>`** (`src/parsers/generic_builder.rs`)
  — shared tree-walking logic that consumes any `LanguageDescriptor`. Enforces the complete
  byte-coverage invariant (every source byte maps to exactly one `SemanticNode`).
- **Language newtype wrappers** (e.g. `RustParser`) — thin structs that hold a
  `GenericSemanticTreeBuilder<XxxDescriptor>` and implement `LanguageParser`. Override only
  language-specific behaviour (e.g. `get_context_boundaries` in `RustParser`).

To add a new language:
1. Create `src/parsers/<lang>.rs` with a `XxxDescriptor` implementing `LanguageDescriptor`.
2. Wrap it: `struct XxxParser(GenericSemanticTreeBuilder<XxxDescriptor>)`.
3. Register the extension in `review_engine_builder.rs`.

## Test Infrastructure

### Comprehensive Fixture Coverage
**100+ structured test fixtures** organized by language and change type:

```
tests/fixtures/{language}/
├── content_changes/     # Identifier renames, literal changes, type modifications
├── structural_changes/  # Function additions, class modifications, import changes
├── reorder_changes/     # Parameter reordering, field rearrangement
├── kind_changes/        # Type transformations (struct→enum, sync→async)
└── complex_combinations/ # Multi-faceted refactoring scenarios
```

### Fixture Format
```json
{
  "name": "rust_identifier_rename",
  "language": "rust",
  "category": "content_changes",
  "old_code": "...",
  "new_code": "...",
  "expected_changes": [...],
  "performance_expectations": {"max_duration_ms": 120}
}
```

### Bug Tracking System
Bug reproduction tests live in `tests/bug_*.rs` and follow TDD principles.
All previously-active bugs have been fixed; see `bugs.md` for the full history.
When filing a new bug: write a failing test first, then fix, then update `bugs.md`.

## Development Guidelines

### Core Principles
- **Tree-sitter Only**: No string/regex operations for code analysis
- **Fail Fast**: No defensive programming or fallbacks
- **Zero Warnings**: All compiler and clippy warnings must be resolved

### Testing Strategy
- **TDD for Bug Fixes**: Always write failing test first
- **Realistic Fixtures**: Prefer `realistic_fixtures` test for real-world scenarios
- **Comprehensive Coverage**: Test structural, content, and positional changes

### Architecture Rules
- **Self-contained Core**: diffviz-core has no dependencies on review/git layers
- **Pure Functions**: AST operations should be deterministic and side-effect free
- **Clean Interfaces**: All public APIs should be well-typed and documented

## Performance Characteristics

**Optimized for Large Codebases:**
- SHA-256 Merkle tree hashing for efficient comparisons
- Structured change detection strategies
- O(log n) tree comparison algorithms

This crate represents the **domain expertise** that makes DiffViz valuable - deep semantic understanding of code changes across multiple programming languages.
