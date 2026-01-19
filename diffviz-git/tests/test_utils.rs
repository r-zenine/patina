//! Testing utilities for Git repository operations

use git2::{Repository, Signature, Time};
use std::path::Path;
use tempfile::TempDir;

/// A temporary Git repository for testing purposes
pub struct TestRepo {
    temp_dir: TempDir,
    pub repo: Repository,
}

impl Default for TestRepo {
    fn default() -> Self {
        Self::new()
    }
}

impl TestRepo {
    /// Creates a new temporary Git repository with proper configuration
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let repo = Repository::init(temp_dir.path()).expect("Failed to init git repo");

        // Configure repo
        let mut config = repo.config().expect("Failed to get config");
        config
            .set_str("user.name", "Test User")
            .expect("Failed to set user.name");
        config
            .set_str("user.email", "test@example.com")
            .expect("Failed to set user.email");

        Self { temp_dir, repo }
    }

    /// Gets the path to the temporary repository
    pub fn path(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Commits a file with the given path and content to the repository
    /// Returns the commit OID
    pub fn commit_file(&mut self, path: &str, content: &str) -> git2::Oid {
        let file_path = self.temp_dir.path().join(path);

        // Create parent directories if needed
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).expect("Failed to create parent directories");
        }

        // Write file content
        std::fs::write(&file_path, content).expect("Failed to write file");

        // Add to index
        let mut index = self.repo.index().expect("Failed to get index");
        index
            .add_path(Path::new(path))
            .expect("Failed to add path to index");
        index.write().expect("Failed to write index");

        // Create commit
        let tree_oid = index.write_tree().expect("Failed to write tree");
        let tree = self.repo.find_tree(tree_oid).expect("Failed to find tree");

        let signature = Signature::new("Test User", "test@example.com", &Time::new(1234567890, 0))
            .expect("Failed to create signature");

        let commit_message = format!("Add {path}");

        if let Ok(head) = self.repo.head() {
            let parent_commit = head.peel_to_commit().expect("Failed to peel to commit");
            self.repo
                .commit(
                    Some("HEAD"),
                    &signature,
                    &signature,
                    &commit_message,
                    &tree,
                    &[&parent_commit],
                )
                .expect("Failed to create commit")
        } else {
            self.repo
                .commit(
                    Some("HEAD"),
                    &signature,
                    &signature,
                    &commit_message,
                    &tree,
                    &[],
                )
                .expect("Failed to create commit")
        }
    }

    /// Deletes a file from the repository and commits the change
    /// Returns the commit OID
    pub fn delete_file(&mut self, path: &str) -> git2::Oid {
        // Remove file from filesystem
        std::fs::remove_file(self.temp_dir.path().join(path)).expect("Failed to remove file");

        // Remove from index
        let mut index = self.repo.index().expect("Failed to get index");
        index
            .remove_path(Path::new(path))
            .expect("Failed to remove path from index");
        index.write().expect("Failed to write index");

        // Create commit
        let tree_oid = index.write_tree().expect("Failed to write tree");
        let tree = self.repo.find_tree(tree_oid).expect("Failed to find tree");
        let signature = Signature::new("Test User", "test@example.com", &Time::new(1234567890, 0))
            .expect("Failed to create signature");
        let parent = self.repo.head().unwrap().peel_to_commit().unwrap();

        self.repo
            .commit(
                Some("HEAD"),
                &signature,
                &signature,
                &format!("Delete {path}"),
                &tree,
                &[&parent],
            )
            .expect("Failed to create commit")
    }

    /// Gets the commit hash for a given revision reference
    pub fn get_commit_hash(&self, rev: &str) -> String {
        let obj = self
            .repo
            .revparse_single(rev)
            .expect("Failed to parse revision");
        obj.id().to_string()
    }
}
