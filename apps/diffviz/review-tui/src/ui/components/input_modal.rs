//! Input modal component for comments, instructions, and edits.
//!
//! The modal chrome (popup framing, cursor rendering) is generic and lives
//! in `tui_design::input_modal`; this module only resolves the review-domain
//! title/placeholder text for the current `InputMode`.

use ratatui::{Frame, layout::Rect};
use tui_design::Theme;
use tui_design::input_modal as generic;

use crate::state::{InputMode, UiState};
use diffviz_review::engines::ReviewEngine;

/// Render input modal for text input modes
pub fn render(f: &mut Frame, area: Rect, ui_state: &UiState, review_engine: &ReviewEngine) {
    if !ui_state.is_in_input_mode() {
        return;
    }

    let theme = Theme::mocha();
    let (title, _prompt, placeholder) = get_modal_content(&ui_state.input_mode, review_engine);

    generic::render(
        f,
        area,
        &title,
        &ui_state.input_buffer,
        ui_state.input_cursor,
        placeholder,
        &theme,
    );
}

fn get_modal_content(
    input_mode: &InputMode,
    review_engine: &ReviewEngine,
) -> (String, &'static str, &'static str) {
    match input_mode {
        InputMode::Instruction { reviewable_id } => {
            let existing = review_engine
                .state()
                .get_instructions(reviewable_id)
                .and_then(|v| v.first());
            let title = match existing {
                Some(instr) => {
                    format!("Edit {}'s note - {}", instr.author, reviewable_id.file_path)
                }
                None => format!("Add Instruction - {}", reviewable_id.file_path),
            };
            (
                title,
                "Enter your instruction for this diff:",
                "Type your instruction here...",
            )
        }
        InputMode::DecisionInstruction { decision_number } => {
            let existing = review_engine
                .get_decision_instructions(*decision_number)
                .and_then(|v| v.into_iter().next());
            let title = match existing {
                Some(instr) => {
                    format!("Edit {}'s note - Decision #{decision_number}", instr.author)
                }
                None => format!("Add Instruction - Decision #{decision_number}"),
            };
            (
                title,
                "Enter your instruction for this decision:",
                "Type your instruction here...",
            )
        }
        InputMode::Navigation => ("".to_string(), "", ""),
    }
}
