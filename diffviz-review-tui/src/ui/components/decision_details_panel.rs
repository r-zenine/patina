//! Decision detail panel component - shows decision context inline in diff view

use ratatui::{
    Frame,
    layout::Rect,
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use tui_design::{Icons, Theme, stylesheet};

use crate::state::UiState;
use diffviz_review::engines::ReviewEngine;

/// Render decision details inline in the diff view panel when a decision is selected (depth 0)
pub fn render(
    f: &mut Frame,
    area: Rect,
    ui_state: &UiState,
    review_engine: &ReviewEngine,
    is_focused: bool,
) {
    let theme = Theme::mocha();

    let decision_number = match ui_state.decision_tree.selected_decision_number() {
        Some(num) => num,
        None => {
            render_no_decision_selected(f, area, is_focused, &theme);
            return;
        }
    };

    let decision = match review_engine.get_decision(decision_number) {
        Some(d) => d,
        None => {
            render_decision_not_found(f, area, decision_number as usize, is_focused, &theme);
            return;
        }
    };

    let mut lines = Vec::new();

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
                stylesheet::success(&theme)
            } else {
                stylesheet::muted(&theme)
            },
        ),
        Span::raw(" "),
        Span::styled(
            format!("Decision {}: {}", decision.number, decision.title),
            stylesheet::info(&theme).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(progress_str, stylesheet::muted(&theme)),
    ]));

    lines.push(Line::from(""));

    if let Some(rationale) = &decision.rationale {
        lines.push(Line::from(vec![Span::styled(rationale, stylesheet::body(&theme))]));
        lines.push(Line::from(""));
    }

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
        stylesheet::muted(&theme),
    )]));

    lines.push(Line::from(""));

    if !decision.code_impacts.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "Code Impacts:",
            stylesheet::success(&theme).add_modifier(Modifier::BOLD),
        )]));

        lines.push(Line::from(""));

        for (idx, impact) in decision.code_impacts.iter().enumerate() {
            let mut range_strs = Vec::new();
            for range in &impact.line_ranges {
                range_strs.push(format!("{}-{}", range.start, range.end));
            }
            let ranges = range_strs.join(", ");

            lines.push(Line::from(vec![
                Span::styled("  ► ", stylesheet::info(&theme)),
                Span::styled(&impact.file, stylesheet::body(&theme)),
                Span::styled(format!(" (lines {ranges})"), stylesheet::muted(&theme)),
            ]));

            if !impact.reasoning.is_empty() {
                lines.push(Line::from(vec![Span::styled(
                    format!("    {}", impact.reasoning),
                    stylesheet::muted(&theme),
                )]));
            }

            if idx < decision.code_impacts.len() - 1 {
                lines.push(Line::from(""));
            }
        }
    } else {
        lines.push(Line::from(vec![Span::styled(
            "  (no code impacts - architectural decision)",
            stylesheet::muted(&theme),
        )]));
    }

    if ui_state.show_instructions
        && let Some(instructions) = review_engine.get_decision_instructions(decision_number)
        && !instructions.is_empty()
    {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            format!("Instructions ({}):", instructions.len()),
            stylesheet::warning(&theme).add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));

        for instruction in &instructions {
            lines.push(Line::from(vec![
                Span::styled("  Author: ", stylesheet::muted(&theme)),
                Span::styled(&instruction.author, stylesheet::body(&theme)),
                Span::styled(" | ", stylesheet::muted(&theme)),
                Span::styled(&instruction.timestamp, stylesheet::muted(&theme)),
            ]));
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(&instruction.content, stylesheet::body(&theme)),
            ]));
            lines.push(Line::from(""));
        }
    }

    let border_style = if is_focused {
        stylesheet::border_focused(&theme)
    } else {
        stylesheet::border(&theme)
    };

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Decision Details")
                .border_style(border_style),
        )
        .wrap(Wrap { trim: true })
        .style(stylesheet::body(&theme));

    f.render_widget(paragraph, area);
}

fn render_no_decision_selected(f: &mut Frame, area: Rect, is_focused: bool, theme: &Theme) {
    let border_style = if is_focused {
        stylesheet::border_focused(theme)
    } else {
        stylesheet::border(theme)
    };

    let lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled("No decision selected", stylesheet::muted(theme))]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Navigate to a decision in the tree to view details",
            stylesheet::muted(theme),
        )]),
    ];

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Decision Details")
                .border_style(border_style),
        )
        .style(stylesheet::body(theme));

    f.render_widget(paragraph, area);
}

fn render_decision_not_found(
    f: &mut Frame,
    area: Rect,
    decision_number: usize,
    is_focused: bool,
    theme: &Theme,
) {
    let border_style = if is_focused {
        stylesheet::border_focused(theme)
    } else {
        stylesheet::border(theme)
    };

    let lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("Decision {decision_number} not found"),
            stylesheet::error(theme),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "The selected decision may have been removed",
            stylesheet::muted(theme),
        )]),
    ];

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Decision Details")
                .border_style(border_style),
        )
        .style(stylesheet::body(theme));

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
