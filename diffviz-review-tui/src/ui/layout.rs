//! TUI layout management and responsive design

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Main layout chunks for the TUI
pub struct MainLayout {
    pub file_list: Rect,
    pub diff_view: Rect,
    pub status_bar: Rect,
}

/// Create the main layout with file list, diff view, and status bar
pub fn create_main_layout(area: Rect) -> MainLayout {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),    // Main content area
            Constraint::Length(1), // Status bar
        ])
        .split(area);

    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25), // Rail panel
            Constraint::Length(1),      // Gap — exposes base bg, creates elevation contrast
            Constraint::Min(0),         // Diff view — takes all remaining width
        ])
        .split(main_chunks[0]);

    MainLayout {
        file_list: content_chunks[0],
        diff_view: content_chunks[2],
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
