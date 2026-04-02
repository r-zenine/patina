//! Input modal component for comments, instructions, and edits

use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::{
    state::{InputMode, UiState},
    theme::{Colors, Styles},
    ui::layout::centered_popup,
};

/// Render input modal for text input modes
pub fn render(f: &mut Frame, area: Rect, ui_state: &UiState) {
    if !ui_state.is_in_input_mode() {
        return;
    }

    let popup_area = centered_popup(area, 70, 40);

    // Clear the area behind the modal
    f.render_widget(Clear, popup_area);

    // Get modal content based on input mode
    let (title, _prompt, placeholder) = get_modal_content(&ui_state.input_mode);

    // Create input content with cursor
    let input_content =
        create_input_display(&ui_state.input_buffer, ui_state.input_cursor, placeholder);

    // Create the modal
    let modal = Paragraph::new(input_content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Styles::focused_border())
                .style(Style::default().bg(Colors::BLACK)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(modal, popup_area);

    // Render help text at the bottom
    let help_area = Rect {
        x: popup_area.x,
        y: popup_area.y + popup_area.height - 3,
        width: popup_area.width,
        height: 1,
    };

    let help_text = Line::from(vec![
        Span::styled("Enter: Submit  ", Style::default().fg(Colors::CYAN)),
        Span::styled("Esc: Cancel", Style::default().fg(Colors::RED)),
    ]);

    let help_paragraph = Paragraph::new(help_text).style(Styles::muted());
    f.render_widget(help_paragraph, help_area);
}

/// Get modal content based on input mode
fn get_modal_content(input_mode: &InputMode) -> (String, &'static str, &'static str) {
    match input_mode {
        InputMode::Instruction { reviewable_id } => (
            format!("Add Instruction - {}", reviewable_id.file_path),
            "Enter your instruction for this diff:",
            "Type your instruction here...",
        ),
        InputMode::Navigation => ("".to_string(), "", ""), // Should not happen
    }
}

/// Create input display with cursor visualization
fn create_input_display(buffer: &str, cursor_pos: usize, placeholder: &str) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    if buffer.is_empty() {
        // Show placeholder text
        lines.push(Line::from(vec![Span::styled(
            placeholder.to_string(),
            Styles::muted(),
        )]));
        lines.push(Line::from(""));
    } else {
        // Split buffer into lines for display
        let input_lines: Vec<&str> = buffer.lines().collect();

        if input_lines.is_empty() {
            // Empty buffer with cursor
            lines.push(Line::from(vec![Span::styled(
                "█",
                Style::default().bg(Colors::WHITE).fg(Colors::BLACK),
            )]));
        } else {
            // Calculate cursor line and column
            let mut char_count = 0;
            let mut cursor_line = 0;
            let mut cursor_col = 0;

            for (line_idx, line) in input_lines.iter().enumerate() {
                let line_len = line.len() + 1; // +1 for newline
                if char_count + line_len > cursor_pos {
                    cursor_line = line_idx;
                    cursor_col = cursor_pos - char_count;
                    break;
                }
                char_count += line_len;
            }

            // Render each line with cursor if applicable
            for (line_idx, line) in input_lines.iter().enumerate() {
                if line_idx == cursor_line {
                    lines.push(create_line_with_cursor(line, cursor_col));
                } else {
                    lines.push(Line::from(vec![Span::styled(
                        (*line).to_string(),
                        Style::default(),
                    )]));
                }
            }
        }
    }

    lines
}

/// Create a line with cursor visualization
fn create_line_with_cursor(line: &str, cursor_col: usize) -> Line<'static> {
    let chars: Vec<char> = line.chars().collect();
    let mut spans = Vec::new();

    for (i, &ch) in chars.iter().enumerate() {
        if i == cursor_col {
            // Show cursor before this character
            spans.push(Span::styled(
                "█",
                Style::default().bg(Colors::WHITE).fg(Colors::BLACK),
            ));
            spans.push(Span::styled(ch.to_string(), Style::default()));
        } else {
            spans.push(Span::styled(ch.to_string(), Style::default()));
        }
    }

    // If cursor is at the end of the line
    if cursor_col >= chars.len() {
        spans.push(Span::styled(
            "█",
            Style::default().bg(Colors::WHITE).fg(Colors::BLACK),
        ));
    }

    Line::from(spans)
}
