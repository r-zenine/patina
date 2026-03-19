//! Decision detail panel component - shows decision context inline in diff view

use ratatui::{
    Frame,
    layout::Rect,
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::state::UiState;
use crate::theme::{Icons, Styles};
use diffviz_review::engines::ReviewEngine;

/// Render decision details inline in the diff view panel when a decision is selected (depth 0)
pub fn render(
    f: &mut Frame,
    area: Rect,
    ui_state: &UiState,
    review_engine: &ReviewEngine,
    is_focused: bool,
) {
    let decision_number = match ui_state.decision_tree.selected_decision_number() {
        Some(num) => num,
        None => {
            render_no_decision_selected(f, area, is_focused);
            return;
        }
    };

    let decision = match review_engine.get_decision(decision_number) {
        Some(d) => d,
        None => {
            render_decision_not_found(f, area, decision_number as usize, is_focused);
            return;
        }
    };

    // Build content lines
    let mut lines = Vec::new();

    // Title with decision number and approval status
    let is_approved = review_engine.is_decision_approved(decision_number);
    let approval_icon = if is_approved {
        Icons::APPROVED
    } else {
        Icons::NOT_APPROVED
    };

    let (approved_count, total_count) = review_engine
        .state()
        .decision_approval_progress(decision_number);
    let progress_str = format!("({approved_count}/{total_count})");

    lines.push(Line::from(vec![
        Span::styled(
            approval_icon,
            if is_approved {
                Styles::success()
            } else {
                Styles::muted()
            },
        ),
        Span::raw(" "),
        Span::styled(
            format!("Decision {}: {}", decision.number, decision.title),
            Styles::info().add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(progress_str, Styles::muted()),
    ]));

    lines.push(Line::from("")); // Spacer

    // Rationale (optional)
    if let Some(rationale) = &decision.rationale {
        lines.push(Line::from(vec![Span::styled(rationale, Styles::primary())]));
        lines.push(Line::from("")); // Spacer
    }

    // Code impacts summary
    let impact_count = decision.code_impacts.len();
    let file_count = decision
        .code_impacts
        .iter()
        .map(|impact| &impact.file)
        .collect::<std::collections::HashSet<_>>()
        .len();

    lines.push(Line::from(vec![Span::styled(
        format!(
            "This decision affects {} file{} with {} code impact{}",
            file_count,
            if file_count == 1 { "" } else { "s" },
            impact_count,
            if impact_count == 1 { "" } else { "s" }
        ),
        Styles::muted(),
    )]));

    lines.push(Line::from("")); // Spacer

    // Code impacts detail section
    if !decision.code_impacts.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "Code Impacts:",
            Styles::success().add_modifier(Modifier::BOLD),
        )]));

        lines.push(Line::from("")); // Spacer

        // List each code impact
        for (idx, impact) in decision.code_impacts.iter().enumerate() {
            // File and line ranges
            let mut range_strs = Vec::new();
            for range in &impact.line_ranges {
                range_strs.push(format!("{}-{}", range.start, range.end));
            }
            let ranges = range_strs.join(", ");

            lines.push(Line::from(vec![
                Span::styled("  ► ", Styles::info()),
                Span::styled(&impact.file, Styles::primary()),
                Span::styled(format!(" (lines {ranges})"), Styles::muted()),
            ]));

            // Reasoning
            if !impact.reasoning.is_empty() {
                lines.push(Line::from(vec![Span::styled(
                    format!("    {}", impact.reasoning),
                    Styles::muted(),
                )]));
            }

            // Add space between impacts (except after last one)
            if idx < decision.code_impacts.len() - 1 {
                lines.push(Line::from(""));
            }
        }
    } else {
        lines.push(Line::from(vec![Span::styled(
            "  (no code impacts - architectural decision)",
            Styles::muted(),
        )]));
    }

    // Create paragraph with appropriate border style
    let border_style = if is_focused {
        Styles::border_focused()
    } else {
        Styles::border()
    };

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Decision Details")
                .border_style(border_style),
        )
        .wrap(Wrap { trim: true })
        .style(Styles::primary());

    f.render_widget(paragraph, area);
}

/// Render placeholder when no decision is selected
fn render_no_decision_selected(f: &mut Frame, area: Rect, is_focused: bool) {
    let border_style = if is_focused {
        Styles::border_focused()
    } else {
        Styles::border()
    };

    let lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled("No decision selected", Styles::muted())]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Navigate to a decision in the tree to view details",
            Styles::muted(),
        )]),
    ];

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Decision Details")
                .border_style(border_style),
        )
        .style(Styles::primary());

    f.render_widget(paragraph, area);
}

/// Render error state when decision is not found
fn render_decision_not_found(f: &mut Frame, area: Rect, decision_number: usize, is_focused: bool) {
    let border_style = if is_focused {
        Styles::border_focused()
    } else {
        Styles::border()
    };

    let lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("Decision {decision_number} not found"),
            Styles::error(),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "The selected decision may have been removed",
            Styles::muted(),
        )]),
    ];

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Decision Details")
                .border_style(border_style),
        )
        .style(Styles::primary());

    f.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_render_decision_details_panel() {
        // Rendering tests require Frame setup; validation is done through integration tests
    }
}
