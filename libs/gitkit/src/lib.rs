//! Pure git infrastructure: open a repo, resolve commits, read file content at a
//! ref, list changed files, compute add/delete stats. No dependency on any
//! review/diff-orchestration domain layer.

use git2::{DiffOptions, Repository};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
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

pub type Result<T> = std::result::Result<T, Error>;

/// Review-agnostic mirror of diffviz-review's `FileStats`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RawFileStats {
    pub additions: u32,
    pub deletions: u32,
    pub total_changes: u32,
}

impl RawFileStats {
    pub fn new(additions: u32, deletions: u32) -> Self {
        Self {
            additions,
            deletions,
            total_changes: additions + deletions,
        }
    }

    pub fn is_creation(&self) -> bool {
        self.additions > 0 && self.deletions == 0
    }

    pub fn is_deletion(&self) -> bool {
        self.additions == 0 && self.deletions > 0
    }

    pub fn is_modification(&self) -> bool {
        self.additions > 0 && self.deletions > 0
    }

    pub fn is_unchanged(&self) -> bool {
        self.additions == 0 && self.deletions == 0
    }
}

/// Review-agnostic mirror of diffviz-review's `FileStatus`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RawFileStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
    Untracked,
}

pub struct GitRepository {
    repo: Repository,
}

impl GitRepository {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        let repo = Repository::open(path).map_err(|source| Error::RepositoryNotFound {
            path: path_str,
            source,
        })?;
        Ok(Self { repo })
    }

    fn resolve_commit_trees(
        &self,
        from_commit: &str,
        to_commit: &str,
    ) -> Result<(git2::Tree<'_>, git2::Tree<'_>)> {
        let from_obj =
            self.repo
                .revparse_single(from_commit)
                .map_err(|source| Error::InvalidCommit {
                    hash: from_commit.to_string(),
                    source,
                })?;

        let to_obj =
            self.repo
                .revparse_single(to_commit)
                .map_err(|source| Error::InvalidCommit {
                    hash: to_commit.to_string(),
                    source,
                })?;

        let from_tree = from_obj
            .as_commit()
            .ok_or_else(|| Error::InvalidCommit {
                hash: from_commit.to_string(),
                source: git2::Error::from_str("Reference does not point to a commit"),
            })?
            .tree()
            .map_err(|source| Error::InvalidCommit {
                hash: from_commit.to_string(),
                source,
            })?;

        let to_tree = to_obj
            .as_commit()
            .ok_or_else(|| Error::InvalidCommit {
                hash: to_commit.to_string(),
                source: git2::Error::from_str("Reference does not point to a commit"),
            })?
            .tree()
            .map_err(|source| Error::InvalidCommit {
                hash: to_commit.to_string(),
                source,
            })?;

        Ok((from_tree, to_tree))
    }

    fn create_git_diff(
        &self,
        from_tree: &git2::Tree,
        to_tree: &git2::Tree,
    ) -> Result<git2::Diff<'_>> {
        let mut diff_options = DiffOptions::new();
        diff_options.context_lines(3);

        self.repo
            .diff_tree_to_tree(Some(from_tree), Some(to_tree), Some(&mut diff_options))
            .map_err(Error::Git)
    }

    fn get_head_tree(&self) -> Result<git2::Tree<'_>> {
        let head = self.repo.head().map_err(Error::Git)?;
        let head_commit = head.peel_to_commit().map_err(Error::Git)?;
        head_commit.tree().map_err(Error::Git)
    }

    /// Get file content at a specific commit/ref. Returns `Ok(None)` if the
    /// file doesn't exist there. `commit_ref` may be `"working-directory"`,
    /// `"index"`, or a regular commit reference.
    pub fn get_file_content_at_commit(
        &self,
        file_path: &str,
        commit_ref: &str,
    ) -> Result<Option<String>> {
        if commit_ref == "working-directory" {
            let full_path = self
                .repo
                .workdir()
                .ok_or_else(|| Error::PatchCreationFailed {
                    file: file_path.to_string(),
                    reason: "Repository has no working directory".to_string(),
                })?
                .join(file_path);

            return match std::fs::read_to_string(&full_path) {
                Ok(content) => Ok(Some(content)),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
                Err(e) => Err(Error::Io(e)),
            };
        }

        if commit_ref == "index" {
            let index = self.repo.index().map_err(Error::Git)?;
            return match index.get_path(Path::new(file_path), 0) {
                Some(entry) => {
                    let blob = self.repo.find_blob(entry.id).map_err(Error::Git)?;
                    match std::str::from_utf8(blob.content()) {
                        Ok(content) => Ok(Some(content.to_string())),
                        Err(_) => Err(Error::PatchCreationFailed {
                            file: file_path.to_string(),
                            reason: "File contains invalid UTF-8".to_string(),
                        }),
                    }
                }
                None => Ok(None),
            };
        }

        let commit_obj = match self.repo.revparse_single(commit_ref) {
            Ok(obj) => obj,
            Err(_) => return Ok(None),
        };

        let commit = match commit_obj.as_commit() {
            Some(c) => c,
            None => return Ok(None),
        };

        let tree = commit.tree().map_err(Error::Git)?;

        match tree.get_path(Path::new(file_path)) {
            Ok(tree_entry) => {
                let blob = self.repo.find_blob(tree_entry.id()).map_err(Error::Git)?;
                match std::str::from_utf8(blob.content()) {
                    Ok(content) => Ok(Some(content.to_string())),
                    Err(_) => Err(Error::PatchCreationFailed {
                        file: file_path.to_string(),
                        reason: "File contains invalid UTF-8".to_string(),
                    }),
                }
            }
            Err(_) => Ok(None),
        }
    }

    pub fn get_file_stats_for_commits(
        &self,
        file_path: &str,
        from_commit: &str,
        to_commit: &str,
    ) -> Result<RawFileStats> {
        if from_commit == "HEAD" && to_commit == "working-directory" {
            return self.get_working_directory_stats_for_file(file_path);
        }

        self.get_file_stats_from_git_diff(file_path, from_commit, to_commit)
    }

    pub fn get_working_directory_stats_for_file(&self, file_path: &str) -> Result<RawFileStats> {
        self.get_working_directory_stats_from_git_diff(file_path)
    }

    pub fn get_file_stats_from_git_diff(
        &self,
        file_path: &str,
        from_commit: &str,
        to_commit: &str,
    ) -> Result<RawFileStats> {
        let (from_tree, to_tree) = self.resolve_commit_trees(from_commit, to_commit)?;
        let git_diff = self.create_git_diff(&from_tree, &to_tree)?;

        let (additions, deletions) = count_additions_deletions(&git_diff, file_path);

        Ok(RawFileStats::new(additions, deletions))
    }

    pub fn get_working_directory_stats_from_git_diff(
        &self,
        file_path: &str,
    ) -> Result<RawFileStats> {
        let head_tree = self.get_head_tree()?;
        let mut diff_opts = git2::DiffOptions::new();
        diff_opts.include_untracked(true);

        let git_diff = self
            .repo
            .diff_tree_to_workdir_with_index(Some(&head_tree), Some(&mut diff_opts))
            .map_err(Error::Git)?;

        let (additions, deletions) = count_additions_deletions(&git_diff, file_path);

        Ok(RawFileStats::new(additions, deletions))
    }

    pub fn get_diff_files(
        &self,
        from_commit: &str,
        to_commit: &str,
    ) -> Result<Vec<(String, RawFileStatus)>> {
        let (from_tree, to_tree) = self.resolve_commit_trees(from_commit, to_commit)?;
        let git_diff = self.create_git_diff(&from_tree, &to_tree)?;

        collect_diff_files(&git_diff)
    }

    pub fn get_working_directory_files(&self) -> Result<Vec<(String, RawFileStatus)>> {
        let head_tree = self.get_head_tree()?;
        let mut diff_opts = git2::DiffOptions::new();
        diff_opts.include_untracked(true);

        let git_diff = self
            .repo
            .diff_tree_to_workdir_with_index(Some(&head_tree), Some(&mut diff_opts))
            .map_err(Error::Git)?;

        collect_diff_files(&git_diff)
    }

    /// Resolve a commit hash to its parent commit hash. Used for building
    /// diff queries that show the exact commit's changes (commit^..commit).
    pub fn resolve_parent_commit(&self, commit_hash: &str) -> Result<String> {
        let commit_obj =
            self.repo
                .revparse_single(commit_hash)
                .map_err(|source| Error::InvalidCommit {
                    hash: commit_hash.to_string(),
                    source,
                })?;

        let commit = commit_obj.as_commit().ok_or_else(|| Error::InvalidCommit {
            hash: commit_hash.to_string(),
            source: git2::Error::new(
                git2::ErrorCode::InvalidSpec,
                git2::ErrorClass::Reference,
                "Not a commit object",
            ),
        })?;

        let parent = commit.parent(0).map_err(|source| Error::InvalidCommit {
            hash: commit_hash.to_string(),
            source,
        })?;

        Ok(parent.id().to_string())
    }
}

