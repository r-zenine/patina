//! Test harness for input processing
//!
//! Run input sequences through HeadlessApp and capture resulting state.

use crate::app::HeadlessApp;
use crate::Result;
use diffviz_review::engines::ReviewEngine;

use super::input_parser::parse_input_sequence;
use super::snapshot::StateSnapshot;

/// Test harness for validating input sequence → state transformations
pub struct InputTestHarness {
    app: HeadlessApp,
}

impl InputTestHarness {
    /// Create a new input test harness
    pub fn new(review_engine: ReviewEngine) -> Self {
        Self {
            app: HeadlessApp::new(review_engine),
        }
    }

    /// Run an input sequence and return final state snapshot
    pub fn run_sequence_final_state(&mut self, input: &str) -> Result<StateSnapshot> {
        let events = parse_input_sequence(input)?;

        for event in events {
            self.app.process_key_event(event)?;
        }

        Ok(StateSnapshot::from_ui_state(&self.app.ui_state))
    }

    /// Run an input sequence and return snapshots after each event
    pub fn run_sequence(&mut self, input: &str) -> Result<Vec<StateSnapshot>> {
        let events = parse_input_sequence(input)?;
        let mut snapshots = Vec::new();

        // Capture initial state
        snapshots.push(StateSnapshot::from_ui_state(&self.app.ui_state));

        for event in events {
            self.app.process_key_event(event)?;
            snapshots.push(StateSnapshot::from_ui_state(&self.app.ui_state));
        }

        Ok(snapshots)
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
    fn test_input_harness_creation() {
        let engine = create_test_engine();
        let _harness = InputTestHarness::new(engine);
    }

    #[test]
    fn test_run_sequence_returns_snapshot() {
        let engine = create_test_engine();
        let mut harness = InputTestHarness::new(engine);

        let snapshot = harness.run_sequence_final_state("j").unwrap();
        assert!(!snapshot.should_quit);
    }
}
