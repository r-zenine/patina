//! Diff view component showing formatted ReviewableDiff content

use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::{
    state::{FocusPanel, UiState},
    theme::{Colors, Icons, Styles},
    ui::components::{
        decision_details_panel,
        renderable_diff_widget::{GutterBracketMap, GutterPosition, RenderableDiffWidget},
    },
};
use diffviz_review::{engines::ReviewEngine, state::ReviewableDiff};

/// Render the diff view panel
pub fn render(f: &mut Frame, area: Rect, ui_state: &UiState, review_engine: &ReviewEngine) {
    let is_focused = matches!(ui_state.focused_panel, FocusPanel::DiffView);

    // Route based on selected depth in decision tree
    match ui_state.decision_tree.selected_path.depth() {
        0 => {
            // Decision selected - show decision details inline
            render_decision_details(f, area, ui_state, review_engine, is_focused);
        }
        1 => {
            // Chunk selected - show diff content
            if let Some(reviewable_id) = ui_state.current_reviewable_id() {
                if let Some(reviewable_diff) = review_engine.get_reviewable_diff(&reviewable_id) {
                    render_diff_content(
                        f,
                        area,
                        ui_state,
                        review_engine,
                        reviewable_diff,
                        is_focused,
                    );
                } else {
                    render_no_diff_selected(f, area, is_focused);
                }
            } else {
                render_no_diff_selected(f, area, is_focused);
            }
        }
        _ => {
            // Unknown depth - show placeholder
            render_no_diff_selected(f, area, is_focused);
        }
    }
}

/// Render decision details inline when a decision is selected (depth 0)
fn render_decision_details(
    f: &mut Frame,
    area: Rect,
    ui_state: &UiState,
    review_engine: &ReviewEngine,
    is_focused: bool,
) {
    decision_details_panel::render(f, area, ui_state, review_engine, is_focused);
}

/// Render the actual diff content
fn render_diff_content(
    f: &mut Frame,
    area: Rect,
    ui_state: &UiState,
    review_engine: &ReviewEngine,
    reviewable_diff: &ReviewableDiff,
    is_focused: bool,
) {
    // Check approval status
    let approval_icon = if review_engine.state().is_approved(&reviewable_diff.id) {
        Icons::APPROVED
    } else {
        Icons::PENDING
    };

    // Create simple title
    let title = format!(
        "{} {} (L{}-{})",
        approval_icon,
        reviewable_diff.file_path,
        reviewable_diff.id.line_range.start_line,
        reviewable_diff.id.line_range.end_line
    );

    // Get RenderableDiff object
    if let Some(renderable_diff) = review_engine.get_renderable_diff_object(&reviewable_diff.id) {
        // Get cursor line number from cursor index
        let cursor_line_num = if ui_state.cursor_index < renderable_diff.lines.len() {
            Some(renderable_diff.lines[ui_state.cursor_index].line_number)
        } else {
            None
        };

        // Build combined title with semantic info and cited-range annotation when multiple
        // decision-log ranges collapsed to this same semantic unit.
        let cited_annotation = if reviewable_diff.cited_ranges.len() > 1 {
            let ranges = reviewable_diff
                .cited_ranges
                .iter()
                .map(|(s, e)| format!("L{s}-{e}"))
                .collect::<Vec<_>>()
                .join(", ");
            format!(" [impacts: {ranges}]")
        } else {
            String::new()
        };
        let combined_title = format!(
            "{} ({}){}",
            title, renderable_diff.metadata.boundary_name, cited_annotation
        );

        // Build instruction gutter map for bracket visualization
        let instruction_map = build_instruction_gutter_map(reviewable_diff, review_engine);

        // Use RenderableDiffWidget for rich display with combined title
        let widget = RenderableDiffWidget::new(&renderable_diff)
            .with_context(ui_state.show_all_context)
            .highlight_semantics(ui_state.highlight_semantics)
            .with_scroll_offset(ui_state.scroll_offset)
            .with_selection(ui_state.selection_range)
            .with_cursor_line(cursor_line_num)
            .with_title(combined_title)
            .with_border_style(if is_focused {
                Styles::focused_border()
            } else {
                Styles::border()
            })
            .with_instruction_indicators(&instruction_map);

        // Render the RenderableDiffWidget directly (it has its own border)
        f.render_widget(widget, area);
    } else {
        // No RenderableDiff available - show error
        render_no_diff_available(f, area, is_focused, title);
    }

    // Show review metadata
    render_review_metadata(f, area, reviewable_diff, review_engine, ui_state);

    // Show instructions overlay if toggled
    if ui_state.show_instructions {
        render_instructions_overlay(f, area, reviewable_diff, review_engine, ui_state);
    }
}

