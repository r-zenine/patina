//! Serialize UI state for testing.

use crate::state::{InputMode, UiState};
use serde::{Deserialize, Serialize};

/// JSON-serializable snapshot of UI state for testing.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct StateSnapshot {
    /// Current input mode: "Navigation" or "FixInstruction".
    pub input_mode: String,
    pub input_buffer: String,
    pub input_cursor: usize,
    pub should_quit: bool,
    pub leader_active: bool,
    pub leader_submenu: Option<char>,
    pub show_help: bool,

    /// DrillNav mode: "Browse" or "Drill".
    pub nav_mode: String,
    /// Cursor over symptom cards; Some only while browsing.
    pub browse_symptom: Option<usize>,
    /// Drilled symptom index; Some only while drilled in.
    pub drill_symptom: Option<usize>,
    /// Site cursor within the focused symptom.
    pub drill_site: Option<usize>,
    /// Drill viewport page offset for the focused symptom.
    pub drill_page_offset: Option<usize>,

    pub status_message: Option<String>,
}

impl StateSnapshot {
    pub fn from_ui_state(ui_state: &UiState) -> Self {
        Self {
            input_mode: match &ui_state.input_mode {
                InputMode::Navigation => "Navigation".to_string(),
                InputMode::FixInstruction { .. } => "FixInstruction".to_string(),
            },
            input_buffer: ui_state.input_buffer.clone(),
            input_cursor: ui_state.input_cursor,
            should_quit: ui_state.should_quit,
            leader_active: ui_state.leader.is_active(),
            leader_submenu: ui_state.leader.submenu(),
            show_help: ui_state.show_help,
            nav_mode: ui_state.nav_mode().to_string(),
            browse_symptom: ui_state.browse_cursor(),
            drill_symptom: ui_state.drill_position().map(|(s, _)| s),
            drill_site: ui_state.drill_position().map(|(_, c)| c),
            drill_page_offset: ui_state.drill_page_offset(),
            status_message: ui_state.status_message().map(str::to_string),
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

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
            nav_mode: "Browse".to_string(),
            browse_symptom: Some(0),
            drill_symptom: None,
            drill_site: None,
            drill_page_offset: None,
            status_message: None,
        };

        let json = snapshot.to_json().unwrap();
        assert!(json.contains("\"nav_mode\": \"Browse\""));

        let deserialized = StateSnapshot::from_json(&json).unwrap();
        assert_eq!(deserialized.nav_mode, "Browse");
    }
}
