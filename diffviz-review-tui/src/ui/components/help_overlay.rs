//! Help overlay component showing all keybindings
//!
//! Displays a comprehensive reference of all available keybindings when '?' is pressed.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::{state::UiState, theme::Colors};

pub fn render(f: &mut Frame, ui_state: &UiState) {
    if !ui_state.show_help {
        return;
    }

    let area = centered_popup(f.area(), 80, 80);

    f.render_widget(Clear, area);

    let content = create_help_content();

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Keybindings Help (? to close)")
        .border_style(Style::default().fg(Colors::CYAN));

    let paragraph = Paragraph::new(content)
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn create_help_content() -> Vec<Line<'static>> {
    vec![
        Line::from(""),
        create_section_header("DECISION TREE NAVIGATION"),
        create_help_line("j/k or ↓/↑", "Navigate decisions in tree"),
        create_help_line("Tab", "Expand/collapse selected decision"),
        create_help_line("Enter", "Expand decision or drill into files"),
        Line::from(""),
        create_section_header("PANE NAVIGATION"),
        create_help_line("← (left)", "Focus left pane (decision tree)"),
        create_help_line("→ (right)", "Focus right pane (code view)"),
        create_help_line("h/l", "Same as arrow keys (vim-style)"),
        Line::from(""),
        create_section_header("SCROLLING & NAVIGATION"),
        create_help_line("g/G", "Jump to top/bottom of list"),
        create_help_line("Ctrl+u/Ctrl+d", "Page up/down"),
        create_help_line("PageUp/PageDown", "Page up/down"),
        create_help_line("Ctrl+j/k", "Scroll inactive panel"),
        Line::from(""),
        create_section_header("VISUAL & SELECTION"),
        create_help_line("v", "Toggle visual selection mode"),
        Line::from(""),
        create_section_header("DECISION CONTEXT"),
        create_help_line("Space d", "Show decision modal"),
        Line::from(""),
        create_section_header("REVIEW ACTIONS (Space + key)"),
        create_help_line("Space a a", "Toggle approve current diff"),
        create_help_line("Space a f", "Approve all diffs in file"),
        Line::from(""),
        create_help_line("Space c c", "Add comment"),
        create_help_line("Space c t", "Toggle comments overlay"),
        Line::from(""),
        create_help_line("Space i i", "Add instruction"),
        create_help_line("Space i t", "Toggle instructions overlay"),
        create_help_line("Space i e", "Edit instruction"),
        Line::from(""),
        create_help_line("Space t s", "Toggle semantic highlighting"),
        create_help_line("Space t c", "Toggle context folding"),
        create_help_line("Space t r", "Toggle reasoning annotations"),
        Line::from(""),
        create_help_line("Space e a", "Export all"),
        Line::from(""),
        create_section_header("UTILITY"),
        create_help_line("?", "Toggle this help overlay"),
        create_help_line("q", "Quit application"),
        create_help_line("Esc", "Cancel / close overlay"),
        Line::from(""),
    ]
}

fn create_section_header(text: &str) -> Line<'static> {
    Line::from(vec![Span::styled(
        format!("  {text}"),
        Style::default()
            .fg(Colors::CYAN)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
    )])
}

fn create_help_line(keys: &str, description: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled("    ", Style::default()),
        Span::styled(format!("{keys:<20}"), Style::default().fg(Colors::YELLOW)),
        Span::styled(description.to_string(), Style::default().fg(Colors::WHITE)),
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
