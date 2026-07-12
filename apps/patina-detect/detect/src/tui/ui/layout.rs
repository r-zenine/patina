//! TUI layout management.

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Main layout chunks: full-width DrillNav content over a one-line status bar.
pub struct MainLayout {
    pub content: Rect,
    pub status_bar: Rect,
}

pub fn create_main_layout(area: Rect) -> MainLayout {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(area);

    MainLayout {
        content: main_chunks[0],
        status_bar: main_chunks[1],
    }
}
