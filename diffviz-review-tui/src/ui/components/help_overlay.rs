//! Help overlay component showing all keybindings
//!
//! Displays a comprehensive reference of all available keybindings when '?' is pressed.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use tui_design::{Theme, stylesheet};

use crate::state::UiState;

pub fn render(f: &mut Frame, ui_state: &UiState) {
    if !ui_state.show_help {
        return;
    }

    let theme = Theme::mocha();
    let area = centered_popup(f.area(), 80, 80);

    f.render_widget(Clear, area);

    let content = create_help_content(&theme);

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Keybindings Help (? to close)")
        .border_style(stylesheet::border_focused(&theme))
        .style(stylesheet::layer_elevated(&theme));

    let paragraph = Paragraph::new(content)
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn create_help_content(theme: &Theme) -> Vec<Line<'static>> {
    vec![
        Line::from(""),
        create_section_header("NAVIGATION", theme),
        create_help_line("j/k or ↓/↑", "Browse: decisions · Drill: chunks", theme),
        create_help_line("h/l or ←/→", "Drill: switch sibling file", theme),
        create_help_line("Enter", "Drill into the focused decision", theme),
        create_help_line("Esc", "Back to Browse", theme),
        create_help_line("g/G", "Jump to first/last", theme),
        create_help_line("Ctrl+u/d, PgUp/Dn", "Page up/down", theme),
        Line::from(""),
        create_section_header("CHUNK CARD (Drill)", theme),
        create_help_line("Tab", "Expand/collapse context lines", theme),
        create_help_line("i", "Expand/collapse the chunk's note", theme),
        Line::from(""),
        create_section_header("REVIEW ACTIONS", theme),
        create_help_line("a", "Toggle approve (decision or chunk)", theme),
        create_help_line("n", "Add or append a note", theme),
        Line::from(""),
        create_section_header("LEADER MENU (Space + key)", theme),
        create_help_line("Space a a/d", "Toggle approve chunk/decision", theme),
        create_help_line("Space a f", "Approve all chunks in file", theme),
        create_help_line("Space t r", "Toggle reasoning annotations", theme),
        Line::from(""),
        create_section_header("UTILITY", theme),
        create_help_line("?", "Toggle this help overlay", theme),
        create_help_line("q / Ctrl+c", "Quit application", theme),
        Line::from(""),
    ]
}

fn create_section_header(text: &str, theme: &Theme) -> Line<'static> {
    Line::from(vec![Span::styled(
        format!("  {text}"),
        stylesheet::keybind_key(theme).add_modifier(Modifier::UNDERLINED),
    )])
}

fn create_help_line(keys: &str, description: &str, theme: &Theme) -> Line<'static> {
    Line::from(vec![
        Span::raw("    "),
        Span::styled(format!("{keys:<20}"), stylesheet::warning(theme)),
        Span::styled(description.to_string(), stylesheet::body(theme)),
    ])
}

fn centered_popup(r: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
