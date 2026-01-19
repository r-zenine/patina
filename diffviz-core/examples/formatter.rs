//! Git-diff style formatter using RenderableDiff line-based structure
//!
//! This module provides clean, readable formatting by leveraging the RenderableDiff
//! abstraction which converts tree-based semantic analysis to line-based display.

use diffviz_core::{renderable_diff::RenderableDiff, reviewable_diff::ReviewableDiff};

/// ANSI color codes for git-diff style output
pub struct Colors;

impl Colors {
    pub const HEADER: &'static str = "\x1b[96m"; // Bright Cyan
    pub const UNCHANGED: &'static str = "\x1b[37m"; // Light Gray
    pub const RESET: &'static str = "\x1b[0m";
}

/// Format a ReviewableDiff with git-diff style colored output using idiomatic traits
pub fn format_reviewable_diff_with_colors(diff: &ReviewableDiff) -> String {
    // Convert to renderable format using idiomatic Into trait
    let renderable: RenderableDiff = diff.into();

    // Format using the line-based structure
    format_renderable_diff(&renderable)
}

/// Format a RenderableDiff with clean git-diff style output
fn format_renderable_diff(diff: &RenderableDiff) -> String {
    let mut output = String::new();

    // Clean header
    output.push_str(&format!(
        "{}📦 {} ({}) - {} changes merged{}\n",
        Colors::HEADER,
        diff.metadata.boundary_name,
        format!("{:?}", diff.language).to_lowercase(),
        diff.metadata.total_changes,
        Colors::RESET
    ));

    output.push_str(&format!(
        "{}────────────────────────────────────────────────────────────{}\n",
        Colors::UNCHANGED,
        Colors::RESET
    ));

    // Process lines with smart folding
    let mut hidden_count = 0;

    for line in &diff.lines {
        if line.should_fold() {
            hidden_count += 1;
            continue;
        }

        // Show hidden line indicator if we have accumulated hidden lines
        if hidden_count > 0 {
            output.push_str(&format!(
                "{}  ... {} lines hidden ...{}\n",
                Colors::UNCHANGED,
                hidden_count,
                Colors::RESET
            ));
            hidden_count = 0;
        }

        // Format visible line using the line's built-in display style
        let (prefix, color) = line.get_display_style();
        output.push_str(&format!(
            "{}{} {}{}\n",
            color,
            prefix,
            line.content,
            Colors::RESET
        ));
    }

    // Handle any remaining hidden lines at the end
    if hidden_count > 0 {
        output.push_str(&format!(
            "{}  ... {} lines hidden ...{}\n",
            Colors::UNCHANGED,
            hidden_count,
            Colors::RESET
        ));
    }

    output
}

/// Example main function demonstrating the formatter
fn main() {
    println!("Git-diff style formatter example");
    println!("This is a library module for formatting ReviewableDiff instances.");
    println!("See reviewable_diff_demo.rs or boundary_merging_demo.rs for usage examples.");
}
