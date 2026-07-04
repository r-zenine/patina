use diffviz_review::errors::DiffVizError;
use diffviz_review::providers::{DiffProvider, FileStats, FileStatus};
use diffviz_review::{DiffQuery, GitRef};
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

    // TODO: BROKEN - Hunk type doesn't exist, needs refactoring
    /*
    fn get_file_hunks_for_commits(
        &self,
        file_path: &str,
        from_commit: &str,
        to_commit: &str,
    ) -> std::result::Result<Vec<Hunk>, Box<dyn std::error::Error>> {
        let (from_tree, to_tree) = self.resolve_commit_trees(from_commit, to_commit)?;
        let git_diff = self.create_git_diff(&from_tree, &to_tree)?;

        self.extract_hunks_from_git_diff(&git_diff, file_path, from_commit, to_commit)
    }
    */

    // TODO: All methods below are broken - multiple missing types (Hunk, RetrievalContext, etc.)
    /*
        fn get_working_directory_hunks_for_file(
            &self,
            file_path: &str,
        ) -> std::result::Result<Vec<Hunk>, Box<dyn std::error::Error>> {
            let head_tree = self.get_head_tree()?;
            let mut diff_opts = git2::DiffOptions::new();
            diff_opts.include_untracked(true);
            diff_opts.context_lines(3);

            let git_diff = self
                .repo
                .diff_tree_to_workdir_with_index(Some(&head_tree), Some(&mut diff_opts))
                .map_err(GitError::Git)?;

            self.extract_hunks_from_git_diff(&git_diff, file_path, "HEAD", "working-directory")
        }

        fn extract_hunks_from_git_diff(
            &self,
            git_diff: &git2::Diff,
            target_file_path: &str,
            from_commit: &str,
            to_commit: &str,
        ) -> std::result::Result<Vec<Hunk>, Box<dyn std::error::Error>> {
            // HunkRef is no longer needed - using RetrievalContext directly

            let mut hunks = Vec::new();

            // Iterate through all deltas (file changes) in the diff
            for (delta_idx, delta) in git_diff.deltas().enumerate() {
                // Check if this delta is for our target file
                let file_path = delta
                    .new_file()
                    .path()
                    .or_else(|| delta.old_file().path())
                    .and_then(|p| p.to_str())
                    .unwrap_or("");

                if file_path != target_file_path {
                    continue;
                }

                // Create a patch for this specific file
                let patch = match git2::Patch::from_diff(git_diff, delta_idx)? {
                    Some(p) => p,
                    None => continue, // Binary or unchanged file
                };

                // Iterate through all hunks in the patch
                for hunk_idx in 0..patch.num_hunks() {
                    let (hunk_info, _total_lines) = patch.hunk(hunk_idx)?;

                    let old_start = hunk_info.old_start();
                    let old_lines = hunk_info.old_lines();
                    let new_start = hunk_info.new_start();
                    let new_lines = hunk_info.new_lines();

                    // Note: Line data is no longer collected here - it's provided via DiffProvider::get_file_content
                    // This method now only creates hunk metadata

                    // Create context directly
                    let context = if from_commit == "HEAD" && to_commit == "working-directory" {
                        RetrievalContext::UnstagedChanges {
                            start_line: new_start,
                            line_count: new_lines,
                        }
                    } else {
                        RetrievalContext::CommitComparison {
                            from_commit: from_commit.to_string(),
                            to_commit: to_commit.to_string(),
                            old_start,
                            old_lines,
                            new_start,
                            new_lines,
                        }
                    };

                    // Create and add the hunk (only metadata - lines are provided via DiffProvider)
                    let hunk = Hunk {
                        header: Some(format!(
                            "@@ -{old_start},{old_lines} +{new_start},{new_lines} @@"
                        )),
                        file_path: target_file_path.to_string(),
                        context,
                    };

                    hunks.push(hunk);
                }
            }

            Ok(hunks)
        }
    }

    /// Implementation of LineRetriever for GitRepository
    impl LineRetriever for GitRepository {
        fn retrieve_lines(
            &self,
            context: &RetrievalContext,
            file_path: &str,
        ) -> std::result::Result<Vec<String>, ContextExpansionError> {
            self.retrieve_lines(context, file_path).map_err(|e| {
                ContextExpansionError::RetrievalFailed {
                    message: e.to_string(),
                }
            })
        }
    }

    /// Implementation of DiffProvider for GitRepository
    impl DiffProvider for GitRepository {
        fn get_file_content(
            &self,
            file_path: &str,
            query: &DiffQuery,
        ) -> std::result::Result<FileContent, Box<dyn std::error::Error>> {
            match query {
                DiffQuery::CommitComparison {
                    from_commit,
                    to_commit,
                } => {
                    let old_content = self.get_file_content_at_commit(file_path, from_commit)?;
                    let new_content = self.get_file_content_at_commit(file_path, to_commit)?;
                    Ok(FileContent::Comparison {
                        old_content,
                        new_content,
                    })
                }
                DiffQuery::WorkingDirectory => {
                    let head_content = self.get_file_content_at_commit(file_path, "HEAD")?;
                    let working_content =
                        self.get_file_content_at_commit(file_path, "working-directory")?;
                    Ok(FileContent::WorkingDirectory {
                        head_content,
                        working_content,
                    })
                }
                DiffQuery::CommitToHead { from_commit } => {
                    let commit_content = self.get_file_content_at_commit(file_path, from_commit)?;
                    let head_content = self.get_file_content_at_commit(file_path, "HEAD")?;
                    Ok(FileContent::CommitToHead {
                        commit_content,
                        head_content,
                    })
                }
                DiffQuery::HeadToCommit { to_commit } => {
                    let head_content = self.get_file_content_at_commit(file_path, "HEAD")?;
                    let commit_content = self.get_file_content_at_commit(file_path, to_commit)?;
                    Ok(FileContent::HeadToCommit {
                        head_content,
                        commit_content,
                    })
                }
            }
        }

        fn get_file_diff_complete(
            &self,
            file_path: &str,
            query: &DiffQuery,
        ) -> std::result::Result<Vec<Hunk>, Box<dyn std::error::Error>> {
            // Get hunks using existing implementation
            let hunks = self.get_file_hunks(file_path, query)?;

            // Note: Level 1 validation is now simplified since hunks no longer contain line data
            // The authoritative statistics are provided via get_file_stats method

            Ok(hunks)
        }

        fn get_file_stats(
            &self,
            file_path: &str,
            query: &DiffQuery,
        ) -> std::result::Result<FileStats, Box<dyn std::error::Error>> {
            match query {
                DiffQuery::CommitComparison {
                    from_commit,
                    to_commit,
                } => Ok(self.get_file_stats_for_commits(file_path, from_commit, to_commit)?),
                DiffQuery::WorkingDirectory => {
                    Ok(self.get_working_directory_stats_for_file(file_path)?)
                }
                DiffQuery::CommitToHead { from_commit } => {
                    Ok(self.get_file_stats_for_commits(file_path, from_commit, "HEAD")?)
                }
                DiffQuery::HeadToCommit { to_commit } => {
                    Ok(self.get_file_stats_for_commits(file_path, "HEAD", to_commit)?)
                }
            }
        }
        fn get_changed_files(
            &self,
            query: &DiffQuery,
        ) -> std::result::Result<Vec<(String, FileStatus)>, Box<dyn std::error::Error>> {
            match query {
                DiffQuery::CommitComparison {
                    from_commit,
                    to_commit,
                } => self.get_diff_files(from_commit, to_commit),
                DiffQuery::WorkingDirectory => self.get_working_directory_files(),
                DiffQuery::CommitToHead { from_commit } => self.get_diff_files(from_commit, "HEAD"),
                DiffQuery::HeadToCommit { to_commit } => self.get_diff_files("HEAD", to_commit),
            }
        }

        fn get_file_hunks(
            &self,
            file_path: &str,
            query: &DiffQuery,
        ) -> std::result::Result<Vec<Hunk>, Box<dyn std::error::Error>> {
            match query {
                DiffQuery::CommitComparison {
                    from_commit,
                    to_commit,
                } => self.get_file_hunks_for_commits(file_path, from_commit, to_commit),
                DiffQuery::WorkingDirectory => self.get_working_directory_hunks_for_file(file_path),
                DiffQuery::CommitToHead { from_commit } => {
                    self.get_file_hunks_for_commits(file_path, from_commit, "HEAD")
                }
                DiffQuery::HeadToCommit { to_commit } => {
                    self.get_file_hunks_for_commits(file_path, "HEAD", to_commit)
                }
            }
        }
    }
    */
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
        line_range: &diffviz_review::entities::reviewable_diff_id::LineRange,
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
