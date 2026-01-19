//! Unique identifier for ReviewableDiffs in the review system
//!
//! This module defines the ReviewableDiffId type which serves as the universal
//! identifier for review items, replacing the legacy ChunkId system.

use crate::entities::git_ref::{DiffQuery, GitRef};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};

/// Line range in a file (1-based line numbers, 0-based columns)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LineRange {
    /// 1-based start line number
    pub start_line: usize,
    /// 1-based end line number (inclusive)
    pub end_line: usize,
    /// 0-based start column
    pub start_column: usize,
    /// 0-based end column
    pub end_column: usize,
}

/// Unique identifier for a ReviewableDiff
///
/// This triplet uniquely identifies a semantic diff unit:
/// - `query`: The type of diff query (commit comparison, working directory, etc.)
/// - `file_path`: The file containing the change
/// - `line_range`: The line range in the file where the change occurs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewableDiffId {
    pub query: DiffQuery,
    pub file_path: String,
    pub line_range: LineRange,
}

impl ReviewableDiffId {
    /// Create a new ReviewableDiffId
    pub fn new(query: DiffQuery, file_path: String, line_range: LineRange) -> Self {
        Self {
            query,
            file_path,
            line_range,
        }
    }

    /// Get the file path
    pub fn file_path(&self) -> &str {
        &self.file_path
    }

    /// Get the diff query
    pub fn query(&self) -> &DiffQuery {
        &self.query
    }

    /// Get the line range
    pub fn line_range(&self) -> LineRange {
        self.line_range
    }

    /// Check if this ID is for the same file and query as another
    pub fn same_file_and_query(&self, other: &Self) -> bool {
        self.file_path == other.file_path && self.query == other.query
    }
}

impl fmt::Display for ReviewableDiffId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let query_str = format_diff_query(&self.query);

        write!(
            f,
            "{}:{}:L{}-{}",
            query_str, self.file_path, self.line_range.start_line, self.line_range.end_line
        )
    }
}

impl PartialEq for ReviewableDiffId {
    fn eq(&self, other: &Self) -> bool {
        self.query == other.query
            && self.file_path == other.file_path
            && self.line_range == other.line_range
    }
}

impl Eq for ReviewableDiffId {}

impl Hash for ReviewableDiffId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash all three components for unique identification
        self.query.hash(state);
        self.file_path.hash(state);
        self.line_range.start_line.hash(state);
        self.line_range.end_line.hash(state);
        // Note: We don't hash column positions as they're less significant for identity
    }
}

impl PartialOrd for ReviewableDiffId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ReviewableDiffId {
    fn cmp(&self, other: &Self) -> Ordering {
        // First compare by file path
        match self.file_path.cmp(&other.file_path) {
            Ordering::Equal => {}
            ord => return ord,
        }

        // Then by query (this ensures same-query items group together)
        match compare_queries(&self.query, &other.query) {
            Ordering::Equal => {}
            ord => return ord,
        }

        // Finally by line range (start line, then end line)
        match self.line_range.start_line.cmp(&other.line_range.start_line) {
            Ordering::Equal => self.line_range.end_line.cmp(&other.line_range.end_line),
            ord => ord,
        }
    }
}

/// Helper enum describing the semantic category of a diff query
#[derive(Debug, PartialEq, Eq)]
enum QueryCategory<'a> {
    WorkingDirectory,
    CommitToHead { from: &'a str },
    HeadToCommit { to: &'a str },
    CommitComparison { from: &'a str, to: &'a str },
    Other,
}

fn categorize_query(query: &DiffQuery) -> QueryCategory<'_> {
    use GitRef::*;
    match (&query.from, &query.to) {
        (Head, Unstaged) | (Staged, Unstaged) => QueryCategory::WorkingDirectory,
        (Commit(from), Head) => QueryCategory::CommitToHead { from },
        (Head, Commit(to)) => QueryCategory::HeadToCommit { to },
        (Commit(from), Commit(to)) => QueryCategory::CommitComparison { from, to },
        _ => QueryCategory::Other,
    }
}

