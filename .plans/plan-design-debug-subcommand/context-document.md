# Context Document for Debug Subcommand

## Behavioral Specification

Build a `diffviz debug` subcommand that exposes all 7 pipeline phases in structured JSON format. This enables transparency into how DiffViz groups and renders code changes, making it easier for agents to understand semantic analysis decisions and debug unexpected diff behavior.

## Codebase Patterns to Follow

- **Command Pattern**: Implement CommandExecutor trait (see DebugExpansion in main.rs for precedent)
- **Dependency Injection**: Use Environment pattern for dependency resolution (fluent builder in environment.rs)
- **Pipeline Reuse**: Leverage ReviewEngineBuilder to avoid duplicating phases 1-5
- **Serialization**: Use serde for domain type serialization (wrapper structs for non-Serialize types)
- **Error Handling**: Propagate structured errors via Result<>, validate inputs (file exists, git refs valid, language supported)
- **Core-then-Integrate Strategy**: Build phases sequentially, each compiling and passing tests before next begins

## Technical Constraints

1. **Zero Compiler Warnings**: All code must pass `cargo clippy --workspace` and `cargo check --workspace`
2. **No Tree-sitter Fallbacks**: diffviz-core forbids regexp/string-based analysis — use tree-sitter or fail fast
3. **ReviewableDiff as Contract**: Use ReviewableDiff from diffviz-core as the authoritative diff representation
4. **DiffProvider Abstraction**: Access git data only through DiffProvider trait, not direct git2 calls
5. **JSON Schema Compliance**: All phase outputs must serialize to valid JSON matching design-doc structure
6. **Line-Range Filtering**: Implement overlap-based filtering (matching TUI code-impact logic): `start <= range_end && end >= range_start`
7. **Backward Compatibility**: No changes to existing commands, core crates, or public APIs

## Codebase Architecture Summary

DiffViz uses clean architecture with modular Rust workspace:

- **diffviz-core** (domain core) - Tree-sitter semantic analysis, ReviewableDiff, RenderableDiff
- **diffviz-review** (orchestration) - ReviewEngineBuilder, ReviewState, review workflows
- **diffviz-git** (infrastructure) - Git operations via DiffProvider trait
- **diffviz-llm** (infrastructure) - LLM integrations
- **diffviz-cli** (entry point) - Commands, Environment, CLI orchestration

The debug subcommand is a CLI integration point that reuses ReviewEngineBuilder (phases 1-5 orchestration) and formats output for transparency.

## Research Findings

No new technologies required. Debug subcommand uses existing:
- **serde/serde_json** - Already in workspace for serialization
- **ReviewEngineBuilder** - Already orchestrates all 7 pipeline phases
- **DiffProvider** - Existing abstraction for git operations
- **Tree-sitter** - Core layer already uses for semantic analysis

Approach: Wrap ReviewEngineBuilder output in JSON-serializable types using serde.
