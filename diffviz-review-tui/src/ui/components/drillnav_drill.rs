//! DrillNav Drill view — inside a decision: pinned file header, dot
//! pagination for sibling files, chunk cards scrolling below.
//!
//! Each chunk card holds the chunk's note (CardTier::Body, truncated to one
//! row with `…` until `i` expands it) above its code lines
//! (CardTier::Content, context lines hidden until Tab expands them). The
//! focused chunk carries the accent bar; approved chunks a `✓` badge.
//! Scrolling combines `scroll_into_view` on the focused chunk with the
//! Ctrl-d/u page offset, clamped to the real content height.

use diffviz_core::renderable_diff::ChangeType;
use diffviz_review::engines::ReviewEngine;
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use tui_design::{CardTier, HierarchicalCard, Icons, Theme, render_drill_header, scroll_into_view};

use super::drillnav_common::{
    content_rect, dot_pagination_line, line_change_type, line_has_change, make_card, note_for,
    note_rows, wrap_text,
};
use crate::state::UiState;
use diffviz_review::{Instruction, ReviewableDiffId};

/// Render the Drill view. Must only be called while `UiState` is drilled in.
pub fn render(f: &mut Frame, area: Rect, ui_state: &UiState, engine: &ReviewEngine) {
    let (decision_idx, file_idx, cursor) = ui_state
        .drill_position()
        .expect("drill view rendered outside Drill mode");
    let theme = Theme::mocha();
    let cr = content_rect(area);
    let text_width = (cr.width as usize).saturating_sub(4);

    let decisions = engine.get_all_decisions();
    let decision = decisions[decision_idx];
    let drill_decision = &ui_state.drill_index().decisions[decision_idx];
    let file = &drill_decision.files[file_idx];
    let chunk_ids = &file.chunks;

    // The code impact reasoning for this file. Index files derive from the
    // decision's impacts, so a missing impact is a broken invariant.
    let impact = decision
        .code_impacts
        .iter()
        .find(|ci| ci.file == file.path)
        .expect("file path from drill index should match a decision impact");

    // Anchored header: file label + impact reasoning at CardTier::Header.
    let is_decision_approved = engine.is_decision_approved(decision.number);
    let header_card = HierarchicalCard::new(cr.width);
    let mut header_lines: Vec<Line<'static>> = Vec::new();
    let file_row_card = if is_decision_approved {
        header_card.with_badge(Icons::APPROVED, theme.accents.green)
    } else {
        header_card
    };
    header_lines.push(file_row_card.at(
        CardTier::Header,
        vec![Span::styled(
            format!("{} {}", Icons::FILE_MODIFIED, file.path),
            Style::default()
                .fg(theme.surface.text())
                .add_modifier(Modifier::BOLD),
        )],
        &theme,
    ));
    for text_line in wrap_text(&impact.reasoning, text_width) {
        header_lines.push(header_card.at(
            CardTier::Header,
            vec![Span::styled(
                text_line,
                Style::default().fg(theme.surface.subtext1()),
            )],
            &theme,
        ));
    }
    let below_header_area = render_drill_header(f, cr, header_lines, &theme);

    // Dot pagination — one mantle-level line between the header and chunks.
    let total_siblings = drill_decision.files.len();
    let content_area = if total_siblings > 1 {
        let [dots_area, rest] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(below_header_area);
        f.render_widget(
            Paragraph::new(dot_pagination_line(
                cr.width,
                file_idx,
                total_siblings,
                &theme,
            )),
            dots_area,
        );
        rest
    } else {
        below_header_area
    };

    let n = chunk_ids.len();
    let mut chunk_lines: Vec<Line<'static>> = Vec::new();

    for (i, chunk_id) in chunk_ids.iter().enumerate() {
        let is_approved = engine.state().is_approved(chunk_id);
        let card = make_card(cr.width, i == cursor, theme.accents.lavender);
        let chunk_expanded = ui_state.drill_chunk_expanded(i);
        let note_expanded = ui_state.drill_chunk_note_expanded(i);

        let note = note_for(engine, chunk_id);

        // Note rows — CardTier::Body, above the code.
        if let Some(instr) = note {
            let wrap_width = text_width.saturating_sub(6);
            let mut rows = note_rows(instr, wrap_width);
            let has_more = rows.len() > 1;
            if !note_expanded {
                let first = rows.into_iter().next().unwrap_or_default();
                rows = vec![if has_more {
                    format!("{}…", first.trim_end_matches(' '))
                } else {
                    first
                }];
            }
            for (row, text_line) in rows.into_iter().enumerate() {
                let icon_col = if row == 0 {
                    Span::styled(
                        Icons::HAS_INSTRUCTIONS,
                        Style::default().fg(theme.accents.yellow),
                    )
                } else {
                    Span::styled("  ", Style::default())
                };
                let row_card = if row == 0 && is_approved {
                    card.with_badge(Icons::APPROVED, theme.accents.green)
                } else {
                    card
                };
                chunk_lines.push(row_card.at(
                    CardTier::Body,
                    vec![
                        Span::styled(" ", Style::default()),
                        icon_col,
                        Span::styled(
                            format!("   {}", text_line),
                            Style::default().fg(theme.surface.subtext1()),
                        ),
                    ],
                    &theme,
                ));
            }
        }

        // Code rows — CardTier::Content; context lines only when expanded.
        if let Some(renderable) = engine.get_renderable_diff_object(chunk_id) {
            let has_note = note.is_some();
            let visible_lines: Vec<_> = renderable
                .lines
                .iter()
                .filter(|line| chunk_expanded || line_has_change(line))
                .collect();

            for (line_idx, line) in visible_lines.iter().enumerate() {
                let ct = line_change_type(line);
                let (fg, sigil) = match &ct {
                    Some(ChangeType::Added) => (theme.accents.green, "+"),
                    Some(ChangeType::Deleted) => (theme.accents.red, "-"),
                    _ => (theme.surface.subtext0(), " "),
                };
                let row_card = if !has_note && line_idx == 0 && is_approved {
                    card.with_badge(Icons::APPROVED, theme.accents.green)
                } else {
                    card
                };
                chunk_lines.push(row_card.at(
                    CardTier::Content,
                    vec![
                        Span::styled(
                            format!("{:>3} ", line.line_number),
                            Style::default().fg(theme.surface.overlay0()),
                        ),
                        Span::styled(sigil, Style::default().fg(fg).add_modifier(Modifier::BOLD)),
                        Span::styled(format!(" {}", line.content), Style::default().fg(fg)),
                    ],
                    &theme,
                ));
            }
        }

        if i + 1 < n {
            chunk_lines.push(Line::styled(
                format!("{:^width$}", "···", width = cr.width as usize),
                Style::default()
                    .fg(theme.surface.overlay0())
                    .bg(theme.surface.mantle()),
            ));
        }
    }

    let heights: Vec<u16> = chunk_ids
        .iter()
        .enumerate()
        .map(|(i, chunk_id)| {
            let h = visible_line_count(
                engine,
                chunk_id,
                note_for(engine, chunk_id),
                ui_state.drill_chunk_expanded(i),
                ui_state.drill_chunk_note_expanded(i),
                text_width,
            );
            if i + 1 < n { h + 1 } else { h }
        })
        .collect();

    // Focused chunk stays visible; the Ctrl-d/u page offset (unclamped in
    // state, D3/D6) is clamped here against the real content height.
    let base_scroll = scroll_into_view(&heights, cursor, content_area.height);
    let total_height: u16 = heights.iter().sum();
    let max_scroll = total_height.saturating_sub(content_area.height);
    let page_offset = ui_state.drill_page_offset().unwrap_or(0);
    let scroll = base_scroll
        .saturating_add(page_offset.min(u16::MAX as usize) as u16)
        .min(max_scroll);
    f.render_widget(
        Paragraph::new(chunk_lines).scroll((scroll, 0)),
        content_area,
    );
}

/// Rendered height of one chunk card (note rows + visible code rows),
/// excluding the between-chunk separator.
fn visible_line_count(
    engine: &ReviewEngine,
    chunk_id: &ReviewableDiffId,
    note: Option<&Instruction>,
    expanded: bool,
    note_expanded: bool,
    text_width: usize,
) -> u16 {
    let code_lines = if let Some(renderable) = engine.get_renderable_diff_object(chunk_id) {
        if expanded {
            renderable.lines.len() as u16
        } else {
            renderable
                .lines
                .iter()
                .filter(|l| line_has_change(l))
                .count() as u16
        }
    } else {
        0
    };
    let note_lines = if let Some(instr) = note {
        if note_expanded {
            note_rows(instr, text_width.saturating_sub(6)).len() as u16
        } else {
            1
        }
    } else {
        0
    };
    code_lines + note_lines
}
