//! Generic DrillNav rendering: Browse (top-level card list) and Drill
//! (pinned header + scrolling leaf cards) views.
//!
//! Neither view knows about any specific domain model — callers implement
//! [`DrillGroup`] (Browse) and [`DrillChunk`] (Drill) over their own types
//! (e.g. a review's `Decision`/`ReviewableDiff`, or a detector's
//! `Symptom`/`Site`) and hand back pre-formatted strings/rows. All "what does
//! expanded/approved/annotated mean" logic stays on the caller's side; this
//! module only owns layout, scrolling, and card chrome.
//!
//! Layout: content capped at 120 columns, centered; surface bg fills full
//! column width. Surface ramp (dark theme, lighter = higher elevation):
//!   group/file header      → surface1 (CardTier::Header — pinnable)
//!   notes, rationale, meta → surface0 (CardTier::Body)
//!   leaf content (code)    → base     (CardTier::Content)
//!   separators, dots       → mantle   (widget floor)

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use crate::card::{CardTier, HierarchicalCard, render_drill_header, separator_line};
use crate::scroll::scroll_into_view;
use crate::tokens::Theme;

/// Maximum content width; wider terminals center the content column.
pub const CONTENT_WIDTH: u16 = 120;

/// The centered content column within `area`.
pub fn content_rect(area: Rect) -> Rect {
    let w = CONTENT_WIDTH.min(area.width);
    let x = area.x + (area.width - w) / 2;
    Rect {
        x,
        width: w,
        ..area
    }
}

pub fn plural_s(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}

/// Hard-splits a word that can't fit on any line (long paths, URLs) into
/// `max_cols`-sized pieces; words that fit come back whole.
fn split_oversized(word: &str, max_cols: usize) -> Vec<String> {
    if word.chars().count() <= max_cols {
        return vec![word.to_string()];
    }
    let mut pieces = Vec::new();
    let mut chars = word.chars().peekable();
    while chars.peek().is_some() {
        pieces.push(chars.by_ref().take(max_cols).collect());
    }
    pieces
}

/// Greedy word wrap for prose (rationales, reasoning, notes).
pub fn wrap_text(text: &str, max_cols: usize) -> Vec<String> {
    let max_cols = max_cols.max(1);
    let mut result = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        for piece in split_oversized(word, max_cols) {
            if current.is_empty() {
                current = piece;
            } else if current.chars().count() + 1 + piece.chars().count() <= max_cols {
                current.push(' ');
                current.push_str(&piece);
            } else {
                result.push(std::mem::take(&mut current));
                current = piece;
            }
        }
    }
    if !current.is_empty() {
        result.push(current);
    }
    result
}

/// A card with the focus accent bar applied when focused.
pub fn make_card(col_width: u16, focused: bool, accent_color: Color) -> HierarchicalCard {
    if focused {
        HierarchicalCard::new(col_width).focused(accent_color)
    } else {
        HierarchicalCard::new(col_width)
    }
}

/// Centered dot row on a mantle background — signals h/l sibling navigation.
/// Active dot uses the accent color; inactive dots use overlay0.
pub fn dot_pagination_line(
    col_width: u16,
    current: usize,
    total: usize,
    theme: &Theme,
) -> Line<'static> {
    let mantle = theme.surface.mantle();
    let active = theme.accents.lavender;
    let passive = theme.surface.overlay0();

    let dot_section = total * 2 - 1; // "● ○ ○" = n dots + (n-1) spaces
    let left_pad = (col_width as usize).saturating_sub(dot_section) / 2;
    let right_fill = (col_width as usize).saturating_sub(left_pad + dot_section);

    let mut spans: Vec<Span<'static>> = Vec::with_capacity(total * 2 + 2);
    spans.push(Span::styled(
        " ".repeat(left_pad),
        Style::default().bg(mantle),
    ));
    for i in 0..total {
        if i > 0 {
            spans.push(Span::styled(" ", Style::default().bg(mantle)));
        }
        let (dot, color) = if i == current {
            ("●", active)
        } else {
            ("○", passive)
        };
        spans.push(Span::styled(dot, Style::default().fg(color).bg(mantle)));
    }
    spans.push(Span::styled(
        " ".repeat(right_fill),
        Style::default().bg(mantle),
    ));
    Line::from(spans)
}

// ── Browse view ─────────────────────────────────────────────────────────

