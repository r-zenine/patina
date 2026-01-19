# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

DiffViz is an LLM-powered code review guide tool that transforms overwhelming code diffs into manageable, step-by-step review experiences. The project uses a clean architecture inspired by the Sam project, with a modular Rust workspace structure.

## Knowledge acquisition Guidelines.
When you need to deep dive into a crate to understand a behaviour, you must read `onboarding.md` first
When the file does not exists ask the onboarding agent to build it for you.
If there are git diffs in the crate, ask the onboarding agent to update the `onboarding.md` document first.

## MANDATORY: Always read onboarding.md before code analysis
Before analyzing any crate's code or proposing changes, or working on fixing a test, or generating any code to a crate:
1. ALWAYS read the crate's `onboarding.md` file first
2. If onboarding.md doesn't exist, use the onboarding agent to create it
3. Only after understanding the architecture from onboarding.md should you read source files
4. If you start code analysis without reading onboarding.md, STOP and read it first
5. Unless specifically asked by me, it is strictly forbiden to introduce fallbacks, or to consider backward compatibility

This prevents surface-level analysis that misses critical architectural constraints and design principles.

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

## Clean Architecture Structure

The project follows clean architecture principles with clear separation of concerns:

```
diffviz/                      # Workspace root
├── diffviz-cli/             # CLI entry point, command orchestration
├── diffviz-core/            # Semantic analysis, ReviewableDiff, RenderableDiff (THE CORE)
├── diffviz-review/          # Review orchestration, workflows, review processes
├── diffviz-git/             # Git operations and diff parsing
├── diffviz-llm/             # LLM client abstractions and implementations
└── diffviz-utils/           # Shared utilities and common functions
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


## Development Rules
- Technical And Functional never change together:  When faced with a change that required both a technical refactoring and a behaviour change you are forbidden to do both at once. You should always change the structure first ( do the refactoring ) with the same behaviour and only then change the behaviour. 


## Key Architectural Patterns

### Entity-Centric Design
- Core semantic models in `diffviz-core/src/` (`ReviewableDiff`, `RenderableDiff`, etc.)
- Review entities in `diffviz-review/src/entities/`
- Business engines in `diffviz-review/src/engines/`
- Core algorithms in `diffviz-review/src/algorithms/`

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

## Development Guidelines

### When Adding New Features
0. Prefer small incremental improvements over big overhauls, prefer TDD when possible
1. Start with core domain modeling in `diffviz-core/` if it's core semantic analysis logic
2. For review orchestration, implement in `diffviz-review/entities` and `diffviz-review/engines`
3. Add infrastructure implementations in appropriate crates
4. Wire dependencies through Environment in CLI layer
5. Always make sure your design does not break architecture rules
6. Under no circumstances are you allowed to introduce a fallback method.

### Testing Architecture Best Practices

**Testing the diffviz-core crate** 
this crate contains an extensive test suite that has to be run fully every time you make a change to the code of diffviz-core
when asked to debug an issue occuring in this create, your first mission is to understand why the test suite did not capture the regression already 
and figure out a way to update it to capture de regression. As a result of this, you should have at least of test failing. then you can focus on a fix. 
When given the choice to add a test, prefer to add it to the realistic_fixtures test if you have realistic data at your disposal

**Structured Test Utilities:**
- Create dedicated `test_utils.rs` modules for reusable test infrastructure
- Implement test-specific methods directly on domain types (e.g., `FileDiff::assert_matches()`)
- Avoid standalone helper functions that duplicate type functionality
- Use structured test data builders (e.g., `TestRepo` for Git operations)

**Test Organization:**
- Unit tests for core semantic analysis logic in diffviz-core crate
- Unit tests for review orchestration logic in diffviz-review crate
- Integration tests for infrastructure layers  
- End-to-end tests through CLI entry points
- Keep test utilities centralized to prevent implementation drift

**Testing Anti-patterns to Avoid:**
- Standalone assertion helpers that mirror type functionality
- Inline test setup code that gets duplicated across test files
- Test-only implementations that diverge from production code

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

## Known Bugs & Testing Strategy

**Bug Organization:**
Bugs are systematically tracked using ignored tests that reproduce the issue, linked to GitHub tickets.

**Bug Test Structure:**
```
crate_name/
├── tests/
│   ├── bugs/              # Active bug reproductions (ignored)
│   │   ├── mod.rs
│   │   ├── issue_123.rs   # One file per bug for complex cases
│   │   └── issue_125.rs
│   ├── regression/         # Fixed bugs (must pass)
│   │   └── mod.rs
│   └── integration/        # Normal tests
```

**Bug Test Format:**
```rust
#[test]
#[ignore = "Bug #123: Semantic pairing fails for nested imports"]
fn bug_123_nested_import_pairing() {
    // Reproduction code that currently fails
    panic!("Expected semantic pairing but got deletion/addition");
}
```

**When Fixing Bugs:**
1. Find the ignored test in `tests/bugs/` for the issue number
2. Remove `#[ignore]` attribute
3. Fix the bug until test passes
4. Move test to `tests/regression/` to prevent future regressions
5. Update the crate's `bugs.md` file to mark as fixed
6. Close the GitHub issue with reference to the fix

**Bug Discovery Commands:**
- `cargo test -- --ignored` - Show all known bugs in a crate
- `cargo test --package <crate-name> -- --ignored` - Show bugs for specific crate
- `rg "#\[ignore.*Bug #" --type rust -g "tests/**/*.rs"` - Find all bug tests

**Each Crate Should Have:**
- `tests/bugs/` directory for active bug reproduction tests
- `tests/regression/` directory for fixed bug tests
- `bugs.md` file listing active and fixed bugs with test locations


**When asked to build a plan** 
- Never highlight timelines, durations, or build any time based breakdown. 
- Focus on the tasks don't bother with time estimates. 
- Make sure the breakdown the tasks into small steps.
- Never consider backward compatibility unless explicitely requested. 

## Mandatory : 
- Never any circumstances produce time estimates in any document you are asked to produce unless explicitely asked. 
- Don't consider backward compatibility, migration strategies, or performance unless specifically asked.
- Don't consider rollback strategy unless specifically asked