/// Render error when RenderableDiff is not available
fn render_no_diff_available(f: &mut Frame, area: Rect, is_focused: bool, title: String) {
    let message = vec![
        Line::from(vec![Span::styled("Unable to render diff", Styles::muted())]),
        Line::from(vec![Span::styled(
            "RenderableDiff not available",
            Styles::muted(),
        )]),
    ];

    let paragraph = Paragraph::new(message)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(if is_focused {
                    Styles::focused_border()
                } else {
                    Styles::border()
                }),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

/// Render message when no diff is selected
fn render_no_diff_selected(f: &mut Frame, area: Rect, is_focused: bool) {
    let message = vec![
        Line::from(vec![Span::styled("No diff selected", Styles::muted())]),
        Line::from(vec![Span::styled(
            "Navigate to a file to view differences",
            Styles::muted(),
        )]),
    ];

    let paragraph = Paragraph::new(message)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Diff View")
                .border_style(if is_focused {
                    Styles::focused_border()
                } else {
                    Styles::border()
                }),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

/// Render review metadata (comments, approvals) in a small overlay
fn render_review_metadata(
    f: &mut Frame,
    area: Rect,
    _reviewable_diff: &ReviewableDiff,
    review_engine: &ReviewEngine,
    ui_state: &UiState,
) {
    let state = review_engine.state();
    // TODO: Get actual reviewable_id from ReviewableDiff once API is available
    let reviewable_id = ui_state.current_reviewable_id().unwrap(); // &reviewable_diff.id;

    // Check for instructions
    if state.has_instructions(&reviewable_id) {
        let instructions = state.get_instructions(&reviewable_id);
        let instruction_count = instructions.map_or(0, |i| i.len());

        // Render instruction indicator
        let indicator_area = Rect {
            x: area.x + area.width - 8,
            y: area.y,
            width: 8,
            height: 1,
        };

        let instruction_indicator =
            Paragraph::new(format!("{} {}", Icons::INSTRUCTION_MODE, instruction_count))
                .style(Style::default().fg(Colors::YELLOW));

        f.render_widget(instruction_indicator, indicator_area);
    }
}

/// Render instructions overlay when toggled on
fn render_instructions_overlay(
    f: &mut Frame,
    area: Rect,
    _reviewable_diff: &ReviewableDiff,
    review_engine: &ReviewEngine,
    ui_state: &UiState,
) {
    let state = review_engine.state();
    let reviewable_id = ui_state.current_reviewable_id().unwrap();

    if let Some(instructions) = state.get_instructions(&reviewable_id) {
        // Create overlay area (bottom 40% of screen)
        let overlay_height = (area.height as f32 * 0.4) as u16;
        let overlay_area = ratatui::layout::Rect {
            x: area.x,
            y: area.y + area.height - overlay_height,
            width: area.width,
            height: overlay_height,
        };

        // Build instruction lines
        let mut lines = vec![Line::from(vec![Span::styled(
            format!("📋 Instructions ({})", instructions.len()),
            Style::default()
                .fg(Colors::YELLOW)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )])];

        for (idx, instruction) in instructions.iter().enumerate() {
            if idx > 0 {
                lines.push(Line::from(""));
            }
            lines.push(Line::from(vec![
                Span::styled("Author: ", Style::default().fg(Colors::WHITE)),
                Span::styled(&instruction.author, Style::default().fg(Colors::YELLOW)),
                Span::styled(" | ", Styles::muted()),
                Span::styled(&instruction.timestamp, Styles::muted()),
            ]));
            lines.push(Line::from(Span::styled(
                &instruction.content,
                Style::default().fg(Colors::WHITE),
            )));
        }

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .title("Instructions (I to toggle)")
                    .border_style(Style::default().fg(Colors::YELLOW)),
            )
            .wrap(Wrap { trim: false });

        f.render_widget(paragraph, overlay_area);
    }
}

/// Build a gutter bracket map for instruction visualization
///
/// Maps line numbers from the ReviewableDiffId to their gutter bracket positions
/// based on the instructions registered for the current file.
fn build_instruction_gutter_map(
    reviewable_diff: &ReviewableDiff,
    review_engine: &ReviewEngine,
) -> GutterBracketMap {
    let mut map = GutterBracketMap::new();
    let state = review_engine.state();

    // Get instructions for this file path (not just this specific diff)
    // We need to check all instructions and see if they apply to this file
    for (reviewable_id, instructions) in state.instructions.instructions.iter() {
        // Only include instructions for the same file and query
        if reviewable_id.file_path == reviewable_diff.id.file_path
            && reviewable_id.query == reviewable_diff.id.query
        {
            for _instruction in instructions {
                // Map the line range to gutter positions
                let start_line = reviewable_id.line_range.start_line;
                let end_line = reviewable_id.line_range.end_line;

                if start_line == end_line {
                    // Single line instruction: just use start marker
                    map.insert(start_line, GutterPosition::InstructionStart);
                } else {
                    // Multi-line instruction: start, middle, end
                    map.insert(start_line, GutterPosition::InstructionStart);
                    for line in (start_line + 1)..end_line {
                        map.insert(line, GutterPosition::InstructionMiddle);
                    }
                    map.insert(end_line, GutterPosition::InstructionEnd);
                }
            }
        }
    }

    map
}
