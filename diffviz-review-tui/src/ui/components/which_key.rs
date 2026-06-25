//! Which-key style overlay component for leader key menu
//!
//! Displays available commands when the leader key is active, with timeout indicator.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::{
    state::UiState,
    theme::{Colors, Styles},
};

/// Render the which-key overlay when leader is active (Spacemacs-style bottom panel)
pub fn render(f: &mut Frame, ui_state: &UiState) {
    if !ui_state.leader_active {
        return;
    }

    let area = bottom_panel(f.area());

    // Clear the area
    f.render_widget(Clear, area);

    let content = match ui_state.leader_submenu {
        None => create_root_menu(ui_state),
        Some('a') => create_actions_submenu(ui_state),
        Some('i') => create_instructions_submenu(),
        Some('t') => create_toggles_submenu(),
        _ => create_root_menu(ui_state),
    };

    let title = match ui_state.leader_submenu {
        None => " Leader Menu (Space)",
        Some('a') => " Actions (Space + a)",
        Some('i') => " Instructions (Space + i)",
        Some('t') => " Toggles (Space + t)",
        _ => " Leader Menu",
    };

    // Add timeout indicator
    let timeout_text = if let Some(remaining) = ui_state.leader_timeout_remaining() {
        format!(" [{}s] ", remaining.as_secs())
    } else {
        " ".to_string()
    };

    let block = Block::default()
        .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
        .title(format!("{title}{timeout_text}"))
        .border_style(Style::default().fg(Colors::CYAN));

    let paragraph = Paragraph::new(content).block(block);
    f.render_widget(paragraph, area);
}

fn create_root_menu(_ui_state: &UiState) -> Vec<Line<'static>> {
    vec![
        Line::from(""),
        create_compact_line(vec![
            ("a", "Actions"),
            ("i", "Instructions"),
            ("t", "Toggles"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled("Esc", Style::default().fg(Colors::CYAN)),
            Span::styled(" cancel", Styles::muted()),
        ]),
    ]
}

fn create_actions_submenu(ui_state: &UiState) -> Vec<Line<'static>> {
    let mut items = vec![("a", "Approve diff"), ("f", "Approve file")];

    // Add decision approval option if at depth 0
    if ui_state.decision_tree.selected_path.depth() == 0 {
        items.push(("d", "Approve decision"));
    }

    vec![
        Line::from(""),
        create_compact_line(items),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled("Esc", Style::default().fg(Colors::CYAN)),
            Span::styled(" back", Styles::muted()),
        ]),
    ]
}

fn create_instructions_submenu() -> Vec<Line<'static>> {
    vec![
        Line::from(""),
        create_compact_line(vec![("i", "Add instruction"), ("t", "Toggle view")]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled("Esc", Style::default().fg(Colors::CYAN)),
            Span::styled(" back", Styles::muted()),
        ]),
    ]
}

fn create_toggles_submenu() -> Vec<Line<'static>> {
    vec![
        Line::from(""),
        create_compact_line(vec![
            ("s", "Semantic highlighting"),
            ("c", "Context folding"),
            ("r", "Reasoning annotations"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled("Esc", Style::default().fg(Colors::CYAN)),
            Span::styled(" back", Styles::muted()),
        ]),
    ]
}

/// Create a compact horizontal line with multiple key bindings (Spacemacs-style)
fn create_compact_line(items: Vec<(&str, &str)>) -> Line<'static> {
    let mut spans = vec![Span::styled("  ", Style::default())];

    for (i, (key, description)) in items.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("    ", Style::default()));
        }

        // Key in brackets with bold cyan
        spans.push(Span::styled(
            format!("[{key}]"),
            Style::default()
                .fg(Colors::CYAN)
                .add_modifier(Modifier::BOLD),
        ));

        // Description in white
        spans.push(Span::styled(" ", Style::default()));
        spans.push(Span::styled(
            description.to_string(),
            Style::default().fg(Colors::WHITE),
        ));
    }

    Line::from(spans)
}

/// Create a Spacemacs-style bottom panel that spans the full width
fn bottom_panel(r: Rect) -> Rect {
    // Calculate height based on content - use about 10 lines for the menu
    let height = 10;

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),         // Top area (main content)
            Constraint::Length(height), // Bottom panel for which-key
        ])
        .split(r);

    layout[1]
}
