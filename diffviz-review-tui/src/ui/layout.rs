//! TUI layout management and responsive design

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Main layout chunks for the TUI: full-width DrillNav content over a
/// one-line status bar.
pub struct MainLayout {
    pub content: Rect,
    pub status_bar: Rect,
}

/// Create the main layout with the DrillNav content area and status bar
pub fn create_main_layout(area: Rect) -> MainLayout {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),    // DrillNav content area
            Constraint::Length(1), // Status bar
        ])
        .split(area);

    MainLayout {
        content: main_chunks[0],
        status_bar: main_chunks[1],
    }
}

/// Create centered popup area for modals
pub fn centered_popup(area: Rect, width_percent: u16, height_percent: u16) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height_percent) / 2),
            Constraint::Percentage(height_percent),
            Constraint::Percentage((100 - height_percent) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_percent) / 2),
            Constraint::Percentage(width_percent),
            Constraint::Percentage((100 - width_percent) / 2),
        ])
        .split(popup_layout[1])[1]
}
