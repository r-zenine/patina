use super::MockDiffProvider;
use crate::engines::review_engine::test_helpers::test_range;
use crate::entities::git_ref::GitRef;
use crate::providers::DiffProvider;

// ===== Hash Calculation Tests =====

#[test]
fn test_calculate_file_hash_known_content() {
    use sha2::{Digest, Sha256};

    let mut mock_provider = MockDiffProvider::new();
    mock_provider.add_file_content("test.rs", &GitRef::head(), "hello world\n");

    let hash = mock_provider
        .get_file_hash("test.rs", &GitRef::head())
        .unwrap();

    let mut hasher = Sha256::new();
    hasher.update(b"hello world\n");
    let expected = format!("{:x}", hasher.finalize());

    assert_eq!(hash, expected);
}

#[test]
fn test_calculate_file_hash_identical_content_identical_hash() {
    let mut mock_provider = MockDiffProvider::new();
    mock_provider.add_file_content("file1.rs", &GitRef::head(), "same content\n");
    mock_provider.add_file_content("file2.rs", &GitRef::head(), "same content\n");

    let hash1 = mock_provider
        .get_file_hash("file1.rs", &GitRef::head())
        .unwrap();
    let hash2 = mock_provider
        .get_file_hash("file2.rs", &GitRef::head())
        .unwrap();

    assert_eq!(hash1, hash2);
}

#[test]
fn test_calculate_file_hash_different_content_different_hash() {
    let mut mock_provider = MockDiffProvider::new();
    mock_provider.add_file_content("file1.rs", &GitRef::head(), "content A\n");
    mock_provider.add_file_content("file2.rs", &GitRef::head(), "content B\n");

    let hash1 = mock_provider
        .get_file_hash("file1.rs", &GitRef::head())
        .unwrap();
    let hash2 = mock_provider
        .get_file_hash("file2.rs", &GitRef::head())
        .unwrap();

    assert_ne!(hash1, hash2);
}

#[test]
fn test_calculate_file_hash_crlf_normalization() {
    let mut mock_provider = MockDiffProvider::new();
    mock_provider.add_file_content("crlf.rs", &GitRef::head(), "line1\r\nline2\r\n");
    mock_provider.add_file_content("lf.rs", &GitRef::head(), "line1\nline2\n");

    let hash_crlf = mock_provider
        .get_file_hash("crlf.rs", &GitRef::head())
        .unwrap();
    let hash_lf = mock_provider
        .get_file_hash("lf.rs", &GitRef::head())
        .unwrap();

    // Should be identical after normalization
    assert_eq!(hash_crlf, hash_lf);
}

#[test]
fn test_calculate_file_hash_lf_unchanged() {
    use sha2::{Digest, Sha256};

    let mut mock_provider = MockDiffProvider::new();
    let content = "line1\nline2\n";
    mock_provider.add_file_content("test.rs", &GitRef::head(), content);

    let hash = mock_provider
        .get_file_hash("test.rs", &GitRef::head())
        .unwrap();

    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let expected = format!("{:x}", hasher.finalize());

    assert_eq!(hash, expected);
}

// ===== Content Snapshot Extraction Tests =====

#[test]
fn test_extract_content_snapshot_middle_lines() {
    let mut mock_provider = MockDiffProvider::new();
    let content = "line1\nline2\nline3\nline4\nline5\nline6\nline7\nline8\nline9\nline10\n";
    mock_provider.add_file_content("test.rs", &GitRef::head(), content);

    let line_range = test_range(3, 5);

    let snapshot = mock_provider
        .get_content_snapshot("test.rs", &GitRef::head(), &line_range)
        .unwrap();

    assert_eq!(snapshot, Some("line3\nline4\nline5".to_string()));
}

#[test]
fn test_extract_content_snapshot_start_of_file() {
    let mut mock_provider = MockDiffProvider::new();
    let content = "line1\nline2\nline3\n";
    mock_provider.add_file_content("test.rs", &GitRef::head(), content);

    let line_range = test_range(1, 2);

    let snapshot = mock_provider
        .get_content_snapshot("test.rs", &GitRef::head(), &line_range)
        .unwrap();

    assert_eq!(snapshot, Some("line1\nline2".to_string()));
}

#[test]
fn test_extract_content_snapshot_end_of_file() {
    let mut mock_provider = MockDiffProvider::new();
    let content = "line1\nline2\nline3\n";
    mock_provider.add_file_content("test.rs", &GitRef::head(), content);

    let line_range = test_range(2, 3);

    let snapshot = mock_provider
        .get_content_snapshot("test.rs", &GitRef::head(), &line_range)
        .unwrap();

    assert_eq!(snapshot, Some("line2\nline3".to_string()));
}

#[test]
fn test_extract_content_snapshot_beyond_file_bounds() {
    let mut mock_provider = MockDiffProvider::new();
    let content = "line1\nline2\nline3\n";
    mock_provider.add_file_content("test.rs", &GitRef::head(), content);

    let line_range = test_range(10, 15);

    let snapshot = mock_provider
        .get_content_snapshot("test.rs", &GitRef::head(), &line_range)
        .unwrap();

    assert_eq!(snapshot, None);
}

#[test]
fn test_extract_content_snapshot_empty_range() {
    let mut mock_provider = MockDiffProvider::new();
    let content = "line1\nline2\nline3\n";
    mock_provider.add_file_content("test.rs", &GitRef::head(), content);

    let line_range = test_range(2, 2);

    let snapshot = mock_provider
        .get_content_snapshot("test.rs", &GitRef::head(), &line_range)
        .unwrap();

    assert_eq!(snapshot, Some("line2".to_string()));
}
