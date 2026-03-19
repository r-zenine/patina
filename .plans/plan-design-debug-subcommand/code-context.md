# Code Context for Debug Subcommand

## Relevant Classes and Functions

### Review Engine & Builder
- **ReviewEngineBuilder** (`diffviz-review/src/review_engine_builder.rs:23-42`) - Orchestrates pipeline phases 1-5: git query → semantic analysis → review engine creation
- **ReviewEngine** (`diffviz-review/src/review_engine.rs`) - Container for ReviewState with mutable operations
- **ReviewState** (`diffviz-review/src/state/mod.rs:33-48`) - Centralized session state storing reviewable_diffs, approvals, decisions, journey

### Core Domain Types
- **ReviewableDiff** (`diffviz-core/src/reviewable_diff.rs:18-24`) - Self-contained reviewable change unit with boundary, source code, metadata
- **DiffNode** (`diffviz-core/src/reviewable_diff.rs:27-34`) - Hierarchical AST node with change_status, relevance score, semantic_kind
- **RenderableDiff** (`diffviz-core/src/renderable_diff/mod.rs:24-29`) - Line-oriented representation optimized for TUI display with annotations

### Semantic Analysis Types
- **SemanticTree** (`diffviz-core/src/semantic_ast.rs:27-77`) - Higher-level abstraction grouping TreeSitter tokens into semantic constructs
- **SemanticNode** (`diffviz-core/src/semantic_ast.rs:40-50`) - Individual semantic unit (function, class, module) with metadata and children
- **SemanticUnitType** (`diffviz-core/src/semantic_ast.rs:52-77`) - Universal category enum (DataStructure, Function, Module, etc.) working across languages

### Infrastructure & Dependency Injection
- **CommandExecutor trait** (`diffviz-cli/src/commands/mod.rs:14-17`) - Common contract for CLI subcommands: `execute(&self, environment: Environment) -> Result<()>`
- **DiffProvider trait** (`diffviz-review/src/providers/mod.rs:68-108`) - Abstraction for git operations: get_changed_files, get_source_code, get_content_snapshot, get_file_hash
- **Environment** (`diffviz-cli/src/environment.rs:40-68`) - Dependency injection container with fluent builder pattern
- **Commands enum** (`diffviz-cli/src/main.rs:60-100`) - Clap subcommands registration point

## Key Files to Reference

### Core Layer (diffviz-core)
- `diffviz-core/src/reviewable_diff.rs` - ReviewableDiff and DiffNode definitions
- `diffviz-core/src/renderable_diff/mod.rs` - RenderableDiff and RenderableLine for display
- `diffviz-core/src/semantic_ast.rs` - SemanticTree and SemanticNode hierarchies
- `diffviz-core/src/ast.rs` - ProgrammingLanguage enum and language classification

### Review Layer (diffviz-review)
- `diffviz-review/src/review_engine_builder.rs` - ReviewEngineBuilder orchestrating phases
- `diffviz-review/src/state/mod.rs` - ReviewState storage and lifecycle
- `diffviz-review/src/providers/mod.rs` - DiffProvider trait definition

### CLI Layer (diffviz-cli)
- `diffviz-cli/src/main.rs` - Commands enum and entry point
- `diffviz-cli/src/commands/mod.rs` - CommandExecutor trait and command registration
- `diffviz-cli/src/environment.rs` - Environment builder and dependency injection

## Serialization & JSON
- `serde` / `serde_json` - Used throughout for struct serialization
- Target: Serialize ReviewableDiff, DiffNode, RenderableDiff for JSON output
- Custom serializers needed for types without built-in Serialize derive

## Testing Patterns
- **Unit Test Location**: `diffviz-core/tests/` - Semantic analysis, DiffNode hierarchies
- **Integration Test Location**: `diffviz-review/tests/` - ReviewEngineBuilder output
- **CLI Test Location**: `diffviz-cli/tests/` - End-to-end command invocation
- **Test Utilities**:
  - `diffviz-core/src/test_utils.rs` - Test fixtures and helpers
  - `diffviz-review/src/test_utils.rs` - Mock providers and test data

## Configuration & Environment
- **Environment Variables**: None currently required (Config loaded from ~/.config/diffviz/config.toml)
- **Dependencies**:
  - serde/serde_json for serialization
  - tree-sitter for AST analysis
  - git2-rs for git operations (via DiffProvider)

## Related Existing Commands
- **DebugExpansion** (`diffviz-cli/src/main.rs:91-99`) - Similar debug-style command that runs review pipeline
- **Review** (`diffviz-cli/src/main.rs:70-78`) - Main review pipeline command with file filtering
- **Show** (`diffviz-cli/src/main.rs:60-68`) - Simple file diff display command
