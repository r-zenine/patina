//! Git reference types for clean state modeling
//!
//! This module provides types for representing Git states and diff queries
//! in a type-safe way, eliminating hardcoded strings and providing clear
//! semantics for Git operations.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

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

impl std::str::FromStr for GitRef {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s {
            "HEAD" => GitRef::Head,
            "staged" => GitRef::Staged,
            "unstaged" | "working_tree" => GitRef::Unstaged,
            other => GitRef::Commit(other.to_string()),
        })
    }
}

impl std::str::FromStr for DiffQuery {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "HEAD..unstaged" => Ok(DiffQuery::head_to_unstaged()),
            _ => {
                if let Some((from, to)) = s.split_once("..") {
                    Ok(DiffQuery::new(
                        GitRef::commit(from.to_string()),
                        GitRef::commit(to.to_string()),
                    ))
                } else {
                    Err(format!("Invalid query format: {s}"))
                }
            }
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

enum QueryCategory<'a> {
    WorkingDirectory,
    CommitToHead { from: &'a str },
    HeadToCommit { to: &'a str },
    CommitComparison { from: &'a str, to: &'a str },
    Other,
}

fn categorize_query(query: &DiffQuery) -> QueryCategory<'_> {
    match (&query.from, &query.to) {
        (GitRef::Head | GitRef::Staged, GitRef::Unstaged) => QueryCategory::WorkingDirectory,
        (GitRef::Commit(from), GitRef::Head) => QueryCategory::CommitToHead { from },
        (GitRef::Head, GitRef::Commit(to)) => QueryCategory::HeadToCommit { to },
        (GitRef::Commit(from), GitRef::Commit(to)) => QueryCategory::CommitComparison { from, to },
        _ => QueryCategory::Other,
    }
}

impl std::fmt::Display for DiffQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match categorize_query(self) {
            QueryCategory::WorkingDirectory => write!(f, "working"),
            QueryCategory::CommitToHead { from } => {
                write!(f, "{}..HEAD", &from[..from.len().min(7)])
            }
            QueryCategory::HeadToCommit { to } => {
                write!(f, "HEAD..{}", &to[..to.len().min(7)])
            }
            QueryCategory::CommitComparison { from, to } => {
                write!(
                    f,
                    "{}..{}",
                    &from[..from.len().min(7)],
                    &to[..to.len().min(7)]
                )
            }
            QueryCategory::Other => write!(f, "{}..{}", self.from, self.to),
        }
    }
}

impl PartialOrd for DiffQuery {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DiffQuery {
    fn cmp(&self, other: &Self) -> Ordering {
        let priority = |cat: &QueryCategory<'_>| match cat {
            QueryCategory::WorkingDirectory => 0u8,
            QueryCategory::CommitToHead { .. } => 1,
            QueryCategory::HeadToCommit { .. } => 2,
            QueryCategory::CommitComparison { .. } => 3,
            QueryCategory::Other => 4,
        };

        let cat_self = categorize_query(self);
        let cat_other = categorize_query(other);

        match priority(&cat_self).cmp(&priority(&cat_other)) {
            Ordering::Equal => match (cat_self, cat_other) {
                (
                    QueryCategory::CommitToHead { from: f1 },
                    QueryCategory::CommitToHead { from: f2 },
                ) => f1.cmp(f2),
                (
                    QueryCategory::HeadToCommit { to: t1 },
                    QueryCategory::HeadToCommit { to: t2 },
                ) => t1.cmp(t2),
                (
                    QueryCategory::CommitComparison { from: f1, to: t1 },
                    QueryCategory::CommitComparison { from: f2, to: t2 },
                ) => match f1.cmp(f2) {
                    Ordering::Equal => t1.cmp(t2),
                    ord => ord,
                },
                _ => Ordering::Equal,
            },
            ord => ord,
        }
    }
}

impl std::fmt::Display for GitRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitRef::Commit(hash) => write!(f, "{}", &hash[..hash.len().min(7)]),
            GitRef::Head => write!(f, "HEAD"),
            GitRef::Staged => write!(f, "STAGED"),
            GitRef::Unstaged => write!(f, "UNSTAGED"),
        }
    }
}
