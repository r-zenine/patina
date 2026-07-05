use crate::entities::decision::{CodeImpact, Decision, DecisionLineRange};
use crate::entities::git_ref::{DiffQuery, GitRef};
use crate::providers::mock_provider::MockDiffProvider;
use crate::review_engine_builder::ReviewEngineBuilder;

// Bug: When a decision has multiple line ranges that all expand to the same semantic unit
// (e.g., two ranges both inside the same function), build_from_decisions pushes the decision
// number into decision_index once per range. The result is decision_index[fn_id] = [1, 1]
// instead of [1], and get_decision_reviewable_diffs() returns duplicate entries, causing
// the TUI to render the same diff twice under that decision.
//
// Expected: get_decision_reviewable_diffs() returns exactly one entry per unique
// (ReviewableDiffId, decision_number) pair, regardless of how many ranges collapsed
// onto that unit.
//
// Actual: returns N entries when N ranges all expand to the same semantic unit.
#[test]
fn test_two_ranges_in_same_function_do_not_produce_duplicate_diffs() {
    // A function whose body spans multiple lines — two distinct sub-ranges
    // (parameters and body) both belong to the same semantic unit (fn compute_total).
    let old_source = "fn compute_total(a: i32, b: i32) -> i32 {\n    a + b\n}\n";
    // new_source (7 lines):
    // 1: fn compute_total(
    // 2:     alpha: i32,
    // 3:     beta: i32,
    // 4: ) -> i32 {
    // 5:     let sum = alpha + beta;
    // 6:     sum
    // 7: }
    let new_source = "fn compute_total(\n    alpha: i32,\n    beta: i32,\n) -> i32 {\n    let sum = alpha + beta;\n    sum\n}\n";

    let old_ref = GitRef::commit("abc111".to_string());
    let new_ref = GitRef::commit("def222".to_string());

    let mut provider = MockDiffProvider::new();
    provider.add_file_content("src/math.rs", &old_ref, old_source);
    provider.add_file_content("src/math.rs", &new_ref, new_source);

    // One decision, two ranges — lines 2-3 (params) and lines 5-6 (body).
    // Both expand to fn compute_total (lines 1-7) → same ReviewableDiffId.
    let decision = Decision {
        number: 1,
        title: "Refactor compute_total".to_string(),
        rationale: None,
        code_impacts: vec![CodeImpact {
            file: "src/math.rs".to_string(),
            line_ranges: vec![
                DecisionLineRange { start: 2, end: 3 },
                DecisionLineRange { start: 5, end: 6 },
            ],
            reasoning: "Two sub-ranges inside the same function".to_string(),
        }],
    };

    let query = DiffQuery::new(old_ref, new_ref);
    let builder = ReviewEngineBuilder::new(Box::new(provider), "test_author".to_string());
    let engine = builder.build_from_decisions(vec![decision], query).unwrap();

    let diffs = engine.get_decision_reviewable_diffs();

    assert_eq!(
        diffs.len(),
        1,
        "Two ranges resolving to the same semantic unit must yield exactly 1 DecisionReviewableDiff \
         (got {}) — duplicates cause the TUI to render the diff twice",
        diffs.len()
    );

    // Both original cited ranges must be recorded on the single merged entry.
    let reviewable = engine
        .get_reviewable_diff(&diffs[0].chunk_id)
        .expect("ReviewableDiff must exist for the returned chunk_id");
    assert_eq!(
        reviewable.cited_ranges.len(),
        2,
        "Both original code-impact ranges must be collected on the merged ReviewableDiff, \
         got: {:?}",
        reviewable.cited_ranges
    );
    assert!(
        reviewable.cited_ranges.contains(&(2, 3)),
        "cited_ranges must include the first range (2-3), got: {:?}",
        reviewable.cited_ranges
    );
    assert!(
        reviewable.cited_ranges.contains(&(5, 6)),
        "cited_ranges must include the second range (5-6), got: {:?}",
        reviewable.cited_ranges
    );
}

// Bug: build_from_decisions crashes when a decision's code_impacts references a
// file that was deleted in the target commit.
//
// Expected: builder skips (or gracefully handles) deleted files and returns Ok.
// Actual:   builder calls get_source_code on the "to" ref, gets "File not found",
//           and propagates DiffVizError::Git("Failed to get new source for …").
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
