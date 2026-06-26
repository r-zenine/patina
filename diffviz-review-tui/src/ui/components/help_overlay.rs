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
        .border_style(stylesheet::border_focused(&theme));

    let paragraph = Paragraph::new(content)
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn create_help_content(theme: &Theme) -> Vec<Line<'static>> {
    vec![
        Line::from(""),
        create_section_header("DECISION TREE NAVIGATION", theme),
        create_help_line("j/k or ↓/↑", "Navigate decisions in tree", theme),
        create_help_line("Tab", "Expand/collapse selected decision", theme),
        create_help_line("Enter", "Expand decision or drill into files", theme),
        Line::from(""),
        create_section_header("PANE NAVIGATION", theme),
        create_help_line("← (left)", "Focus left pane (decision tree)", theme),
        create_help_line("→ (right)", "Focus right pane (code view)", theme),
        create_help_line("h/l", "Same as arrow keys (vim-style)", theme),
        Line::from(""),
        create_section_header("SCROLLING & NAVIGATION", theme),
        create_help_line("g/G", "Jump to top/bottom of list", theme),
        create_help_line("Ctrl+u/Ctrl+d", "Page up/down", theme),
        create_help_line("PageUp/PageDown", "Page up/down", theme),
        create_help_line("Ctrl+j/k", "Scroll inactive panel", theme),
        Line::from(""),
        create_section_header("VISUAL & SELECTION", theme),
        create_help_line("v", "Toggle visual selection mode", theme),
        Line::from(""),
        create_section_header("DECISION CONTEXT", theme),
        create_help_line("Space d", "Show decision modal", theme),
        Line::from(""),
        create_section_header("REVIEW ACTIONS (Space + key)", theme),
        create_help_line("Space a a", "Toggle approve current diff", theme),
        create_help_line("Space a f", "Approve all diffs in file", theme),
        Line::from(""),
        create_help_line("Space c c", "Add comment", theme),
        create_help_line("Space c t", "Toggle comments overlay", theme),
        Line::from(""),
        create_help_line("Space i i", "Add instruction", theme),
        create_help_line("Space i t", "Toggle instructions overlay", theme),
        create_help_line("Space i e", "Edit instruction", theme),
        Line::from(""),
        create_help_line("Space t s", "Toggle semantic highlighting", theme),
        create_help_line("Space t c", "Toggle context folding", theme),
        create_help_line("Space t r", "Toggle reasoning annotations", theme),
        Line::from(""),
        create_help_line("Space e a", "Export all", theme),
        Line::from(""),
        create_section_header("UTILITY", theme),
        create_help_line("?", "Toggle this help overlay", theme),
        create_help_line("q", "Quit application", theme),
        create_help_line("Esc", "Cancel / close overlay", theme),
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
