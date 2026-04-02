//! Serialize UI state for testing
//!
//! Converts UiState to JSON-serializable StateSnapshot containing
//! test-relevant fields including decision tree state.

use crate::state::{FocusPanel, InputMode, UiState};
use serde::{Deserialize, Serialize};

/// JSON-serializable snapshot of UI state for testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    /// Which panel has focus: "FileList" or "DiffView"
    pub focused_panel: String,

    /// Current cursor position in diff view
    pub cursor_index: usize,

    /// Current scroll offset
    pub scroll_offset: usize,

    /// Current input mode: "Navigation", "Instruction", or "Edit"
    pub input_mode: String,

    /// Input buffer content
    pub input_buffer: String,

    /// Input cursor position
    pub input_cursor: usize,

    /// Whether to show all context
    pub show_all_context: bool,

    /// Application should quit
    pub should_quit: bool,

    /// Current file list selection index
    pub file_list_selection: usize,

    /// Whether to highlight semantics
    pub highlight_semantics: bool,

    /// Whether leader key is active
    pub leader_active: bool,

    /// Leader submenu (if active)
    pub leader_submenu: Option<char>,

    /// Whether to show help
    pub show_help: bool,

    /// Whether to show instructions
    pub show_instructions: bool,

    /// Selection anchor (if active)
    pub selection_anchor: Option<usize>,

    /// Selection range (start, end) if active
    pub selection_range: Option<(usize, usize)>,

    /// Currently expanded files in the tree
    pub expanded_files: Vec<String>,

    /// Current decision tree selection path (decision_index, chunk_index)
    pub decision_tree_path: (usize, Option<usize>),
}

impl StateSnapshot {
    /// Create snapshot from UI state
    pub fn from_ui_state(ui_state: &UiState) -> Self {
        Self {
            focused_panel: match ui_state.focused_panel {
                FocusPanel::FileList => "FileList".to_string(),
                FocusPanel::DiffView => "DiffView".to_string(),
            },
            cursor_index: ui_state.cursor_index,
            scroll_offset: ui_state.scroll_offset,
            input_mode: match &ui_state.input_mode {
                InputMode::Navigation => "Navigation".to_string(),
                InputMode::Instruction { .. } => "Instruction".to_string(),
            },
            input_buffer: ui_state.input_buffer.clone(),
            input_cursor: ui_state.input_cursor,
            show_all_context: ui_state.show_all_context,
            should_quit: ui_state.should_quit,
            file_list_selection: ui_state.file_list_selection,
            highlight_semantics: ui_state.highlight_semantics,
            leader_active: ui_state.leader_active,
            leader_submenu: ui_state.leader_submenu,
            show_help: ui_state.show_help,
            show_instructions: ui_state.show_instructions,
            selection_anchor: ui_state.selection_anchor,
            selection_range: ui_state.selection_range,
            expanded_files: ui_state.expanded_files.iter().cloned().collect(),
            decision_tree_path: (
                ui_state.decision_tree.selected_path.decision_index,
                ui_state.decision_tree.selected_path.chunk_index,
            ),
        }
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_serialization() {
        let snapshot = StateSnapshot {
            focused_panel: "FileList".to_string(),
            cursor_index: 42,
            scroll_offset: 10,
            input_mode: "Navigation".to_string(),
            input_buffer: String::new(),
            input_cursor: 0,
            show_all_context: false,
            should_quit: false,
            file_list_selection: 0,
            highlight_semantics: true,
            leader_active: false,
            leader_submenu: None,
            show_help: false,
            show_instructions: false,
            selection_anchor: None,
            selection_range: None,
            expanded_files: vec![],
            decision_tree_path: (0, None),
        };

        let json = snapshot.to_json().unwrap();
        assert!(json.contains("\"cursor_index\": 42"));

        let deserialized = StateSnapshot::from_json(&json).unwrap();
        assert_eq!(deserialized.cursor_index, 42);
    }
}
