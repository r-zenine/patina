use crate::entities::decision::{CodeImpact, Decision, DecisionLineRange};
use crate::entities::git_ref::{DiffQuery, GitRef};
use crate::providers::mock_provider::MockDiffProvider;
use crate::review_engine_builder::ReviewEngineBuilder;

// Bug: build_from_decisions crashes when a decision's code_impacts references a
// file that was deleted in the target commit.
//
// Expected: builder skips (or gracefully handles) deleted files and returns Ok.
// Actual:   builder calls get_source_code on the "to" ref, gets "File not found",
//           and propagates DiffVizError::Git("Failed to get new source for …").
#[ignore]
#[test]
fn test_build_from_decisions_does_not_crash_on_deleted_file() {
    let old_ref = GitRef::commit("abc123".to_string());
    let new_ref = GitRef::commit("def456".to_string());

    let mut provider = MockDiffProvider::new();
    // File exists only at the old ref — it was deleted at new_ref.
    provider.add_file_content(
        "src/error.rs",
        &old_ref,
        "use thiserror::Error;\n\
         \n\
         #[derive(Debug, Error)]\n\
         pub enum SamTuiError {\n\
             #[error(\"Terminal error: {0}\")]\n\
             Terminal(#[from] std::io::Error),\n\
         }\n",
    );

    let decision = Decision {
        number: 1,
        title: "Delete SamTuiError".to_string(),
        rationale: Some("Replaced by tui_harness::TuiError".to_string()),
        code_impacts: vec![CodeImpact {
            file: "src/error.rs".to_string(),
            line_ranges: vec![DecisionLineRange { start: 1, end: 7 }],
            reasoning: "Deleted — file removed entirely".to_string(),
        }],
    };

    let query = DiffQuery::new(old_ref, new_ref);
    let builder = ReviewEngineBuilder::new(Box::new(provider), "test_author".to_string());

    let result = builder.build_from_decisions(vec![decision], query);

    // Should not error on a deleted file.
    assert!(
        result.is_ok(),
        "build_from_decisions should handle deleted files gracefully, got: {:?}",
        result.err()
    );
}
