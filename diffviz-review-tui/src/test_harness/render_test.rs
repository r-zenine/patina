//! Test harness for rendering visual output
//!
//! Render UiState using ratatui's TestBackend and capture visual output as text.

use ratatui::{backend::TestBackend, buffer::Buffer, Terminal};

use crate::{state::UiState, ui, Result};
use diffviz_review::engines::ReviewEngine;

/// Test harness for validating UI rendering
pub struct RenderTestHarness {
    /// Terminal dimensions for rendering
    width: u16,
    height: u16,
}

impl RenderTestHarness {
    /// Create a new render test harness with default terminal size
    pub fn new() -> Self {
        Self::with_size(80, 24)
    }

    /// Create a render test harness with specific terminal dimensions
    pub fn with_size(width: u16, height: u16) -> Self {
        Self { width, height }
    }

    /// Render a UI state and return the visual output as a string
    pub fn render(&self, ui_state: &mut UiState, review_engine: &ReviewEngine) -> Result<String> {
        let mut terminal = Terminal::new(TestBackend::new(self.width, self.height))?;

        terminal.draw(|f| {
            ui::draw(f, ui_state, review_engine);
        })?;

        let buffer = terminal.backend().buffer();
        Ok(buffer_to_string(buffer, self.width, self.height))
    }
}

impl Default for RenderTestHarness {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a ratatui buffer to a string representation for testing
fn buffer_to_string(buffer: &Buffer, width: u16, height: u16) -> String {
    let mut output = String::new();

    for y in 0..height {
        let mut line = String::new();
        for x in 0..width {
            let cell = &buffer[(x, y)];
            line.push_str(cell.symbol());
        }
        // Remove trailing whitespace from lines but preserve newlines
        let trimmed = line.trim_end_matches(' ');
        output.push_str(trimmed);
        output.push('\n');
    }

    // Remove trailing newlines
    output.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_harness::StateSnapshot;
    use diffviz_review::providers::mock_provider::MockDiffProvider;
    use diffviz_review::{DiffQuery, GitRef, ReviewEngineBuilder};

    fn create_test_engine() -> ReviewEngine {
        let mock_provider =
            MockDiffProvider::from_review_fixtures().expect("Failed to load test fixtures");
        let review_engine_builder =
            ReviewEngineBuilder::new(Box::new(mock_provider), "test-user".to_string());
        let diff_query = DiffQuery::new(GitRef::Head, GitRef::Unstaged);
        review_engine_builder
            .build(diff_query)
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

        // Should have multiple lines of output
        assert!(lines.len() > 0, "Output should contain at least one line");
    }
}
