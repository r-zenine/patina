//! Input modal component for comments, instructions, and edits

use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Clear, Padding, Paragraph, Wrap},
};
use tui_design::{Theme, stylesheet};

use crate::{
    state::{InputMode, UiState},
    ui::layout::centered_popup,
};
use diffviz_review::engines::ReviewEngine;

/// Render input modal for text input modes
pub fn render(f: &mut Frame, area: Rect, ui_state: &UiState, review_engine: &ReviewEngine) {
    if !ui_state.is_in_input_mode() {
        return;
    }

    let theme = Theme::mocha();
    let popup_area = centered_popup(area, 70, 40);

    f.render_widget(Clear, popup_area);

    let (title, _prompt, placeholder) = get_modal_content(&ui_state.input_mode, review_engine);

    let input_content = create_input_display(
        &ui_state.input_buffer,
        ui_state.input_cursor,
        placeholder,
        &theme,
    );

    let modal = Paragraph::new(input_content)
        .block(
            Block::default()
                .title(format!("  {title}"))
                .padding(Padding::new(2, 2, 1, 1))
                .style(stylesheet::layer_elevated(&theme)),
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
        Span::styled("Enter: Submit  ", stylesheet::keybind_key(&theme)),
        Span::styled("Esc: Cancel", stylesheet::error(&theme)),
    ]);

    let help_paragraph = Paragraph::new(help_text).style(stylesheet::muted(&theme));
    f.render_widget(help_paragraph, help_area);
}

fn get_modal_content(
    input_mode: &InputMode,
    review_engine: &ReviewEngine,
) -> (String, &'static str, &'static str) {
    match input_mode {
        InputMode::Instruction { reviewable_id } => {
            let existing = review_engine
                .state()
                .get_instructions(reviewable_id)
                .and_then(|v| v.first());
            let title = match existing {
                Some(instr) => format!(
                    "Append to {}'s note - {}",
                    instr.author, reviewable_id.file_path
                ),
                None => format!("Add Instruction - {}", reviewable_id.file_path),
            };
            (
                title,
                "Enter your instruction for this diff:",
                "Type your instruction here...",
            )
        }
        InputMode::DecisionInstruction { decision_number } => {
            let existing = review_engine
                .get_decision_instructions(*decision_number)
                .and_then(|v| v.into_iter().next());
            let title = match existing {
                Some(instr) => format!(
                    "Append to {}'s note - Decision #{decision_number}",
                    instr.author
                ),
                None => format!("Add Instruction - Decision #{decision_number}"),
            };
            (
                title,
                "Enter your instruction for this decision:",
                "Type your instruction here...",
            )
        }
        InputMode::Navigation => ("".to_string(), "", ""),
    }
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
