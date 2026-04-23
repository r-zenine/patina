//! Thin wrapper around `tui_harness::CombinedTestHarness` for diffviz-review-tui.

use diffviz_review::engines::ReviewEngine;

use crate::{Result, app::HeadlessApp};

use super::snapshot::StateSnapshot;

pub use tui_harness::CombinedTestResult;

/// Combined test harness for full integration testing
///
/// Chains input processing and rendering to validate complete workflows
/// from keyboard input through state changes to visual output.
pub struct CombinedTestHarness {
    inner: tui_harness::CombinedTestHarness<HeadlessApp>,
}

impl CombinedTestHarness {
    /// Create a new combined test harness
    pub fn new(review_engine: ReviewEngine) -> Self {
        Self {
            inner: tui_harness::CombinedTestHarness::new(HeadlessApp::new(review_engine)),
        }
    }

    /// Create a combined test harness with custom render dimensions
    pub fn with_render_size(review_engine: ReviewEngine, width: u16, height: u16) -> Self {
        Self {
            inner: tui_harness::CombinedTestHarness::with_render_size(
                HeadlessApp::new(review_engine),
                width,
                height,
            ),
        }
    }

    /// Run an input sequence capturing both state and visual at each step
    pub fn run_sequence_with_renders(
        &mut self,
        input: &str,
    ) -> Result<Vec<CombinedTestResult<StateSnapshot>>> {
        self.inner
            .run_sequence_with_renders(input)
            .map_err(anyhow::Error::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use diffviz_review::providers::mock_provider::MockDiffProvider;
    use diffviz_review::{DiffQuery, GitRef, ReviewEngineBuilder};

    fn create_test_engine() -> ReviewEngine {
        let mock_provider =
            MockDiffProvider::from_review_fixtures().expect("Failed to load test fixtures");
        let review_engine_builder =
            ReviewEngineBuilder::new(Box::new(mock_provider), "test-user".to_string());
        let diff_query = DiffQuery::new(GitRef::Head, GitRef::Unstaged);
        review_engine_builder
            .build_from_decisions(vec![], diff_query)
            .expect("Failed to build ReviewEngine")
    }

    #[test]
    fn test_combined_harness_creation() {
        let engine = create_test_engine();
        let _harness = CombinedTestHarness::new(engine);
    }

    #[test]
    fn test_combined_harness_with_custom_size() {
        let engine = create_test_engine();
        let _harness = CombinedTestHarness::with_render_size(engine, 120, 40);
    }
}