/// Pre-formatted header content for one Browse card.
pub struct GroupHeader {
    /// Bold title span, e.g. `"#3 Fix retry logic"`.
    pub title: String,
    /// Optional single-glyph gutter badge (icon, color) — e.g. an approved
    /// checkmark. Must be exactly 1 terminal cell wide.
    pub badge: Option<(&'static str, Color)>,
    /// Trailing meta text with no leading padding, e.g.
    /// `"3 files · 2/5 chunks approved"`.
    pub meta: String,
}

/// One top-level card in the Browse view (a review's Decision, a detector's
/// Symptom, ...). All methods return owned/pre-formatted data — this trait
/// carries no domain types.
pub trait DrillGroup {
    fn header(&self) -> GroupHeader;
    fn rationale(&self) -> Option<String>;
    /// Pre-formatted child preview rows (already icon-prefixed).
    fn children_preview(&self) -> Vec<String>;
}

/// Render the Browse view: a scrollable list of hierarchical cards, one per
/// group, with the focused card carrying the accent bar.
pub fn render_browse<G: DrillGroup>(
    f: &mut Frame,
    area: Rect,
    groups: &[G],
    cursor: usize,
    theme: &Theme,
) {
    let cr = content_rect(area);
    let text_width = (cr.width as usize).saturating_sub(4);
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut heights: Vec<u16> = Vec::new();

    lines.push(separator_line(cr.width, theme.surface.mantle()));

    for (i, group) in groups.iter().enumerate() {
        let card_start = lines.len();
        let focused = i == cursor;
        let card = make_card(cr.width, focused, theme.accents.lavender);
        let header = group.header();

        let label_card = match header.badge {
            Some((ch, color)) => card.with_badge(ch, color),
            None => card,
        };
        lines.push(label_card.at(
            CardTier::Header,
            vec![
                Span::styled(
                    header.title,
                    Style::default()
                        .fg(theme.surface.text())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("  {}", header.meta),
                    Style::default().fg(theme.surface.overlay0()),
                ),
            ],
            theme,
        ));

        if let Some(rationale) = group.rationale() {
            for text_line in wrap_text(&rationale, text_width) {
                lines.push(card.at(
                    CardTier::Header,
                    vec![Span::styled(
                        format!("· {}", text_line),
                        Style::default().fg(theme.surface.subtext1()),
                    )],
                    theme,
                ));
            }
        }

        for preview in group.children_preview() {
            lines.push(card.at(
                CardTier::Body,
                vec![Span::styled(
                    preview,
                    Style::default().fg(theme.surface.text()),
                )],
                theme,
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

// ── Drill view ──────────────────────────────────────────────────────────

/// Pinned header content for the Drill view.
pub struct FileHeader {
    /// Bold label span, e.g. `"~ src/lib.rs"`.
    pub label: String,
    /// Optional single-glyph gutter badge, same contract as [`GroupHeader::badge`].
    pub badge: Option<(&'static str, Color)>,
    /// Reasoning/summary prose shown under the label (word-wrapped here).
    pub reasoning: String,
}

/// Semantic color role for one code row — kept generic (no `ChangeType`
/// import) so this crate never depends on a domain diff model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineColor {
    Added,
    Deleted,
    Neutral,
}

/// One renderable code line within a chunk card.
pub struct CodeRow {
    pub line_number: usize,
    pub color: LineColor,
    pub content: String,
}

/// A reasoning callout anchored above a specific code line.
pub struct AnnotationRow {
    /// Chunk-relative line number (matches [`CodeRow::line_number`]) this
    /// annotation renders above.
    pub trigger_line: usize,
    pub label: String,
    pub reasoning: String,
}

/// Icons used for the two auto-placed per-chunk badges.
pub struct DrillChunkIcons {
    /// Placed in the icon column of a note's first row.
    pub note_marker: (&'static str, Color),
    /// Placed in the gutter badge column of a chunk's first rendered row
    /// (note row if present, otherwise the first code row) when approved.
    pub approved_badge: (&'static str, Color),
}

/// One leaf card in the Drill view (a review chunk, a detector site, ...).
/// Expansion/collapse state is entirely the caller's concern — `note_rows`
/// and `code_rows` must already reflect the current expand state.
pub trait DrillChunk {
    fn is_approved(&self) -> bool;
    /// `None` when there is no note; otherwise pre-wrapped/pre-truncated
    /// display rows for the current expand state.
    fn note_rows(&self) -> Option<Vec<String>>;
    /// Pre-filtered for the current expand state (e.g. context lines hidden
    /// until expanded).
    fn code_rows(&self) -> Vec<CodeRow>;
    /// Empty when annotations are toggled off.
    fn annotations(&self) -> Vec<AnnotationRow>;
}

/// Render the Drill view: pinned file header, optional dot pagination for
/// sibling files, then chunk cards scrolling below.
#[allow(clippy::too_many_arguments)]
pub fn render_drill<C: DrillChunk>(
    f: &mut Frame,
    area: Rect,
    header: FileHeader,
    siblings: Option<(usize, usize)>,
    chunks: &[C],
    cursor: usize,
    page_offset: usize,
    theme: &Theme,
    icons: &DrillChunkIcons,
) {
    let cr = content_rect(area);
    let text_width = (cr.width as usize).saturating_sub(4);

    let header_card = HierarchicalCard::new(cr.width);
    let mut header_lines: Vec<Line<'static>> = Vec::new();
    let file_row_card = match header.badge {
        Some((ch, color)) => header_card.with_badge(ch, color),
        None => header_card,
    };
    header_lines.push(file_row_card.at(
        CardTier::Header,
        vec![Span::styled(
            header.label,
            Style::default()
                .fg(theme.surface.text())
                .add_modifier(Modifier::BOLD),
        )],
        theme,
    ));
    for text_line in wrap_text(&header.reasoning, text_width) {
        header_lines.push(header_card.at(
            CardTier::Header,
            vec![Span::styled(
                text_line,
                Style::default().fg(theme.surface.subtext1()),
            )],
            theme,
        ));
    }
    let below_header_area = render_drill_header(f, cr, header_lines, theme);

    // Dot pagination — one mantle-level line between the header and chunks.
    let content_area = if let Some((current, total)) = siblings {
        let [dots_area, rest] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(below_header_area);
        f.render_widget(
            Paragraph::new(dot_pagination_line(cr.width, current, total, theme)),
            dots_area,
        );
        rest
    } else {
        below_header_area
    };

    let n = chunks.len();
    let mut chunk_line_groups: Vec<Vec<Line<'static>>> = Vec::with_capacity(n);

    for (i, chunk) in chunks.iter().enumerate() {
        let mut lines: Vec<Line<'static>> = Vec::new();
        let is_approved = chunk.is_approved();
        let card = make_card(cr.width, i == cursor, theme.accents.lavender);
        let note = chunk.note_rows();
        let has_note = note.is_some();

        // Note rows — CardTier::Body, above the code.
        if let Some(rows) = &note {
            for (row, text_line) in rows.iter().enumerate() {
                let icon_col = if row == 0 {
                    Span::styled(
                        icons.note_marker.0,
                        Style::default().fg(icons.note_marker.1),
                    )
                } else {
                    Span::styled("  ", Style::default())
                };
                let row_card = if row == 0 && is_approved {
                    card.with_badge(icons.approved_badge.0, icons.approved_badge.1)
                } else {
                    card
                };
                lines.push(row_card.at(
                    CardTier::Body,
                    vec![
                        Span::styled(" ", Style::default()),
                        icon_col,
                        Span::styled(
                            format!("   {}", text_line),
                            Style::default().fg(theme.surface.subtext1()),
                        ),
                    ],
                    theme,
                ));
            }
        }

        // Code rows — CardTier::Content. Reasoning annotations (mauve, no
        // sigil) render just above their trigger line.
        let code_rows = chunk.code_rows();
        let annotations = chunk.annotations();
        let mut annotation_emitted = vec![false; annotations.len()];

        for (line_idx, row) in code_rows.iter().enumerate() {
            for (annotation, emitted) in annotations.iter().zip(annotation_emitted.iter_mut()) {
                if !*emitted && row.line_number >= annotation.trigger_line {
                    lines.extend(annotation_rows(annotation, card, text_width, theme));
                    *emitted = true;
                }
            }

            let (fg, sigil) = match row.color {
                LineColor::Added => (theme.accents.green, "+"),
                LineColor::Deleted => (theme.accents.red, "-"),
                LineColor::Neutral => (theme.surface.subtext0(), " "),
            };
            let row_card = if !has_note && line_idx == 0 && is_approved {
                card.with_badge(icons.approved_badge.0, icons.approved_badge.1)
            } else {
                card
            };
            lines.push(row_card.at(
                CardTier::Content,
                vec![
                    Span::styled(
                        format!("{:>3} ", row.line_number),
                        Style::default().fg(theme.surface.overlay0()),
                    ),
                    Span::styled(sigil, Style::default().fg(fg).add_modifier(Modifier::BOLD)),
                    Span::styled(format!(" {}", row.content), Style::default().fg(fg)),
                ],
                theme,
            ));
        }

        chunk_line_groups.push(lines);
    }

    let mut chunk_lines: Vec<Line<'static>> = Vec::new();
    let mut heights: Vec<u16> = Vec::with_capacity(n);
    for (i, group) in chunk_line_groups.into_iter().enumerate() {
        let h = group.len() as u16;
        chunk_lines.extend(group);
        if i + 1 < n {
            chunk_lines.push(Line::styled(
                format!("{:^width$}", "···", width = cr.width as usize),
                Style::default()
                    .fg(theme.surface.overlay0())
                    .bg(theme.surface.mantle()),
            ));
            heights.push(h + 1);
        } else {
            heights.push(h);
        }
    }

    // Focused chunk stays visible; the page offset (unclamped upstream) is
    // clamped here against the real content height.
    let base_scroll = scroll_into_view(&heights, cursor, content_area.height);
    let total_height: u16 = heights.iter().sum();
    let max_scroll = total_height.saturating_sub(content_area.height);
    let scroll = base_scroll
        .saturating_add(page_offset.min(u16::MAX as usize) as u16)
        .min(max_scroll);
    f.render_widget(
        Paragraph::new(chunk_lines).scroll((scroll, 0)),
        content_area,
    );
}

/// Wrapped rows for one reasoning annotation: mauve, no sigil.
fn annotation_rows(
    annotation: &AnnotationRow,
    card: HierarchicalCard,
    text_width: usize,
    theme: &Theme,
) -> Vec<Line<'static>> {
    wrap_text(
        &format!("{}  {}", annotation.label, annotation.reasoning),
        text_width,
    )
    .into_iter()
    .map(|text_line| {
        card.at(
            CardTier::Content,
            vec![
                Span::styled("    ", Style::default()),
                Span::styled(text_line, Style::default().fg(theme.accents.mauve)),
            ],
            theme,
        )
    })
    .collect()
}
