//! Main TUI application state and business logic integration
//!
//! This module contains the core ReviewTuiApp that coordinates between
//! the ReviewEngine (business logic) and the UI state (presentation).

use crossterm::event::KeyEvent;
use ratatui::Frame;

use diffviz_review::engines::ReviewEngine;
use tui_harness::ELMApp;

use crate::{
    Result,
    command::{Command, execute_command},
    error::ReviewTuiError,
    events::{BusinessEvent, UiEvent, handle_key_event, ui_event_to_business_event},
    state::UiState,
    state_snapshot::StateSnapshot,
    ui,
};

/// Main TUI application that coordinates ReviewEngine and UI
pub struct ReviewTuiApp {
    /// Business logic engine
    review_engine: ReviewEngine,

    /// UI navigation and display state
    ui_state: UiState,
}

impl ReviewTuiApp {
    /// Create a new TUI application
    pub fn new(review_engine: ReviewEngine) -> Result<Self> {
        let ui_state = Self::initialize_ui_state(&review_engine);
        Ok(Self {
            review_engine,
            ui_state,
        })
    }

    /// Initialize UI state from review engine
    fn initialize_ui_state(review_engine: &ReviewEngine) -> UiState {
        let mut ui_state = UiState::new();

        ui_state.decision_tree =
            crate::decision_navigation::DecisionNavigationTree::build_from_review_engine(
                review_engine,
            );

        if ui_state.decision_tree.selected_chunk_id().is_some() {
            ui_state.reset_scroll();
        }

        ui_state
    }

    /// Consume the app and return the underlying ReviewEngine.
    pub fn into_review_engine(self) -> ReviewEngine {
        self.review_engine
    }

    /// Run the main application loop using the tui-harness runtime.
    pub fn run(&mut self) -> Result<()> {
        tui_harness::run_app(self)?;
        Ok(())
    }

    /// Process a single key event and update state, returning command to execute
    pub fn process_key_event(&mut self, key: KeyEvent) -> Result<Command> {
        process_key_event_impl(&mut self.review_engine, &mut self.ui_state, key)
    }
}

impl ELMApp for ReviewTuiApp {
    type Snapshot = StateSnapshot;
    type Error = ReviewTuiError;

    fn dispatch_key(&mut self, key: KeyEvent) -> std::result::Result<(), ReviewTuiError> {
        let command = self
            .process_key_event(key)
            .map_err(|e| ReviewTuiError::KeyDispatch(e.into()))?;
        execute_command(command).map_err(|e| ReviewTuiError::CommandExecution(e.into()))?;
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        ui::draw(frame, &self.ui_state, &self.review_engine);
    }

    fn should_quit(&self) -> bool {
        self.ui_state.should_quit
    }

    fn snapshot(&self) -> StateSnapshot {
        StateSnapshot::from_ui_state(&self.ui_state)
    }

    fn on_tick(&mut self) {
        if self.ui_state.leader_active && self.ui_state.is_leader_timed_out() {
            self.ui_state.deactivate_leader();
        }
    }
}

// ---------------------------------------------------------------------------
// Shared implementations — called by both ReviewTuiApp and HeadlessApp
// ---------------------------------------------------------------------------

fn process_key_event_impl(
    engine: &mut ReviewEngine,
    ui_state: &mut UiState,
    key: KeyEvent,
) -> Result<Command> {
    if let Some(ui_event) = handle_key_event(
        key,
        &ui_state.input_mode,
        ui_state.leader_active,
        ui_state.leader_submenu,
    ) {
        handle_ui_event_impl(engine, ui_state, &ui_event)?;

        if let Some(business_event) = ui_event_to_business_event(&ui_event, ui_state) {
            let command = handle_business_event_impl(engine, business_event)?;

            if matches!(ui_event, UiEvent::SubmitInput) {
                ui_state.exit_input_mode();
            }

            return Ok(command);
        }
    }

    Ok(Command::None)
}

