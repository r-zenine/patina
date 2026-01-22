//! Decision detail modal component - shows decision context and code impacts

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::state::UiState;
use crate::ui::layout::centered_popup;
use diffviz_review::engines::ReviewEngine;

/// Render the decision detail modal when showing decision context
pub fn render(f: &mut Frame, area: Rect, ui_state: &UiState, review_engine: &ReviewEngine) {
    if !ui_state.decision_tree.show_decision_modal {
        return;
    }

    let decision_number = match ui_state.decision_tree.selected_decision_number() {
        Some(num) => num,
        None => return,
    };

    let decision = match review_engine.get_decision(decision_number) {
        Some(d) => d,
        None => return,
    };

    // Create centered popup area
    let popup_area = centered_popup(area, 75, 70);

    // Build modal content
    let mut lines = Vec::new();

    // Title
    lines.push(Line::from(vec![Span::styled(
        format!("Decision {}: {}", decision.number, decision.title),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]));

    lines.push(Line::from("")); // Spacer

    // Summary
    lines.push(Line::from(vec![Span::raw(&decision.summary)]));
    lines.push(Line::from("")); // Spacer

    // Decision log reference
    if let Some(log_line) = decision.decision_log_line {
        lines.push(Line::from(vec![
            Span::styled("Decision log line: ", Style::default().fg(Color::DarkGray)),
            Span::raw(log_line.to_string()),
        ]));
        lines.push(Line::from("")); // Spacer
    }

    // Code impacts header
    lines.push(Line::from(vec![Span::styled(
        "Code Impacts:",
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    )]));

    if decision.code_impacts.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "  (no code impacts - architectural decision)",
            Style::default().fg(Color::DarkGray),
        )]));
    } else {
        // List each code impact
        for (idx, impact) in decision.code_impacts.iter().enumerate() {
            // File and line ranges
            let mut range_strs = Vec::new();
            for range in &impact.line_ranges {
                range_strs.push(format!("{}-{}", range.start, range.end));
            }
            let ranges = range_strs.join(", ");

            lines.push(Line::from(vec![
                Span::raw(format!("  ► {}", impact.file)),
                Span::styled(
                    format!(" (lines {ranges})"),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));

            // Change type and confidence on next line
            let change_type_str = match impact.change_type {
                diffviz_review::entities::decision::ChangeType::Addition => "Addition",
                diffviz_review::entities::decision::ChangeType::Modification => "Modification",
                diffviz_review::entities::decision::ChangeType::Deletion => "Deletion",
            };

            let confidence_str = match impact.confidence {
                diffviz_review::entities::decision::Confidence::High => "HIGH",
                diffviz_review::entities::decision::Confidence::Medium => "MEDIUM",
                diffviz_review::entities::decision::Confidence::Low => "LOW",
            };

            let confidence_color = match impact.confidence {
                diffviz_review::entities::decision::Confidence::High => Color::Green,
                diffviz_review::entities::decision::Confidence::Medium => Color::Yellow,
                diffviz_review::entities::decision::Confidence::Low => Color::Red,
            };

            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::raw(change_type_str),
                Span::raw(" • "),
                Span::styled(confidence_str, Style::default().fg(confidence_color)),
            ]));

            // Reasoning on next line
            lines.push(Line::from(vec![Span::styled(
                format!("    {}", impact.reasoning),
                Style::default().fg(Color::DarkGray),
            )]));

            // Add space between impacts (except after last one)
            if idx < decision.code_impacts.len() - 1 {
                lines.push(Line::from(""));
            }
        }
    }

    lines.push(Line::from("")); // Spacer before footer

    // Footer with instructions
    lines.push(Line::from(vec![
        Span::styled("[Esc]", Style::default().fg(Color::Cyan)),
        Span::raw(" Close  "),
        Span::styled("[↑↓]", Style::default().fg(Color::Cyan)),
        Span::raw(" Navigate  "),
        Span::styled("[→]", Style::default().fg(Color::Cyan)),
        Span::raw(" View files"),
    ]));

    // Create paragraph with wrapping
    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Black)),
        )
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(Color::White));

    f.render_widget(paragraph, popup_area);
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_render_decision_detail_modal() {
        // Rendering tests require Frame setup; validation is done through integration tests
    }
}
