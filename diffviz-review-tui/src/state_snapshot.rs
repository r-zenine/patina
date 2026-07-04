//! Serialize UI state for testing
//!
//! Converts UiState to JSON-serializable StateSnapshot containing
//! test-relevant DrillNav fields.

use crate::state::{InputMode, UiState};
use serde::{Deserialize, Serialize};

/// JSON-serializable snapshot of UI state for testing
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct StateSnapshot {
    /// Current input mode: "Navigation", "Instruction", or "DecisionInstruction"
    pub input_mode: String,

    /// Input buffer content
    pub input_buffer: String,

    /// Input cursor position
    pub input_cursor: usize,

    /// Application should quit
    pub should_quit: bool,

    /// Whether leader key is active
    pub leader_active: bool,

    /// Leader submenu (if active)
    pub leader_submenu: Option<char>,

    /// Whether to show help
    pub show_help: bool,

    /// Whether to show inline reasoning annotations
    pub show_reasoning: bool,

    /// DrillNav mode: "Browse" or "Drill"
    pub nav_mode: String,

    /// Cursor over decision cards; Some only while browsing
    pub browse_cursor: Option<usize>,

    /// Drilled decision index; Some only while drilled in
    pub drill_decision: Option<usize>,

    /// Focused sibling-file index within the drilled decision
    pub drill_file: Option<usize>,

    /// Chunk cursor within the focused file
    pub drill_chunk: Option<usize>,

    /// Whether the focused chunk has expanded code context
    pub drill_context_expanded: Option<bool>,

    /// Whether the focused chunk has its note expanded
    pub drill_note_expanded: Option<bool>,

    /// Drill viewport page offset for the focused file
    pub drill_page_offset: Option<usize>,

    /// One-shot status-bar error message (cleared on next keypress)
    pub status_message: Option<String>,
}

impl StateSnapshot {
    /// Create snapshot from UI state
    pub fn from_ui_state(ui_state: &UiState) -> Self {
        Self {
            input_mode: match &ui_state.input_mode {
                InputMode::Navigation => "Navigation".to_string(),
                InputMode::Instruction { .. } => "Instruction".to_string(),
                InputMode::DecisionInstruction { .. } => "DecisionInstruction".to_string(),
            },
            input_buffer: ui_state.input_buffer.clone(),
            input_cursor: ui_state.input_cursor,
            should_quit: ui_state.should_quit,
            leader_active: ui_state.leader_active,
            leader_submenu: ui_state.leader_submenu,
            show_help: ui_state.show_help,
            show_reasoning: ui_state.show_reasoning,
            nav_mode: ui_state.nav_mode().to_string(),
            browse_cursor: ui_state.browse_cursor(),
            drill_decision: ui_state.drill_position().map(|(d, _, _)| d),
            drill_file: ui_state.drill_position().map(|(_, f, _)| f),
            drill_chunk: ui_state.drill_position().map(|(_, _, c)| c),
            drill_context_expanded: ui_state.drill_context_expanded(),
            drill_note_expanded: ui_state.drill_note_expanded(),
            drill_page_offset: ui_state.drill_page_offset(),
            status_message: ui_state.status_message().map(str::to_string),
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
            input_mode: "Navigation".to_string(),
            input_buffer: String::new(),
            input_cursor: 0,
            should_quit: false,
            leader_active: false,
            leader_submenu: None,
            show_help: false,
            show_reasoning: false,
            nav_mode: "Browse".to_string(),
            browse_cursor: Some(0),
            drill_decision: None,
            drill_file: None,
            drill_chunk: None,
            drill_context_expanded: None,
            drill_note_expanded: None,
            drill_page_offset: None,
            status_message: None,
        };

        let json = snapshot.to_json().unwrap();
        assert!(json.contains("\"nav_mode\": \"Browse\""));

        let deserialized = StateSnapshot::from_json(&json).unwrap();
        assert_eq!(deserialized.nav_mode, "Browse");
    }
}
