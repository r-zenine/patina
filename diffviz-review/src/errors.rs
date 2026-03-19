use thiserror::Error;

#[derive(Debug, Error)]
pub enum DiffVizError {
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Processing failed: {0}")]
    ProcessingFailed(String),

    #[error("Git operation failed: {0}")]
    Git(String),

    #[error("Repository error: {0}")]
    Repository(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parse error: {0}")]
    YamlParse(#[from] serde_yaml::Error),

    // Review-specific errors
    #[error("Review operation failed")]
    Review(#[from] ReviewError),
}

/// Specific errors that can occur during review operations
#[derive(Debug, Error)]
pub enum ReviewError {
    #[error("Navigation failed: {reason}")]
    NavigationFailed {
        reason: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("ReviewableDiff review failed for {reviewable_id}: {reason}")]
    ReviewableDiffReviewFailed {
        reviewable_id: String,
        reason: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Review context invalid: {details}")]
    InvalidContext {
        details: String,
        current_file_path: Option<String>,
        current_reviewable_diff_id: Option<String>,
    },

    #[error("User interaction failed: {operation}")]
    InteractionFailed {
        operation: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Review completion failed: {reason}")]
    CompletionFailed {
        reason: String,
        approved_reviewable_diffs: usize,
        total_reviewable_diffs: usize,
    },

    #[error("File not found in diff: {file_path} at index {file_index}")]
    FileNotFound {
        file_path: String,
        file_index: usize,
    },

    #[error("ReviewableDiff not found: {reviewable_id}")]
    ReviewableDiffNotFound {
        reviewable_id: String,
        file_path: String,
    },

    #[error("Review engine not initialized")]
    EngineNotInitialized,

    #[error("Review session cancelled by user")]
    UserCancelled,

    #[error("Display error: failed to render {item_type}")]
    DisplayError {
        item_type: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error(
        "Instruction overlap detected: cannot add instruction for {new_id} because it overlaps with existing instruction at {existing_id}"
    )]
    InstructionOverlap { new_id: String, existing_id: String },

    #[error("Export failed: {reason}")]
    ExportFailed { reason: String },

    #[error("Import failed: {reason}")]
    ImportFailed { reason: String },
}

pub type Result<T> = std::result::Result<T, DiffVizError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use std::io;

    #[test]
    fn test_invalid_operation_error() {
        let error = DiffVizError::InvalidOperation("test operation".to_string());
        assert_eq!(error.to_string(), "Invalid operation: test operation");
    }

    #[test]
    fn test_processing_failed_error() {
        let error = DiffVizError::ProcessingFailed("parsing failed".to_string());
        assert_eq!(error.to_string(), "Processing failed: parsing failed");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let diffviz_error: DiffVizError = io_error.into();

        match diffviz_error {
            DiffVizError::Io(_) => {} // Expected
            _ => panic!("Expected Io error variant"),
        }
    }

    #[test]
    fn test_result_type_alias() {
        let success: Result<String> = Ok("success".to_string());
        let failure: Result<String> = Err(DiffVizError::InvalidOperation("failed".to_string()));

        assert!(success.is_ok());
        assert!(failure.is_err());
    }

    #[test]
    fn test_error_chain_display() {
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "Access denied");
        let diffviz_error = DiffVizError::Io(io_error);
        let error_string = diffviz_error.to_string();
        assert!(error_string.contains("IO error"));
    }

    #[test]
    fn test_error_source_chain() {
        let io_error = io::Error::new(io::ErrorKind::TimedOut, "Operation timed out");
        let diffviz_error = DiffVizError::Io(io_error);

        // Test that the source chain is preserved
        assert!(diffviz_error.source().is_some());
    }

    #[test]
    fn test_git_error_formatting() {
        let git_error = DiffVizError::Git("Failed to parse commit".to_string());
        assert_eq!(
            git_error.to_string(),
            "Git operation failed: Failed to parse commit"
        );
    }

    #[test]
    fn test_repository_error_formatting() {
        let repo_error = DiffVizError::Repository("Could not access .git directory".to_string());
        assert_eq!(
            repo_error.to_string(),
            "Repository error: Could not access .git directory"
        );
    }

    #[test]
    fn test_review_error_navigation_failed() {
        let review_error = ReviewError::NavigationFailed {
            reason: "No files available".to_string(),
            source: None,
        };
        assert_eq!(
            review_error.to_string(),
            "Navigation failed: No files available"
        );
    }

    #[test]
    fn test_review_error_reviewable_diff_review_failed() {
        let review_error = ReviewError::ReviewableDiffReviewFailed {
            reviewable_id: "working:main.rs:L1-10".to_string(),
            reason: "User cancelled".to_string(),
            source: None,
        };
        assert!(review_error
            .to_string()
            .contains("ReviewableDiff review failed"));
        assert!(review_error.to_string().contains("working:main.rs:L1-10"));
        assert!(review_error.to_string().contains("User cancelled"));
    }

    #[test]
    fn test_review_error_invalid_context() {
        let review_error = ReviewError::InvalidContext {
            details: "File path not found".to_string(),
            current_file_path: Some("test.rs".to_string()),
            current_reviewable_diff_id: Some("working:test.rs:L1-5".to_string()),
        };
        assert!(review_error.to_string().contains("Review context invalid"));
        assert!(review_error.to_string().contains("File path not found"));
    }

    #[test]
    fn test_review_error_interaction_failed() {
        let review_error = ReviewError::InteractionFailed {
            operation: "get_user_input".to_string(),
            source: None,
        };
        assert_eq!(
            review_error.to_string(),
            "User interaction failed: get_user_input"
        );
    }

    #[test]
    fn test_review_error_completion_failed() {
        let review_error = ReviewError::CompletionFailed {
            reason: "Incomplete review".to_string(),
            approved_reviewable_diffs: 2,
            total_reviewable_diffs: 5,
        };
        assert!(review_error
            .to_string()
            .contains("Review completion failed"));
        assert!(review_error.to_string().contains("Incomplete review"));
    }

    #[test]
    fn test_review_error_file_not_found() {
        let review_error = ReviewError::FileNotFound {
            file_path: "missing.rs".to_string(),
            file_index: 3,
        };
        assert!(review_error.to_string().contains("File not found in diff"));
        assert!(review_error.to_string().contains("missing.rs"));
        assert!(review_error.to_string().contains("index 3"));
    }

    #[test]
    fn test_review_error_reviewable_diff_not_found() {
        let review_error = ReviewError::ReviewableDiffNotFound {
            reviewable_id: "working:test.rs:L10-20".to_string(),
            file_path: "test.rs".to_string(),
        };
        assert!(review_error
            .to_string()
            .contains("ReviewableDiff not found"));
        assert!(review_error.to_string().contains("working:test.rs:L10-20"));
    }

    #[test]
    fn test_review_error_engine_not_initialized() {
        let review_error = ReviewError::EngineNotInitialized;
        assert_eq!(review_error.to_string(), "Review engine not initialized");
    }

    #[test]
    fn test_review_error_user_cancelled() {
        let review_error = ReviewError::UserCancelled;
        assert_eq!(review_error.to_string(), "Review session cancelled by user");
    }

    #[test]
    fn test_review_error_display_error() {
        let review_error = ReviewError::DisplayError {
            item_type: "FileDiff".to_string(),
            source: None,
        };
        assert!(review_error.to_string().contains("Display error"));
        assert!(review_error.to_string().contains("FileDiff"));
    }

    #[test]
    fn test_diff_viz_error_review_conversion() {
        let review_error = ReviewError::UserCancelled;
        let diff_viz_error: DiffVizError = review_error.into();

        match diff_viz_error {
            DiffVizError::Review(_) => {} // Expected
            _ => panic!("Expected Review error variant"),
        }
    }

    #[test]
    fn test_review_error_with_source() {
        use std::io;
        let io_error = io::Error::new(io::ErrorKind::BrokenPipe, "Connection lost");
        let review_error = ReviewError::InteractionFailed {
            operation: "read_input".to_string(),
            source: Some(Box::new(io_error)),
        };

        assert!(review_error.source().is_some());
        assert_eq!(
            review_error.to_string(),
            "User interaction failed: read_input"
        );
    }
}
