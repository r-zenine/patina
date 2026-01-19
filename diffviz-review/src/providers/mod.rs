//! Diff provider abstractions for review workflows
//!
//! This module contains the DiffProvider trait and related types that define
//! the interface between the review layer and infrastructure layers (like git).
//! The review layer defines what diff capabilities it needs, and infrastructure
//! layers implement these interfaces.

pub mod mock_provider;

use crate::entities::git_ref::{DiffQuery, GitRef};
use serde::{Deserialize, Serialize};

/// Git statistics for a specific file with validation capabilities
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileStats {
    /// Number of lines added
    pub additions: u32,
    /// Number of lines removed  
    pub deletions: u32,
    /// Total lines changed (additions + deletions)
    pub total_changes: u32,
}

impl FileStats {
    /// Create new FileStats
    pub fn new(additions: u32, deletions: u32) -> Self {
        Self {
            additions,
            deletions,
            total_changes: additions + deletions,
        }
    }

    /// Check if this represents a file creation (only additions)
    pub fn is_creation(&self) -> bool {
        self.additions > 0 && self.deletions == 0
    }

    /// Check if this represents a file deletion (only deletions)
    pub fn is_deletion(&self) -> bool {
        self.additions == 0 && self.deletions > 0
    }

    /// Check if this represents a file modification (both additions and deletions)
    pub fn is_modification(&self) -> bool {
        self.additions > 0 && self.deletions > 0
    }

    /// Check if no changes (shouldn't happen in practice)
    pub fn is_unchanged(&self) -> bool {
        self.additions == 0 && self.deletions == 0
    }
}

/// Git file change status for organizing review workflows  
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
    Untracked,
}

/// Trait for providing diff data to review processing
/// This implements dependency inversion - infrastructure layers implement this interface
pub trait DiffProvider {
    /// Get all files with changes for the given diff query
    fn get_changed_files(
        &self,
        query: &DiffQuery,
    ) -> std::result::Result<Vec<(String, FileStatus)>, Box<dyn std::error::Error>>;

    /// Get authoritative git statistics for validation
    /// This provides the ground truth for line counts that hunks and chunks must match
    fn get_file_stats(
        &self,
        file_path: &str,
        query: &DiffQuery,
    ) -> std::result::Result<FileStats, Box<dyn std::error::Error>>;

    /// Get source code content for a file at a specific Git reference
    /// This enables semantic analysis by providing the actual file content
    fn get_source_code(
        &self,
        file_path: &str,
        git_ref: &GitRef,
    ) -> std::result::Result<String, Box<dyn std::error::Error>>;
}