fn count_additions_deletions(git_diff: &git2::Diff, file_path: &str) -> (u32, u32) {
    let mut additions = 0;
    let mut deletions = 0;

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

        if let Ok(Some(patch)) = git2::Patch::from_diff(git_diff, delta_idx) {
            for hunk_idx in 0..patch.num_hunks() {
                let num_lines = patch.num_lines_in_hunk(hunk_idx).unwrap_or(0);
                for line_idx in 0..num_lines {
                    if let Ok(diff_line) = patch.line_in_hunk(hunk_idx, line_idx) {
                        match diff_line.origin() {
                            '+' => additions += 1,
                            '-' => deletions += 1,
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    (additions, deletions)
}

fn collect_diff_files(git_diff: &git2::Diff) -> Result<Vec<(String, RawFileStatus)>> {
    let mut files = Vec::new();
    git_diff
        .foreach(
            &mut |delta, _progress| {
                let status = match delta.status() {
                    git2::Delta::Added => RawFileStatus::Added,
                    git2::Delta::Deleted => RawFileStatus::Deleted,
                    git2::Delta::Modified => RawFileStatus::Modified,
                    git2::Delta::Renamed => RawFileStatus::Renamed,
                    git2::Delta::Copied => RawFileStatus::Copied,
                    git2::Delta::Untracked => RawFileStatus::Untracked,
                    _ => RawFileStatus::Modified,
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
        )
        .map_err(Error::Git)?;

    Ok(files)
}