fn compare_queries(a: &DiffQuery, b: &DiffQuery) -> Ordering {
    let priority = |category: &QueryCategory<'_>| match category {
        QueryCategory::WorkingDirectory => 0,
        QueryCategory::CommitToHead { .. } => 1,
        QueryCategory::HeadToCommit { .. } => 2,
        QueryCategory::CommitComparison { .. } => 3,
        QueryCategory::Other => 4,
    };

    let cat_a = categorize_query(a);
    let cat_b = categorize_query(b);

    match priority(&cat_a).cmp(&priority(&cat_b)) {
        Ordering::Equal => match (cat_a, cat_b) {
            (
                QueryCategory::CommitToHead { from: f1 },
                QueryCategory::CommitToHead { from: f2 },
            ) => f1.cmp(f2),
            (QueryCategory::HeadToCommit { to: t1 }, QueryCategory::HeadToCommit { to: t2 }) => {
                t1.cmp(t2)
            }
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

fn format_diff_query(query: &DiffQuery) -> String {
    match categorize_query(query) {
        QueryCategory::WorkingDirectory => "working".to_string(),
        QueryCategory::CommitToHead { from } => format!("{}..HEAD", shorten_ref(from)),
        QueryCategory::HeadToCommit { to } => format!("HEAD..{}", shorten_ref(to)),
        QueryCategory::CommitComparison { from, to } => {
            format!("{}..{}", shorten_ref(from), shorten_ref(to))
        }
        QueryCategory::Other => format!("{}->{:?}", format_git_ref(&query.from), &query.to),
    }
}

fn shorten_ref(reference: &str) -> String {
    let len = reference.len();
    let end = len.min(7);
    reference[..end].to_string()
}

fn format_git_ref(git_ref: &GitRef) -> String {
    match git_ref {
        GitRef::Commit(hash) => shorten_ref(hash),
        GitRef::Head => "HEAD".to_string(),
        GitRef::Staged => "STAGED".to_string(),
        GitRef::Unstaged => "UNSTAGED".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::git_ref::GitRef;

    fn create_test_id() -> ReviewableDiffId {
        ReviewableDiffId::new(
            DiffQuery::head_to_unstaged(),
            "src/main.rs".to_string(),
            LineRange {
                start_line: 10,
                end_line: 20,
                start_column: 0,
                end_column: 0,
            },
        )
    }

    #[test]
    fn test_reviewable_diff_id_creation() {
        let id = create_test_id();
        assert_eq!(id.file_path(), "src/main.rs");
        assert_eq!(id.line_range().start_line, 10);
        assert_eq!(id.line_range().end_line, 20);
    }

    #[test]
    fn test_equality() {
        let id1 = create_test_id();
        let id2 = create_test_id();
        assert_eq!(id1, id2);

        let id3 = ReviewableDiffId::new(
            DiffQuery::head_to_unstaged(),
            "src/main.rs".to_string(),
            LineRange {
                start_line: 15,
                end_line: 25,
                start_column: 0,
                end_column: 0,
            },
        );
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_ordering() {
        let id1 = ReviewableDiffId::new(
            DiffQuery::head_to_unstaged(),
            "src/a.rs".to_string(),
            LineRange {
                start_line: 10,
                end_line: 20,
                start_column: 0,
                end_column: 0,
            },
        );

        let id2 = ReviewableDiffId::new(
            DiffQuery::head_to_unstaged(),
            "src/b.rs".to_string(),
            LineRange {
                start_line: 10,
                end_line: 20,
                start_column: 0,
                end_column: 0,
            },
        );

        assert!(id1 < id2); // a.rs comes before b.rs

        let id3 = ReviewableDiffId::new(
            DiffQuery::head_to_unstaged(),
            "src/a.rs".to_string(),
            LineRange {
                start_line: 5,
                end_line: 15,
                start_column: 0,
                end_column: 0,
            },
        );

        assert!(id3 < id1); // Line 5 comes before line 10
    }

    #[test]
    fn test_display() {
        let id = ReviewableDiffId::new(
            DiffQuery::new(
                GitRef::Commit("abc123def456".to_string()),
                GitRef::Commit("789xyz000111".to_string()),
            ),
            "src/main.rs".to_string(),
            LineRange {
                start_line: 10,
                end_line: 20,
                start_column: 0,
                end_column: 0,
            },
        );

        let display = format!("{id}");
        assert_eq!(display, "abc123d..789xyz0:src/main.rs:L10-20");
    }

    #[test]
    fn test_same_file_and_query() {
        let id1 = create_test_id();
        let id2 = ReviewableDiffId::new(
            DiffQuery::head_to_unstaged(),
            "src/main.rs".to_string(),
            LineRange {
                start_line: 30,
                end_line: 40,
                start_column: 0,
                end_column: 0,
            },
        );

        assert!(id1.same_file_and_query(&id2));

        let id3 = ReviewableDiffId::new(
            DiffQuery::head_to_unstaged(),
            "src/other.rs".to_string(),
            LineRange {
                start_line: 10,
                end_line: 20,
                start_column: 0,
                end_column: 0,
            },
        );

        assert!(!id1.same_file_and_query(&id3));
    }
}
