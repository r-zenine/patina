//! Theme and styling definitions for the TUI
//!
//! This module defines colors, icons, and styles used throughout the interface,
//! providing a consistent visual experience.

use ratatui::style::{Color, Style};

/// Color palette for the TUI
pub struct Colors;

impl Colors {
    // Base colors
    pub const BACKGROUND: Color = Color::Reset;
    pub const TEXT_PRIMARY: Color = Color::White;
    pub const TEXT_SECONDARY: Color = Color::Gray;
    pub const TEXT_MUTED: Color = Color::DarkGray;

    // Status colors
    pub const SUCCESS: Color = Color::Green;
    pub const WARNING: Color = Color::Yellow;
    pub const ERROR: Color = Color::Red;
    pub const INFO: Color = Color::Blue;

    // Diff colors
    pub const DIFF_ADDED: Color = Color::Green;
    pub const DIFF_REMOVED: Color = Color::Red;
    pub const DIFF_CONTEXT: Color = Color::Gray;
    pub const DIFF_ADDED_BG: Color = Color::Rgb(0, 64, 0);
    pub const DIFF_REMOVED_BG: Color = Color::Rgb(64, 0, 0);

    // File status colors
    pub const FILE_ADDED: Color = Color::Green;
    pub const FILE_DELETED: Color = Color::Red;
    pub const FILE_MODIFIED: Color = Color::Yellow;
    pub const FILE_RENAMED: Color = Color::Blue;
    pub const FILE_COPIED: Color = Color::Cyan;

    // Additional UI colors
    pub const CYAN: Color = Color::Cyan;
    pub const RED: Color = Color::Red;
    pub const GREEN: Color = Color::Green;
    pub const BLUE: Color = Color::Blue;
    pub const YELLOW: Color = Color::Yellow;
    pub const WHITE: Color = Color::White;
    pub const BLACK: Color = Color::Black;

    // UI element colors
    pub const BORDER: Color = Color::Gray;
    pub const BORDER_FOCUSED: Color = Color::Blue;
    pub const SELECTION: Color = Color::Blue;
    pub const SELECTION_BG: Color = Color::Rgb(0, 32, 64);
}

/// Icons and symbols used in the interface
pub struct Icons;

impl Icons {
    // File status icons
    pub const FILE_ADDED: &'static str = "+";
    pub const FILE_DELETED: &'static str = "-";
    pub const FILE_MODIFIED: &'static str = "~";
    pub const FILE_RENAMED: &'static str = "→";
    pub const FILE_COPIED: &'static str = "⊃";

    // Diff line prefixes
    pub const DIFF_ADDED: &'static str = "+";
    pub const DIFF_REMOVED: &'static str = "-";
    pub const DIFF_CONTEXT: &'static str = " ";

    // Review status icons
    pub const APPROVED: &'static str = "✓";
    pub const NOT_APPROVED: &'static str = "○";
    pub const HAS_COMMENTS: &'static str = "💬";
    pub const HAS_INSTRUCTIONS: &'static str = "📝";
    pub const HAS_SUGGESTIONS: &'static str = "💡";
    pub const PENDING: &'static str = "⏳";
    pub const PARTIAL: &'static str = "⚠";
    pub const COMMENT: &'static str = "💬";

    // Navigation icons
    pub const FOLDER_CLOSED: &'static str = "📁";
    pub const FOLDER_OPEN: &'static str = "📂";
    pub const FILE: &'static str = "📄";

    // Input mode icons
    pub const COMMENT_MODE: &'static str = "💬";
    pub const INSTRUCTION_MODE: &'static str = "📝";
    pub const EDIT_MODE: &'static str = "✏️";
}

/// Common styles used throughout the interface
pub struct Styles;

impl Styles {
    pub fn primary() -> Style {
        Style::default().fg(Colors::TEXT_PRIMARY)
    }

    pub fn secondary() -> Style {
        Style::default().fg(Colors::TEXT_SECONDARY)
    }

    pub fn muted() -> Style {
        Style::default().fg(Colors::TEXT_MUTED)
    }

    pub fn success() -> Style {
        Style::default().fg(Colors::SUCCESS)
    }

    pub fn warning() -> Style {
        Style::default().fg(Colors::WARNING)
    }

    pub fn error() -> Style {
        Style::default().fg(Colors::ERROR)
    }

    pub fn info() -> Style {
        Style::default().fg(Colors::INFO)
    }

    pub fn border() -> Style {
        Style::default().fg(Colors::BORDER)
    }

    pub fn border_focused() -> Style {
        Style::default().fg(Colors::BORDER_FOCUSED)
    }

    pub fn selection() -> Style {
        Style::default()
            .fg(Colors::SELECTION)
            .bg(Colors::SELECTION_BG)
    }

    pub fn diff_added() -> Style {
        Style::default().fg(Colors::DIFF_ADDED)
    }

    pub fn diff_removed() -> Style {
        Style::default().fg(Colors::DIFF_REMOVED)
    }

    pub fn diff_context() -> Style {
        Style::default().fg(Colors::DIFF_CONTEXT)
    }

    pub fn diff_added_bg() -> Style {
        Style::default()
            .fg(Colors::DIFF_ADDED)
            .bg(Colors::DIFF_ADDED_BG)
    }

    pub fn diff_removed_bg() -> Style {
        Style::default()
            .fg(Colors::DIFF_REMOVED)
            .bg(Colors::DIFF_REMOVED_BG)
    }

    pub fn focused_border() -> Style {
        Style::default().fg(Colors::BORDER_FOCUSED)
    }

    pub fn status_bar() -> Style {
        Style::default().fg(Colors::TEXT_PRIMARY)
    }

    pub fn scrollbar() -> Style {
        Style::default().fg(Colors::TEXT_SECONDARY)
    }
}
