use ratatui::style::{Modifier, Style};

use crate::tokens::Theme;

// --- Borders ---

pub fn border(theme: &Theme) -> Style {
    Style::default().fg(theme.surface[3])
}

pub fn border_focused(theme: &Theme) -> Style {
    Style::default().fg(theme.surface[4])
}

// --- Text hierarchy (fg + modifier only, no bg) ---

pub fn title_active(theme: &Theme) -> Style {
    Style::default()
        .fg(theme.surface[7])
        .add_modifier(Modifier::BOLD)
}

pub fn title_inactive(theme: &Theme) -> Style {
    Style::default().fg(theme.surface[6])
}

pub fn body(theme: &Theme) -> Style {
    Style::default().fg(theme.surface[7])
}

pub fn muted(theme: &Theme) -> Style {
    Style::default().fg(theme.surface[5])
}

pub fn metadata(theme: &Theme) -> Style {
    Style::default()
        .fg(theme.surface[5])
        .add_modifier(Modifier::ITALIC)
}

// --- Interactive ---

pub fn keybind_key(theme: &Theme) -> Style {
    Style::default()
        .fg(theme.accents.lavender)
        .add_modifier(Modifier::BOLD)
}

pub fn keybind_desc(theme: &Theme) -> Style {
    Style::default().fg(theme.surface[5])
}

// --- State (the only two bg exceptions) ---

pub fn selection(theme: &Theme) -> Style {
    Style::default()
        .fg(theme.surface[7])
        .bg(theme.surface[2])
}

pub fn cursor(theme: &Theme) -> Style {
    Style::default()
        .fg(theme.surface[0])
        .bg(theme.surface[7])
}

// --- Status bar ---

pub fn status_bar(theme: &Theme) -> Style {
    Style::default().fg(theme.surface[6])
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
    Style::default().fg(theme.surface[5])
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
