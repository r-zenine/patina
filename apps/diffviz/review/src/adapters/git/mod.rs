use crate::errors::DiffVizError;
use crate::providers::{DiffProvider, FileStats, FileStatus};
use crate::{DiffQuery, GitRef};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitError {
    #[error(transparent)]
    Core(#[from] gitkit::Error),
}

impl From<GitError> for DiffVizError {
    fn from(err: GitError) -> Self {
        match err {
            GitError::Core(gitkit::Error::Git(git_err)) => DiffVizError::Git(format!("{git_err}")),
            GitError::Core(gitkit::Error::RepositoryNotFound { path, source }) => {
                DiffVizError::Repository(format!("Repository not found at '{path}': {source}"))
            }
            GitError::Core(gitkit::Error::InvalidCommit { hash, source }) => {
                DiffVizError::InvalidOperation(format!("Invalid commit '{hash}': {source}"))
            }
            GitError::Core(gitkit::Error::StagingFailed { file, reason, .. }) => {
                DiffVizError::Git(format!("Failed to stage hunk in file '{file}': {reason}"))
            }
            GitError::Core(gitkit::Error::PatchCreationFailed { file, reason }) => {
                DiffVizError::ProcessingFailed(format!(
                    "Failed to create patch for file '{file}': {reason}"
                ))
            }
            GitError::Core(gitkit::Error::Io(io_err)) => DiffVizError::Io(io_err),
            GitError::Core(gitkit::Error::ValidationFailed {
                file,
                hunk_additions,
                hunk_deletions,
                git_additions,
                git_deletions,
            }) => DiffVizError::ProcessingFailed(format!(
                "Level 1 validation failed for file '{file}': hunks show +{hunk_additions}/{hunk_deletions} but git stats show +{git_additions}/{git_deletions}"
            )),
            GitError::Core(gitkit::Error::UnexpectedDeltaStatus { file, status }) => {
                DiffVizError::Git(format!(
                    "Unexpected delta status '{status}' for file '{file}'"
                ))
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, GitError>;

/// Adapter over `gitkit::GitRepository`. Exists as a local newtype (rather
/// than using `gitkit::GitRepository` directly) so `DiffProvider` can be
/// implemented on it without violating the orphan rule.
pub struct GitRepository {
    inner: gitkit::GitRepository,
}

impl GitRepository {
    pub fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        Ok(Self {
            inner: gitkit::GitRepository::open(path)?,
        })
    }

    /// Resolve a commit hash to its parent commit hash
    /// Used for building diff queries that show the exact commit's changes (commit^..commit)
    pub fn resolve_parent_commit(&self, commit_hash: &str) -> Result<String> {
        Ok(self.inner.resolve_parent_commit(commit_hash)?)
    }
}

fn map_stats(raw: gitkit::RawFileStats) -> FileStats {
    FileStats::new(raw.additions, raw.deletions)
}

fn map_status(raw: gitkit::RawFileStatus) -> FileStatus {
    match raw {
        gitkit::RawFileStatus::Added => FileStatus::Added,
        gitkit::RawFileStatus::Modified => FileStatus::Modified,
        gitkit::RawFileStatus::Deleted => FileStatus::Deleted,
        gitkit::RawFileStatus::Renamed => FileStatus::Renamed,
        gitkit::RawFileStatus::Copied => FileStatus::Copied,
        gitkit::RawFileStatus::Untracked => FileStatus::Untracked,
    }
}

/// Minimal working implementation of DiffProvider for GitRepository
/// TODO: Restore full functionality once missing types are available
impl DiffProvider for GitRepository {
    fn get_changed_files(
        &self,
        query: &DiffQuery,
    ) -> std::result::Result<Vec<(String, FileStatus)>, Box<dyn std::error::Error>> {
        let from_ref = git_ref_to_string(&query.from)?;
        let to_ref = git_ref_to_string(&query.to)?;

        // Handle special cases for working directory comparisons
        let raw_files = match (&query.from, &query.to) {
            (GitRef::Head, GitRef::Unstaged) => self.inner.get_working_directory_files()?,
            (GitRef::Staged, GitRef::Unstaged) => self.inner.get_working_directory_files()?, // TODO: Implement staged vs unstaged
            _ => self.inner.get_diff_files(&from_ref, &to_ref)?,
        };

        Ok(raw_files
            .into_iter()
            .map(|(path, status)| (path, map_status(status)))
            .collect())
    }

    fn get_file_stats(
        &self,
        file_path: &str,
        query: &DiffQuery,
    ) -> std::result::Result<FileStats, Box<dyn std::error::Error>> {
        let from_ref = git_ref_to_string(&query.from)?;
        let to_ref = git_ref_to_string(&query.to)?;

        // Handle special cases for working directory comparisons
        let raw_stats = match (&query.from, &query.to) {
            (GitRef::Head, GitRef::Unstaged) => {
                self.inner.get_working_directory_stats_for_file(file_path)?
            }
            (GitRef::Staged, GitRef::Unstaged) => {
                // TODO: Implement proper staged vs unstaged stats
                self.inner.get_working_directory_stats_for_file(file_path)?
            }
            _ => self
                .inner
                .get_file_stats_for_commits(file_path, &from_ref, &to_ref)?,
        };

        Ok(map_stats(raw_stats))
    }

    fn get_source_code(
        &self,
        file_path: &str,
        git_ref: &GitRef,
    ) -> std::result::Result<String, Box<dyn std::error::Error>> {
        let ref_string = git_ref_to_string(git_ref)?;

        let commit_ref = match git_ref {
            GitRef::Unstaged => "working-directory",
            GitRef::Staged => "HEAD", // TODO: Implement reading from index
            _ => ref_string.as_str(),
        };

        let content = self
            .inner
            .get_file_content_at_commit(file_path, commit_ref)?
            .ok_or_else(|| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("File not found: {file_path}"),
                )) as Box<dyn std::error::Error>
            })?;

        Ok(content)
    }

    fn get_file_hash(
        &self,
        file_path: &str,
        git_ref: &GitRef,
    ) -> std::result::Result<String, Box<dyn std::error::Error>> {
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
    ) -> std::result::Result<Option<String>, Box<dyn std::error::Error>> {
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

/// Helper function to convert GitRef to string for git operations
fn git_ref_to_string(git_ref: &GitRef) -> Result<String> {
    match git_ref {
        GitRef::Commit(hash) => Ok(hash.clone()),
        GitRef::Head => Ok("HEAD".to_string()),
        GitRef::Staged => Ok("--staged".to_string()), // Special marker for staged content
        GitRef::Unstaged => Ok("--unstaged".to_string()), // Special marker for working directory
    }
}
