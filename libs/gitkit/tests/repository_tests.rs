use gitkit::test_utils::TestRepo;
use gitkit::{Error, GitRepository, RawFileStatus};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_repository_not_found() {
    let result = GitRepository::open("/definitely/does/not/exist");

    match result {
        Err(Error::RepositoryNotFound { path, .. }) => {
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
        Err(Error::RepositoryNotFound { .. }) => (),
        _ => panic!("Expected RepositoryNotFound error when opening a regular file"),
    }
}

#[test]
fn test_repository_open_valid_repo() {
    let test_repo = TestRepo::new();
    let repo = GitRepository::open(test_repo.path());
    assert!(repo.is_ok());
}

#[test]
fn test_get_file_content_at_commit() {
    let mut test_repo = TestRepo::new();
    let commit_oid = test_repo.commit_file("hello.txt", "hello world\n");
    let repo = GitRepository::open(test_repo.path()).unwrap();

    let content = repo
        .get_file_content_at_commit("hello.txt", &commit_oid.to_string())
        .unwrap();
    assert_eq!(content, Some("hello world\n".to_string()));
}

#[test]
fn test_get_file_content_at_commit_not_found() {
    let mut test_repo = TestRepo::new();
    let commit_oid = test_repo.commit_file("hello.txt", "hello world\n");
    let repo = GitRepository::open(test_repo.path()).unwrap();

    let content = repo
        .get_file_content_at_commit("missing.txt", &commit_oid.to_string())
        .unwrap();
    assert_eq!(content, None);
}

#[test]
fn test_get_file_content_working_directory() {
    let mut test_repo = TestRepo::new();
    test_repo.commit_file("hello.txt", "committed content\n");
    fs::write(test_repo.path().join("hello.txt"), "uncommitted content\n").unwrap();
    let repo = GitRepository::open(test_repo.path()).unwrap();

    let content = repo
        .get_file_content_at_commit("hello.txt", "working-directory")
        .unwrap();
    assert_eq!(content, Some("uncommitted content\n".to_string()));
}

#[test]
fn test_get_file_content_working_directory_missing() {
    let mut test_repo = TestRepo::new();
    test_repo.commit_file("hello.txt", "committed content\n");
    let repo = GitRepository::open(test_repo.path()).unwrap();

    let content = repo
        .get_file_content_at_commit("missing.txt", "working-directory")
        .unwrap();
    assert_eq!(content, None);
}

#[test]
fn test_get_file_content_index() {
    let mut test_repo = TestRepo::new();
    test_repo.commit_file("hello.txt", "committed content\n");
    let repo = GitRepository::open(test_repo.path()).unwrap();

    let content = repo
        .get_file_content_at_commit("hello.txt", "index")
        .unwrap();
    assert_eq!(content, Some("committed content\n".to_string()));
}

#[test]
fn test_get_diff_files_status_mapping() {
    let mut test_repo = TestRepo::new();
    let first = test_repo.commit_file("a.txt", "one\n");
    test_repo.commit_file("b.txt", "two\n");
    let second = test_repo.get_commit_hash("HEAD");
    let repo = GitRepository::open(test_repo.path()).unwrap();

    let files = repo.get_diff_files(&first.to_string(), &second).unwrap();

    assert_eq!(files.len(), 1);
    assert_eq!(files[0].0, "b.txt");
    assert!(matches!(files[0].1, RawFileStatus::Added));
}

#[test]
fn test_get_diff_files_deletion_status() {
    let mut test_repo = TestRepo::new();
    let first = test_repo.commit_file("a.txt", "one\n");
    test_repo.delete_file("a.txt");
    let second = test_repo.get_commit_hash("HEAD");
    let repo = GitRepository::open(test_repo.path()).unwrap();

    let files = repo.get_diff_files(&first.to_string(), &second).unwrap();

    assert_eq!(files.len(), 1);
    assert_eq!(files[0].0, "a.txt");
    assert!(matches!(files[0].1, RawFileStatus::Deleted));
}

#[test]
fn test_get_working_directory_files() {
    let mut test_repo = TestRepo::new();
    test_repo.commit_file("a.txt", "one\n");
    fs::write(test_repo.path().join("new.txt"), "new file\n").unwrap();
    let repo = GitRepository::open(test_repo.path()).unwrap();

    let files = repo.get_working_directory_files().unwrap();
    assert!(files.iter().any(|(path, _)| path == "new.txt"));
}

#[test]
fn test_get_file_stats_for_commits() {
    let mut test_repo = TestRepo::new();
    let first = test_repo.commit_file("a.txt", "line1\n");
    test_repo.commit_file("a.txt", "line1\nline2\nline3\n");
    let second = test_repo.get_commit_hash("HEAD");
    let repo = GitRepository::open(test_repo.path()).unwrap();

    let stats = repo
        .get_file_stats_for_commits("a.txt", &first.to_string(), &second)
        .unwrap();

    assert_eq!(stats.additions, 2);
    assert_eq!(stats.deletions, 0);
}

#[test]
fn test_resolve_parent_commit() {
    let mut test_repo = TestRepo::new();
    let first = test_repo.commit_file("a.txt", "one\n");
    test_repo.commit_file("a.txt", "two\n");
    let second = test_repo.get_commit_hash("HEAD");
    let repo = GitRepository::open(test_repo.path()).unwrap();

    let parent = repo.resolve_parent_commit(&second).unwrap();
    assert_eq!(parent, first.to_string());
}

#[test]
fn test_resolve_parent_commit_no_parent() {
    let mut test_repo = TestRepo::new();
    let first = test_repo.commit_file("a.txt", "one\n");
    let repo = GitRepository::open(test_repo.path()).unwrap();

    let result = repo.resolve_parent_commit(&first.to_string());
    assert!(matches!(result, Err(Error::InvalidCommit { .. })));
}
