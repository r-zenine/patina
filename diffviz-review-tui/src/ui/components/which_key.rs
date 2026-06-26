//! Which-key style overlay component for leader key menu
//!
//! Displays available commands when the leader key is active, with timeout indicator.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};
use tui_design::{Theme, stylesheet};

use crate::state::UiState;

/// Render the which-key overlay when leader is active (Spacemacs-style bottom panel)
pub fn render(f: &mut Frame, ui_state: &UiState) {
    if !ui_state.leader_active {
        return;
    }

    let theme = Theme::mocha();
    let area = bottom_panel(f.area());

    f.render_widget(Clear, area);

    let content = match ui_state.leader_submenu {
        None => create_root_menu(ui_state, &theme),
        Some('a') => create_actions_submenu(ui_state, &theme),
        Some('i') => create_instructions_submenu(&theme),
        Some('t') => create_toggles_submenu(&theme),
        _ => create_root_menu(ui_state, &theme),
    };

    let title = match ui_state.leader_submenu {
        None => " Leader Menu (Space)",
        Some('a') => " Actions (Space + a)",
        Some('i') => " Instructions (Space + i)",
        Some('t') => " Toggles (Space + t)",
        _ => " Leader Menu",
    };

    let timeout_text = if let Some(remaining) = ui_state.leader_timeout_remaining() {
        format!(" [{}s] ", remaining.as_secs())
    } else {
        " ".to_string()
    };

    let block = Block::default()
        .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
        .title(format!("{title}{timeout_text}"))
        .border_style(stylesheet::border_focused(&theme));

    let paragraph = Paragraph::new(content).block(block);
    f.render_widget(paragraph, area);
}

fn create_root_menu(_ui_state: &UiState, theme: &Theme) -> Vec<Line<'static>> {
    vec![
        Line::from(""),
        create_compact_line(vec![
            ("a", "Actions"),
            ("i", "Instructions"),
            ("t", "Toggles"),
        ], theme),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("Esc", stylesheet::keybind_key(theme)),
            Span::styled(" cancel", stylesheet::muted(theme)),
        ]),
    ]
}

fn create_actions_submenu(ui_state: &UiState, theme: &Theme) -> Vec<Line<'static>> {
    let mut items = vec![("a", "Approve diff"), ("f", "Approve file")];

    if ui_state.decision_tree.selected_path.depth() == 0 {
        items.push(("d", "Approve decision"));
    }

    vec![
        Line::from(""),
        create_compact_line(items, theme),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("Esc", stylesheet::keybind_key(theme)),
            Span::styled(" back", stylesheet::muted(theme)),
        ]),
    ]
}

fn create_instructions_submenu(theme: &Theme) -> Vec<Line<'static>> {
    vec![
        Line::from(""),
        create_compact_line(vec![("i", "Add instruction"), ("t", "Toggle view")], theme),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("Esc", stylesheet::keybind_key(theme)),
            Span::styled(" back", stylesheet::muted(theme)),
        ]),
    ]
}

fn create_toggles_submenu(theme: &Theme) -> Vec<Line<'static>> {
    vec![
        Line::from(""),
        create_compact_line(vec![
            ("s", "Semantic highlighting"),
            ("c", "Context folding"),
            ("r", "Reasoning annotations"),
        ], theme),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("Esc", stylesheet::keybind_key(theme)),
            Span::styled(" back", stylesheet::muted(theme)),
        ]),
    ]
}

fn create_compact_line(items: Vec<(&str, &str)>, theme: &Theme) -> Line<'static> {
    let mut spans = vec![Span::raw("  ")];

    for (i, (key, description)) in items.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("    "));
        }

        spans.push(Span::styled(
            format!("[{key}]"),
            stylesheet::keybind_key(theme).add_modifier(Modifier::BOLD),
        ));

        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            description.to_string(),
            stylesheet::body(theme),
        ));
    }

    Line::from(spans)
}

fn bottom_panel(r: Rect) -> Rect {
    let height = 10;

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(height),
        ])
        .split(r);

    layout[1]
}
