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
// Terminal background should be set to mantle (#181825) — the true floor.
// Panels rise from there: base → surface0 → surface1

pub fn layer_base(theme: &Theme) -> Style {
    Style::default().bg(theme.surface.crust())
}

pub fn layer_raised(theme: &Theme) -> Style {
    Style::default().bg(theme.surface.surface0())
}

pub fn layer_elevated(theme: &Theme) -> Style {
    Style::default().bg(theme.surface.surface1())
}

// --- State (the only two bg exceptions) ---

pub fn selection(theme: &Theme) -> Style {
    Style::default()
        .fg(theme.surface.text())
        .bg(theme.surface.surface0())
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
