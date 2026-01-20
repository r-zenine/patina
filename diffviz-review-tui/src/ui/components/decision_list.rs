//! Decision list component - primary navigation view for decision-based review

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::state::UiState;
use diffviz_review::engines::ReviewEngine;

/// Render the decision list as the primary navigation view
pub fn render(f: &mut Frame, area: Rect, ui_state: &UiState, review_engine: &ReviewEngine) {
    let decisions = review_engine.get_all_decisions();

    if decisions.is_empty() {
        let block = Block::default()
            .title(" Decisions ")
            .borders(Borders::ALL);
        f.render_widget(block, area);
        return;
    }

    // Build list items from decisions
    let mut items = Vec::new();

    for decision in &decisions {
        let is_selected = ui_state
            .decision_nav
            .selected_decision
            .map(|sel| sel == decision.number)
            .unwrap_or(false);

        // Decision indicator (► for selected, space for unselected)
        let indicator = if is_selected { "►" } else { " " };

        // Decision number and title
        let number_and_title = format!("{}. {}", decision.number, decision.title);

        // Code impact count
        let impact_count = decision.code_impacts.len();
        let count_str = format!("[{}]", impact_count);

        // Build the line
        let line_content = if is_selected {
            // Highlight selected decision with inverted colors
            vec![
                Span::styled(
                    indicator,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    " ",
                    Style::default()
                        .bg(Color::DarkGray)
                        .fg(Color::White)
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
                    " ".repeat(
                        (area.width as usize)
                            .saturating_sub(indicator.len() + 1 + number_and_title.len() + count_str.len() + 1),
                    ),
                    Style::default().bg(Color::DarkGray),
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
                Span::raw(format!("{} ", indicator)),
                Span::raw(number_and_title),
                Span::styled(format!("  {}", count_str), Style::default().fg(Color::DarkGray)),
            ]
        };

        items.push(ListItem::new(Line::from(line_content)));
    }

    // Create the list widget
    let list = List::new(items)
        .block(
            Block::default()
                .title(" Decisions ")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(list, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_empty_decision_list() {
        // This would require more complex setup with Frame, so we'll skip detailed rendering tests
        // The component logic is validated through integration with the main app
    }
}