fn handle_ui_event_impl(
    engine: &ReviewEngine,
    ui_state: &mut UiState,
    event: &UiEvent,
) -> Result<()> {
    let total_lines = get_total_lines_impl(engine, ui_state);

    match event {
        UiEvent::Quit => {
            ui_state.quit();
        }

        UiEvent::NavigateUp => match ui_state.focused_panel {
            crate::state::FocusPanel::FileList => {
                ui_state.decision_tree.navigate_prev();
                ui_state.reset_scroll();
            }
            crate::state::FocusPanel::DiffView => {
                ui_state.cursor_up();
            }
        },

        UiEvent::NavigateDown => match ui_state.focused_panel {
            crate::state::FocusPanel::FileList => {
                ui_state.decision_tree.navigate_next();
                ui_state.reset_scroll();
            }
            crate::state::FocusPanel::DiffView => {
                ui_state.cursor_down(total_lines);
            }
        },

        UiEvent::NavigateLeft => {
            ui_state.focused_panel = crate::state::FocusPanel::FileList;
        }

        UiEvent::NavigateRight => {
            ui_state.focused_panel = crate::state::FocusPanel::DiffView;
        }

        UiEvent::NavigateToTop => match ui_state.focused_panel {
            crate::state::FocusPanel::FileList => {
                ui_state.navigate_to_first_in_tree();
            }
            crate::state::FocusPanel::DiffView => {
                ui_state.cursor_to_top();
            }
        },

        UiEvent::NavigateToBottom => match ui_state.focused_panel {
            crate::state::FocusPanel::FileList => {
                ui_state.navigate_to_last_in_tree();
            }
            crate::state::FocusPanel::DiffView => {
                ui_state.cursor_to_bottom(total_lines);
            }
        },

        UiEvent::NavigatePageUp => match ui_state.focused_panel {
            crate::state::FocusPanel::FileList => {
                ui_state.page_up();
            }
            crate::state::FocusPanel::DiffView => {
                ui_state.cursor_page_up(total_lines);
            }
        },

        UiEvent::NavigatePageDown => match ui_state.focused_panel {
            crate::state::FocusPanel::FileList => {
                ui_state.page_down();
            }
            crate::state::FocusPanel::DiffView => {
                ui_state.cursor_page_down(total_lines);
            }
        },

        UiEvent::ToggleFocus => {
            ui_state.toggle_focus();
        }

        UiEvent::ToggleContextDisplay => {
            ui_state.toggle_context_display();
            ui_state.deactivate_leader();
        }

        UiEvent::ScrollUp => {
            ui_state.scroll_up(1);
        }

        UiEvent::ScrollDown => {
            ui_state.scroll_down(1);
        }

        UiEvent::ScrollPageUp => {
            ui_state.page_up();
        }

        UiEvent::ScrollPageDown => {
            ui_state.page_down();
        }

        UiEvent::ScrollInactivePanelUp => match ui_state.focused_panel {
            crate::state::FocusPanel::FileList => {
                ui_state.scroll_up(1);
            }
            crate::state::FocusPanel::DiffView => {
                ui_state.decision_tree.navigate_prev();
                ui_state.reset_scroll();
            }
        },

        UiEvent::ScrollInactivePanelDown => match ui_state.focused_panel {
            crate::state::FocusPanel::FileList => {
                ui_state.scroll_down(1);
            }
            crate::state::FocusPanel::DiffView => {
                ui_state.decision_tree.navigate_next();
                ui_state.reset_scroll();
            }
        },

        UiEvent::EnterInstructionMode => {
            if let Some(reviewable_id) = ui_state.current_reviewable_id() {
                ui_state.start_instruction_input(reviewable_id);
            } else if let Some(decision_number) = ui_state.current_decision_number() {
                ui_state.start_decision_instruction_input(decision_number);
            }
            ui_state.deactivate_leader();
        }

        UiEvent::ExitInputMode | UiEvent::CancelInput => {
            ui_state.exit_input_mode();
        }

        UiEvent::InputChar(c) => {
            ui_state.input_char(*c);
        }

        UiEvent::DeleteChar => {
            ui_state.delete_char();
        }

        UiEvent::MoveCursorLeft => {
            ui_state.move_cursor_left();
        }

        UiEvent::MoveCursorRight => {
            ui_state.move_cursor_right();
        }

        UiEvent::MoveCursorHome => {
            ui_state.move_cursor_home();
        }

        UiEvent::MoveCursorEnd => {
            ui_state.move_cursor_end();
        }

        UiEvent::ToggleApprove | UiEvent::ApproveFile => {
            ui_state.deactivate_leader();
        }

        UiEvent::SubmitInput => {}

        UiEvent::SelectCurrent => match ui_state.focused_panel {
            crate::state::FocusPanel::FileList => {
                ui_state.decision_tree.expand_current();
                ui_state.reset_scroll();
            }
            crate::state::FocusPanel::DiffView => {}
        },

        UiEvent::ToggleSemanticHighlight => {
            ui_state.toggle_semantic_highlight();
            ui_state.deactivate_leader();
        }

        UiEvent::ToggleRangeSelection => match ui_state.focused_panel {
            crate::state::FocusPanel::DiffView => {
                ui_state.toggle_range_selection(total_lines);
            }
            crate::state::FocusPanel::FileList => {}
        },

        UiEvent::ToggleDecisionExpansion => {
            ui_state.decision_tree.toggle_expansion();
            ui_state.reset_scroll();
        }

        UiEvent::ToggleInstructions => {
            ui_state.toggle_instructions();
            ui_state.deactivate_leader();
        }

        UiEvent::ActivateLeader => {
            ui_state.activate_leader();
        }

        UiEvent::EnterLeaderSubmenu(submenu) => {
            ui_state.enter_leader_submenu(*submenu);
        }

        UiEvent::DeactivateLeader => {
            ui_state.deactivate_leader();
        }

        UiEvent::LeaderTimeout => {
            ui_state.deactivate_leader();
        }

        UiEvent::ToggleHelp => {
            ui_state.show_help = !ui_state.show_help;
        }

        UiEvent::ExportAll => {
            ui_state.deactivate_leader();
        }

        UiEvent::Refresh
        | UiEvent::NavigateToFile(_)
        | UiEvent::DeleteForward
        | UiEvent::MoveCursorWordLeft
        | UiEvent::MoveCursorWordRight => {}
    }

    Ok(())
}

