use diffviz_git::{GitError, GitRepository};
use diffviz_review::errors::DiffVizError;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_repository_not_found() {
    let result = GitRepository::open("/definitely/does/not/exist");

    match result {
        Err(GitError::Core(gitkit::Error::RepositoryNotFound { path, .. })) => {
            assert!(path.contains("/definitely/does/not/exist"));
        }
        _ => panic!("Expected RepositoryNotFound error"),
    }
}

#[test]
fn test_repository_open_regular_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("not_a_repo.txt");
    fs::write(&file_path, "just a file").unwrap();

    let result = GitRepository::open(&file_path);

    match result {
        Err(GitError::Core(gitkit::Error::RepositoryNotFound { .. })) => (),
        _ => panic!("Expected RepositoryNotFound error when opening a regular file"),
    }
}

#[test]
fn test_git_error_conversion_to_diffviz_error() {
    // Create a mock git2::Error for testing
    let git2_error = git2::Error::from_str("Some git operation failed");
    let git_error = GitError::Core(gitkit::Error::Git(git2_error));
    let diffviz_error: DiffVizError = git_error.into();

    match diffviz_error {
        DiffVizError::Git(msg) => {
            // Just ensure we got a Git error with some message
            assert!(!msg.is_empty(), "Error message should not be empty");
        }
        _ => panic!("Expected Git error"),
    }
}

#[test]
fn test_invalid_commit_conversion_to_diffviz_error() {
    let git2_error = git2::Error::from_str("revision not found");
    let git_error = GitError::Core(gitkit::Error::InvalidCommit {
        hash: "bad_hash".to_string(),
        source: git2_error,
    });
    let diffviz_error: DiffVizError = git_error.into();

    match diffviz_error {
        DiffVizError::InvalidOperation(msg) => {
            assert!(msg.contains("Invalid commit 'bad_hash'"));
        }
        _ => panic!("Expected InvalidOperation error"),
    }
}

#[test]
fn test_repository_not_found_conversion_to_diffviz_error() {
    let git2_error = git2::Error::from_str("repository not found");
    let git_error = GitError::Core(gitkit::Error::RepositoryNotFound {
        path: "/path/not/found".to_string(),
        source: git2_error,
    });
    let diffviz_error: DiffVizError = git_error.into();

    match diffviz_error {
        DiffVizError::Repository(msg) => {
            assert!(msg.contains("/path/not/found"));
        }
        _ => panic!("Expected Repository error"),
    }
}

#[test]
fn test_staging_error_with_detailed_message() {
    // Test that staging errors include detailed context
    let staging_error = GitError::Core(gitkit::Error::StagingFailed {
        file: "test_file.rs".to_string(),
        reason: "Failed to apply patch to index".to_string(),
        source: None,
    });

    let error_msg = format!("{staging_error}");
    assert!(error_msg.contains("test_file.rs"));
    assert!(error_msg.contains("Failed to apply patch to index"));

    // Test conversion to DiffVizError
    let diffviz_error: DiffVizError = staging_error.into();
    let diffviz_msg = format!("{diffviz_error}");
    assert!(diffviz_msg.contains("test_file.rs"));
    assert!(diffviz_msg.contains("Failed to apply patch to index"));
}

#[test]
fn test_patch_creation_error_with_detailed_message() {
    let patch_error = GitError::Core(gitkit::Error::PatchCreationFailed {
        file: "broken_file.rs".to_string(),
        reason: "Invalid hunk header: 'malformed'".to_string(),
    });

    let error_msg = format!("{patch_error}");
    assert!(error_msg.contains("broken_file.rs"));
    assert!(error_msg.contains("Invalid hunk header"));

    // Test conversion to DiffVizError
    let diffviz_error: DiffVizError = patch_error.into();
    let diffviz_msg = format!("{diffviz_error}");
    assert!(diffviz_msg.contains("broken_file.rs"));
    assert!(diffviz_msg.contains("Invalid hunk header"));
}
