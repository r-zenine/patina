//! Combined test harness for full integration testing
//!
//! Chain input processing and rendering to capture both state transitions
//! and visual output at each step of the test sequence.

use diffviz_review::engines::ReviewEngine;

use crate::{app::HeadlessApp, Result};

use super::input_parser::parse_input_sequence;
use super::render_test::RenderTestHarness;
use super::snapshot::StateSnapshot;

/// Combined test result: state snapshot + visual output at each step
#[derive(Debug, Clone)]
pub struct CombinedTestResult {
    /// State snapshot after this event
    pub state: StateSnapshot,
    /// Visual output for this state
    pub visual: String,
}

/// Combined test harness for full integration testing
///
/// Chains input processing and rendering to validate complete workflows
/// from keyboard input through state changes to visual output.
pub struct CombinedTestHarness {
    app: HeadlessApp,
    render_harness: RenderTestHarness,
}

impl CombinedTestHarness {
    /// Create a new combined test harness
    pub fn new(review_engine: ReviewEngine) -> Self {
        Self {
            app: HeadlessApp::new(review_engine),
            render_harness: RenderTestHarness::new(),
        }
    }

    /// Create a combined test harness with custom render dimensions
    pub fn with_render_size(review_engine: ReviewEngine, width: u16, height: u16) -> Self {
        Self {
            app: HeadlessApp::new(review_engine),
            render_harness: RenderTestHarness::with_size(width, height),
        }
    }

    /// Run an input sequence capturing both state and visual at each step
    pub fn run_sequence_with_renders(&mut self, input: &str) -> Result<Vec<CombinedTestResult>> {
        let events = parse_input_sequence(input)?;
        let mut results = Vec::new();

        // Capture initial state and render
        let state = StateSnapshot::from_ui_state(&self.app.ui_state);
        let visual = self
            .render_harness
            .render(&mut self.app.ui_state, &self.app.review_engine)?;
        results.push(CombinedTestResult { state, visual });

        // Process each event and capture state + render
        for event in events {
            self.app.process_key_event(event)?;
            let state = StateSnapshot::from_ui_state(&self.app.ui_state);
            let visual = self
                .render_harness
                .render(&mut self.app.ui_state, &self.app.review_engine)?;
            results.push(CombinedTestResult { state, visual });
        }

        Ok(results)
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