fn handle_business_event_impl(engine: &mut ReviewEngine, event: BusinessEvent) -> Result<Command> {
    let author = engine.author().to_string();

    match event {
        BusinessEvent::ToggleApprove { reviewable_id } => {
            if engine.state().is_approved(&reviewable_id) {
                engine.reject(reviewable_id)?;
            } else {
                engine.approve(reviewable_id, author)?;
            }
            Ok(Command::None)
        }

        BusinessEvent::ApproveFile { file_path } => {
            engine.approve_all_in_file(&file_path, author)?;
            Ok(Command::None)
        }

        BusinessEvent::AddInstruction {
            reviewable_id,
            content,
        } => {
            if !content.trim().is_empty() {
                engine.add_instruction(reviewable_id, content, author)?;
            }
            Ok(Command::None)
        }

        BusinessEvent::AddDecisionInstruction {
            decision_number,
            content,
        } => {
            if !content.trim().is_empty() {
                engine.add_decision_instruction(decision_number, content, author)?;
            }
            Ok(Command::None)
        }

        BusinessEvent::ExportInstructions => {
            let json = engine.export_instructions_json()?;
            let filename = "instructions-all.json".to_string();
            Ok(Command::Batch(vec![
                Command::WriteFile {
                    path: filename.clone(),
                    content: json,
                },
                Command::ShowMessage {
                    message: format!("Exported instructions to {filename}"),
                },
            ]))
        }

        BusinessEvent::ToggleApproveDecision { decision_number } => {
            if engine.is_decision_approved(decision_number) {
                engine.reject_decision(decision_number)?;
            } else {
                engine.approve_decision(decision_number, author)?;
            }
            Ok(Command::None)
        }

        BusinessEvent::SaveSession | BusinessEvent::LoadSession { .. } => Ok(Command::None),
    }
}

fn get_total_lines_impl(engine: &ReviewEngine, ui_state: &UiState) -> usize {
    if let Some(reviewable_id) = ui_state.current_reviewable_id()
        && let Some(diff) = engine.get_renderable_diff_object(&reviewable_id)
    {
        return diff.lines.len();
    }
    0
}
