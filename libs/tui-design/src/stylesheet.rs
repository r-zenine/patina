use ratatui::style::{Modifier, Style};

use crate::tokens::Theme;

// --- Borders ---

pub fn border(theme: &Theme) -> Style {
    Style::default().fg(theme.surface.overlay0())
}

pub fn border_focused(theme: &Theme) -> Style {
    Style::default().fg(theme.surface.overlay2())
}

// --- Text hierarchy (fg + modifier only, no bg) ---

pub fn title_active(theme: &Theme) -> Style {
    Style::default()
        .fg(theme.surface.text())
        .add_modifier(Modifier::BOLD)
}

pub fn title_inactive(theme: &Theme) -> Style {
    Style::default().fg(theme.surface.subtext1())
}

pub fn body(theme: &Theme) -> Style {
    Style::default().fg(theme.surface.text())
}

pub fn muted(theme: &Theme) -> Style {
    Style::default().fg(theme.surface.subtext0())
}

pub fn metadata(theme: &Theme) -> Style {
    Style::default()
        .fg(theme.surface.subtext0())
        .add_modifier(Modifier::ITALIC)
}

// --- Interactive ---

pub fn keybind_key(theme: &Theme) -> Style {
    Style::default()
        .fg(theme.accents.lavender)
        .add_modifier(Modifier::BOLD)
}

pub fn keybind_desc(theme: &Theme) -> Style {
    Style::default().fg(theme.surface.subtext0())
}

// --- Surface layers (bg-based elevation, dark theme: lighter = higher) ---
// Canonical elevation ramp (matches `CardTier` in card.rs):
//   crust    → terminal floor: fill the full frame     = terminal_floor
//   mantle   → widget floor: separators, gaps,
//              pinned-header containers                = widget_floor
//   base     → raw content (code lines, diffs)         = CardTier::Content
//   surface0 → secondary info, metadata, panels        = CardTier::Body / layer_raised
//   surface1 → labels, summaries, modals, popups       = CardTier::Header / layer_elevated
//   surface2 → selection state only — never structural = selection
//
// `layer_raised`/`layer_elevated` are the panel-granularity view of the same
// two tiers `CardTier::Body`/`CardTier::Header` use per-row; sharing those
// colors is intentional alignment, not a collision.

/// Terminal floor (crust). Fill the whole frame with this before drawing.
pub fn terminal_floor(theme: &Theme) -> Style {
    Style::default().bg(theme.surface.crust())
}

/// Widget floor (mantle) — separators, gaps, and pinned-header containers.
pub fn widget_floor(theme: &Theme) -> Style {
    Style::default().bg(theme.surface.mantle())
}

pub fn layer_raised(theme: &Theme) -> Style {
    Style::default().bg(theme.surface.surface0())
}

pub fn layer_elevated(theme: &Theme) -> Style {
    Style::default().bg(theme.surface.surface1())
}

// --- State (the only two bg exceptions) ---
//
// Selection sits on surface2 — one luminance step above the highest
// structural tier and used by nothing else — so a selected row can never
// blend into `CardTier` or layer elevation. In card-based views still prefer
// `HierarchicalCard::focused` (accent bar); use `selection` for flat lists
// and range highlights.

pub fn selection(theme: &Theme) -> Style {
    Style::default()
        .fg(theme.surface.text())
        .bg(theme.surface.surface2())
}

pub fn cursor(theme: &Theme) -> Style {
    Style::default()
        .fg(theme.surface.crust())
        .bg(theme.surface.text())
}

// --- Status bar ---

pub fn status_bar(theme: &Theme) -> Style {
    Style::default().fg(theme.surface.subtext1())
}

// --- Diff ---

pub fn diff_added(theme: &Theme) -> Style {
    Style::default().fg(theme.accents.green)
}

pub fn diff_removed(theme: &Theme) -> Style {
    Style::default().fg(theme.accents.red)
}

pub fn diff_modified(theme: &Theme) -> Style {
    Style::default().fg(theme.accents.yellow)
}

pub fn diff_context(theme: &Theme) -> Style {
    Style::default().fg(theme.surface.subtext0())
}

// --- File status ---

pub fn file_added(theme: &Theme) -> Style {
    Style::default().fg(theme.accents.green)
}

pub fn file_removed(theme: &Theme) -> Style {
    Style::default().fg(theme.accents.red)
}

pub fn file_modified(theme: &Theme) -> Style {
    Style::default().fg(theme.accents.yellow)
}

pub fn file_renamed(theme: &Theme) -> Style {
    Style::default().fg(theme.accents.sky)
}

pub fn file_copied(theme: &Theme) -> Style {
    Style::default().fg(theme.accents.mauve)
}

// --- Semantic status ---

pub fn success(theme: &Theme) -> Style {
    Style::default().fg(theme.accents.green)
}

pub fn warning(theme: &Theme) -> Style {
    Style::default().fg(theme.accents.peach)
}

pub fn error(theme: &Theme) -> Style {
    Style::default().fg(theme.accents.red)
}

pub fn info(theme: &Theme) -> Style {
    Style::default().fg(theme.accents.sky)
}
