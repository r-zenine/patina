//! Thin wrapper around `tui_harness::RenderTestHarness` for diffviz-review-tui.

use ratatui::Frame;
use tui_harness::ELMApp;

use crate::{Result, state::UiState, ui};
use diffviz_review::engines::ReviewEngine;

/// Test harness for validating UI rendering
pub struct RenderTestHarness {
    inner: tui_harness::RenderTestHarness,
}

impl RenderTestHarness {
    /// Create a new render test harness with default terminal size
    pub fn new() -> Self {
        Self {
            inner: tui_harness::RenderTestHarness::new(),
        }
    }

    /// Create a render test harness with specific terminal dimensions
    pub fn with_size(width: u16, height: u16) -> Self {
        Self {
            inner: tui_harness::RenderTestHarness::with_size(width, height),
        }
    }

    /// Render a UI state and return the visual output as a string
    pub fn render(&self, ui_state: &mut UiState, review_engine: &ReviewEngine) -> Result<String> {
        let adapter = RenderAdapter {
            ui_state,
            review_engine,
        };
        self.inner.render(&adapter).map_err(anyhow::Error::from)
    }
}

impl Default for RenderTestHarness {
    fn default() -> Self {
        Self::new()
    }
}

/// Local adapter so a bare (UiState, ReviewEngine) pair implements ELMApp for rendering.
struct RenderAdapter<'a> {
    ui_state: &'a UiState,
    review_engine: &'a ReviewEngine,
}

impl ELMApp for RenderAdapter<'_> {
    type Snapshot = ();
    type Error = std::convert::Infallible;

    fn draw(&self, frame: &mut Frame) {
        ui::draw(frame, self.ui_state, self.review_engine);
    }

    fn dispatch_key(
        &mut self,
        _key: crossterm::event::KeyEvent,
    ) -> std::result::Result<(), std::convert::Infallible> {
        Ok(())
    }

    fn should_quit(&self) -> bool {
        false
    }

    fn snapshot(&self) {}
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
    fn test_render_harness_creation() {
        let _harness = RenderTestHarness::new();
    }

    #[test]
    fn test_render_with_custom_size() {
        let _harness = RenderTestHarness::with_size(120, 40);
    }

    #[test]
    fn test_render_ui_state() {
        let engine = create_test_engine();
        let harness = RenderTestHarness::new();
        let mut ui_state = UiState::new();

        ui_state.decision_tree =
            crate::decision_navigation::DecisionNavigationTree::build_from_review_engine(&engine);

        let output = harness
            .render(&mut ui_state, &engine)
            .expect("Render failed");
        assert!(!output.is_empty());
    }

    #[test]
    fn test_render_output_contains_lines() {
        let engine = create_test_engine();
        let harness = RenderTestHarness::new();
        let mut ui_state = UiState::new();

        ui_state.decision_tree =
            crate::decision_navigation::DecisionNavigationTree::build_from_review_engine(&engine);

        let output = harness
            .render(&mut ui_state, &engine)
            .expect("Render failed");
        let lines: Vec<&str> = output.lines().collect();

        assert!(!lines.is_empty(), "Output should contain at least one line");
    }
}
