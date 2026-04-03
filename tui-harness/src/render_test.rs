//! Generic render test harness.
//!
//! Render any ELMApp using ratatui's TestBackend and capture visual output as text.

use ratatui::{Terminal, backend::TestBackend, buffer::Buffer};

use crate::{Result, traits::ELMApp};

/// Headless test harness for validating UI rendering.
pub struct RenderTestHarness {
    width: u16,
    height: u16,
}

impl RenderTestHarness {
    /// Create a render test harness with default terminal size (80×24).
    pub fn new() -> Self {
        Self::with_size(80, 24)
    }

    /// Create a render test harness with specific terminal dimensions.
    pub fn with_size(width: u16, height: u16) -> Self {
        Self { width, height }
    }

    /// Render an ELMApp and return the visual output as a string.
    pub fn render<M: ELMApp>(&self, app: &M) -> Result<String> {
        let mut terminal = Terminal::new(TestBackend::new(self.width, self.height))?;

        terminal.draw(|f| app.draw(f))?;

        let buffer = terminal.backend().buffer();
        Ok(buffer_to_string(buffer, self.width, self.height))
    }
}

impl Default for RenderTestHarness {
    fn default() -> Self {
        Self::new()
    }
}

fn buffer_to_string(buffer: &Buffer, width: u16, height: u16) -> String {
    let mut output = String::new();

    for y in 0..height {
        let mut line = String::new();
        for x in 0..width {
            let cell = &buffer[(x, y)];
            line.push_str(cell.symbol());
        }
        let trimmed = line.trim_end_matches(' ');
        output.push_str(trimmed);
        output.push('\n');
    }

    output.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_harness_creation() {
        let _harness = RenderTestHarness::new();
    }

    #[test]
    fn test_render_with_custom_size() {
        let _harness = RenderTestHarness::with_size(120, 40);
    }
}
