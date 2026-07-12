//! Main TUI application state and business logic integration.
//!
//! This module coordinates between `TriageData` (the loaded symptoms and
//! their precomputed renderings), the triage `Baseline` (persistence), and
//! `UiState` (presentation).

use crossterm::event::KeyEvent;
use ratatui::Frame;

use crate::entities::{Symptom, SymptomId, TriageVerdict};
use crate::persistence::Baseline;
use diffviz_core::ReviewableDiff;
use tui_harness::{Affordance, AppDescription, ELMApp, ModeDoc};

use crate::tui::rendering::{self, RenderingError};
use crate::tui::{
    Result,
    command::{Command, execute_command},
    error::TriageTuiError,
    events::{BusinessEvent, UiEvent, bindings, handle_key_event, ui_event_to_business_event},
    state::UiState,
    state_snapshot::StateSnapshot,
    ui,
};

/// The loaded, untriaged symptoms plus a precomputed `ReviewableDiff` per
/// site (index-aligned: `core_diffs[i][j]` renders `symptoms[i].sites[j]`).
/// Immutable for the life of a triage session — verdicts are recorded to
/// the `Baseline`, not reflected back into this list (the acceptance
/// criterion is "the *next* `detect` run excludes it", not live removal).
pub struct TriageData {
    pub symptoms: Vec<Symptom>,
    pub core_diffs: Vec<Vec<ReviewableDiff>>,
}

impl TriageData {
    pub fn from_symptoms(
        root: &std::path::Path,
        symptoms: Vec<Symptom>,
    ) -> Result<Self, RenderingError> {
        let core_diffs = rendering::render_all(root, symptoms.iter())?;
        Ok(TriageData {
            symptoms,
            core_diffs,
        })
    }
}

/// Main TUI application that coordinates `TriageData`, the `Baseline`, and
/// `UiState`.
pub struct TriageApp {
    data: TriageData,
    baseline: Baseline,
    ui_state: UiState,
}

impl TriageApp {
    pub fn new(data: TriageData, baseline: Baseline) -> Self {
        let ui_state = UiState::new(data.symptoms.len());
        Self {
            data,
            baseline,
            ui_state,
        }
    }

    /// Run the main application loop using the tui-harness runtime.
    pub fn run(&mut self) -> Result<()> {
        tui_harness::run_app(self)?;
        Ok(())
    }

    pub fn process_key_event(&mut self, key: KeyEvent) -> Result<Command> {
        process_key_event_impl(&self.data, &mut self.baseline, &mut self.ui_state, key)
    }
}

impl ELMApp for TriageApp {
    type Snapshot = StateSnapshot;
    type Error = TriageTuiError;

