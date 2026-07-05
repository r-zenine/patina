//! Shared fixtures for the DrillNav contract tests.
//!
//! The decisions mirror `src/main.rs::create_hardcoded_decisions_vec()` so the
//! CLI harness (`--test-input` / `--test-full`) and these tests exercise the
//! same structure. Verified against the mock fixtures (Phase 0 audit):
//!
//! - Decision 1 (idx 0): 2 files — `src/config/reader.rs` (2 chunks),
//!   `src/models/calculator.rs` (7 chunks) — 9 chunks total
//! - Decision 2 (idx 1): 1 file — `src/network/client.rs` (1 chunk)
//! - Decision 3 (idx 2): 3 files — `src/components/Greeting.tsx` (3 chunks),
//!   `src/data/fetcher.py` (1 chunk), `src/types/api.ts` (1 chunk)
//!
//! File indices follow lexicographic order of file paths (the DrillNav
//! sibling order); chunk indices follow ascending start line.

// Each integration-test crate compiles this module independently; not every
// crate uses every helper.
#![allow(dead_code)]

use diffviz_review::engines::ReviewEngine;
use diffviz_review::providers::mock_provider::MockDiffProvider;
use diffviz_review::{
    CodeImpact, Decision, DecisionLineRange, DiffQuery, GitRef, ReviewEngineBuilder,
    ReviewableDiffId,
};
use diffviz_review_tui::ReviewTuiApp;
use diffviz_review_tui::test_harness::parse_input_sequence;

/// Engine with the audited multi-file / multi-chunk decision structure above.
pub fn create_drillnav_engine() -> ReviewEngine {
    let mock_provider =
        MockDiffProvider::from_review_fixtures().expect("Failed to load test fixtures");
    let review_engine_builder =
        ReviewEngineBuilder::new(Box::new(mock_provider), "test-user".to_string());
    let diff_query = DiffQuery::new(GitRef::Head, GitRef::Unstaged);

    let decisions = vec![
        Decision {
            number: 1,
            title: "Refactor calculator model module".to_string(),
            rationale: Some("Extract calculator logic into separate, testable module".to_string()),
            code_impacts: vec![
                CodeImpact {
                    file: "src/models/calculator.rs".to_string(),
                    line_ranges: vec![DecisionLineRange { start: 1, end: 72 }],
                    reasoning: "Calculator model structure refactoring".to_string(),
                },
                CodeImpact {
                    file: "src/config/reader.rs".to_string(),
                    line_ranges: vec![DecisionLineRange { start: 1, end: 7 }],
                    reasoning: "Configuration reader updates".to_string(),
                },
            ],
        },
        Decision {
            number: 2,
            title: "Improve error handling in network client".to_string(),
            rationale: Some(
                "Standardize error types and add context to error messages".to_string(),
            ),
            code_impacts: vec![
                CodeImpact {
                    file: "src/network/client.rs".to_string(),
                    line_ranges: vec![DecisionLineRange { start: 1, end: 6 }],
                    reasoning: "Network error handling improvements".to_string(),
                },
                CodeImpact {
                    file: "src/models/calculator.rs".to_string(),
                    line_ranges: vec![DecisionLineRange { start: 20, end: 40 }],
                    reasoning: "Calculator model error handling enhancements".to_string(),
                },
            ],
        },
        Decision {
            number: 3,
            title: "Add structured logging throughout application".to_string(),
            rationale: Some(
                "Architectural decision: use tracing crate for observability".to_string(),
            ),
            code_impacts: vec![
                CodeImpact {
                    file: "src/data/fetcher.py".to_string(),
                    line_ranges: vec![DecisionLineRange { start: 1, end: 5 }],
                    reasoning: "Add logging to async data fetching operations".to_string(),
                },
                CodeImpact {
                    file: "src/components/Greeting.tsx".to_string(),
                    line_ranges: vec![DecisionLineRange { start: 1, end: 49 }],
                    reasoning: "Add component lifecycle logging".to_string(),
                },
                CodeImpact {
                    file: "src/types/api.ts".to_string(),
                    line_ranges: vec![DecisionLineRange { start: 1, end: 9 }],
                    reasoning: "Add API type validation logging".to_string(),
                },
            ],
        },
    ];

    review_engine_builder
        .build_from_decisions(decisions, diff_query)
        .expect("Failed to build ReviewEngine")
}

/// Engine with no decisions at all — the empty-review edge case used to
/// exercise error surfacing (approving with nothing selected must fail).
pub fn create_empty_engine() -> ReviewEngine {
    let mock_provider =
        MockDiffProvider::from_review_fixtures().expect("Failed to load test fixtures");
    let review_engine_builder =
        ReviewEngineBuilder::new(Box::new(mock_provider), "test-user".to_string());
    let diff_query = DiffQuery::new(GitRef::Head, GitRef::Unstaged);
    review_engine_builder
        .build_from_decisions(vec![], diff_query)
        .expect("Failed to build ReviewEngine")
}

/// Drive a fresh app through an input sequence and hand back the engine so
/// tests can assert business state (approvals, notes) directly.
pub fn drive_app(engine: ReviewEngine, input: &str) -> ReviewEngine {
    let mut app = ReviewTuiApp::new(engine).expect("Failed to create ReviewTuiApp");
    let steps = parse_input_sequence(input).expect("Failed to parse input sequence");
    for step in steps {
        step.apply(&mut app).expect("Failed to apply input step");
    }
    app.into_review_engine()
}

/// Chunk IDs for a decision × file, ordered by start line (the DrillNav
/// chunk-cursor order).
pub fn chunks_for_file(
    engine: &ReviewEngine,
    decision_number: u32,
    file_path: &str,
) -> Vec<ReviewableDiffId> {
    let mut ids: Vec<ReviewableDiffId> = engine
        .get_decision_reviewable_diffs()
        .into_iter()
        .filter(|d| d.decision_number == decision_number && d.chunk_id.file_path() == file_path)
        .map(|d| d.chunk_id)
        .collect();
    ids.sort_by_key(|id| id.line_range().start_line);
    ids
}
