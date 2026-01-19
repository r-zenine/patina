//! ANSI color constants for CLI output formatting

/// ANSI color codes for git-diff style output
pub struct Colors;

impl Colors {
    pub const HEADER: &'static str = "\x1b[96m"; // Bright Cyan
    pub const UNCHANGED: &'static str = "\x1b[37m"; // Light Gray
    pub const ADDED: &'static str = "\x1b[32m"; // Green
    pub const DELETED: &'static str = "\x1b[31m"; // Red
    pub const MODIFIED: &'static str = "\x1b[33m"; // Yellow
    pub const RESET: &'static str = "\x1b[0m";
}
