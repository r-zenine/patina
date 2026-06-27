//! Decision tree component - hierarchical view for decision-based review
//!
//! Displays decisions with expandable chunks in a tree structure.
//! Tree layout: Decision (level 0) → Chunks (level 1)

use ratatui::{
    Frame,
    layout::Rect,
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, List, ListItem},
};
use tui_design::{Icons, Theme, stylesheet};

use crate::decision_navigation::FlattenedNodeKind;
use crate::state::UiState;
use diffviz_review::engines::ReviewEngine;

/// Render the decision tree as the primary navigation view
pub fn render(f: &mut Frame, area: Rect, ui_state: &UiState, review_engine: &ReviewEngine, is_focused: bool) {
    let theme = Theme::mocha();
    let flattened = ui_state.decision_tree.flatten();

    let title_style = if is_focused {
        stylesheet::title_active(&theme)
    } else {
        stylesheet::title_inactive(&theme)
    };

    let block = Block::default()
        .title(ratatui::text::Span::styled(" Decisions ", title_style))
        .style(stylesheet::layer_base(&theme));

    if flattened.is_empty() {
        f.render_widget(block, area);
        return;
    }

    let mut items = Vec::new();
    for node in &flattened {
        let is_selected = node.path == ui_state.decision_tree.selected_path;

        let item_line = match &node.kind {
            FlattenedNodeKind::Decision { number, expanded } => {
                build_decision_item(*number, *expanded, is_selected, review_engine, area.width, &theme)
            }
            FlattenedNodeKind::Chunk {
                decision_num: _,
                chunk_id,
                display_name,
            } => build_chunk_item(chunk_id, display_name, is_selected, review_engine, &theme),
        };

        items.push(ListItem::new(item_line));
    }

    let list = List::new(items)
        .block(block)
        .style(stylesheet::body(&theme));

    f.render_widget(list, area);
}

fn build_decision_item<'a>(
    decision_num: u32,
    expanded: bool,
    is_selected: bool,
    review_engine: &'a ReviewEngine,
    area_width: u16,
    theme: &Theme,
) -> Line<'a> {
    let decision = review_engine.get_decision(decision_num).unwrap();

    let expand_indicator = if expanded { "▼" } else { "▶" };
    let selection_indicator = if is_selected { "►" } else { " " };

    let is_approved = review_engine.is_decision_approved(decision_num);
    let approval_icon = if is_approved {
        Icons::APPROVED
    } else {
        Icons::NOT_APPROVED
    };

    let number_and_title = format!("{}. {}", decision.number, decision.title);

    let (approved_count, total_count) = review_engine
        .state()
        .decision_approval_progress(decision_num);
    let progress_str = format!("({approved_count}/{total_count})");

    let impact_count = decision.code_impacts.len();
    let count_str = format!("[{impact_count}]");

    let instruction_count = review_engine
        .get_decision_instructions(decision_num)
        .map_or(0, |v| v.len());
    let instruction_badge = if instruction_count > 0 {
        format!(" {} {}", Icons::INSTRUCTION_MODE, instruction_count)
    } else {
        String::new()
    };

    let line_content = if is_selected {
        let approval_style = if is_approved {
            stylesheet::success(theme)
        } else {
            stylesheet::muted(theme)
        };
        let sel_bg = theme.surface.surface0();
        vec![
            Span::styled(
                selection_indicator,
                stylesheet::keybind_key(theme).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {expand_indicator} "),
                stylesheet::warning(theme)
                    .bg(sel_bg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{approval_icon} "),
                approval_style.bg(sel_bg).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                number_and_title.clone(),
                stylesheet::body(theme)
                    .bg(sel_bg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " ".repeat((area_width as usize).saturating_sub(
                    selection_indicator.len()
                        + 3
                        + approval_icon.len()
                        + 1
                        + number_and_title.len()
                        + progress_str.len()
                        + count_str.len()
                        + 2,
                )),
                stylesheet::body(theme).bg(sel_bg),
            ),
            Span::styled(
                format!("{progress_str} "),
                stylesheet::muted(theme)
                    .bg(sel_bg)
                    .add_modifier(Modifier::DIM),
            ),
            Span::styled(
                count_str,
                stylesheet::info(theme)
                    .bg(sel_bg)
                    .add_modifier(Modifier::DIM),
            ),
            Span::styled(
                instruction_badge,
                stylesheet::warning(theme)
                    .bg(sel_bg)
                    .add_modifier(Modifier::BOLD),
            ),
        ]
    } else {
        vec![
            Span::raw(format!("{selection_indicator} {expand_indicator} ")),
            Span::styled(
                format!("{approval_icon} "),
                if is_approved {
                    stylesheet::success(theme)
                } else {
                    stylesheet::muted(theme)
                },
            ),
            Span::raw(number_and_title),
            Span::styled(
                format!(" {progress_str} "),
                stylesheet::muted(theme),
            ),
            Span::styled(
                format!(" {count_str}"),
                stylesheet::muted(theme),
            ),
            Span::styled(instruction_badge, stylesheet::warning(theme)),
        ]
    };

    Line::from(line_content)
}

fn build_chunk_item(
    chunk_id: &diffviz_review::ReviewableDiffId,
    display_name: &str,
    is_selected: bool,
    review_engine: &ReviewEngine,
    theme: &Theme,
) -> Line<'static> {
    let selection_indicator = if is_selected { "►" } else { " " };
    let indent = "  ";

    let is_approved = review_engine.state().is_approved(chunk_id);
    let approval_icon = if is_approved {
        Icons::APPROVED
    } else {
        Icons::NOT_APPROVED
    };

    let instruction_count = review_engine
        .state()
        .get_instructions(chunk_id)
        .map_or(0, |i| i.len());
    let instruction_badge = if instruction_count > 0 {
        format!(" {} {}", Icons::INSTRUCTION_MODE, instruction_count)
    } else {
        String::new()
    };

    let line_content = if is_selected {
        let sel_bg = theme.surface.surface0();
        vec![
            Span::styled(
                format!("{selection_indicator}{indent}"),
                stylesheet::body(theme)
                    .bg(sel_bg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{approval_icon} "),
                if is_approved {
                    stylesheet::success(theme)
                } else {
                    stylesheet::muted(theme)
                }
                .bg(sel_bg)
                .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                display_name.to_string(),
                stylesheet::body(theme)
                    .bg(sel_bg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                instruction_badge,
                stylesheet::warning(theme)
                    .bg(sel_bg)
                    .add_modifier(Modifier::BOLD),
            ),
        ]
    } else {
        vec![
            Span::raw(format!("{selection_indicator}{indent}")),
            Span::styled(
                format!("{approval_icon} "),
                if is_approved {
                    stylesheet::success(theme)
                } else {
                    stylesheet::muted(theme)
                },
            ),
            Span::raw(display_name.to_string()),
            Span::styled(instruction_badge, stylesheet::warning(theme)),
        ]
    };

    Line::from(line_content)
}
