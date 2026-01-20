//! Test-only TUI binary for predictable manual testing
//!
//! This standalone binary always uses curated fixtures from diffviz-review
//! to provide predictable, reproducible content for manual TUI validation.
//! Never requires a git repository - always works with the same test data.

use anyhow::Result;
use diffviz_review::providers::mock_provider::MockDiffProvider;
use diffviz_review::{
    ChangeType, CodeImpact, Confidence, Decision, DecisionLineRange, DiffQuery, GitRef,
    ReviewDecisions, ReviewEngineBuilder,
};
use diffviz_review_tui::ReviewTuiApp;

fn main() -> Result<()> {
    // Initialize tracing for debugging
    tracing_subscriber::fmt::init();

    // Always create test review engine - no conditionals
    let review_engine = create_test_review_engine()?;

    // Create and run the TUI application
    let mut app = ReviewTuiApp::new(review_engine)?;
    app.run()?;

    Ok(())
}

/// Create a test ReviewEngine using curated fixtures
fn create_test_review_engine() -> Result<diffviz_review::engines::ReviewEngine> {
    // Use curated fixtures from diffviz-review crate
    let mock_provider = MockDiffProvider::from_review_fixtures()
        .map_err(|e| anyhow::anyhow!("Failed to load test fixtures: {}", e))?;

    // Standard ReviewEngine creation using ReviewEngineBuilder
    let review_engine_builder =
        ReviewEngineBuilder::new(Box::new(mock_provider), "dev-user".to_string());
    let diff_query = DiffQuery::new(GitRef::Head, GitRef::Unstaged);
    let mut review_engine = review_engine_builder
        .build(diff_query)
        .map_err(|e| anyhow::anyhow!("Failed to build ReviewEngine: {}", e))?;

    // Phase 1: Add hardcoded decision data for TUI testing
    let decisions = create_hardcoded_decisions();
    review_engine.set_decisions(decisions);

    Ok(review_engine)
}

/// Create hardcoded decisions for Phase 1 TUI testing
/// These demonstrate decision-based review structure without requiring JSON loading
fn create_hardcoded_decisions() -> ReviewDecisions {
    let mut decisions = ReviewDecisions::new();

    // Decision 1: Refactor authentication module
    decisions.add_decision(Decision {
        number: 1,
        title: "Refactor authentication module".to_string(),
        summary: "Extract authentication logic into separate, testable module".to_string(),
        decision_log_line: Some(15),
        code_impacts: vec![
            CodeImpact {
                file: "src/lib.rs".to_string(),
                line_ranges: vec![DecisionLineRange { start: 1, end: 50 }],
                change_type: ChangeType::Modification,
                confidence: Confidence::High,
                reasoning: "Main library module imports new auth module".to_string(),
            },
            CodeImpact {
                file: "src/auth.rs".to_string(),
                line_ranges: vec![DecisionLineRange { start: 1, end: 100 }],
                change_type: ChangeType::Addition,
                confidence: Confidence::High,
                reasoning: "New authentication module with login, logout functions".to_string(),
            },
        ],
    });

    // Decision 2: Add error handling improvements
    decisions.add_decision(Decision {
        number: 2,
        title: "Improve error handling across modules".to_string(),
        summary: "Standardize error types and add context to error messages".to_string(),
        decision_log_line: Some(28),
        code_impacts: vec![
            CodeImpact {
                file: "src/lib.rs".to_string(),
                line_ranges: vec![DecisionLineRange { start: 1, end: 50 }],
                change_type: ChangeType::Modification,
                confidence: Confidence::Medium,
                reasoning: "Adds error context to library result types".to_string(),
            },
            CodeImpact {
                file: "src/auth.rs".to_string(),
                line_ranges: vec![DecisionLineRange {
                    start: 50,
                    end: 100,
                }],
                change_type: ChangeType::Modification,
                confidence: Confidence::High,
                reasoning: "Implements custom error types for auth failures".to_string(),
            },
        ],
    });

    // Decision 3: Add logging infrastructure (no-code decision)
    decisions.add_decision(Decision {
        number: 3,
        title: "Add structured logging throughout application".to_string(),
        summary: "Architectural decision: use tracing crate for observability".to_string(),
        decision_log_line: Some(42),
        code_impacts: vec![],
    });

    decisions
}
