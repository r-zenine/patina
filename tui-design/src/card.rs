use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use unicode_width::UnicodeWidthStr;

use crate::tokens::Theme;

// Width of the accent bar prefix on every card line ("▌ " or "  ").
const INDENT: usize = 2;

/// The three elevation tiers of a hierarchical card.
///
/// Maps content role to surface elevation:
/// - `Header`  → surface1 — node label, summary prose (pinnable when drilling in)
/// - `Body`    → surface0 — secondary info, metadata, children preview
/// - `Content` → base     — raw content: code lines, diffs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardTier {
    Header,
    Body,
    Content,
}

impl CardTier {
    /// Returns the background color for this tier from the given theme.
    pub fn bg(self, theme: &Theme) -> Color {
        match self {
            CardTier::Header => theme.surface.surface1(),
            CardTier::Body => theme.surface.surface0(),
            CardTier::Content => theme.surface.base(),
        }
    }
}

/// A focused hierarchical card that applies a side accent bar and surface
/// background to every line it produces.
///
/// Call `.focused(color)` to activate the accent bar; omit for unfocused cards.
/// Call `.with_badge(ch, color)` to place a single character in the second gutter
/// column — use this for per-row state indicators (e.g. approval `✓`).
/// Use `.line()` / `.blank()` / `.range_separator()` to produce lines, then
/// push them into your `Vec<Line>` before passing to a `Paragraph`.
///
/// Between-card separators (no accent, no card ownership) — use `separator_line`.
#[derive(Clone, Copy)]
pub struct HierarchicalCard {
    col_width: u16,
    accent: Option<Color>,
    /// Single character placed in the second gutter column with its own fg color.
    /// When `None` the column is a plain space. Width must be exactly 1 terminal cell.
    badge: Option<(&'static str, Color)>,
}

impl HierarchicalCard {
    pub fn new(col_width: u16) -> Self {
        Self {
            col_width,
            accent: None,
            badge: None,
        }
    }

    pub fn focused(mut self, color: Color) -> Self {
        self.accent = Some(color);
        self
    }

    /// Sets a single-character badge in the second gutter column.
    /// Returns a copy so the caller can use the badged variant for one row
    /// and the original for the rest.
    pub fn with_badge(mut self, ch: &'static str, color: Color) -> Self {
        debug_assert_eq!(
            ch.width(),
            1,
            "badge must be exactly 1 terminal cell wide, got {ch:?}"
        );
        self.badge = Some((ch, color));
        self
    }

    /// A content line: spans are given fg colors; bg and accent bar are applied here.
    pub fn line<'a>(&self, spans: Vec<Span<'a>>, bg: Color) -> Line<'a> {
        content_line(self.col_width, spans, bg, self.accent, self.badge)
    }

    /// A blank line at the given elevation (no content spans).
    pub fn blank(&self, bg: Color) -> Line<'static> {
        blank_line(self.col_width, bg, self.accent, self.badge)
    }

    /// A centered `···` line separating non-contiguous code ranges within this card.
    pub fn range_separator(&self, fg: Color, bg: Color) -> Line<'static> {
        range_separator_line(self.col_width, fg, bg, self.accent, self.badge)
    }

    /// Like `line`, but takes a `CardTier` instead of a raw background color.
    /// Prefer this over `line` when you want the tier's semantic elevation enforced.
    pub fn at<'a>(&self, tier: CardTier, spans: Vec<Span<'a>>, theme: &Theme) -> Line<'a> {
        self.line(spans, tier.bg(theme))
    }
}

