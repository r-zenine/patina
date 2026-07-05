//! Main TUI application state and business logic integration
//!
//! This module contains the core ReviewTuiApp that coordinates between
//! the ReviewEngine (business logic) and the UI state (presentation).

use crossterm::event::KeyEvent;
use ratatui::Frame;

use diffviz_review::engines::ReviewEngine;
use tui_harness::{Affordance, AppDescription, ELMApp, ModeDoc};

use crate::{
    Result,
    command::{Command, execute_command},
    error::ReviewTuiError,
    events::{BusinessEvent, UiEvent, bindings, handle_key_event, ui_event_to_business_event},
    state::{DrillDecision, DrillFile, DrillIndex, UiState},
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
        ui_state.set_drill_index(build_drill_index(review_engine));
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
        if self.ui_state.leader.is_active() && self.ui_state.is_leader_timed_out() {
            self.ui_state.deactivate_leader();
        }
    }

    fn describe(&self) -> Option<AppDescription> {
        Some(AppDescription {
            app: env!("CARGO_PKG_NAME").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            modes: vec![
                ModeDoc {
                    name: "Navigation".to_string(),
                    description: "Browse decisions and drill into files/chunks; \
                                  Space opens the leader-key menu"
                        .to_string(),
                },
                ModeDoc {
                    name: "Instruction".to_string(),
                    description: "Text input for a note on the focused chunk".to_string(),
                },
                ModeDoc {
                    name: "DecisionInstruction".to_string(),
                    description: "Text input for a note on the focused decision".to_string(),
                },
            ],
            bindings: bindings::REGISTRY.binding_docs(),
        })
    }

    fn affordances(&self) -> Vec<Affordance> {
        let scope = bindings::scope_of(
            &self.ui_state.input_mode,
            self.ui_state.leader.is_active(),
            self.ui_state.leader.submenu(),
        );

        bindings::REGISTRY.affordances(scope)
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
    // The status message is one-shot (D7): any keypress clears it before the
    // key is handled, so an error set below survives exactly one snapshot.
    ui_state.clear_status_message();

    if let Some(ui_event) = handle_key_event(
        key,
        &ui_state.input_mode,
        ui_state.leader.is_active(),
        ui_state.leader.submenu(),
    ) {
        handle_ui_event_impl(engine, ui_state, &ui_event)?;

        match ui_event_to_business_event(&ui_event, ui_state) {
            Some(business_event) => {
                let result = handle_business_event_impl(engine, business_event);

                if matches!(ui_event, UiEvent::SubmitInput) {
                    ui_state.exit_input_mode();
                }

                return match result {
                    Ok(command) => Ok(command),
                    // Business failures surface in the status bar instead of
                    // aborting the app (D7); fatal errors keep propagating
                    // from the layers above.
                    Err(e) => {
                        ui_state.set_status_message(format!("{e}"));
                        Ok(Command::None)
                    }
                };
            }
            None => {
                // An approve keypress that resolves to no target (empty
                // review) is an error, not a silent no-op (D7).
                if matches!(ui_event, UiEvent::ToggleApprove) {
                    ui_state.set_status_message("Nothing to approve".to_string());
                }
            }
        }
    }

    Ok(Command::None)
}

fn handle_ui_event_impl(
    engine: &ReviewEngine,
    ui_state: &mut UiState,
    event: &UiEvent,
) -> Result<()> {
    match event {
        UiEvent::Quit => {
            ui_state.quit();
        }

        // ── DrillNav navigation ─────────────────────────────────────────
        UiEvent::NavigateUp => {
            ui_state.navigate_up();
        }

        UiEvent::NavigateDown => {
            ui_state.navigate_down();
        }

        UiEvent::NavigateLeft => {
            ui_state.navigate_left();
        }

        UiEvent::NavigateRight => {
            ui_state.navigate_right();
        }

        UiEvent::NavigateToTop => {
            ui_state.to_top();
        }

        UiEvent::NavigateToBottom => {
            ui_state.to_bottom();
        }

        UiEvent::NavigatePageUp => {
            ui_state.drill_page_up();
        }

        UiEvent::NavigatePageDown => {
            ui_state.drill_page_down();
        }

        UiEvent::SelectCurrent => {
            ui_state.drill_in();
        }

        UiEvent::Back => {
            ui_state.back();
        }

        UiEvent::ToggleChunkContext => {
            ui_state.toggle_context();
        }

        UiEvent::ToggleNoteExpansion => {
            ui_state.toggle_note_expansion();
        }

        UiEvent::EnterInstructionMode => {
            if let Some(reviewable_id) = ui_state.current_reviewable_id() {
                let existing_content = engine
                    .state()
                    .get_instructions(&reviewable_id)
                    .and_then(|v| v.first())
                    .map(|instr| instr.content.clone());
                ui_state.start_instruction_input(reviewable_id, existing_content);
            } else if let Some(decision_number) = ui_state.current_decision_number() {
                let existing_content = engine
                    .get_decision_instructions(decision_number)
                    .and_then(|v| v.into_iter().next())
                    .map(|instr| instr.content.clone());
                ui_state.start_decision_instruction_input(decision_number, existing_content);
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

        UiEvent::ToggleReasoning => {
            ui_state.toggle_reasoning();
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

        // Reachable in input mode but not yet implemented (no keybinding
        // conflict — Delete/Ctrl-Left/Ctrl-Right just don't edit yet).
        UiEvent::DeleteForward | UiEvent::MoveCursorWordLeft | UiEvent::MoveCursorWordRight => {}
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
                engine.edit_instruction(reviewable_id, content, author)?;
            }
            Ok(Command::None)
        }

        BusinessEvent::AddDecisionInstruction {
            decision_number,
            content,
        } => {
            if !content.trim().is_empty() {
                engine.edit_decision_instruction(decision_number, content, author)?;
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

/// Build the precomputed decision→files→chunks index (D6): decisions in
/// engine order, files sorted lexicographically, chunks by ascending start
/// line — the DrillNav sibling and cursor order.
fn build_drill_index(engine: &ReviewEngine) -> DrillIndex {
    let pairs = engine.get_decision_reviewable_diffs();
    let decisions = engine
        .get_all_decisions()
        .into_iter()
        .map(|decision| {
            let mut paths: Vec<String> = pairs
                .iter()
                .filter(|pair| pair.decision_number == decision.number)
                .map(|pair| pair.chunk_id.file_path().to_string())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();
            paths.sort();

            let files = paths
                .into_iter()
                .map(|path| {
                    let mut chunks: Vec<_> = pairs
                        .iter()
                        .filter(|pair| {
                            pair.decision_number == decision.number
                                && pair.chunk_id.file_path() == path
                        })
                        .map(|pair| pair.chunk_id.clone())
                        .collect();
                    chunks.sort_by_key(|id| id.line_range().start_line);
                    DrillFile { path, chunks }
                })
                .collect();

            DrillDecision {
                number: decision.number,
                files,
            }
        })
        .collect();

    DrillIndex { decisions }
}
