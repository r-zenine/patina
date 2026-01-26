//! Theme and styling definitions for the TUI
//!
//! This module defines colors, icons, and styles used throughout the interface,
//! providing a consistent visual experience.
//!
//! Uses a Dracula-inspired color scheme for excellent readability and reduced eye strain.

use ratatui::style::{Color, Modifier, Style};

/// Color palette for the TUI (Dracula-inspired)
pub struct Colors;

impl Colors {
    // Dracula base colors
    // Background: #282a36, Foreground: #f8f8f2, Selection: #44475a, Comment: #6272a4
    pub const BACKGROUND: Color = Color::Reset;
    pub const TEXT_PRIMARY: Color = Color::Rgb(248, 248, 242); // Off-white foreground
    pub const TEXT_SECONDARY: Color = Color::Rgb(98, 114, 164); // Dracula comment
    pub const TEXT_MUTED: Color = Color::Rgb(68, 71, 90); // Dracula selection
    pub const ACCENT_1: Color = Color::Rgb(139, 233, 253); // Dracula cyan (bright)
    pub const ACCENT_2: Color = Color::Rgb(189, 147, 249); // Dracula purple
    pub const ACCENT_3: Color = Color::Rgb(255, 166, 0); // Saturated orange (high contrast)

    // Status colors (Dracula-adjusted)
    pub const SUCCESS: Color = Color::Rgb(80, 250, 123); // Dracula green (bright)
    pub const WARNING: Color = Color::Rgb(214, 190, 110); // Muted golden yellow
    pub const ERROR: Color = Color::Rgb(255, 121, 198); // Dracula pink
    pub const INFO: Color = Color::Rgb(139, 233, 253); // Dracula cyan

    // Diff colors (Dracula-adjusted for better readability)
    pub const DIFF_ADDED: Color = Color::Rgb(80, 250, 123); // Dracula green
    pub const DIFF_REMOVED: Color = Color::Rgb(255, 121, 198); // Dracula pink
    pub const DIFF_CONTEXT: Color = Color::Rgb(98, 114, 164); // Dracula comment

    // File status colors (Dracula-adjusted)
    pub const FILE_ADDED: Color = Color::Rgb(80, 250, 123); // Green
    pub const FILE_DELETED: Color = Color::Rgb(255, 121, 198); // Pink
    pub const FILE_MODIFIED: Color = Color::Rgb(214, 190, 110); // Muted golden yellow
    pub const FILE_RENAMED: Color = Color::Rgb(139, 233, 253); // Cyan
    pub const FILE_COPIED: Color = Color::Rgb(189, 147, 249); // Purple

    // Additional UI colors (Dracula palette)
    pub const CYAN: Color = Color::Rgb(139, 233, 253);
    pub const RED: Color = Color::Rgb(255, 85, 85); // Dracula red
    pub const GREEN: Color = Color::Rgb(80, 250, 123);
    pub const BLUE: Color = Color::Rgb(139, 233, 253);
    pub const YELLOW: Color = Color::Rgb(214, 190, 110); // Muted golden yellow
    pub const WHITE: Color = Color::Rgb(248, 248, 242);
    pub const BLACK: Color = Color::Rgb(40, 42, 54);

    // UI element colors
    pub const BORDER: Color = Color::Rgb(98, 114, 164); // Dracula comment
    pub const BORDER_FOCUSED: Color = Color::Rgb(139, 233, 253); // Cyan for focus
    pub const SELECTION: Color = Color::Rgb(248, 248, 242); // Primary text
    pub const SELECTION_BG: Color = Color::Rgb(98, 114, 164); // Comment color bg
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

    pub fn diff_added_with_bg() -> Style {
        Style::default()
            .fg(Colors::DIFF_ADDED)
            .add_modifier(Modifier::BOLD)
    }

    pub fn diff_removed_with_bg() -> Style {
        Style::default()
            .fg(Colors::DIFF_REMOVED)
            .add_modifier(Modifier::BOLD)
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
