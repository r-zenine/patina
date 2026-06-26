//! Diff view component showing formatted ReviewableDiff content

use ratatui::{
    Frame,
    layout::Rect,
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use tui_design::{Icons, Theme, stylesheet};

use crate::{
    state::{FocusPanel, UiState},
    ui::components::{
        decision_details_panel,
        renderable_diff_widget::{
            GutterBracketMap, GutterPosition, ReasoningAnnotation, RenderableDiffWidget,
        },
    },
};
use diffviz_review::{engines::ReviewEngine, state::ReviewableDiff};

/// Render the diff view panel
pub fn render(f: &mut Frame, area: Rect, ui_state: &UiState, review_engine: &ReviewEngine) {
    let is_focused = matches!(ui_state.focused_panel, FocusPanel::DiffView);

    match ui_state.decision_tree.selected_path.depth() {
        0 => {
            render_decision_details(f, area, ui_state, review_engine, is_focused);
        }
        1 => {
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
            render_no_diff_selected(f, area, is_focused);
        }
    }
}

fn render_decision_details(
    f: &mut Frame,
    area: Rect,
    ui_state: &UiState,
    review_engine: &ReviewEngine,
    is_focused: bool,
) {
    decision_details_panel::render(f, area, ui_state, review_engine, is_focused);
}

fn render_diff_content(
    f: &mut Frame,
    area: Rect,
    ui_state: &UiState,
    review_engine: &ReviewEngine,
    reviewable_diff: &ReviewableDiff,
    is_focused: bool,
) {
    let theme = Theme::mocha();

    let approval_icon = if review_engine.state().is_approved(&reviewable_diff.id) {
        Icons::APPROVED
    } else {
        Icons::PENDING
    };

    let title = format!(
        "{} {} (L{}-{})",
        approval_icon,
        reviewable_diff.file_path,
        reviewable_diff.id.line_range.start_line,
        reviewable_diff.id.line_range.end_line
    );

    if let Some(renderable_diff) = review_engine.get_renderable_diff_object(&reviewable_diff.id) {
        let cursor_line_num = if ui_state.cursor_index < renderable_diff.lines.len() {
            Some(renderable_diff.lines[ui_state.cursor_index].line_number)
        } else {
            None
        };

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

        let decisions = review_engine.get_decisions_for_diff(&reviewable_diff.id);

        let decision_badge = if !ui_state.show_reasoning && !decisions.is_empty() {
            let labels = decisions
                .iter()
                .map(|d| format!("D{}", d.number))
                .collect::<Vec<_>>()
                .join(",");
            format!("  ◆ {labels}")
        } else {
            String::new()
        };

        let combined_title = format!(
            "{} ({}){}{}",
            title, renderable_diff.metadata.boundary_name, cited_annotation, decision_badge
        );

        let diff_start = renderable_diff.metadata.overall_line_range.start_line;
        let annotations: Vec<ReasoningAnnotation> = if ui_state.show_reasoning {
            decisions
                .iter()
                .flat_map(|decision| {
                    decision
                        .code_impacts
                        .iter()
                        .filter(|impact| impact.file == reviewable_diff.file_path)
                        .map(|impact| {
                            let abs_trigger = impact
                                .line_ranges
                                .iter()
                                .map(|r| r.start)
                                .min()
                                .unwrap_or(reviewable_diff.id.line_range.start_line)
                                .max(reviewable_diff.id.line_range.start_line);
                            let trigger_line =
                                abs_trigger.saturating_sub(diff_start).saturating_add(1);
                            ReasoningAnnotation {
                                trigger_line,
                                label: format!("D{}", decision.number),
                                reasoning: impact.reasoning.clone(),
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .collect()
        } else {
            vec![]
        };

        let instruction_map = build_instruction_gutter_map(reviewable_diff, review_engine);

        let widget = RenderableDiffWidget::new(&renderable_diff)
            .with_context(ui_state.show_all_context)
            .highlight_semantics(ui_state.highlight_semantics)
            .with_scroll_offset(ui_state.scroll_offset)
            .with_selection(ui_state.selection_range)
            .with_cursor_line(cursor_line_num)
            .with_title(combined_title)
            .with_border_style(if is_focused {
                stylesheet::border_focused(&theme)
            } else {
                stylesheet::border(&theme)
            })
            .with_instruction_indicators(&instruction_map)
            .with_reasoning_annotations(&annotations);

        f.render_widget(widget, area);
    } else {
        render_no_diff_available(f, area, is_focused, title);
    }

    render_review_metadata(f, area, reviewable_diff, review_engine, ui_state);

    if ui_state.show_instructions {
        render_instructions_overlay(f, area, reviewable_diff, review_engine, ui_state);
    }
}

fn render_no_diff_available(f: &mut Frame, area: Rect, is_focused: bool, title: String) {
    let theme = Theme::mocha();
    let message = vec![
        Line::from(vec![Span::styled("Unable to render diff", stylesheet::muted(&theme))]),
        Line::from(vec![Span::styled(
            "RenderableDiff not available",
            stylesheet::muted(&theme),
        )]),
    ];

    let paragraph = Paragraph::new(message)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(if is_focused {
                    stylesheet::border_focused(&theme)
                } else {
                    stylesheet::border(&theme)
                }),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn render_no_diff_selected(f: &mut Frame, area: Rect, is_focused: bool) {
    let theme = Theme::mocha();
    let message = vec![
        Line::from(vec![Span::styled("No diff selected", stylesheet::muted(&theme))]),
        Line::from(vec![Span::styled(
            "Navigate to a file to view differences",
            stylesheet::muted(&theme),
        )]),
    ];

    let paragraph = Paragraph::new(message)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Diff View")
                .border_style(if is_focused {
                    stylesheet::border_focused(&theme)
                } else {
                    stylesheet::border(&theme)
                }),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn render_review_metadata(
    f: &mut Frame,
    area: Rect,
    _reviewable_diff: &ReviewableDiff,
    review_engine: &ReviewEngine,
    ui_state: &UiState,
) {
    let theme = Theme::mocha();
    let state = review_engine.state();
    let reviewable_id = ui_state.current_reviewable_id().unwrap();

    if state.has_instructions(&reviewable_id) {
        let instructions = state.get_instructions(&reviewable_id);
        let instruction_count = instructions.map_or(0, |i| i.len());

        let indicator_area = Rect {
            x: area.x + area.width - 8,
            y: area.y,
            width: 8,
            height: 1,
        };

        let instruction_indicator =
            Paragraph::new(format!("{} {}", Icons::INSTRUCTION_MODE, instruction_count))
                .style(stylesheet::warning(&theme));

        f.render_widget(instruction_indicator, indicator_area);
    }
}

fn render_instructions_overlay(
    f: &mut Frame,
    area: Rect,
    _reviewable_diff: &ReviewableDiff,
    review_engine: &ReviewEngine,
    ui_state: &UiState,
) {
    let theme = Theme::mocha();
    let state = review_engine.state();
    let reviewable_id = ui_state.current_reviewable_id().unwrap();

    if let Some(instructions) = state.get_instructions(&reviewable_id) {
        let overlay_height = (area.height as f32 * 0.4) as u16;
        let overlay_area = ratatui::layout::Rect {
            x: area.x,
            y: area.y + area.height - overlay_height,
            width: area.width,
            height: overlay_height,
        };

        let mut lines = vec![Line::from(vec![Span::styled(
            format!("📋 Instructions ({})", instructions.len()),
            stylesheet::warning(&theme).add_modifier(Modifier::BOLD),
        )])];

        for (idx, instruction) in instructions.iter().enumerate() {
            if idx > 0 {
                lines.push(Line::from(""));
            }
            lines.push(Line::from(vec![
                Span::styled("Author: ", stylesheet::muted(&theme)),
                Span::styled(&instruction.author, stylesheet::warning(&theme)),
                Span::styled(" | ", stylesheet::muted(&theme)),
                Span::styled(&instruction.timestamp, stylesheet::muted(&theme)),
            ]));
            lines.push(Line::from(Span::styled(
                &instruction.content,
                stylesheet::body(&theme),
            )));
        }

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Instructions (I to toggle)")
                    .border_style(stylesheet::warning(&theme)),
            )
            .wrap(Wrap { trim: false });

        f.render_widget(paragraph, overlay_area);
    }
}

fn build_instruction_gutter_map(
    reviewable_diff: &ReviewableDiff,
    review_engine: &ReviewEngine,
) -> GutterBracketMap {
    let mut map = GutterBracketMap::new();
    let state = review_engine.state();

    for (reviewable_id, instructions) in state.instructions.instructions.iter() {
        if reviewable_id.file_path == reviewable_diff.id.file_path
            && reviewable_id.query == reviewable_diff.id.query
        {
            for _instruction in instructions {
                let start_line = reviewable_id.line_range.start_line;
                let end_line = reviewable_id.line_range.end_line;

                if start_line == end_line {
                    map.insert(start_line, GutterPosition::InstructionStart);
                } else {
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