/// Renders the parent node's label+summary as a sticky anchored header.
///
/// Wraps `content_lines` in top and bottom `separator_line`s, fills the area
/// with a mantle background, and returns the remaining `Rect` below for children.
///
/// Typical usage in a drill view:
/// ```ignore
/// let header_card = HierarchicalCard::new(cr.width);
/// let content_lines = vec![
///     header_card.at(CardTier::Header, label_spans, theme),
///     header_card.at(CardTier::Header, summary_spans, theme),
/// ];
/// let children_area = render_drill_header(frame, cr, content_lines, theme);
/// ```
pub fn render_drill_header<'a>(
    f: &mut Frame,
    area: Rect,
    content_lines: Vec<Line<'a>>,
    theme: &Theme,
) -> Rect {
    let header_height = (content_lines.len() as u16 + 1).min(area.height);
    let [header_area, children_area] =
        Layout::vertical([Constraint::Length(header_height), Constraint::Fill(1)]).areas(area);

    let mut lines: Vec<Line<'a>> = Vec::with_capacity(header_height as usize);
    lines.push(separator_line(area.width, theme.surface.mantle()));
    lines.extend(content_lines);

    f.render_widget(
        Paragraph::new("").style(Style::default().bg(theme.surface.mantle())),
        header_area,
    );
    f.render_widget(Paragraph::new(lines), header_area);

    children_area
}

/// Blank accent-free line — used for between-card gaps, never owned by a card.
pub fn separator_line(col_width: u16, bg: Color) -> Line<'static> {
    blank_line(col_width, bg, None, None)
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Produces the two gutter characters: accent glyph (col 0) and badge (col 1).
fn accent_bar(
    accent: Option<Color>,
    badge: Option<(&'static str, Color)>,
    bg: Color,
) -> [Span<'static>; 2] {
    let col0 = match accent {
        Some(c) => Span::styled("▌", Style::default().fg(c).bg(bg)),
        None => Span::styled(" ", Style::default().bg(bg)),
    };
    let col1 = match badge {
        Some((ch, color)) => Span::styled(ch, Style::default().fg(color).bg(bg)),
        None => Span::styled(" ", Style::default().bg(bg)),
    };
    [col0, col1]
}

fn content_line<'a>(
    col_width: u16,
    spans: Vec<Span<'a>>,
    bg: Color,
    accent: Option<Color>,
    badge: Option<(&'static str, Color)>,
) -> Line<'a> {
    // Measure in terminal cells, not chars — double-width glyphs (emoji, CJK)
    // would otherwise skew the trailing background fill.
    let content_len: usize = spans.iter().map(|s| s.content.width()).sum();
    let used = INDENT + content_len;
    let trailing = (col_width as usize).saturating_sub(used);
    let [g0, g1] = accent_bar(accent, badge, bg);
    let mut all: Vec<Span<'a>> = vec![g0, g1];
    for s in spans {
        all.push(Span::styled(s.content, s.style.bg(bg)));
    }
    all.push(Span::styled(" ".repeat(trailing), Style::default().bg(bg)));
    Line::from(all)
}

fn blank_line(
    col_width: u16,
    bg: Color,
    accent: Option<Color>,
    badge: Option<(&'static str, Color)>,
) -> Line<'static> {
    let trailing = (col_width as usize).saturating_sub(INDENT);
    let [g0, g1] = accent_bar(accent, badge, bg);
    Line::from(vec![
        g0,
        g1,
        Span::styled(" ".repeat(trailing), Style::default().bg(bg)),
    ])
}

fn range_separator_line(
    col_width: u16,
    fg: Color,
    bg: Color,
    accent: Option<Color>,
    badge: Option<(&'static str, Color)>,
) -> Line<'static> {
    let left_pad = (col_width as usize).saturating_sub(3) / 2;
    let pad = left_pad.saturating_sub(INDENT);
    let trailing = (col_width as usize).saturating_sub(INDENT + pad + 3);
    let [g0, g1] = accent_bar(accent, badge, bg);
    Line::from(vec![
        g0,
        g1,
        Span::styled(" ".repeat(pad), Style::default().bg(bg)),
        Span::styled("···", Style::default().fg(fg).bg(bg)),
        Span::styled(" ".repeat(trailing), Style::default().bg(bg)),
    ])
}
