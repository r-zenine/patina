//! Shared rendering helpers for the DrillNav views.
//!
//! Layout: content capped at 120 columns, centered; surface bg fills full
//! column width. Surface ramp (dark theme, lighter = higher elevation):
//!   decision/file header → surface1 (CardTier::Header — pinnable)
//!   notes, file preview  → surface0 (CardTier::Body)
//!   code lines           → base     (CardTier::Content)
//!   separators, dots     → mantle   (widget floor)

use diffviz_core::RenderableLine;
use diffviz_core::renderable_diff::ChangeType;
use diffviz_review::{Instruction, ReviewableDiffId, engines::ReviewEngine};
use ratatui::prelude::*;
use tui_design::{HierarchicalCard, Theme};

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

/// The chunk's single note. One note per chunk is a product invariant —
/// adding an instruction to an annotated chunk appends to the existing note.
pub fn note_for<'a>(
    engine: &'a ReviewEngine,
    chunk_id: &ReviewableDiffId,
) -> Option<&'a Instruction> {
    engine
        .state()
        .get_instructions(chunk_id)
        .and_then(|v| v.first())
}

/// Wrapped display rows for a note: authors prefix the first contribution;
/// each appended contribution starts on its own row (note content is
/// newline-separated under the single-note model).
pub fn note_rows(instr: &Instruction, wrap_width: usize) -> Vec<String> {
    let text = format!("{}: {}", instr.author, instr.content);
    text.split('\n')
        .flat_map(|segment| wrap_text(segment, wrap_width))
        .collect()
}

pub fn line_change_type(line: &RenderableLine<'_>) -> Option<ChangeType> {
    line.annotations.first().and_then(|a| a.change_type.clone())
}

pub fn line_has_change(line: &RenderableLine<'_>) -> bool {
    line.annotations
        .first()
        .and_then(|a| a.change_type.as_ref())
        .is_some()
}
