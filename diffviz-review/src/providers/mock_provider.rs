//! Mock DiffProvider implementation for testing with fixtures
//!
//! This module provides MockDiffProvider that loads curated test fixtures
//! to enable predictable TUI testing without requiring git repositories.
//!
//! Phase 6: Context Expansion Integration
//! MockDiffProvider works with ReviewEngineBuilder which applies context expansion
//! to produce ReviewableDiffs with rich ContextNode trees and varied relevance scores.
//! The pipeline: fixtures → MockDiffProvider → ReviewEngineBuilder → semantic analysis + context expansion → ReviewableDiffs

use crate::entities::git_ref::{DiffQuery, GitRef};
use crate::providers::{DiffProvider, FileStats, FileStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReviewFixture {
    pub name: String,
    pub file_path: String,
    pub language: String,
    pub description: String,
    pub old_code: String,
    pub new_code: String,
    pub expected_line_stats: LineStats,
    pub metadata: FixtureMetadata,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LineStats {
    pub additions: u32,
    pub deletions: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FixtureMetadata {
    pub complexity_level: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Error)]
pub enum MockProviderError {
    #[error("Failed to read fixture file: {path}")]
    FixtureRead {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse fixture JSON: {path}")]
    FixtureParse {
        path: String,
        #[source]
        source: serde_json::Error,
    },

    #[error("Fixture directory not found")]
    FixtureDirectoryMissing,
}

pub struct MockDiffProvider {
    fixtures: HashMap<String, ReviewFixture>,
    changed_files: Vec<(String, FileStatus)>,
    file_contents: HashMap<(String, GitRef), String>, // (file_path, git_ref) -> content
}

impl MockDiffProvider {
    /// Create an empty MockDiffProvider for testing
    pub fn new() -> Self {
        Self {
            fixtures: HashMap::new(),
            changed_files: Vec::new(),
            file_contents: HashMap::new(),
        }
    }

    /// Add file content for a specific git ref (for testing)
    pub fn add_file_content(&mut self, file_path: &str, git_ref: &GitRef, content: &str) {
        let key = (file_path.to_string(), git_ref.clone());
        self.file_contents.insert(key, content.to_string());
    }

    /// Create MockDiffProvider from review crate fixtures
    pub fn from_review_fixtures() -> Result<Self, MockProviderError> {
        let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures");

        if !fixture_dir.exists() {
            return Err(MockProviderError::FixtureDirectoryMissing);
        }

        let mut fixtures = HashMap::new();
        let mut changed_files = Vec::new();

        // Load all JSON fixtures from the directory, in filename order:
        // read_dir order is filesystem-dependent, and changed_files order
        // flows into decision/chunk ordering downstream — sorting keeps
        // fixture-driven runs reproducible across machines and sessions.
        let mut paths: Vec<PathBuf> = std::fs::read_dir(&fixture_dir)
            .map_err(|e| MockProviderError::FixtureRead {
                path: fixture_dir.display().to_string(),
                source: e,
            })?
            .map(|entry| {
                entry
                    .map(|e| e.path())
                    .map_err(|e| MockProviderError::FixtureRead {
                        path: fixture_dir.display().to_string(),
                        source: e,
                    })
            })
            .collect::<Result<_, _>>()?;
        paths.sort();

        for path in paths {
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content =
                    std::fs::read_to_string(&path).map_err(|e| MockProviderError::FixtureRead {
                        path: path.display().to_string(),
                        source: e,
                    })?;

                let fixture: ReviewFixture = serde_json::from_str(&content).map_err(|e| {
                    MockProviderError::FixtureParse {
                        path: path.display().to_string(),
                        source: e,
                    }
                })?;

                changed_files.push((fixture.file_path.clone(), FileStatus::Modified));
                fixtures.insert(fixture.file_path.clone(), fixture);
            }
        }

        Ok(MockDiffProvider {
            fixtures,
            changed_files,
            file_contents: HashMap::new(),
        })
    }
}

impl Default for MockDiffProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl DiffProvider for MockDiffProvider {
    fn get_changed_files(
        &self,
        _query: &DiffQuery,
    ) -> Result<Vec<(String, FileStatus)>, Box<dyn std::error::Error>> {
        Ok(self.changed_files.clone())
    }

    fn get_file_stats(
        &self,
        file_path: &str,
        _query: &DiffQuery,
    ) -> Result<FileStats, Box<dyn std::error::Error>> {
        let fixture = self
            .fixtures
            .get(file_path)
            .ok_or_else(|| format!("Fixture not found for path: {file_path}"))?;

        Ok(FileStats::new(
            fixture.expected_line_stats.additions,
            fixture.expected_line_stats.deletions,
        ))
    }

    fn get_source_code(
        &self,
        file_path: &str,
        git_ref: &GitRef,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Check file_contents map first (for test data)
        let key = (file_path.to_string(), git_ref.clone());
        if let Some(content) = self.file_contents.get(&key) {
            return Ok(content.clone());
        }

        // Fall back to fixtures
        let fixture = self
            .fixtures
            .get(file_path)
            .ok_or_else(|| format!("File not found: {file_path} at ref {git_ref:?}"))?;

        match git_ref {
            GitRef::Head => Ok(fixture.old_code.clone()), // "Before" state
            GitRef::Unstaged => Ok(fixture.new_code.clone()), // "After" state
            _ => Ok(fixture.old_code.clone()),            // Default to old_code for other refs
        }
    }

    fn get_file_hash(
        &self,
        file_path: &str,
        git_ref: &GitRef,
    ) -> Result<String, Box<dyn std::error::Error>> {
        use sha2::{Digest, Sha256};

        // Get the content (reuse get_source_code)
        let content = self.get_source_code(file_path, git_ref)?;

        // Normalize line endings (CRLF -> LF)
        let normalized_content = content.replace("\r\n", "\n");

        // Calculate SHA-256 hash
        let mut hasher = Sha256::new();
        hasher.update(normalized_content.as_bytes());
        let hash = hasher.finalize();

        Ok(format!("{hash:x}"))
    }

    fn get_content_snapshot(
        &self,
        file_path: &str,
        git_ref: &GitRef,
        line_range: &crate::entities::reviewable_diff_id::LineRange,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        // Get the full content
        let content = self.get_source_code(file_path, git_ref)?;
        let lines: Vec<&str> = content.lines().collect();

        // Validate line range (1-based)
        if line_range.start_line == 0 || line_range.end_line == 0 {
            return Ok(None);
        }

        if line_range.start_line > lines.len() || line_range.end_line > lines.len() {
            return Ok(None);
        }

        if line_range.start_line > line_range.end_line {
            return Ok(None);
        }

        // Extract the requested lines (convert from 1-based to 0-based indexing)
        let start_idx = line_range.start_line - 1;
        let end_idx = line_range.end_line; // end_line is inclusive, so this is correct for slicing

        let snapshot = lines[start_idx..end_idx].join("\n");
        Ok(Some(snapshot))
    }
}

#[cfg(test)]
#[path = "mock_provider_tests.rs"]
mod tests;
