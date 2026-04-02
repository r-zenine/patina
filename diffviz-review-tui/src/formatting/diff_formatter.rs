//! TUI formatter for converting RenderableDiff to ratatui components
//!
//! This module is inspired by the formatter in diffviz-core/examples/formatter.rs
//! but adapted for ratatui display instead of terminal ANSI output.

use crate::theme::Styles;
use diffviz_core::{renderable_diff::RenderableDiff, reviewable_diff::ReviewableDiff};
use ratatui::{
    style::Style,
    text::{Line, Span},
};

/// Formatted diff ready for TUI display
#[derive(Debug, Clone)]
pub struct FormattedDiff {
    pub header_lines: Vec<Line<'static>>,
    pub content_lines: Vec<Line<'static>>,
    pub metadata: DiffMetadata,
}

/// Metadata about the formatted diff
#[derive(Debug, Clone)]
pub struct DiffMetadata {
    pub boundary_name: String,
    pub language: String,
    pub total_changes: usize,
    pub total_lines: usize,
    pub visible_lines: usize,
    pub hidden_lines: usize,
}

/// TUI-specific diff formatter
pub struct TuiDiffFormatter {
    pub show_all_context: bool,
}

impl Default for TuiDiffFormatter {
    fn default() -> Self {
        Self {
            show_all_context: true,
        }
    }
}

impl TuiDiffFormatter {
    /// Create a new formatter
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether to show all context lines or fold irrelevant ones
    pub fn with_context_display(mut self, show_all: bool) -> Self {
        self.show_all_context = show_all;
        self
    }

    /// Format a ReviewableDiff for TUI display
    pub fn format_reviewable_diff(&self, reviewable_diff: &ReviewableDiff) -> FormattedDiff {
        let renderable = RenderableDiff::try_from(reviewable_diff)
            .expect("ReviewableDiff should produce valid RenderableDiff");
        self.format_renderable_diff(&renderable)
    }

    /// Format a RenderableDiff for TUI display
    pub fn format_renderable_diff(&self, diff: &RenderableDiff) -> FormattedDiff {
        let mut header_lines = Vec::new();
        let mut content_lines = Vec::new();
        let mut hidden_count = 0;
        let mut visible_count = 0;

        // Create header
        header_lines.push(self.create_header_line(diff));
        header_lines.push(self.create_separator_line());

        // Process diff lines with folding logic
        for line in &diff.lines {
            if !self.show_all_context && line.should_fold() {
                hidden_count += 1;
                continue;
            }

            // Show hidden line indicator if we have accumulated hidden lines
            if hidden_count > 0 {
                content_lines.push(self.create_hidden_lines_indicator(hidden_count));
                hidden_count = 0;
            }

            // Format visible line
            let formatted_line = self.format_diff_line(line);
            content_lines.push(formatted_line);
            visible_count += 1;
        }

        // Handle any remaining hidden lines at the end
        if hidden_count > 0 {
            content_lines.push(self.create_hidden_lines_indicator(hidden_count));
        }

        let metadata = DiffMetadata {
            boundary_name: diff.metadata.boundary_name.clone(),
            language: format!("{:?}", diff.language).to_lowercase(),
            total_changes: diff.metadata.total_changes,
            total_lines: diff.lines.len(),
            visible_lines: visible_count,
            hidden_lines: hidden_count,
        };

        FormattedDiff {
            header_lines,
            content_lines,
            metadata,
        }
    }

    /// Create the header line with diff metadata
    fn create_header_line(&self, diff: &RenderableDiff) -> Line<'static> {
        let content = format!(
            "📦 {} ({}) - {} changes merged",
            diff.metadata.boundary_name,
            format!("{:?}", diff.language).to_lowercase(),
            diff.metadata.total_changes
        );

        Line::from(vec![Span::styled(content, Styles::info())])
    }

    /// Create separator line
    fn create_separator_line(&self) -> Line<'static> {
        let separator = "─".repeat(60);
        Line::from(vec![Span::styled(separator, Styles::border())])
    }

    /// Create indicator for hidden lines
    fn create_hidden_lines_indicator(&self, count: usize) -> Line<'static> {
        let content = format!("  ... {count} lines hidden ...");
        Line::from(vec![Span::styled(content, Styles::muted())])
    }

    /// Format a single diff line
    fn format_diff_line(
        &self,
        line: &diffviz_core::renderable_diff::RenderableLine,
    ) -> Line<'static> {
        Line::from(vec![Span::styled(
            format!("{}: {}", line.line_number, line.content),
            Style::default(),
        )])
    }
}
