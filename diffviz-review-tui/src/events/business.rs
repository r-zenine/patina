//! Business event handling that requires ReviewEngine operations
//!
//! This module defines business events that represent review operations
//! and provides conversion from UI events to business events.

use crate::events::UiEvent;
use crate::state::{InputMode, UiState};
use diffviz_review::engines::review_engine::ExportScope;
use diffviz_review::ReviewableDiffId;

/// Business events that require ReviewEngine operations
#[derive(Debug, Clone)]
pub enum BusinessEvent {
    /// Approve or unapprove a ReviewableDiff
    ToggleApprove { reviewable_id: ReviewableDiffId },

    /// Approve all ReviewableDiffs in a file
    ApproveFile { file_path: String },

    /// Add an instruction to a ReviewableDiff
    AddInstruction {
        reviewable_id: ReviewableDiffId,
        content: String,
    },

    /// Edit content of a ReviewableDiff
    EditContent {
        reviewable_id: ReviewableDiffId,
        new_content: String,
    },

    /// Export instructions to JSON file
    ExportInstructions { scope: ExportScope },

    /// Save current review session
    SaveSession,

    /// Load a previous review session
    LoadSession { session_id: String },
}

/// Convert UI events to business events based on current state
pub fn ui_event_to_business_event(ui_event: &UiEvent, ui_state: &UiState) -> Option<BusinessEvent> {
    match ui_event {
        UiEvent::ToggleApprove => {
            ui_state
                .current_reviewable_id
                .as_ref()
                .map(|id| BusinessEvent::ToggleApprove {
                    reviewable_id: id.clone(),
                })
        }

        UiEvent::ApproveFile => {
            ui_state
                .current_file_path
                .as_ref()
                .map(|path| BusinessEvent::ApproveFile {
                    file_path: path.clone(),
                })
        }

        UiEvent::SubmitInput => match &ui_state.input_mode {
            InputMode::Instruction { reviewable_id } => Some(BusinessEvent::AddInstruction {
                reviewable_id: reviewable_id.clone(),
                content: ui_state.input_buffer.clone(),
            }),

            InputMode::Edit { reviewable_id } => Some(BusinessEvent::EditContent {
                reviewable_id: reviewable_id.clone(),
                new_content: ui_state.input_buffer.clone(),
            }),

            InputMode::Navigation => None,
        },

        UiEvent::ExportFile => {
            ui_state
                .current_file_path
                .as_ref()
                .map(|path| BusinessEvent::ExportInstructions {
                    scope: ExportScope::SingleFile(path.clone()),
                })
        }

        UiEvent::ExportSingleInstruction => {
            ui_state
                .current_reviewable_id
                .as_ref()
                .map(|id| BusinessEvent::ExportInstructions {
                    scope: ExportScope::SingleInstruction(id.clone()),
                })
        }

        UiEvent::ExportAll => Some(BusinessEvent::ExportInstructions {
            scope: ExportScope::All,
        }),

        // Other UI events don't generate business events
        _ => None,
    }
}
