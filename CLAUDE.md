# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

DiffViz is an LLM-powered code review guide tool that transforms overwhelming code diffs into manageable, step-by-step review experiences. The project uses a clean architecture inspired by the Sam project, with a modular Rust workspace structure.

## Development Commands

### Workspace Operations
- `cargo build --workspace` - Build all workspace crates
- `cargo test --workspace` - Run tests across all crates  
- `cargo check --workspace` - Check compilation for all crates
- `cargo run` - Run the main CLI application (from diffviz-cli)

### Individual Crate Development
- `cargo build --package <crate-name>` - Build specific crate
- `cargo test --package <crate-name>` - Test specific crate
- `cargo check --package <crate-name>` - Check specific crate

### Code Quality
- `cargo fmt --all` - Format all workspace code
- `cargo clippy --workspace` - Lint all workspace crates
- `cargo clippy --package <crate-name>` - Lint specific crate

### Bounded-Context Dependency Policy
- `cargo run -p depcheck -- check-deps` - Fails if any crate under `apps/<context>/` depends (directly or transitively) on a crate in a *different* `apps/<context>/`, or if any `libs/*` crate depends on an `apps/*` crate. Run this after adding any new internal dependency. Backed by [cargo-guppy](https://github.com/facebookincubator/cargo-guppy); policy logic lives in `maintenance/depcheck/src/main.rs`.

### Pre-commit Hook
- One-time setup per clone: `cargo test -p depcheck` (installs the git hook via `cargo-husky`, a dev-dependency of `maintenance/depcheck`).
- The hook (`.cargo-husky/hooks/pre-commit`, copied to `.git/hooks/pre-commit`) runs on every `git commit`: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, then `cargo run -p depcheck -- check-deps`. Edit `.cargo-husky/hooks/pre-commit` to change it — the copy in `.git/hooks/` is regenerated automatically next time `depcheck`'s dev-dependencies are built.

## Clean Architecture Structure

The project follows clean architecture principles with clear separation of concerns:

```
apps/diffviz/                 # diffviz bounded context
├── cli/                     # CLI entry point, command orchestration
├── core/                    # Semantic analysis, ReviewableDiff, RenderableDiff (THE CORE)
├── review/                  # Review orchestration, workflows, review processes
└── review-tui/              # TUI interface
libs/                         # Generic subdomains shared across bounded contexts
├── gitkit/                  # Git operations and diff parsing
├── fsutils/                 # Filesystem helpers
└── ...
```

## Dev rules 
ZERO WARNING RULE: general rules, you are not allow to leave compiler or clippy warning not fixed, after every change, you need to run cargo fmt -- / clippy and cargo check and fix all the warnings 

crate: diffviz-core 
- string based or regexp operations to analyze code are forbidden in this module. Only Tree-sitter is allowed, if it can be done with tree-sitter you need to explicitely ask for permission and provide evidence to support your claim
- FALLBACKS are forbidden in this crate, you need to adopt a fail fast approach, no defensive programming here:





### Dependency Rules

**Core Layer (diffviz-core - THE DOMAIN CORE):**
- Contains the core semantic analysis logic and domain expertise
- Houses key abstractions: `ReviewableDiff`, `RenderableDiff`, AST analysis
- Implements the essential business capabilities that make DiffViz valuable
- Self-contained with TreeSitter, parsing, and semantic intelligence
- This IS what makes DiffViz special - the semantic understanding of code

**Review Layer (diffviz-review):**
- Orchestrates the review process and workflows
- Contains review-specific business logic and coordination
- Manages entities for the review experience pipeline
- Depends on diffviz-core for core semantic capabilities

**FS layer (diffviz-git)**:
- Provides capabilities to identify diffs to review 
- Provides capabilities to retrieve the content of a line range of a given file 
- Depends on both diffviz-review and diffviz-core

**Infrastructure Layers:**
- All other crates depend on diffviz-review (which depends on diffviz-core)
- Infrastructure crates should not depend on each other directly
- Communication happens through review abstractions

**CLI Layer (diffviz-cli):**
- Entry point that composes all dependencies
- Uses Environment pattern for dependency injection
- Orchestrates business operations through engines



## Key Architectural Patterns

### Entity-Centric Design
- Core semantic models in `apps/diffviz/core/src/` (`ReviewableDiff`, `RenderableDiff`, etc.)
- Review entities in `apps/diffviz/review/src/entities/`
- Business engines in `apps/diffviz/review/src/engines/`
- Core algorithms in `apps/diffviz/review/src/algorithms/`

### Environment Pattern
- Dependency injection container in CLI layer
- Assembles all dependencies and provides them to engines
- Enables easy testing and swapping of implementations

### Engine Pattern
- Business operations orchestrated through engines
- Engines accept dependencies through constructor injection
- Clear separation of concerns within business logic

## Language Support

The tool provides deep support for 4 languages with language-specific analysis:
- **Go**: Module/package analysis, goroutine safety, error handling patterns
- **Python**: Import resolution, type hint validation, async/await patterns  
- **TypeScript**: Type system changes, React component analysis, dependency tracking
- **Rust**: Ownership analysis, trait implementations, unsafe code detection

## LLM Integration

Hybrid approach supporting both local and remote models:
- **Local Models**: Ollama integration for privacy and zero API costs
- **API Models**: OpenAI/Anthropic for advanced analysis requiring larger context

## Configuration Strategy

Minimal configuration for MVP with extensibility architecture:
- Global settings: `~/.config/diffviz/config.toml`
- Project overrides: `.diffviz/config.toml` in project root
- Essential settings only (LLM providers, basic preferences)

### Error Handling Best Practices

**Structured Error Design:**
- Use `thiserror` with structured error variants, not just string wrappers
- Preserve source error information with `#[source]` and `#[from]` attributes
- Include contextual information in error variants (paths, identifiers, etc.)

**Example of Well-Structured Errors:**
```rust
#[derive(Debug, Error)]
pub enum GitError {
    #[error("Git operation failed")]
    Git(#[from] git2::Error),

    #[error("Repository not found at path: {path}")]
    RepositoryNotFound {
        path: String,
        #[source]
        source: git2::Error,
    },

    #[error("Invalid commit hash: {hash}")]
    InvalidCommit {
        hash: String,
        #[source]
        source: git2::Error,
    },
}
```

**Error Propagation:**
- Leverage automatic `#[from]` conversions to reduce boilerplate
- Propagate errors through Result types with proper context preservation
- Handle user-facing errors gracefully in CLI layer with meaningful messages

**Error Handling Anti-patterns to Avoid:**
- Converting rich error types to simple strings (loses valuable debug information)
- Creating error variants without preserving the source error chain
- Using generic error messages without specific context

