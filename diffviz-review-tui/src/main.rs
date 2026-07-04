//! Test-only TUI binary for predictable manual testing
//!
//! This standalone binary always uses curated fixtures from diffviz-review
//! to provide predictable, reproducible content for manual TUI validation.
//! Never requires a git repository - always works with the same test data.
//!
//! The agent CLI surface comes from tui-harness:
//!   --describe             machine-readable app manifest
//!   --test-input <seq>     run keys headlessly, print the final snapshot
//!   --test-full <seq>      run keys headlessly, print state + visual per step
//!   (no flags)             interactive TUI

use anyhow::Result;
use diffviz_review::providers::mock_provider::MockDiffProvider;
use diffviz_review::{
    CodeImpact, Decision, DecisionLineRange, DiffQuery, GitRef, ReviewEngineBuilder,
};
use diffviz_review_tui::ReviewTuiApp;
use std::env;

fn main() -> Result<()> {
    // Initialize tracing for debugging
    tracing_subscriber::fmt::init();

    let review_engine = create_test_review_engine()?;
    let app = ReviewTuiApp::new(review_engine)?;
    tui_harness::run_agent_cli(app, env::args().skip(1))?;

    Ok(())
}

/// Create a test ReviewEngine using curated fixtures
fn create_test_review_engine() -> Result<diffviz_review::engines::ReviewEngine> {
    // Use curated fixtures from diffviz-review crate
    let mock_provider = MockDiffProvider::from_review_fixtures()
        .map_err(|e| anyhow::anyhow!("Failed to load test fixtures: {e}"))?;

    // Use decision-based ReviewEngine creation
    let review_engine_builder =
        ReviewEngineBuilder::new(Box::new(mock_provider), "dev-user".to_string());
    let diff_query = DiffQuery::new(GitRef::Head, GitRef::Unstaged);

    // Create hardcoded decisions for TUI testing
    let decisions = create_hardcoded_decisions_vec();

    // Build ReviewEngine using the decision-based pipeline
    let review_engine = review_engine_builder
        .build_from_decisions(decisions, diff_query)
        .map_err(|e| anyhow::anyhow!("Failed to build ReviewEngine from decisions: {e}"))?;

    Ok(review_engine)
}

/// Create hardcoded decisions for TUI testing
/// Returns Vec<Decision> for use with build_from_decisions()
fn create_hardcoded_decisions_vec() -> Vec<Decision> {
    vec![
        // Decision 1: Refactor calculator model module
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
        // Decision 2: Improve network client error handling
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
        // Decision 3: Add logging infrastructure
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
    ]
}
