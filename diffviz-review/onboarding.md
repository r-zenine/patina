# diffviz-review

Review orchestration layer providing ReviewableDiff-based code review workflows with semantic analysis integration and state management.

## Architecture Role

The diffviz-review crate sits in the middle layer of the DiffViz clean architecture, orchestrating review processes between the core semantic analysis capabilities and infrastructure layers. It depends on diffviz-core for semantic understanding while providing high-level review abstractions for CLI and TUI layers.

**Architectural Position:**
- **Depends on:** diffviz-core (semantic analysis, ReviewableDiff, RenderableDiff)
- **Depended by:** diffviz-cli (command orchestration), diffviz-git (file operations), diffviz-llm (AI integration)
- **Layer:** Business logic/Application services layer

## Core Capabilities

• **Review State Management** - Centralized state tracking for multi-file reviews with approval, comment, and suggestion workflows
• **ReviewableDiff Orchestration** - Business logic layer for managing semantic diff units from diffviz-core
• **Review Engine with Caching** - High-performance review processing with RenderableDiff caching for TUI responsiveness
• **Git Integration Abstraction** - DiffProvider trait defining clean interfaces for git operations without git dependencies
• **Multi-language Semantic Processing** - Pipeline for converting git diffs to semantic ReviewableDiffs across 8 programming languages
• **Builder Pattern Architecture** - ReviewEngineBuilder orchestrating complete git-to-review pipeline with dependency injection

## Key Abstractions

**ReviewableDiffId** - Universal identifier triplet (DiffQuery + file_path + LineRange) providing unique addressing for semantic diff units across different git queries with deterministic ordering and display formatting.

**ReviewState** - Centralized business state container managing ReviewableDiffs in a BTreeMap with associated review metadata (approvals, comments, instructions, suggestions) using entity-centric collections indexed by ReviewableDiffId.

**ReviewEngine** - Core business engine providing review operations (approve, comment, navigate) with RenderableDiff caching, bulk operations, and progress tracking. Bridges semantic analysis from diffviz-core with review workflows.

**DiffProvider** - Infrastructure abstraction trait defining required git capabilities (get_changed_files, get_file_stats, get_source_code) enabling dependency inversion and clean testing without git dependencies.

**ReviewEngineBuilder** - Factory orchestrating the complete semantic analysis pipeline from git queries to populated ReviewEngine instances, handling language detection, parsing, semantic tree building, and ReviewableDiff generation.

## Development Rules

**Entity-Centric Design Requirement** - All review data MUST be organized around ReviewableDiffId as the primary key. Collections use ReviewableDiffId for indexing, never legacy chunk IDs or ad-hoc identifiers.

**State Immutability Pattern** - ReviewState update methods return `&mut Self` for chaining while maintaining internal consistency. External state mutations only through controlled methods, never direct field access.

**Dependency Inversion Enforcement** - Review layer defines interfaces (DiffProvider trait) implemented by infrastructure layers. No direct git dependencies, filesystem operations, or external service calls in this crate.

**Semantic Analysis Integration** - All diff processing MUST use diffviz-core semantic analysis pipeline. String-based diff parsing forbidden. Only TreeSitter and semantic trees for code analysis.

**Fail-Fast Error Handling** - Use structured thiserror enums with context preservation. No defensive programming or fallbacks. Operations either succeed with semantic understanding or fail with clear error chains.

## Code Organization

```
diffviz-review/
├── src/
│   ├── lib.rs                    # Public API surface and re-exports
│   ├── entities/                 # Review domain entities
│   │   ├── mod.rs               # Entity module organization
│   │   ├── reviewable_diff_id.rs # Universal diff addressing
│   │   ├── git_ref.rs           # Git state modeling
│   │   ├── comment.rs           # Comment workflow entities
│   │   ├── suggestion.rs        # Code suggestion entities
│   │   ├── approval.rs          # Approval workflow entities
│   │   └── instruction.rs       # Review instruction entities
│   ├── engines/                  # Business logic engines
│   │   ├── mod.rs               # Engine module re-exports
│   │   └── review_engine.rs     # Core review orchestration
│   ├── state/                    # State management
│   │   └── mod.rs               # ReviewState and ReviewableDiff wrapper
│   ├── providers/                # Infrastructure abstractions
│   │   ├── mod.rs               # Provider interfaces
│   │   └── mock_provider.rs     # Test implementations
│   ├── review_engine_builder.rs  # Git-to-ReviewEngine pipeline
│   └── errors.rs                # Structured error handling
└── tests/
    └── fixtures/                # Test data for semantic analysis
        ├── rust_async_conversion.json
        ├── typescript_interface_property.json
        └── python_class_inheritance.json
```

## Testing Strategy

**Fixture-Based Testing** - Uses JSON test fixtures containing realistic code changes with expected semantic analysis results. Tests validate complete pipeline from code strings to ReviewableDiff generation.

**Mock Provider Pattern** - MockDiffProvider implementation enables testing review workflows without git dependencies. Provides controlled test scenarios for different file states and content variations.

**Entity Integration Tests** - Each review entity (Comment, Suggestion, Approval) has comprehensive tests validating collection operations, state transitions, and ReviewableDiffId-based indexing.

**Builder Pipeline Testing** - ReviewEngineBuilder tests cover language detection, file filtering, semantic analysis integration, and error handling across supported programming languages.

**Cache Behavior Validation** - ReviewEngine tests verify RenderableDiff caching performance, invalidation triggers, and memory management under various review operations.

## Integration Patterns

**ReviewEngine Creation Workflow:**
1. CLI layer creates DiffProvider implementation (git operations)
2. ReviewEngineBuilder.new(provider, author) creates builder with dependencies
3. builder.build(DiffQuery) executes semantic pipeline:
   - Get changed files via DiffProvider
   - Filter supported languages (Rust, Python, Go, TypeScript, etc.)
   - Parse and build semantic trees using diffviz-core
   - Generate ReviewableDiffs from semantic pairs
   - Create ReviewEngine with cached RenderableDiffs

**Review State Updates:**
```rust
// Entity-centric review operations
engine.approve(reviewable_id, reviewer, callback);
engine.add_comment(reviewable_id, content, author, callback);
engine.add_instruction(reviewable_id, content, author, callback);

// Bulk operations for efficiency
engine.approve_all_in_file(file_path, reviewer, callback);

// Progress tracking and completion
let progress = engine.get_review_progress();
let summary = engine.complete_review()?;
```

**Caching Integration:**
- RenderableDiff objects cached by ReviewableDiffId for TUI performance
- Cache invalidation on review state changes (approve, comment, etc.)
- Read-only rendering methods for concurrent access patterns
- Memory management via cache clearing and statistics tracking

## Development Tools

**Test Fixtures** - JSON-based test data located in `tests/fixtures/` containing realistic code changes for validating semantic analysis pipeline with expected line statistics and metadata.

**Mock Infrastructure** - `providers/mock_provider.rs` provides DiffProvider implementation for testing review workflows without git dependencies, supporting various file states and content scenarios.

**Error Context Tools** - Structured error types with preserved error chains and contextual information for debugging review pipeline failures and git operation issues.