//! Main TUI application state and business logic integration
//!
//! This module contains the core ReviewTuiApp that coordinates between
//! the ReviewEngine (business logic) and the UI state (presentation).

use crossterm::{
    event::{self, Event, KeyEvent},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use std::time::{Duration, Instant};

use diffviz_review::engines::ReviewEngine;

use crate::{
    Result,
    command::{Command, execute_command},
    events::{BusinessEvent, UiEvent, handle_key_event, ui_event_to_business_event},
    state::UiState,
    ui,
};

/// Main TUI application that coordinates ReviewEngine and UI
pub struct ReviewTuiApp {
    /// Business logic engine
    review_engine: ReviewEngine,

    /// UI navigation and display state
    ui_state: UiState,

    /// Terminal for rendering
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl ReviewTuiApp {
    /// Create a new TUI application
    pub fn new(review_engine: ReviewEngine) -> Result<Self> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        let ui_state = Self::initialize_ui_state(&review_engine);

        Ok(Self {
            review_engine,
            ui_state,
            terminal,
        })
    }

    /// Initialize UI state from review engine
    fn initialize_ui_state(review_engine: &ReviewEngine) -> UiState {
        let mut ui_state = UiState::new();

        // Build decision tree from review engine
        ui_state.decision_tree =
            crate::decision_navigation::DecisionNavigationTree::build_from_review_engine(
                review_engine,
            );

        // Reset scroll for initial state
        if ui_state.decision_tree.selected_chunk_id().is_some() {
            ui_state.reset_scroll();
        }

        ui_state
    }

    /// Consume the app and return the underlying ReviewEngine.
    ///
    /// Performs terminal cleanup before returning (since Drop will not run via ManuallyDrop).
    pub fn into_review_engine(self) -> ReviewEngine {
        let mut manual = std::mem::ManuallyDrop::new(self);
        // Manually run terminal teardown (Drop is suppressed by ManuallyDrop)
        let _ = disable_raw_mode();
        let _ = execute!(manual.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = manual.terminal.show_cursor();
        // SAFETY: ManuallyDrop prevents automatic Drop. We take sole ownership of
        // review_engine via ptr::read. The remaining fields (ui_state, terminal) are
        // leaked, which is acceptable — their cleanup has been done manually above and
        // the process exits shortly after this call.
        unsafe { std::ptr::read(std::ptr::addr_of!(manual.review_engine)) }
    }

    /// Run the main application loop at a fixed ~60fps cadence.
    ///
    /// Each iteration drains all pending events within a 16ms frame window,
    /// then renders once. This prevents a full redraw between every key-repeat
    /// event when the user holds a navigation key.
    pub fn run(&mut self) -> Result<()> {
        const FRAME_DURATION: Duration = Duration::from_millis(16); // ~60fps

        // Start one frame in the past so the first iteration renders immediately.
        let mut last_render = Instant::now() - FRAME_DURATION;

        loop {
            // Drain all input events for this frame window
            let frame_deadline = last_render + FRAME_DURATION;
            self.drain_events_for_frame(frame_deadline)?;

            // Leader timeout: checked once per frame (fires within 33ms of expiry)
            if self.ui_state.leader_active && self.ui_state.is_leader_timed_out() {
                self.ui_state.deactivate_leader();
            }

            // Single render covering all events collected this frame
            self.render()?;
            last_render = Instant::now();

            if self.ui_state.should_quit {
                break;
            }
        }

        Ok(())
    }

    /// Render the current UI state
    fn render(&mut self) -> Result<()> {
        self.terminal.draw(|f| {
            ui::draw(f, &self.ui_state, &self.review_engine);
        })?;
        Ok(())
    }

    /// Drain all pending key events until the frame deadline is reached.
    ///
    /// Processes each event and executes its command immediately so that
    /// ordering is preserved. Returns once the deadline has passed or the
    /// event queue is empty, whichever comes first.
    fn drain_events_for_frame(&mut self, deadline: Instant) -> Result<()> {
        loop {
            if self.ui_state.should_quit {
                break;
            }
            let now = Instant::now();
            if now >= deadline {
                break;
            }
            let remaining = deadline - now;
            if event::poll(remaining)? {
                if let Event::Key(key) = event::read()? {
                    let command = self.process_key_event(key)?;
                    execute_command(command)?;
                }
                // Non-key events (resize, mouse) are read and discarded above
            } else {
                // Queue empty within remaining budget
                break;
            }
        }
        Ok(())
    }

    /// Process a single key event and update state, returning command to execute
    pub fn process_key_event(&mut self, key: KeyEvent) -> Result<Command> {
        process_key_event_impl(&mut self.review_engine, &mut self.ui_state, key)
    }
}

impl Drop for ReviewTuiApp {
    fn drop(&mut self) {
        // Restore terminal - always try to clean up even if errors occur
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

/// Headless application for testing without terminal
#[cfg(feature = "test-harness")]
pub struct HeadlessApp {
    /// Business logic engine
    pub review_engine: ReviewEngine,

    /// UI navigation and display state
    pub ui_state: UiState,
}

#[cfg(feature = "test-harness")]
impl HeadlessApp {
    /// Create a new headless application
    pub fn new(review_engine: ReviewEngine) -> Self {
        let ui_state = ReviewTuiApp::initialize_ui_state(&review_engine);

        Self {
            review_engine,
            ui_state,
        }
    }

    /// Process a single key event and update state, returning command to execute
    pub fn process_key_event(&mut self, key: KeyEvent) -> Result<Command> {
        process_key_event_impl(&mut self.review_engine, &mut self.ui_state, key)
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
    // Convert keyboard input to UI event, passing leader state
    if let Some(ui_event) = handle_key_event(
        key,
        &ui_state.input_mode,
        ui_state.leader_active,
        ui_state.leader_submenu,
    ) {
        // Handle UI-only events first
        handle_ui_event_impl(engine, ui_state, &ui_event)?;

        // Convert to business events and handle through ReviewEngine
        if let Some(business_event) = ui_event_to_business_event(&ui_event, ui_state) {
            let command = handle_business_event_impl(engine, business_event)?;

            // Exit input mode after successful business operation
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
    // Get total lines once for efficiency (used by multiple navigation handlers)
    let total_lines = get_total_lines_impl(engine, ui_state);

    match event {
        UiEvent::Quit => {
            ui_state.quit();
        }

        UiEvent::NavigateUp => {
            match ui_state.focused_panel {
                crate::state::FocusPanel::FileList => {
                    // Navigate up in the decision tree
                    ui_state.decision_tree.navigate_prev();
                    ui_state.reset_scroll();
                }
                crate::state::FocusPanel::DiffView => {
                    ui_state.cursor_up();
                }
            }
        }

        UiEvent::NavigateDown => {
            match ui_state.focused_panel {
                crate::state::FocusPanel::FileList => {
                    // Navigate down in the decision tree
                    ui_state.decision_tree.navigate_next();
                    ui_state.reset_scroll();
                }
                crate::state::FocusPanel::DiffView => {
                    ui_state.cursor_down(total_lines);
                }
            }
        }

        UiEvent::NavigateLeft => {
            // Switch focus to left pane (decisions tree)
            ui_state.focused_panel = crate::state::FocusPanel::FileList;
        }

        UiEvent::NavigateRight => {
            // Switch focus to right pane (code/diff view)
            ui_state.focused_panel = crate::state::FocusPanel::DiffView;
        }

        UiEvent::NavigateToTop => match ui_state.focused_panel {
            crate::state::FocusPanel::FileList => {
                // Navigate to first item in tree
                ui_state.navigate_to_first_in_tree();
            }
            crate::state::FocusPanel::DiffView => {
                ui_state.cursor_to_top();
            }
        },

        UiEvent::NavigateToBottom => match ui_state.focused_panel {
            crate::state::FocusPanel::FileList => {
                // Navigate to last item in tree
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
            // Deactivate leader after executing complete action
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
            }
            // Deactivate leader after executing complete action
            ui_state.deactivate_leader();
        }

        UiEvent::EnterEditMode => {
            if let Some(reviewable_id) = ui_state.current_reviewable_id() {
                ui_state.start_edit_mode(reviewable_id);
            }
            // Deactivate leader after executing complete action
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

        // These events are handled as business events
        UiEvent::ToggleApprove | UiEvent::ApproveFile => {
            // Deactivate leader after executing complete action
            ui_state.deactivate_leader();
        }

        UiEvent::SubmitInput => {}

        UiEvent::SelectCurrent => {
            match ui_state.focused_panel {
                crate::state::FocusPanel::FileList => {
                    // Expand/collapse the currently selected node
                    ui_state.decision_tree.expand_current();
                    ui_state.reset_scroll();
                }
                crate::state::FocusPanel::DiffView => {
                    // In the code view (right pane) - could toggle approval or other action
                }
            }
        }

        UiEvent::ToggleSemanticHighlight => {
            ui_state.toggle_semantic_highlight();
            // Deactivate leader after executing complete action
            ui_state.deactivate_leader();
        }

        UiEvent::ToggleRangeSelection => match ui_state.focused_panel {
            crate::state::FocusPanel::DiffView => {
                ui_state.toggle_range_selection(total_lines);
            }
            crate::state::FocusPanel::FileList => {}
        },

        UiEvent::ToggleDecisionExpansion => {
            // Toggle expansion of currently selected node
            ui_state.decision_tree.toggle_expansion();
            ui_state.reset_scroll();
        }

        UiEvent::ToggleInstructions => {
            ui_state.toggle_instructions();
            // Deactivate leader after executing complete action
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

        // Export event is handled as a business event
        UiEvent::ExportAll => {
            ui_state.deactivate_leader();
        }

        // Not implemented yet
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
                engine.reject(reviewable_id, None)?;
            } else {
                engine.approve(reviewable_id, author, None)?;
            }
            Ok(Command::None)
        }

        BusinessEvent::ApproveFile { file_path } => {
            engine.approve_all_in_file(&file_path, author, None)?;
            Ok(Command::None)
        }

        BusinessEvent::AddInstruction {
            reviewable_id,
            content,
        } => {
            if !content.trim().is_empty() {
                engine.add_instruction(reviewable_id, content, author, None)?;
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

        // Not implemented yet
        BusinessEvent::EditContent { .. }
        | BusinessEvent::SaveSession
        | BusinessEvent::LoadSession { .. } => Ok(Command::None),
    }
}

fn get_total_lines_impl(engine: &ReviewEngine, ui_state: &UiState) -> usize {
    if let Some(reviewable_id) = ui_state.current_reviewable_id() {
        if let Some(diff) = engine.get_renderable_diff_object(&reviewable_id) {
            return diff.lines.len();
        }
    }
    0
}
