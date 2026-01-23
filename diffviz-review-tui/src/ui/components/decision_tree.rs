//! Decision tree component - hierarchical view for decision-based review
//!
//! Displays decisions with expandable files and chunks in a tree structure.
//! Tree layout: Decision (level 0) → Files (level 1) → Chunks (level 2)

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::decision_navigation::FlattenedNodeKind;
use crate::state::UiState;
use crate::theme::Icons;
use diffviz_review::engines::ReviewEngine;

/// Render the decision tree as the primary navigation view
pub fn render(f: &mut Frame, area: Rect, ui_state: &UiState, review_engine: &ReviewEngine) {
    // Flatten the tree for rendering
    let flattened = ui_state.decision_tree.flatten();

    if flattened.is_empty() {
        let block = Block::default().title(" Decisions ").borders(Borders::ALL);
        f.render_widget(block, area);
        return;
    }

    // Build list items from flattened nodes
    let mut items = Vec::new();
    for node in &flattened {
        let is_selected = node.path == ui_state.decision_tree.selected_path;

        let item_line = match &node.kind {
            FlattenedNodeKind::Decision { number, expanded } => {
                build_decision_item(*number, *expanded, is_selected, review_engine, area.width)
            }
            FlattenedNodeKind::File {
                decision_num,
                path,
                expanded: _,
            } => build_file_item(*decision_num, path, is_selected, review_engine, area.width),
            FlattenedNodeKind::Chunk {
                decision_num: _,
                file_path: _,
                chunk_id,
            } => build_chunk_item(chunk_id, is_selected),
        };

        items.push(ListItem::new(item_line));
    }

    // Create the list widget
    let list = List::new(items)
        .block(Block::default().title(" Decisions ").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    f.render_widget(list, area);
}

/// Build a line for a decision tree node
fn build_decision_item<'a>(
    decision_num: u32,
    expanded: bool,
    is_selected: bool,
    review_engine: &'a ReviewEngine,
    area_width: u16,
) -> Line<'a> {
    let decision = review_engine.get_decision(decision_num).unwrap();

    // Expansion indicator
    let expand_indicator = if expanded { "▼" } else { "▶" };

    // Decision indicator (► for selected, space for unselected)
    let selection_indicator = if is_selected { "►" } else { " " };

    // Approval status
    let is_approved = review_engine.is_decision_approved(decision_num);
    let approval_icon = if is_approved {
        Icons::APPROVED
    } else {
        Icons::NOT_APPROVED
    };

    // Decision number and title
    let number_and_title = format!("{}. {}", decision.number, decision.title);

    // Progress indicator: approved/total chunks
    let (approved_count, total_count) = review_engine
        .state()
        .decision_approval_progress(decision_num);
    let progress_str = format!("({approved_count}/{total_count})");

    // Code impact count
    let impact_count = decision.code_impacts.len();
    let count_str = format!("[{impact_count}]");

    let line_content = if is_selected {
        // Highlight selected decision with inverted colors
        let approval_color = if is_approved {
            Color::Green
        } else {
            Color::DarkGray
        };
        vec![
            Span::styled(
                selection_indicator,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {expand_indicator} "),
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{approval_icon} "),
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(approval_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                number_and_title.clone(),
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " ".repeat((area_width as usize).saturating_sub(
                    selection_indicator.len()
                        + 3
                        + approval_icon.len()
                        + 1
                        + number_and_title.len()
                        + progress_str.len()
                        + count_str.len()
                        + 2,
                )),
                Style::default().bg(Color::DarkGray),
            ),
            Span::styled(
                format!("{progress_str} "),
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::DIM),
            ),
            Span::styled(
                count_str,
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::DIM),
            ),
        ]
    } else {
        vec![
            Span::raw(format!("{selection_indicator} {expand_indicator} ")),
            Span::styled(
                format!("{approval_icon} "),
                Style::default().fg(if is_approved {
                    Color::Green
                } else {
                    Color::DarkGray
                }),
            ),
            Span::raw(number_and_title),
            Span::styled(
                format!(" {progress_str} "),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!(" {count_str}"),
                Style::default().fg(Color::DarkGray),
            ),
        ]
    };

    Line::from(line_content)
}

/// Build a line for a file tree node
fn build_file_item<'a>(
    decision_num: u32,
    file_path: &'a str,
    is_selected: bool,
    review_engine: &'a ReviewEngine,
    area_width: u16,
) -> Line<'a> {
    // Get line ranges for this file in this decision
    let decision = review_engine.get_decision(decision_num).unwrap();
    let mut line_ranges = Vec::new();
    for impact in &decision.code_impacts {
        if impact.file == file_path {
            for range in &impact.line_ranges {
                line_ranges.push(format!("{}-{}", range.start, range.end));
            }
        }
    }

    let selection_indicator = if is_selected { "►" } else { " " };
    let indent = "  ";

    // File name and line ranges
    let file_name = file_path.split('/').next_back().unwrap_or(file_path);
    let lines_str = format!("(L{})", line_ranges.join(","));

    let line_content = if is_selected {
        vec![
            Span::styled(
                selection_indicator,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{indent}{file_name}"),
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " ".repeat((area_width as usize).saturating_sub(
                    selection_indicator.len()
                        + indent.len()
                        + file_name.len()
                        + lines_str.len()
                        + 1,
                )),
                Style::default().bg(Color::DarkGray),
            ),
            Span::styled(
                lines_str,
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::DIM),
            ),
        ]
    } else {
        vec![
            Span::raw(format!("{selection_indicator}{indent}")),
            Span::raw(file_name),
            Span::styled(
                format!("  {lines_str}"),
                Style::default().fg(Color::DarkGray),
            ),
        ]
    };

    Line::from(line_content)
}

/// Build a line for a chunk tree node
fn build_chunk_item(chunk_id: &diffviz_review::ReviewableDiffId, is_selected: bool) -> Line {
    let selection_indicator = if is_selected { "►" } else { " " };
    let indent = "    ";

    let chunk_repr = format!(
        "L{}-{}",
        chunk_id.line_range.start_line, chunk_id.line_range.end_line
    );

    let line_content = if is_selected {
        vec![Span::styled(
            format!("{selection_indicator}{indent}{chunk_repr}"),
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]
    } else {
        vec![Span::raw(format!(
            "{selection_indicator}{indent}{chunk_repr}"
        ))]
    };

    Line::from(line_content)
}
