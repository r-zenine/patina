//! DrillNav Browse view — top-level decisions as hierarchical cards.
//!
//! Each card shows the decision label with approval badge and chunk progress
//! (CardTier::Header), the wrapped rationale (Header), and a preview of the
//! affected files (CardTier::Body). The focused card carries the accent bar;
//! `scroll_into_view` keeps it visible.

use diffviz_review::engines::ReviewEngine;
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use tui_design::{CardTier, Theme, scroll_into_view, separator_line};

use super::drillnav_common::{content_rect, make_card, plural_s, wrap_text};
use crate::state::UiState;
use crate::ui::icons::Icons;

/// Render the Browse view. Must only be called while `UiState` is browsing.
pub fn render(f: &mut Frame, area: Rect, ui_state: &UiState, engine: &ReviewEngine) {
    let cursor = ui_state
        .browse_cursor()
        .expect("browse view rendered outside Browse mode");
    let theme = Theme::mocha();
    let cr = content_rect(area);
    let text_width = (cr.width as usize).saturating_sub(4);
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut heights: Vec<u16> = Vec::new();

    lines.push(separator_line(cr.width, theme.surface.mantle()));

    let decisions = engine.get_all_decisions();
    for (i, decision) in decisions.iter().enumerate() {
        let card_start = lines.len();
        let focused = i == cursor;
        let is_decision_approved = engine.is_decision_approved(decision.number);
        let card = make_card(cr.width, focused, theme.accents.lavender);
        // File count from the drill index, not code_impacts: impacts can
        // declare files that map to zero chunks, and the index is what
        // drilling in will actually show.
        let index_files = &ui_state.drill_index().decisions[i].files;
        let n_files = index_files.len();
        let (chunks_approved, chunks_total) = engine.decision_approval_progress(decision.number);

        // label row — CardTier::Header (surface1)
        let label_card = if is_decision_approved {
            card.with_badge(Icons::APPROVED, theme.accents.green)
        } else {
            card
        };
        lines.push(label_card.at(
            CardTier::Header,
            vec![
                Span::styled(
                    format!("#{} {}", decision.number, decision.title),
                    Style::default()
                        .fg(theme.surface.text())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(
                        "  {} file{} · {}/{} chunks approved",
                        n_files,
                        plural_s(n_files),
                        chunks_approved,
                        chunks_total,
                    ),
                    Style::default().fg(theme.surface.overlay0()),
                ),
            ],
            &theme,
        ));

        // summary rows — CardTier::Header (pinnable block, same elevation as label)
        if let Some(rationale) = &decision.rationale {
            for text_line in wrap_text(rationale, text_width) {
                lines.push(card.at(
                    CardTier::Header,
                    vec![Span::styled(
                        format!("· {}", text_line),
                        Style::default().fg(theme.surface.subtext1()),
                    )],
                    &theme,
                ));
            }
        }

        // children preview — CardTier::Body (surface0, lower elevation)
        for file in index_files {
            lines.push(card.at(
                CardTier::Body,
                vec![Span::styled(
                    format!("{} {}", Icons::FILE_MODIFIED, file.path),
                    Style::default().fg(theme.surface.text()),
                )],
                &theme,
            ));
        }

        lines.push(separator_line(cr.width, theme.surface.mantle()));
        heights.push((lines.len() - card_start) as u16);
    }

    // The leading separator belongs to the first card's height so offsets align.
    if let Some(h) = heights.first_mut() {
        *h += 1;
    }
    let scroll = scroll_into_view(&heights, cursor, cr.height);
    f.render_widget(Paragraph::new(lines).scroll((scroll, 0)), cr);
}
