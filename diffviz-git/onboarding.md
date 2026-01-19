# diffviz-git

Git filesystem layer providing capabilities to identify diffs for review and retrieve file content from the repository.

## Architecture Role

Infrastructure layer crate that serves as the filesystem abstraction between Git repositories and the review orchestration system. Sits above git2 library and provides a clean interface to diffviz-review for Git operations. Depends on both diffviz-review (for abstractions) and diffviz-core (for types), following the clean architecture where infrastructure layers communicate through review abstractions.

**Position in dependency graph:**
- Infrastructure layer depending on diffviz-review (review layer)
- Also depends on diffviz-core for domain types
- Used by diffviz-cli through Environment pattern dependency injection
- Wraps git2 library for Git repository operations

## Core Capabilities

- **Repository Management**: Open and validate Git repositories with structured error handling
- **Diff Identification**: Extract file changes, additions, deletions between commits and working directory
- **Content Retrieval**: Fetch file content at specific commits, HEAD, staged, or working directory states
- **Statistics Extraction**: Provide authoritative addition/deletion counts directly from Git diff operations
- **Reference Resolution**: Convert commit hashes, branch names, tags to Git tree objects for comparison
- **Working Directory Support**: Handle unstaged changes, staged changes, and commit-to-commit diffs

## Key Abstractions

- **GitRepository**: Primary interface wrapping git2::Repository with structured error handling
- **GitError**: Comprehensive error type with contextual information (file paths, commit hashes, reasons)
- **DiffProvider Implementation**: Implements review layer's DiffProvider trait for Git-backed file operations
- **TestRepo**: Test utility for creating temporary repositories with commit operations

## Development Rules

- **Fail-Fast Approach**: No defensive programming or fallbacks - explicit error handling with detailed context
- **Structured Error Design**: Use thiserror with source error chains, never convert to simple strings
- **Git2 Abstraction**: All git2 operations must be wrapped with GitError for consistent error handling
- **No Direct File Access**: File content must be retrieved through Git objects, not filesystem operations (except working directory)
- **Reference Safety**: Always validate commit references before tree operations

## Code Organization

```
diffviz-git/
├── Cargo.toml           # Dependencies: diffviz-review, diffviz-core, git2, thiserror
├── src/
│   └── lib.rs           # Single module containing all Git operations
└── tests/
    ├── test_utils.rs    # TestRepo utility for temporary Git repositories
    └── error_handling.rs # Comprehensive error conversion and handling tests
```

**Architecture Notes:**
- Single-file crate design - all functionality in src/lib.rs
- Contains extensive commented-out sections marked "TODO: BROKEN" indicating ongoing architectural refactoring
- Many methods disabled due to missing types from architectural changes
- Currently implements minimal working DiffProvider with planned restoration of full functionality

## Testing Strategy

**Test Infrastructure:**
- **TestRepo Utility**: Comprehensive temporary repository builder with commit operations
- **Error Conversion Testing**: Validates GitError to DiffVizError conversions preserve context
- **Repository Operations**: Tests opening repositories, handling invalid paths, and file operations

**Test Categories:**
- Unit tests for error handling and conversions (tests/error_handling.rs)
- Integration utilities for repository setup (tests/test_utils.rs)
- TestRepo provides: commit_file, delete_file, get_commit_hash operations
- Structured test data builders following project conventions

**Quality Gates:**
- Error message validation ensuring contextual information preservation
- Git operation testing with proper cleanup via TempDir
- Comprehensive GitError variant coverage in conversion tests

## Integration Patterns

**DiffProvider Implementation:**
- Implements diffviz-review's DiffProvider trait for Git-backed operations
- Supports DiffQuery variants: CommitComparison, WorkingDirectory, CommitToHead, HeadToCommit
- Provides get_changed_files, get_file_stats, get_source_code methods
- Maps GitRef enum to Git references (commits, HEAD, staged, unstaged)

**Error Handling Chain:**
- GitError -> DiffVizError conversion preserves source error information
- Contextual error variants include file paths, commit hashes, operation details
- Structured error design enables precise debugging and user feedback

**Special Reference Handling:**
- GitRef::Unstaged -> working directory file access
- GitRef::Staged -> index/staged content (TODO: full implementation)
- GitRef::Head -> HEAD commit reference
- GitRef::Commit -> specific commit hash resolution

## Development Tools

**TestRepo Capabilities:**
- Automatic temporary directory and repository creation
- Git configuration setup (user.name, user.email) for test commits
- File commit operations with parent directory creation
- File deletion and removal from index operations
- Commit hash extraction for test assertions