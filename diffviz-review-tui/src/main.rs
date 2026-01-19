//! Test-only TUI binary for predictable manual testing
//!
//! This standalone binary always uses curated fixtures from diffviz-review
//! to provide predictable, reproducible content for manual TUI validation.
//! Never requires a git repository - always works with the same test data.

use anyhow::Result;
use diffviz_review::providers::mock_provider::MockDiffProvider;
use diffviz_review::{DiffQuery, GitRef, ReviewEngineBuilder};
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
    let review_engine = review_engine_builder
        .build(diff_query)
        .map_err(|e| anyhow::anyhow!("Failed to build ReviewEngine: {}", e))?;

    Ok(review_engine)
}
