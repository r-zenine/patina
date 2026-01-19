use diffviz_review::errors::DiffVizError;
use diffviz_review::providers::{DiffProvider, FileStats, FileStatus};
use diffviz_review::{DiffQuery, GitRef};
use git2::{DiffOptions, Repository};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitError {
    #[error("Git operation failed: {0}")]
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

    #[error("Failed to stage hunk in file '{file}': {reason}")]
    StagingFailed {
        file: String,
        reason: String,
        #[source]
        source: Option<git2::Error>,
    },

    #[error("Failed to create patch for file '{file}': {reason}")]
    PatchCreationFailed { file: String, reason: String },

    #[error("IO error")]
    Io(#[from] std::io::Error),

    #[error(
        "Level 1 validation failed for file '{file}': hunks show {hunk_additions:+}/{hunk_deletions:-} but git stats show {git_additions:+}/{git_deletions:-}"
    )]
    ValidationFailed {
        file: String,
        hunk_additions: u32,
        hunk_deletions: u32,
        git_additions: u32,
        git_deletions: u32,
    },
}

impl From<GitError> for DiffVizError {
    fn from(err: GitError) -> Self {
        match err {
            GitError::Git(git_err) => DiffVizError::Git(format!("{git_err}")),
            GitError::RepositoryNotFound { path, source } => {
                DiffVizError::Repository(format!("Repository not found at '{path}': {source}"))
            }
            GitError::InvalidCommit { hash, source } => {
                DiffVizError::InvalidOperation(format!("Invalid commit '{hash}': {source}"))
            }
            GitError::StagingFailed { file, reason, .. } => {
                DiffVizError::Git(format!("Failed to stage hunk in file '{file}': {reason}"))
            }
            GitError::PatchCreationFailed { file, reason } => DiffVizError::ProcessingFailed(
                format!("Failed to create patch for file '{file}': {reason}"),
            ),
            GitError::Io(io_err) => DiffVizError::Io(io_err),
            GitError::ValidationFailed {
                file,
                hunk_additions,
                hunk_deletions,
                git_additions,
                git_deletions,
            } => DiffVizError::ProcessingFailed(format!(
                "Level 1 validation failed for file '{file}': hunks show +{hunk_additions}/{hunk_deletions} but git stats show +{git_additions}/{git_deletions}"
            )),
        }
    }
}

pub type Result<T> = std::result::Result<T, GitError>;

pub struct GitRepository {
    repo: Repository,
}

