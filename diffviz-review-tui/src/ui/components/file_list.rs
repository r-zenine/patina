//! File list component with collapsible file groups

use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};
use std::collections::BTreeMap;

use crate::{
    state::{FocusPanel, UiState},
    theme::{Colors, Icons, Styles},
};
use diffviz_review::engines::ReviewEngine;
use diffviz_review::ReviewableDiffId;

/// Render the file list with collapsible groups
pub fn render(f: &mut Frame, area: Rect, ui_state: &mut UiState, review_engine: &ReviewEngine) {
    let reviewable_ids = review_engine.get_ordered_reviewable_ids();
    let is_focused = matches!(ui_state.focused_panel, FocusPanel::FileList);

    // Group ReviewableDiffs by file path
    let mut files: BTreeMap<String, Vec<&ReviewableDiffId>> = BTreeMap::new();
    for &id in &reviewable_ids {
        if let Some(diff) = review_engine.get_reviewable_diff(id) {
            files.entry(diff.file_path.clone()).or_default().push(id);
        }
    }

    // Build list items
    let mut items = Vec::new();
    let mut selection_map = Vec::new(); // Maps list index to (file_path, Option<reviewable_id>)

    for (file_path, diff_ids) in &files {
        // Calculate stats for this file
        let total_count = diff_ids.len();
        let approved_count = diff_ids
            .iter()
            .filter(|&&id| review_engine.state().is_approved(id))
            .count();

        let is_expanded = ui_state.is_file_expanded(file_path);

        // Build file header
        let expand_icon = if is_expanded { "▼" } else { "▶" };

        // Count instructions
        let instruction_count: usize = diff_ids
            .iter()
            .filter_map(|&id| review_engine.state().get_instructions(id))
            .map(|instructions| instructions.len())
            .sum();

        // Build indicators with counts
        let mut indicators = String::new();
        if instruction_count > 0 {
            indicators.push_str(&format!(
                " {} {}",
                Icons::INSTRUCTION_MODE,
                instruction_count
            ));
        }

        // Style based on completion
        let file_style = if approved_count == total_count {
            Style::default().fg(Colors::GREEN)
        } else if approved_count > 0 {
            Style::default().fg(Colors::YELLOW)
        } else {
            Style::default().fg(Colors::WHITE)
        };

        // Add file header line
        let file_line = Line::from(vec![
            Span::styled(format!("{expand_icon} "), Style::default().fg(Colors::CYAN)),
            Span::styled(
                format!("{} ", get_file_name(file_path)),
                file_style.add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("[{approved_count}/{total_count}]"),
                Style::default().fg(Colors::TEXT_MUTED),
            ),
            Span::styled(indicators, Style::default()),
        ]);

        items.push(ListItem::new(file_line));
        selection_map.push((file_path.clone(), None));

        // Add individual diffs if expanded
        if is_expanded {
            for &diff_id in diff_ids {
                if let Some(diff) = review_engine.get_reviewable_diff(diff_id) {
                    let is_approved = review_engine.state().is_approved(diff_id);
                    let has_instruction = review_engine.state().has_instructions(diff_id);

                    let status_icon = if is_approved {
                        Icons::APPROVED
                    } else {
                        Icons::NOT_APPROVED
                    };

                    let mut item_indicators = String::new();
                    if has_instruction {
                        item_indicators.push_str(&format!(" {}", Icons::INSTRUCTION_MODE));
                    }

                    // Extract semantic name if available
                    let semantic_name = extract_semantic_name(diff);

                    let diff_style = if is_approved {
                        Style::default().fg(Colors::GREEN)
                    } else {
                        Style::default().fg(Colors::WHITE)
                    };

                    let diff_line = Line::from(vec![
                        Span::raw("  "),
                        Span::styled(format!("{status_icon} "), diff_style),
                        Span::styled(
                            format!(
                                "L{}-{}",
                                diff.id.line_range.start_line, diff.id.line_range.end_line
                            ),
                            Style::default().fg(Colors::TEXT_MUTED),
                        ),
                        Span::styled(format!(" {semantic_name}"), diff_style),
                        Span::styled(item_indicators, Style::default()),
                    ]);

                    items.push(ListItem::new(diff_line));
                    selection_map.push((file_path.clone(), Some(diff_id)));
                }
            }
        }
    }

    // Create the list widget
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Files")
                .border_style(if is_focused {
                    Styles::focused_border()
                } else {
                    Styles::border()
                }),
        )
        .style(Styles::primary())
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    // Create list state for selection
    let mut list_state = ListState::default();
    list_state.select(Some(
        ui_state
            .file_list_selection
            .min(selection_map.len().saturating_sub(1)),
    ));

    // Store selection map for navigation (we'll need this for handling enter key)
    // For now, we'll handle it through the selection index

    // Render the list
    f.render_stateful_widget(list, area, &mut list_state);
}

/// Extract just the file name from a full path
fn get_file_name(path: &str) -> &str {
    path.split('/').next_back().unwrap_or(path)
}

/// Extract semantic name from ReviewableDiff if available
fn extract_semantic_name(_diff: &diffviz_review::ReviewableDiff) -> String {
    // TODO: Once ReviewableDiff includes semantic info, extract function/class name
    // For now, just return a placeholder or empty string
    // TODO: Extract semantic name from diff metadata when available
    String::new()
}
