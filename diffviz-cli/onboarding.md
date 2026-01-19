# diffviz-cli

CLI entry point that orchestrates command execution and uses the Environment pattern for dependency injection as the composition root.

## Architecture Role

This crate serves as the **composition root** in the clean architecture hierarchy, sitting at the outermost layer. It assembles all workspace dependencies and provides them to business engines through the Environment pattern. The CLI layer depends on all other crates but has no dependents, making it the single place where the entire application is wired together.

Dependencies:
- diffviz-review (review orchestration and business logic)
- diffviz-review-tui (terminal UI implementation)
- diffviz-git (Git operations and diff parsing)
- diffviz-core (semantic analysis and core domain)
- diffviz-llm (LLM client abstractions)
- External: clap, anyhow, tokio, ratatui, crossterm

## Core Capabilities

- **Binary Entry Point**: Provides `diffviz` executable via main.rs
- **Command Line Parsing**: Uses Clap derive API for type-safe argument handling
- **Dependency Injection**: Assembles all application dependencies through Environment pattern
- **Command Orchestration**: Routes subcommands to appropriate handlers with injected dependencies
- **Global Configuration**: Manages repository path, author, verbose logging, and terminal backend settings
- **Error Handling**: Provides user-friendly error messages with anyhow integration
- **Logging Infrastructure**: Configures structured logging to output.log file

## Key Abstractions

### Environment
Central dependency injection container that assembles GitRepository and other core dependencies. Follows builder pattern for configuration and provides methods to access or consume dependencies.

### CommandExecutor
Trait defining the interface for all CLI subcommands. Each command receives an Environment instance and returns a Result, enabling consistent error handling across all operations.

### Commands Module Structure
Organized module hierarchy where each subcommand is implemented as a struct with CommandExecutor trait, encapsulating command-specific logic and parameter validation.

### EnvironmentBuilder
Fluent builder API for constructing Environment instances with various configuration options, providing defaults and validation before dependency assembly.

### Config
Configuration container holding author name, repository path, verbosity settings, and terminal backend selection with sensible defaults.

## Development Rules

- **Single Responsibility**: Each command implementation handles only its specific CLI subcommand logic
- **Dependency Injection**: All external dependencies must flow through the Environment pattern
- **No Business Logic**: CLI layer only orchestrates, never implements domain logic
- **Error Boundaries**: Commands catch and transform domain errors into user-friendly messages
- **Builder Pattern**: Use EnvironmentBuilder for all Environment construction
- **Trait Consistency**: All commands must implement CommandExecutor trait
- **Clean Separation**: Commands coordinate but never directly implement semantic analysis

## Code Organization

```
diffviz-cli/
├── Cargo.toml                    # Binary crate with workspace dependencies
└── src/
    ├── main.rs                   # Entry point, argument parsing, Environment assembly
    ├── environment.rs            # Environment pattern implementation, dependency injection
    └── commands/
        ├── mod.rs               # CommandExecutor trait, shared command functionality
        ├── review.rs            # Interactive TUI subcommand (launches diffviz-review-tui)
        ├── show.rs              # File diff display (uses diffviz-core semantic analysis)
        ├── diagnose.rs          # Debug/diagnostic subcommand (TODO implementation)
        └── formatter.rs         # ANSI color constants for terminal output
```

## Testing Strategy

**Unit Tests**: Environment module contains comprehensive tests for configuration building, dependency injection, and error handling scenarios using temporary Git repositories.

**Integration Points**: Commands are tested through their CommandExecutor interface, ensuring proper Environment consumption and error propagation.

**Test Infrastructure**: Uses tempfile for creating test Git repositories and validates fluent builder API behavior across various configuration scenarios.

**Test Coverage**: Environment builder, configuration defaults, Git repository integration, and error conditions for invalid repository paths.

## Integration Patterns

**Environment Consumption**: Commands take ownership of Environment and consume dependencies as needed. GitRepository is extracted via `into_git_repository()` when implementing DiffProvider trait.

**Command Flow**:
1. Parse CLI args with Clap
2. Build Environment via EnvironmentBuilder
3. Instantiate command struct with parsed parameters
4. Execute command with Environment injection
5. Handle and display results/errors

**Dependency Assembly**: Environment constructs GitRepository from config path and validates repository accessibility before returning to commands.

**Error Transformation**: Commands catch domain-specific errors and transform them into anyhow::Error with user-friendly context for CLI display.