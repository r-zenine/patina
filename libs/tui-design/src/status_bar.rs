//! Generic status-bar chrome: a one-line hint bar where a one-shot message
//! preempts the hints in error styling, plus a fixed input-mode status line.
//! Callers own the actual hint/help text; this module only styles it.

use ratatui::{Frame, layout::Rect, style::Modifier, text::Line, text::Span, widgets::Paragraph};

use crate::stylesheet;
use crate::tokens::Theme;

/// Render the navigation-mode status bar: `override_message` (if any)
/// preempts `hints` in error styling until the caller clears it.
pub fn render_hints(
    f: &mut Frame,
    area: Rect,
    override_message: Option<&str>,
    hints: &str,
    theme: &Theme,
) {
    let (status, status_style) = match override_message {
        Some(msg) => (
            msg.to_string(),
            stylesheet::error(theme).patch(stylesheet::terminal_floor(theme)),
        ),
        None => (
            hints.to_string(),
            stylesheet::status_bar(theme).patch(stylesheet::terminal_floor(theme)),
        ),
    };
    f.render_widget(Paragraph::new(status).style(status_style), area);
}

/// Render the fixed status line shown while in a text-input mode, e.g.
/// `"Instruction: Enter to submit, Esc to cancel"`.
pub fn render_input_status(
    f: &mut Frame,
    area: Rect,
    mode_name: &str,
    help_text: &str,
    theme: &Theme,
) {
    let status_line = Line::from(vec![
        Span::styled(
            format!("{mode_name}: "),
            stylesheet::keybind_key(theme).add_modifier(Modifier::BOLD),
        ),
        Span::styled(help_text, stylesheet::muted(theme)),
    ]);

    let paragraph = Paragraph::new(status_line)
        .style(stylesheet::status_bar(theme).patch(stylesheet::terminal_floor(theme)));
    f.render_widget(paragraph, area);
}
