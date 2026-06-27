use ratatui::prelude::*;

// Width of the accent bar prefix on every card line ("▌ " or "  ").
const INDENT: usize = 2;

/// A focused hierarchical card that applies a side accent bar and surface
/// background to every line it produces.
///
/// Call `.focused(color)` to activate the accent bar; omit for unfocused cards.
/// Use `.line()` / `.blank()` / `.range_separator()` to produce lines, then
/// push them into your `Vec<Line>` before passing to a `Paragraph`.
///
/// Between-card separators (no accent, no card ownership) — use `separator_line`.
pub struct HierarchicalCard {
    col_width: u16,
    accent: Option<Color>,
}

impl HierarchicalCard {
    pub fn new(col_width: u16) -> Self {
        Self { col_width, accent: None }
    }

    pub fn focused(mut self, color: Color) -> Self {
        self.accent = Some(color);
        self
    }

    /// A content line: spans are given fg colors; bg and accent bar are applied here.
    pub fn line<'a>(&self, spans: Vec<Span<'a>>, bg: Color) -> Line<'a> {
        content_line(self.col_width, spans, bg, self.accent)
    }

    /// A blank line at the given elevation (no content spans).
    pub fn blank(&self, bg: Color) -> Line<'static> {
        blank_line(self.col_width, bg, self.accent)
    }

    /// A centered `···` line separating non-contiguous code ranges within this card.
    pub fn range_separator(&self, fg: Color, bg: Color) -> Line<'static> {
        range_separator_line(self.col_width, fg, bg, self.accent)
    }
}

/// Blank accent-free line — used for between-card gaps, never owned by a card.
pub fn separator_line(col_width: u16, bg: Color) -> Line<'static> {
    blank_line(col_width, bg, None)
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn accent_bar(accent: Option<Color>, bg: Color) -> Span<'static> {
    match accent {
        Some(c) => Span::styled("▌ ", Style::default().fg(c).bg(bg)),
        None => Span::styled("  ", Style::default().bg(bg)),
    }
}

fn content_line<'a>(col_width: u16, spans: Vec<Span<'a>>, bg: Color, accent: Option<Color>) -> Line<'a> {
    let content_len: usize = spans.iter().map(|s| s.content.chars().count()).sum();
    let used = INDENT + content_len;
    let trailing = (col_width as usize).saturating_sub(used);
    let mut all: Vec<Span<'a>> = vec![accent_bar(accent, bg)];
    for s in spans {
        all.push(Span::styled(s.content, s.style.bg(bg)));
    }
    all.push(Span::styled(" ".repeat(trailing), Style::default().bg(bg)));
    Line::from(all)
}

fn blank_line(col_width: u16, bg: Color, accent: Option<Color>) -> Line<'static> {
    let trailing = (col_width as usize).saturating_sub(INDENT);
    Line::from(vec![
        accent_bar(accent, bg),
        Span::styled(" ".repeat(trailing), Style::default().bg(bg)),
    ])
}

fn range_separator_line(col_width: u16, fg: Color, bg: Color, accent: Option<Color>) -> Line<'static> {
    let left_pad = (col_width as usize).saturating_sub(3) / 2;
    let pad = left_pad.saturating_sub(INDENT);
    let trailing = (col_width as usize).saturating_sub(INDENT + pad + 3);
    Line::from(vec![
        accent_bar(accent, bg),
        Span::styled(" ".repeat(pad), Style::default().bg(bg)),
        Span::styled("···", Style::default().fg(fg).bg(bg)),
        Span::styled(" ".repeat(trailing), Style::default().bg(bg)),
    ])
}
