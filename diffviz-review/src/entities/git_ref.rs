//! Git reference types for clean state modeling
//!
//! This module provides types for representing Git states and diff queries
//! in a type-safe way, eliminating hardcoded strings and providing clear
//! semantics for Git operations.

use serde::{Deserialize, Serialize};

/// Represents different Git states that can be referenced
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GitRef {
    /// A specific commit (hash, branch, tag, etc.)
    Commit(String),
    /// Current HEAD commit
    Head,
    /// Staged changes in the index
    Staged,
    /// Unstaged changes in the working tree
    Unstaged,
}

/// Represents a diff query between two Git states
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DiffQuery {
    /// Source Git state for comparison
    pub from: GitRef,
    /// Target Git state for comparison
    pub to: GitRef,
}

impl DiffQuery {
    /// Create a new diff query between two Git references
    pub fn new(from: GitRef, to: GitRef) -> Self {
        Self { from, to }
    }

    /// Create a diff query from a commit to HEAD
    pub fn commit_to_head(commit: String) -> Self {
        Self {
            from: GitRef::Commit(commit),
            to: GitRef::Head,
        }
    }

    /// Create a diff query from HEAD to unstaged changes
    pub fn head_to_unstaged() -> Self {
        Self {
            from: GitRef::Head,
            to: GitRef::Unstaged,
        }
    }

    /// Create a diff query from staged to unstaged changes
    pub fn staged_to_unstaged() -> Self {
        Self {
            from: GitRef::Staged,
            to: GitRef::Unstaged,
        }
    }

    /// Create a diff query from HEAD to staged changes
    pub fn head_to_staged() -> Self {
        Self {
            from: GitRef::Head,
            to: GitRef::Staged,
        }
    }
}

impl GitRef {
    /// Create a GitRef for a specific commit
    pub fn commit(commit: String) -> Self {
        Self::Commit(commit)
    }

    /// Create a GitRef for HEAD
    pub fn head() -> Self {
        Self::Head
    }

    /// Create a GitRef for staged changes
    pub fn staged() -> Self {
        Self::Staged
    }

    /// Create a GitRef for unstaged changes
    pub fn unstaged() -> Self {
        Self::Unstaged
    }
}
