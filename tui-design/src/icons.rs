pub struct Icons;

impl Icons {
    // File status
    pub const FILE_ADDED: &'static str = "+";
    pub const FILE_DELETED: &'static str = "-";
    pub const FILE_MODIFIED: &'static str = "~";
    pub const FILE_RENAMED: &'static str = "→";
    pub const FILE_COPIED: &'static str = "⊃";

    // Diff line prefixes
    pub const DIFF_ADDED: &'static str = "+";
    pub const DIFF_REMOVED: &'static str = "-";
    pub const DIFF_CONTEXT: &'static str = " ";

    // Review status
    pub const APPROVED: &'static str = "✓";
    pub const NOT_APPROVED: &'static str = "○";
    pub const HAS_COMMENTS: &'static str = "💬";
    pub const HAS_INSTRUCTIONS: &'static str = "📝";
    pub const HAS_SUGGESTIONS: &'static str = "💡";
    pub const PENDING: &'static str = "⏳";
    pub const PARTIAL: &'static str = "⚠";
    pub const COMMENT: &'static str = "💬";

    // Navigation
    pub const FOLDER_CLOSED: &'static str = "📁";
    pub const FOLDER_OPEN: &'static str = "📂";
    pub const FILE: &'static str = "📄";

    // Input mode
    pub const COMMENT_MODE: &'static str = "💬";
    pub const INSTRUCTION_MODE: &'static str = "📝";
    pub const EDIT_MODE: &'static str = "✏️";
}
