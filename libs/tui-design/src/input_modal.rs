//! Generic text-input modal chrome: centered popup, title block, cursor
//! rendering. Callers own the domain-specific title/placeholder text and the
//! input buffer/cursor position; this module only draws them.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Clear, Padding, Paragraph, Wrap},
};

use crate::stylesheet;
use crate::tokens::Theme;

/// Render a centered text-input modal: `title` in the block header, `buffer`
/// (with a block cursor at `cursor_pos`) or `placeholder` when empty, and a
/// fixed "Enter: Submit  Esc: Cancel" footer.
pub fn render(
    f: &mut Frame,
    area: Rect,
    title: &str,
    buffer: &str,
    cursor_pos: usize,
    placeholder: &str,
    theme: &Theme,
) {
    let popup_area = centered_popup(area, 70, 40);

    f.render_widget(Clear, popup_area);

    let input_content = create_input_display(buffer, cursor_pos, placeholder, theme);

    let modal = Paragraph::new(input_content)
        .block(
            Block::default()
                .title(format!("  {title}"))
                .padding(Padding::new(2, 2, 1, 1))
                .style(stylesheet::layer_elevated(theme)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(modal, popup_area);

    let help_area = Rect {
        x: popup_area.x + 2,
        y: popup_area.y + popup_area.height - 3,
        width: popup_area.width.saturating_sub(2),
        height: 1,
    };

    let help_text = Line::from(vec![
        Span::styled("Enter: Submit  ", stylesheet::keybind_key(theme)),
        Span::styled("Esc: Cancel", stylesheet::error(theme)),
    ]);

    let help_paragraph = Paragraph::new(help_text).style(stylesheet::muted(theme));
    f.render_widget(help_paragraph, help_area);
}

fn create_input_display(
    buffer: &str,
    cursor_pos: usize,
    placeholder: &str,
    theme: &Theme,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    if buffer.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            placeholder.to_string(),
            stylesheet::muted(theme),
        )]));
        lines.push(Line::from(""));
    } else {
        let input_lines: Vec<&str> = buffer.lines().collect();

        if input_lines.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "█",
                stylesheet::cursor(theme),
            )]));
        } else {
            let mut char_count = 0;
            let mut cursor_line = 0;
            let mut cursor_col = 0;

            for (line_idx, line) in input_lines.iter().enumerate() {
                let line_len = line.len() + 1;
                if char_count + line_len > cursor_pos {
                    cursor_line = line_idx;
                    cursor_col = cursor_pos - char_count;
                    break;
                }
                char_count += line_len;
            }

            for (line_idx, line) in input_lines.iter().enumerate() {
                if line_idx == cursor_line {
                    lines.push(create_line_with_cursor(line, cursor_col, theme));
                } else {
                    lines.push(Line::from(line.to_string()));
                }
            }
        }
    }

    lines
}

fn create_line_with_cursor(line: &str, cursor_col: usize, theme: &Theme) -> Line<'static> {
    let chars: Vec<char> = line.chars().collect();
    let mut spans = Vec::new();

    for (i, &ch) in chars.iter().enumerate() {
        if i == cursor_col {
            spans.push(Span::styled("█", stylesheet::cursor(theme)));
            spans.push(Span::raw(ch.to_string()));
        } else {
            spans.push(Span::raw(ch.to_string()));
        }
    }

    if cursor_col >= chars.len() {
        spans.push(Span::styled("█", stylesheet::cursor(theme)));
    }

    Line::from(spans)
}

fn centered_popup(area: Rect, width_percent: u16, height_percent: u16) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height_percent) / 2),
            Constraint::Percentage(height_percent),
            Constraint::Percentage((100 - height_percent) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_percent) / 2),
            Constraint::Percentage(width_percent),
            Constraint::Percentage((100 - width_percent) / 2),
        ])
        .split(popup_layout[1])[1]
}