impl GitRepository {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        let repo = Repository::open(path).map_err(|source| GitError::RepositoryNotFound {
            path: path_str,
            source,
        })?;
        Ok(Self { repo })
    }

    // /// Computes the diff between two commits in the repository.
    // ///
    // /// This function takes commit references (hashes, branch names, tags, etc.) and returns
    // /// a structured diff showing all file changes, hunks, and individual line modifications.
    // ///
    // /// # Arguments
    // ///
    // /// * `from_commit` - Source commit reference (hash, branch, tag, etc.)
    // /// * `to_commit` - Target commit reference to compare against
    // ///
    // /// # Returns
    // ///
    // /// TODO: BROKEN - References non-existent Diff type. Needs refactoring.
    // /// Returns a Diff containing semantic chunk information for code review analysis.
    // ///
    // /// # Example
    // ///
    // /// This method is currently disabled (see TODO above).
    // TODO: BROKEN - Diff type doesn't exist, needs refactoring
    /*
    pub fn get_diff(&self, from_commit: &str, to_commit: &str) -> Result<Diff> {
        // NOTE: In the new architecture, the git layer only provides raw git data.
        // Semantic chunks are created by the semantic layer using ChunkProcessor + DiffProvider.
        // This method now returns an empty diff that should be populated by the semantic layer.

        Ok(Diff {
            from_commit: from_commit.to_string(),
            to_commit: to_commit.to_string(),
            chunks: Vec::new(), // Empty - will be populated by semantic processing
        })
    }
    */

    /// Resolves commit references to their corresponding git tree objects.
    ///
    /// Takes commit references (which can be hashes, branch names, tags, etc.) and converts
    /// them to git2::Tree objects that can be used for diff comparison. This involves
    /// parsing the references, finding the commit objects, and extracting their trees.
    ///
    /// # Arguments
    ///
    /// * `from_commit` - Source commit reference to resolve
    /// * `to_commit` - Target commit reference to resolve
    ///
    /// # Returns
    ///
    /// Returns a tuple of (from_tree, to_tree) or GitError if commits cannot be found.
    fn resolve_commit_trees(
        &self,
        from_commit: &str,
        to_commit: &str,
    ) -> Result<(git2::Tree, git2::Tree)> {
        let from_obj =
            self.repo
                .revparse_single(from_commit)
                .map_err(|source| GitError::InvalidCommit {
                    hash: from_commit.to_string(),
                    source,
                })?;

        let to_obj =
            self.repo
                .revparse_single(to_commit)
                .map_err(|source| GitError::InvalidCommit {
                    hash: to_commit.to_string(),
                    source,
                })?;

        let from_tree = from_obj
            .as_commit()
            .ok_or_else(|| GitError::InvalidCommit {
                hash: from_commit.to_string(),
                source: git2::Error::from_str("Reference does not point to a commit"),
            })?
            .tree()
            .map_err(|source| GitError::InvalidCommit {
                hash: from_commit.to_string(),
                source,
            })?;

        let to_tree = to_obj
            .as_commit()
            .ok_or_else(|| GitError::InvalidCommit {
                hash: to_commit.to_string(),
                source: git2::Error::from_str("Reference does not point to a commit"),
            })?
            .tree()
            .map_err(|source| GitError::InvalidCommit {
                hash: to_commit.to_string(),
                source,
            })?;

        Ok((from_tree, to_tree))
    }

    /// Creates a git diff between two tree objects with appropriate options.
    ///
    /// Configures git diff options for comprehensive comparison including:
    /// - Context lines for better understanding
    /// - Rename detection for moved files
    /// - Appropriate thresholds for rename/copy detection
    ///
    /// # Arguments
    ///
    /// * `from_tree` - Source tree object
    /// * `to_tree` - Target tree object
    ///
    /// # Returns
    ///
    /// Returns a git2::Diff object ready for processing.
    fn create_git_diff(&self, from_tree: &git2::Tree, to_tree: &git2::Tree) -> Result<git2::Diff> {
        let mut diff_options = DiffOptions::new();
        diff_options.context_lines(3);

        self.repo
            .diff_tree_to_tree(Some(from_tree), Some(to_tree), Some(&mut diff_options))
            .map_err(GitError::Git)
    }

    // TODO: BROKEN - Diff type doesn't exist, needs refactoring
    /*
    /// Get the diff for working directory changes (unstaged changes)
    ///
    /// # Returns
    ///
    /// Returns a Diff struct with empty chunks that should be populated by semantic processing
    pub fn get_working_directory_diff(&self) -> Result<Diff> {
        // NOTE: Like get_diff, this returns empty chunks that should be populated
        // by the semantic layer in the new architecture

        Ok(Diff {
            from_commit: "HEAD".to_string(),
            to_commit: "working-directory".to_string(),
            chunks: Vec::new(), // Empty - will be populated by semantic processing
        })
    }

    /// Get the diff for staged changes (changes in index)
    ///
    /// # Returns
    ///
    /// Returns a Diff struct with empty chunks that should be populated by semantic processing
    pub fn get_staged_diff(&self) -> Result<Diff> {
        // NOTE: Like get_diff, this returns empty chunks that should be populated
        // by the semantic layer in the new architecture

        Ok(Diff {
            from_commit: "HEAD".to_string(),
            to_commit: "index".to_string(),
            chunks: Vec::new(), // Empty - will be populated by semantic processing
        })
    }
    */

    // TODO: BROKEN - RetrievalContext type doesn't exist, needs refactoring
    /*
    pub fn stage_file(&self, file_path: &str, _context: &RetrievalContext) -> Result<()> {
        // Get the index
        let mut index = self.repo.index().map_err(GitError::Git)?;

        // Add the file to the index
        index
            .add_path(Path::new(file_path))
            .map_err(|e| GitError::StagingFailed {
                file: file_path.to_string(),
                reason: e.to_string(),
                source: Some(e),
            })?;

        // Write the index
        index.write().map_err(|e| GitError::StagingFailed {
            file: file_path.to_string(),
            reason: format!("Failed to write index: {e}"),
            source: Some(e),
        })?;

        Ok(())
    }
    */

    /// Gets the HEAD tree for diff comparisons
    fn get_head_tree(&self) -> Result<git2::Tree> {
        let head = self.repo.head().map_err(GitError::Git)?;
        let head_commit = head.peel_to_commit().map_err(GitError::Git)?;
        head_commit.tree().map_err(GitError::Git)
    }

    // /// Retrieves the content of specific lines from a file at a given commit.
    // ///
    // /// This method is used to expand semantic chunks with additional context lines
    // /// during the review process. It leverages git's object storage to efficiently
    // /// retrieve historical file content.
    // ///
    // /// # Arguments
    // ///
    // /// * `hunk_ref` - Reference containing file path, commit info, and line range
    // ///
    // /// # Returns
    // ///
    // /// Returns the requested lines as strings, or GitError if retrieval fails.
    // TODO: BROKEN - RetrievalContext type doesn't exist, needs refactoring
    /*
    pub fn retrieve_lines(
        &self,
        context: &RetrievalContext,
        file_path: &str,
    ) -> Result<Vec<String>> {
        let (commit_hash, start_line, line_count) = match context {
            RetrievalContext::SingleCommit {
                commit,
                start_line,
                line_count,
            } => (commit, *start_line, *line_count),
            RetrievalContext::CommitComparison {
                to_commit,
                new_start,
                new_lines,
                ..
            } => (to_commit, *new_start, *new_lines),
            _ => {
                return Err(GitError::PatchCreationFailed {
                    file: file_path.to_string(),
                    reason: "No git commit context available".to_string(),
                });
            }
        };

        // Resolve the commit
        let commit_obj =
            self.repo
                .revparse_single(commit_hash)
                .map_err(|source| GitError::InvalidCommit {
                    hash: commit_hash.to_string(),
                    source,
                })?;

        let commit = commit_obj
            .as_commit()
            .ok_or_else(|| GitError::InvalidCommit {
                hash: commit_hash.to_string(),
                source: git2::Error::from_str("Object is not a commit"),
            })?;

        let tree = commit.tree().map_err(|source| GitError::InvalidCommit {
            hash: commit_hash.to_string(),
            source,
        })?;

        // Find the file in the tree
        let tree_entry =
            tree.get_path(Path::new(file_path))
                .map_err(|e| GitError::PatchCreationFailed {
                    file: file_path.to_string(),
                    reason: format!("File not found in commit {commit_hash}: {e}"),
                })?;

        let blob = self
            .repo
            .find_blob(tree_entry.id())
            .map_err(GitError::Git)?;

        let content =
            std::str::from_utf8(blob.content()).map_err(|_| GitError::PatchCreationFailed {
                file: file_path.to_string(),
                reason: "File contains invalid UTF-8".to_string(),
            })?;

        // Extract the requested line range
        let lines: Vec<&str> = content.lines().collect();
        let start_index = (start_line.saturating_sub(1)) as usize;
        let end_index = (start_index + line_count as usize).min(lines.len());

        let selected_lines = lines[start_index..end_index]
            .iter()
            .map(|&line| line.to_string())
            .collect();

        Ok(selected_lines)
    }
    */
    // Helper methods for DiffProvider implementation

    /// Get file content at a specific commit
    /// Returns None if file doesn't exist at that commit
    fn get_file_content_at_commit(
        &self,
        file_path: &str,
        commit_ref: &str,
    ) -> Result<Option<String>> {
        // Handle special cases
        if commit_ref == "working-directory" {
            // Read from working directory
            let full_path = self
                .repo
                .workdir()
                .ok_or_else(|| GitError::PatchCreationFailed {
                    file: file_path.to_string(),
                    reason: "Repository has no working directory".to_string(),
                })?
                .join(file_path);

            return match std::fs::read_to_string(&full_path) {
                Ok(content) => Ok(Some(content)),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
                Err(e) => Err(GitError::Io(e)),
            };
        }

        if commit_ref == "index" {
            // Read from git index (staged changes)
            let index = self.repo.index().map_err(GitError::Git)?;
            return match index.get_path(Path::new(file_path), 0) {
                Some(entry) => {
                    let blob = self.repo.find_blob(entry.id).map_err(GitError::Git)?;
                    match std::str::from_utf8(blob.content()) {
                        Ok(content) => Ok(Some(content.to_string())),
                        Err(_) => Err(GitError::PatchCreationFailed {
                            file: file_path.to_string(),
                            reason: "File contains invalid UTF-8".to_string(),
                        }),
                    }
                }
                None => Ok(None),
            };
        }

        // Regular commit reference
        let commit_obj = match self.repo.revparse_single(commit_ref) {
            Ok(obj) => obj,
            Err(_) => return Ok(None), // Commit doesn't exist
        };

        let commit = match commit_obj.as_commit() {
            Some(c) => c,
            None => return Ok(None), // Not a commit
        };

        let tree = commit.tree().map_err(GitError::Git)?;

        match tree.get_path(Path::new(file_path)) {
            Ok(tree_entry) => {
                let blob = self
                    .repo
                    .find_blob(tree_entry.id())
                    .map_err(GitError::Git)?;
                match std::str::from_utf8(blob.content()) {
                    Ok(content) => Ok(Some(content.to_string())),
                    Err(_) => Err(GitError::PatchCreationFailed {
                        file: file_path.to_string(),
                        reason: "File contains invalid UTF-8".to_string(),
                    }),
                }
            }
            Err(_) => Ok(None), // File doesn't exist at this commit
        }
    }

    /// Extract git diff statistics for a specific file
    /// This provides authoritative statistics directly from git
    fn get_file_stats_for_commits(
        &self,
        file_path: &str,
        from_commit: &str,
        to_commit: &str,
    ) -> Result<FileStats> {
        if from_commit == "HEAD" && to_commit == "working-directory" {
            return self.get_working_directory_stats_for_file(file_path);
        }

        // Since hunks no longer contain line data, we need to extract stats directly from git
        self.get_file_stats_from_git_diff(file_path, from_commit, to_commit)
    }

    fn get_working_directory_stats_for_file(&self, file_path: &str) -> Result<FileStats> {
        // Since hunks no longer contain line data, extract stats directly from git
        self.get_working_directory_stats_from_git_diff(file_path)
    }

    /// Extract git diff statistics directly from git diff
    fn get_file_stats_from_git_diff(
        &self,
        file_path: &str,
        from_commit: &str,
        to_commit: &str,
    ) -> Result<FileStats> {
        let (from_tree, to_tree) = self.resolve_commit_trees(from_commit, to_commit)?;
        let git_diff = self.create_git_diff(&from_tree, &to_tree)?;

        let mut additions = 0;
        let mut deletions = 0;

        // Iterate through diff to count additions/deletions for the target file
        for (delta_idx, delta) in git_diff.deltas().enumerate() {
            let delta_file_path = delta
                .new_file()
                .path()
                .or_else(|| delta.old_file().path())
                .and_then(|p| p.to_str())
                .unwrap_or("");

            if delta_file_path != file_path {
                continue;
            }

            if let Ok(Some(patch)) = git2::Patch::from_diff(&git_diff, delta_idx) {
                // Count additions and deletions in this patch
                for hunk_idx in 0..patch.num_hunks() {
                    let num_lines = patch.num_lines_in_hunk(hunk_idx).unwrap_or(0);
                    for line_idx in 0..num_lines {
                        if let Ok(diff_line) = patch.line_in_hunk(hunk_idx, line_idx) {
                            match diff_line.origin() {
                                '+' => additions += 1,
                                '-' => deletions += 1,
                                _ => {} // Skip context lines
                            }
                        }
                    }
                }
            }
        }

        Ok(FileStats::new(additions, deletions))
    }

    /// Extract working directory stats directly from git diff
    fn get_working_directory_stats_from_git_diff(&self, file_path: &str) -> Result<FileStats> {
        let head_tree = self.get_head_tree()?;
        let mut diff_opts = git2::DiffOptions::new();
        diff_opts.include_untracked(true);

        let git_diff = self
            .repo
            .diff_tree_to_workdir_with_index(Some(&head_tree), Some(&mut diff_opts))
            .map_err(GitError::Git)?;

        let mut additions = 0;
        let mut deletions = 0;

        // Iterate through diff to count additions/deletions for the target file
        for (delta_idx, delta) in git_diff.deltas().enumerate() {
            let delta_file_path = delta
                .new_file()
                .path()
                .or_else(|| delta.old_file().path())
                .and_then(|p| p.to_str())
                .unwrap_or("");

            if delta_file_path != file_path {
                continue;
            }

            if let Ok(Some(patch)) = git2::Patch::from_diff(&git_diff, delta_idx) {
                // Count additions and deletions in this patch
                for hunk_idx in 0..patch.num_hunks() {
                    let num_lines = patch.num_lines_in_hunk(hunk_idx).unwrap_or(0);
                    for line_idx in 0..num_lines {
                        if let Ok(diff_line) = patch.line_in_hunk(hunk_idx, line_idx) {
                            match diff_line.origin() {
                                '+' => additions += 1,
                                '-' => deletions += 1,
                                _ => {} // Skip context lines
                            }
                        }
                    }
                }
            }
        }

        Ok(FileStats::new(additions, deletions))
    }

    fn get_diff_files(
        &self,
        from_commit: &str,
        to_commit: &str,
    ) -> std::result::Result<Vec<(String, FileStatus)>, Box<dyn std::error::Error>> {
        // Simplified: just extract file paths and statuses directly from git diff
        let (from_tree, to_tree) = self.resolve_commit_trees(from_commit, to_commit)?;
        let git_diff = self.create_git_diff(&from_tree, &to_tree)?;

        let mut files = Vec::new();
        git_diff.foreach(
            &mut |delta, _progress| {
                let status = match delta.status() {
                    git2::Delta::Added => FileStatus::Added,
                    git2::Delta::Deleted => FileStatus::Deleted,
                    git2::Delta::Modified => FileStatus::Modified,
                    git2::Delta::Renamed => FileStatus::Renamed,
                    git2::Delta::Copied => FileStatus::Copied,
                    git2::Delta::Untracked => FileStatus::Untracked,
                    _ => FileStatus::Modified,
                };

                let path = if let Some(new_file) = delta.new_file().path() {
                    new_file.to_string_lossy().to_string()
                } else if let Some(old_file) = delta.old_file().path() {
                    old_file.to_string_lossy().to_string()
                } else {
                    "<unknown>".to_string()
                };

                files.push((path, status));
                true
            },
            None,
            None,
            None,
        )?;

        Ok(files)
    }

    fn get_working_directory_files(
        &self,
    ) -> std::result::Result<Vec<(String, FileStatus)>, Box<dyn std::error::Error>> {
        // Simplified: extract file information directly from git diff
        let head_tree = self.get_head_tree()?;
        let mut diff_opts = git2::DiffOptions::new();
        diff_opts.include_untracked(true);

        let git_diff = self
            .repo
            .diff_tree_to_workdir_with_index(Some(&head_tree), Some(&mut diff_opts))
            .map_err(GitError::Git)?;

        let mut files = Vec::new();
        git_diff.foreach(
            &mut |delta, _progress| {
                let status = match delta.status() {
                    git2::Delta::Added => FileStatus::Added,
                    git2::Delta::Deleted => FileStatus::Deleted,
                    git2::Delta::Modified => FileStatus::Modified,
                    git2::Delta::Renamed => FileStatus::Renamed,
                    git2::Delta::Copied => FileStatus::Copied,
                    git2::Delta::Untracked => FileStatus::Untracked,
                    _ => FileStatus::Modified,
                };

                let path = if let Some(new_file) = delta.new_file().path() {
                    new_file.to_string_lossy().to_string()
                } else if let Some(old_file) = delta.old_file().path() {
                    old_file.to_string_lossy().to_string()
                } else {
                    "<unknown>".to_string()
                };

                files.push((path, status));
                true
            },
            None,
            None,
            None,
        )?;

        Ok(files)
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
        match (&query.from, &query.to) {
            (GitRef::Head, GitRef::Unstaged) => self.get_working_directory_files(),
            (GitRef::Staged, GitRef::Unstaged) => self.get_working_directory_files(), // TODO: Implement staged vs unstaged
            _ => self.get_diff_files(&from_ref, &to_ref),
        }
    }

    fn get_file_stats(
        &self,
        file_path: &str,
        query: &DiffQuery,
    ) -> std::result::Result<FileStats, Box<dyn std::error::Error>> {
        let from_ref = git_ref_to_string(&query.from)?;
        let to_ref = git_ref_to_string(&query.to)?;

        // Handle special cases for working directory comparisons
        match (&query.from, &query.to) {
            (GitRef::Head, GitRef::Unstaged) => {
                Ok(self.get_working_directory_stats_for_file(file_path)?)
            }
            (GitRef::Staged, GitRef::Unstaged) => {
                // TODO: Implement proper staged vs unstaged stats
                Ok(self.get_working_directory_stats_for_file(file_path)?)
            }
            _ => Ok(self.get_file_stats_for_commits(file_path, &from_ref, &to_ref)?),
        }
    }

    fn get_source_code(
        &self,
        file_path: &str,
        git_ref: &GitRef,
    ) -> std::result::Result<String, Box<dyn std::error::Error>> {
        let ref_string = git_ref_to_string(git_ref)?;

        match git_ref {
            GitRef::Unstaged => {
                // Read file from working directory
                let full_path = self
                    .repo
                    .workdir()
                    .ok_or_else(|| {
                        Box::new(GitError::Git(git2::Error::from_str(
                            "Repository has no working directory",
                        )))
                    })?
                    .join(file_path);
                Ok(std::fs::read_to_string(full_path)?)
            }
            GitRef::Staged => {
                // TODO: Implement reading from index
                let content = self
                    .get_file_content_at_commit(file_path, "HEAD")?
                    .ok_or_else(|| {
                        Box::new(GitError::Git(git2::Error::from_str("File not found")))
                    })?;
                Ok(content)
            }
            _ => {
                let content = self
                    .get_file_content_at_commit(file_path, &ref_string)?
                    .ok_or_else(|| {
                        Box::new(GitError::Git(git2::Error::from_str("File not found")))
                    })?;
                Ok(content)
            }
        }
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