    fn dispatch_key(&mut self, key: KeyEvent) -> std::result::Result<(), TriageTuiError> {
        let command = self
            .process_key_event(key)
            .map_err(|e| TriageTuiError::KeyDispatch(e.into()))?;
        execute_command(command).map_err(|e| TriageTuiError::CommandExecution(e.into()))?;
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        ui::draw(frame, &self.ui_state, &self.data);
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
                    description: "Browse untriaged symptoms and drill into their sites; \
                                  Space opens the leader-key menu"
                        .to_string(),
                },
                ModeDoc {
                    name: "FixInstruction".to_string(),
                    description: "Text input for how to fix the focused symptom".to_string(),
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

fn process_key_event_impl(
    data: &TriageData,
    baseline: &mut Baseline,
    ui_state: &mut UiState,
    key: KeyEvent,
) -> Result<Command> {
    ui_state.clear_status_message();

    let Some(ui_event) = handle_key_event(
        key,
        &ui_state.input_mode,
        ui_state.leader.is_active(),
        ui_state.leader.submenu(),
    ) else {
        return Ok(Command::None);
    };

    handle_ui_event_impl(data, ui_state, &ui_event);

    let focused_symptom_id = ui_state
        .current_symptom_index()
        .and_then(|idx| data.symptoms.get(idx))
        .map(|s| &s.id);
    let fix_instruction_symptom_id = match &ui_state.input_mode {
        crate::tui::state::InputMode::FixInstruction { symptom_id } => Some(symptom_id),
        crate::tui::state::InputMode::Navigation => None,
    };

    let business_event = ui_event_to_business_event(
        &ui_event,
        focused_symptom_id,
        fix_instruction_symptom_id,
        &ui_state.input_buffer,
    );

    if let Some(event) = business_event {
        if matches!(ui_event, UiEvent::SubmitInput) {
            ui_state.exit_input_mode();
        }
        return match handle_business_event_impl(baseline, event) {
            Ok((VerdictOutcome::Recorded, command)) => {
                ui_state.back_to_browse_after_verdict();
                Ok(command)
            }
            Ok((VerdictOutcome::Skipped, command)) => Ok(command),
            Err(e) => {
                ui_state.set_status_message(format!("{e}"));
                Ok(Command::None)
            }
        };
    }

    Ok(Command::None)
}

fn handle_ui_event_impl(data: &TriageData, ui_state: &mut UiState, event: &UiEvent) {
    match event {
        UiEvent::Quit => ui_state.quit(),

        UiEvent::NavigateUp => ui_state.navigate_up(),
        UiEvent::NavigateDown => {
            let n_sites = ui_state
                .drill_position()
                .map(|(symptom_idx, _)| data.symptoms[symptom_idx].sites.len())
                .unwrap_or(0);
            ui_state.navigate_down(n_sites);
        }
        UiEvent::NavigatePageUp => ui_state.drill_page_up(),
        UiEvent::NavigatePageDown => ui_state.drill_page_down(),
        UiEvent::SelectCurrent => ui_state.drill_in(),
        UiEvent::Back => ui_state.back(),

        UiEvent::EnterFixInstructionMode => {
            if let Some(idx) = ui_state.current_symptom_index()
                && let Some(symptom) = data.symptoms.get(idx)
            {
                ui_state.start_fix_instruction_input(symptom.id.clone());
            }
            ui_state.deactivate_leader();
        }
        UiEvent::ExitInputMode | UiEvent::CancelInput => ui_state.exit_input_mode(),
        UiEvent::InputChar(c) => ui_state.input_char(*c),
        UiEvent::DeleteChar => ui_state.delete_char(),
        UiEvent::MoveCursorLeft => ui_state.move_cursor_left(),
        UiEvent::MoveCursorRight => ui_state.move_cursor_right(),
        UiEvent::MoveCursorHome => ui_state.move_cursor_home(),
        UiEvent::MoveCursorEnd => ui_state.move_cursor_end(),

        UiEvent::DismissFalsePositive
        | UiEvent::DismissIntentional
        | UiEvent::DismissAcceptedDebt => {
            ui_state.deactivate_leader();
        }

        UiEvent::SubmitInput => {}

        UiEvent::ActivateLeader => ui_state.activate_leader(),
        UiEvent::EnterLeaderSubmenu(submenu) => ui_state.enter_leader_submenu(*submenu),
        UiEvent::DeactivateLeader => ui_state.deactivate_leader(),
        UiEvent::LeaderTimeout => ui_state.deactivate_leader(),

        UiEvent::ToggleHelp => ui_state.show_help = !ui_state.show_help,
    }
}

/// Whether a business event actually changed the baseline (vs. a no-op,
/// e.g. an empty fix instruction) — callers use this to decide whether the
/// UI should navigate back to Browse.
enum VerdictOutcome {
    Recorded,
    Skipped,
}

fn handle_business_event_impl(
    baseline: &mut Baseline,
    event: BusinessEvent,
) -> Result<(VerdictOutcome, Command)> {
    match event {
        BusinessEvent::RecordVerdict {
            symptom_id,
            verdict,
        } => {
            if let TriageVerdict::Fix { instruction } = &verdict
                && instruction.trim().is_empty()
            {
                return Ok((VerdictOutcome::Skipped, Command::None));
            }
            record_verdict(baseline, &symptom_id, verdict)?;
            Ok((VerdictOutcome::Recorded, Command::None))
        }
    }
}

/// Persist a verdict directly on the baseline — the same "accepted
/// pragmatic compromise" `diffviz-review-tui` documents for
/// `self.engine.approve()`: a synchronous domain operation on the app's own
/// persisted state, not I/O the update logic needs to defer through a
/// `Command`.
fn record_verdict(
    baseline: &mut Baseline,
    symptom_id: &SymptomId,
    verdict: TriageVerdict,
) -> Result<()> {
    baseline.record_verdict(symptom_id, verdict)?;
    Ok(())
}
