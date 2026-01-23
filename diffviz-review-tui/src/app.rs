//! Main TUI application state and business logic integration
//!
//! This module contains the core ReviewTuiApp that coordinates between
//! the ReviewEngine (business logic) and the UI state (presentation).

use crossterm::{
    event::{self, Event, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;

use diffviz_review::engines::ReviewEngine;

use crate::{
    command::{execute_command, Command},
    events::{handle_key_event, ui_event_to_business_event, BusinessEvent, UiEvent},
    state::UiState,
    ui, Result,
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

    /// Run the main application loop
    pub fn run(&mut self) -> Result<()> {
        loop {
            // Render UI
            self.render()?;

            // Handle events and get command
            let command = self.handle_events()?;

            // Execute command (side effects)
            execute_command(command)?;

            // Check if should quit
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

    /// Handle input events and return command to execute
    fn handle_events(&mut self) -> Result<Command> {
        // Check leader timeout - modeled as LeaderTimeout event
        if self.ui_state.leader_active && self.ui_state.is_leader_timed_out() {
            self.ui_state.deactivate_leader();
            return Ok(Command::None);
        }

        // Poll for input events with timeout
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                return self.process_key_event(key);
            }
        }

        Ok(Command::None)
    }

    /// Process a single key event and update state, returning command to execute
    pub fn process_key_event(&mut self, key: KeyEvent) -> Result<Command> {
        // Convert keyboard input to UI event, passing leader state
        if let Some(ui_event) = handle_key_event(
            key,
            &self.ui_state.input_mode,
            self.ui_state.leader_active,
            self.ui_state.leader_submenu,
        ) {
            // Handle UI-only events first
            self.handle_ui_event(&ui_event)?;

            // Convert to business events and handle through ReviewEngine
            if let Some(business_event) = ui_event_to_business_event(&ui_event, &self.ui_state) {
                let command = self.handle_business_event(business_event)?;

                // Exit input mode after successful business operation
                if matches!(ui_event, UiEvent::SubmitInput) {
                    self.ui_state.exit_input_mode();
                }

                return Ok(command);
            }
        }

        Ok(Command::None)
    }

    /// Handle UI events that affect navigation and display
    fn handle_ui_event(&mut self, event: &UiEvent) -> Result<()> {
        // Get total lines once for efficiency (used by multiple navigation handlers)
        let total_lines = self.get_total_lines_in_current_diff();

        match event {
            UiEvent::Quit => {
                self.ui_state.quit();
            }

            UiEvent::NavigateUp => {
                match self.ui_state.focused_panel {
                    crate::state::FocusPanel::FileList => {
                        // Navigate up in the decision tree
                        self.ui_state.decision_tree.navigate_prev();
                        self.ui_state.reset_scroll();
                    }
                    crate::state::FocusPanel::DiffView => {
                        self.ui_state.cursor_up();
                    }
                }
            }

            UiEvent::NavigateDown => {
                match self.ui_state.focused_panel {
                    crate::state::FocusPanel::FileList => {
                        // Navigate down in the decision tree
                        self.ui_state.decision_tree.navigate_next();
                        self.ui_state.reset_scroll();
                    }
                    crate::state::FocusPanel::DiffView => {
                        self.ui_state.cursor_down(total_lines);
                    }
                }
            }

            UiEvent::NavigateLeft => {
                // Switch focus to left pane (decisions tree)
                self.ui_state.focused_panel = crate::state::FocusPanel::FileList;
            }

            UiEvent::NavigateRight => {
                // Switch focus to right pane (code/diff view)
                self.ui_state.focused_panel = crate::state::FocusPanel::DiffView;
            }

            UiEvent::NavigateToTop => match self.ui_state.focused_panel {
                crate::state::FocusPanel::FileList => {
                    // Navigate to first item in tree
                    self.ui_state.navigate_to_first_in_tree();
                }
                crate::state::FocusPanel::DiffView => {
                    self.ui_state.cursor_to_top();
                }
            },

            UiEvent::NavigateToBottom => match self.ui_state.focused_panel {
                crate::state::FocusPanel::FileList => {
                    // Navigate to last item in tree
                    self.ui_state.navigate_to_last_in_tree();
                }
                crate::state::FocusPanel::DiffView => {
                    self.ui_state.cursor_to_bottom(total_lines);
                }
            },

            UiEvent::NavigatePageUp => match self.ui_state.focused_panel {
                crate::state::FocusPanel::FileList => {
                    self.ui_state.page_up();
                }
                crate::state::FocusPanel::DiffView => {
                    self.ui_state.cursor_page_up(total_lines);
                }
            },

            UiEvent::NavigatePageDown => match self.ui_state.focused_panel {
                crate::state::FocusPanel::FileList => {
                    self.ui_state.page_down();
                }
                crate::state::FocusPanel::DiffView => {
                    self.ui_state.cursor_page_down(total_lines);
                }
            },

            UiEvent::ToggleFocus => {
                self.ui_state.toggle_focus();
            }

            UiEvent::ToggleContextDisplay => {
                self.ui_state.toggle_context_display();
                // Deactivate leader after executing complete action
                self.ui_state.deactivate_leader();
            }

            UiEvent::ScrollUp => {
                self.ui_state.scroll_up(1);
            }

            UiEvent::ScrollDown => {
                self.ui_state.scroll_down(1);
            }

            UiEvent::ScrollPageUp => {
                self.ui_state.page_up();
            }

            UiEvent::ScrollPageDown => {
                self.ui_state.page_down();
            }

            UiEvent::ScrollInactivePanelUp => match self.ui_state.focused_panel {
                crate::state::FocusPanel::FileList => {
                    self.ui_state.scroll_up(1);
                }
                crate::state::FocusPanel::DiffView => {
                    self.navigate_to_previous_reviewable_diff();
                }
            },

            UiEvent::ScrollInactivePanelDown => match self.ui_state.focused_panel {
                crate::state::FocusPanel::FileList => {
                    self.ui_state.scroll_down(1);
                }
                crate::state::FocusPanel::DiffView => {
                    self.navigate_to_next_reviewable_diff();
                }
            },

            UiEvent::EnterInstructionMode => {
                if let Some(reviewable_id) = self.ui_state.current_reviewable_id() {
                    self.ui_state.start_instruction_input(reviewable_id);
                }
                // Deactivate leader after executing complete action
                self.ui_state.deactivate_leader();
            }

            UiEvent::EnterEditMode => {
                if let Some(reviewable_id) = self.ui_state.current_reviewable_id() {
                    self.ui_state.start_edit_mode(reviewable_id);
                }
                // Deactivate leader after executing complete action
                self.ui_state.deactivate_leader();
            }

            UiEvent::ExitInputMode | UiEvent::CancelInput => {
                self.ui_state.exit_input_mode();
            }

            UiEvent::InputChar(c) => {
                self.ui_state.input_char(*c);
            }

            UiEvent::DeleteChar => {
                self.ui_state.delete_char();
            }

            UiEvent::MoveCursorLeft => {
                self.ui_state.move_cursor_left();
            }

            UiEvent::MoveCursorRight => {
                self.ui_state.move_cursor_right();
            }

            UiEvent::MoveCursorHome => {
                self.ui_state.move_cursor_home();
            }

            UiEvent::MoveCursorEnd => {
                self.ui_state.move_cursor_end();
            }

            // These events are handled as business events
            UiEvent::ToggleApprove | UiEvent::ApproveFile => {
                // Deactivate leader after executing complete action
                self.ui_state.deactivate_leader();
            }

            UiEvent::SubmitInput => {}

            UiEvent::SelectCurrent => {
                match self.ui_state.focused_panel {
                    crate::state::FocusPanel::FileList => {
                        // Expand/collapse the currently selected node
                        self.ui_state.decision_tree.expand_current();
                        self.ui_state.reset_scroll();
                    }
                    crate::state::FocusPanel::DiffView => {
                        // In the code view (right pane) - could toggle approval or other action
                    }
                }
            }

            UiEvent::ToggleSemanticHighlight => {
                self.ui_state.toggle_semantic_highlight();
                // Deactivate leader after executing complete action
                self.ui_state.deactivate_leader();
            }

            UiEvent::ToggleRangeSelection => match self.ui_state.focused_panel {
                crate::state::FocusPanel::DiffView => {
                    self.ui_state.toggle_range_selection(total_lines);
                }
                crate::state::FocusPanel::FileList => {}
            },

            UiEvent::ToggleDecisionExpansion => {
                // Toggle expansion of currently selected node
                self.ui_state.decision_tree.toggle_expansion();
                self.ui_state.reset_scroll();
            }

            UiEvent::ToggleInstructions => {
                self.ui_state.toggle_instructions();
                // Deactivate leader after executing complete action
                self.ui_state.deactivate_leader();
            }

            UiEvent::ActivateLeader => {
                self.ui_state.activate_leader();
            }

            UiEvent::EnterLeaderSubmenu(submenu) => {
                self.ui_state.enter_leader_submenu(*submenu);
            }

            UiEvent::DeactivateLeader => {
                self.ui_state.deactivate_leader();
            }

            UiEvent::LeaderTimeout => {
                self.ui_state.deactivate_leader();
            }

            UiEvent::ToggleHelp => {
                self.ui_state.show_help = !self.ui_state.show_help;
            }

            // Export events are handled as business events
            UiEvent::ExportFile | UiEvent::ExportSingleInstruction | UiEvent::ExportAll => {
                // These are converted to BusinessEvent::ExportInstructions
                // and handled in handle_business_event
                self.ui_state.deactivate_leader();
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

    /// Handle business events that require ReviewEngine operations, returning command to execute
    fn handle_business_event(&mut self, event: BusinessEvent) -> Result<Command> {
        let author = self.review_engine.author().to_string();

        match event {
            BusinessEvent::ToggleApprove { reviewable_id } => {
                if self.review_engine.state().is_approved(&reviewable_id) {
                    self.review_engine.reject(reviewable_id, None)?;
                } else {
                    self.review_engine.approve(reviewable_id, author, None)?;
                }
                Ok(Command::None)
            }

            BusinessEvent::ApproveFile { file_path } => {
                self.review_engine
                    .approve_all_in_file(&file_path, author, None)?;
                Ok(Command::None)
            }

            BusinessEvent::AddInstruction {
                reviewable_id,
                content,
            } => {
                if !content.trim().is_empty() {
                    self.review_engine
                        .add_instruction(reviewable_id, content, author, None)?;
                }
                Ok(Command::None)
            }

            BusinessEvent::ExportInstructions { ref scope } => {
                // Generate JSON export
                let json = self.review_engine.export_instructions_json(scope.clone())?;

                // Determine file path based on scope
                let filename = match scope {
                    diffviz_review::engines::review_engine::ExportScope::SingleFile(path) => {
                        format!("instructions-{}.json", path.replace('/', "-"))
                    }
                    diffviz_review::engines::review_engine::ExportScope::SingleInstruction(_) => {
                        "instructions-single.json".to_string()
                    }
                    diffviz_review::engines::review_engine::ExportScope::All => {
                        "instructions-all.json".to_string()
                    }
                };

                // Return commands for file write and message
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
                if self.review_engine.is_decision_approved(decision_number) {
                    self.review_engine.reject_decision(decision_number)?;
                } else {
                    self.review_engine
                        .approve_decision(decision_number, author)?;
                }
                Ok(Command::None)
            }

            // Not implemented yet
            BusinessEvent::EditContent { .. }
            | BusinessEvent::SaveSession
            | BusinessEvent::LoadSession { .. } => Ok(Command::None),
        }
    }

    // Navigation helper methods

    fn navigate_to_next_reviewable_diff(&mut self) {
        // Navigate to next chunk in tree
        self.ui_state.decision_tree.navigate_next();
        self.ui_state.reset_scroll();
    }

    fn navigate_to_previous_reviewable_diff(&mut self) {
        // Navigate to previous chunk in tree
        self.ui_state.decision_tree.navigate_prev();
        self.ui_state.reset_scroll();
    }

    /// Get total number of lines in the currently displayed diff
    fn get_total_lines_in_current_diff(&self) -> usize {
        if let Some(reviewable_id) = self.ui_state.current_reviewable_id() {
            if let Some(diff) = self
                .review_engine
                .get_renderable_diff_object(&reviewable_id)
            {
                return diff.lines.len();
            }
        }
        0
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
        // Convert keyboard input to UI event, passing leader state
        if let Some(ui_event) = handle_key_event(
            key,
            &self.ui_state.input_mode,
            self.ui_state.leader_active,
            self.ui_state.leader_submenu,
        ) {
            // Handle UI-only events first
            self.handle_ui_event(&ui_event)?;

            // Convert to business events and handle through ReviewEngine
            if let Some(business_event) = ui_event_to_business_event(&ui_event, &self.ui_state) {
                let command = self.handle_business_event(business_event)?;

                // Exit input mode after successful business operation
                if matches!(ui_event, UiEvent::SubmitInput) {
                    self.ui_state.exit_input_mode();
                }

                return Ok(command);
            }
        }

        Ok(Command::None)
    }

    /// Handle UI events that affect navigation and display
    fn handle_ui_event(&mut self, event: &UiEvent) -> Result<()> {
        // Get total lines once for efficiency (used by multiple navigation handlers)
        let total_lines = self.get_total_lines_in_current_diff();

        match event {
            UiEvent::Quit => {
                self.ui_state.quit();
            }

            UiEvent::NavigateUp => {
                match self.ui_state.focused_panel {
                    crate::state::FocusPanel::FileList => {
                        // Navigate up in the decision tree
                        self.ui_state.decision_tree.navigate_prev();
                        self.ui_state.reset_scroll();
                    }
                    crate::state::FocusPanel::DiffView => {
                        self.ui_state.cursor_up();
                    }
                }
            }

            UiEvent::NavigateDown => {
                match self.ui_state.focused_panel {
                    crate::state::FocusPanel::FileList => {
                        // Navigate down in the decision tree
                        self.ui_state.decision_tree.navigate_next();
                        self.ui_state.reset_scroll();
                    }
                    crate::state::FocusPanel::DiffView => {
                        self.ui_state.cursor_down(total_lines);
                    }
                }
            }

            UiEvent::NavigateLeft => {
                // Switch focus to left pane (decisions tree)
                self.ui_state.focused_panel = crate::state::FocusPanel::FileList;
            }

            UiEvent::NavigateRight => {
                // Switch focus to right pane (code/diff view)
                self.ui_state.focused_panel = crate::state::FocusPanel::DiffView;
            }

            UiEvent::NavigateToTop => match self.ui_state.focused_panel {
                crate::state::FocusPanel::FileList => {
                    // Navigate to first item in tree
                    self.ui_state.navigate_to_first_in_tree();
                }
                crate::state::FocusPanel::DiffView => {
                    self.ui_state.cursor_to_top();
                }
            },

            UiEvent::NavigateToBottom => match self.ui_state.focused_panel {
                crate::state::FocusPanel::FileList => {
                    // Navigate to last item in tree
                    self.ui_state.navigate_to_last_in_tree();
                }
                crate::state::FocusPanel::DiffView => {
                    self.ui_state.cursor_to_bottom(total_lines);
                }
            },

            UiEvent::NavigatePageUp => match self.ui_state.focused_panel {
                crate::state::FocusPanel::FileList => {
                    self.ui_state.page_up();
                }
                crate::state::FocusPanel::DiffView => {
                    self.ui_state.cursor_page_up(total_lines);
                }
            },

            UiEvent::NavigatePageDown => match self.ui_state.focused_panel {
                crate::state::FocusPanel::FileList => {
                    self.ui_state.page_down();
                }
                crate::state::FocusPanel::DiffView => {
                    self.ui_state.cursor_page_down(total_lines);
                }
            },

            UiEvent::ToggleFocus => {
                self.ui_state.toggle_focus();
            }

            UiEvent::ToggleContextDisplay => {
                self.ui_state.toggle_context_display();
                // Deactivate leader after executing complete action
                self.ui_state.deactivate_leader();
            }

            UiEvent::ScrollUp => {
                self.ui_state.scroll_up(1);
            }

            UiEvent::ScrollDown => {
                self.ui_state.scroll_down(1);
            }

            UiEvent::ScrollPageUp => {
                self.ui_state.page_up();
            }

            UiEvent::ScrollPageDown => {
                self.ui_state.page_down();
            }

            UiEvent::ScrollInactivePanelUp => match self.ui_state.focused_panel {
                crate::state::FocusPanel::FileList => {
                    self.ui_state.scroll_up(1);
                }
                crate::state::FocusPanel::DiffView => {
                    self.navigate_to_previous_reviewable_diff();
                }
            },

            UiEvent::ScrollInactivePanelDown => match self.ui_state.focused_panel {
                crate::state::FocusPanel::FileList => {
                    self.ui_state.scroll_down(1);
                }
                crate::state::FocusPanel::DiffView => {
                    self.navigate_to_next_reviewable_diff();
                }
            },

            UiEvent::EnterInstructionMode => {
                if let Some(reviewable_id) = self.ui_state.current_reviewable_id() {
                    self.ui_state.start_instruction_input(reviewable_id);
                }
                // Deactivate leader after executing complete action
                self.ui_state.deactivate_leader();
            }

            UiEvent::EnterEditMode => {
                if let Some(reviewable_id) = self.ui_state.current_reviewable_id() {
                    self.ui_state.start_edit_mode(reviewable_id);
                }
                // Deactivate leader after executing complete action
                self.ui_state.deactivate_leader();
            }

            UiEvent::ExitInputMode | UiEvent::CancelInput => {
                self.ui_state.exit_input_mode();
            }

            UiEvent::InputChar(c) => {
                self.ui_state.input_char(*c);
            }

            UiEvent::DeleteChar => {
                self.ui_state.delete_char();
            }

            UiEvent::MoveCursorLeft => {
                self.ui_state.move_cursor_left();
            }

            UiEvent::MoveCursorRight => {
                self.ui_state.move_cursor_right();
            }

            UiEvent::MoveCursorHome => {
                self.ui_state.move_cursor_home();
            }

            UiEvent::MoveCursorEnd => {
                self.ui_state.move_cursor_end();
            }

            // These events are handled as business events
            UiEvent::ToggleApprove | UiEvent::ApproveFile => {
                // Deactivate leader after executing complete action
                self.ui_state.deactivate_leader();
            }

            UiEvent::SubmitInput => {}

            UiEvent::SelectCurrent => {
                match self.ui_state.focused_panel {
                    crate::state::FocusPanel::FileList => {
                        // Expand/collapse the currently selected node
                        self.ui_state.decision_tree.expand_current();
                        self.ui_state.reset_scroll();
                    }
                    crate::state::FocusPanel::DiffView => {
                        // In the code view (right pane) - could toggle approval or other action
                    }
                }
            }

            UiEvent::ToggleSemanticHighlight => {
                self.ui_state.toggle_semantic_highlight();
                // Deactivate leader after executing complete action
                self.ui_state.deactivate_leader();
            }

            UiEvent::ToggleRangeSelection => match self.ui_state.focused_panel {
                crate::state::FocusPanel::DiffView => {
                    self.ui_state.toggle_range_selection(total_lines);
                }
                crate::state::FocusPanel::FileList => {}
            },

            UiEvent::ToggleDecisionExpansion => {
                // Toggle expansion of currently selected node
                self.ui_state.decision_tree.toggle_expansion();
                self.ui_state.reset_scroll();
            }

            UiEvent::ToggleInstructions => {
                self.ui_state.toggle_instructions();
                // Deactivate leader after executing complete action
                self.ui_state.deactivate_leader();
            }

            UiEvent::ActivateLeader => {
                self.ui_state.activate_leader();
            }

            UiEvent::EnterLeaderSubmenu(submenu) => {
                self.ui_state.enter_leader_submenu(*submenu);
            }

            UiEvent::DeactivateLeader => {
                self.ui_state.deactivate_leader();
            }

            UiEvent::LeaderTimeout => {
                self.ui_state.deactivate_leader();
            }

            UiEvent::ToggleHelp => {
                self.ui_state.show_help = !self.ui_state.show_help;
            }

            // Export events are handled as business events
            UiEvent::ExportFile | UiEvent::ExportSingleInstruction | UiEvent::ExportAll => {
                // These are converted to BusinessEvent::ExportInstructions
                // and handled in handle_business_event
                self.ui_state.deactivate_leader();
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

    /// Handle business events that require ReviewEngine operations, returning command to execute
    fn handle_business_event(&mut self, event: BusinessEvent) -> Result<Command> {
        let author = self.review_engine.author().to_string();

        match event {
            BusinessEvent::ToggleApprove { reviewable_id } => {
                if self.review_engine.state().is_approved(&reviewable_id) {
                    self.review_engine.reject(reviewable_id, None)?;
                } else {
                    self.review_engine.approve(reviewable_id, author, None)?;
                }
                Ok(Command::None)
            }

            BusinessEvent::ApproveFile { file_path } => {
                self.review_engine
                    .approve_all_in_file(&file_path, author, None)?;
                Ok(Command::None)
            }

            BusinessEvent::AddInstruction {
                reviewable_id,
                content,
            } => {
                if !content.trim().is_empty() {
                    self.review_engine
                        .add_instruction(reviewable_id, content, author, None)?;
                }
                Ok(Command::None)
            }

            BusinessEvent::ExportInstructions { ref scope } => {
                // Generate JSON export
                let json = self.review_engine.export_instructions_json(scope.clone())?;

                // Determine file path based on scope
                let filename = match scope {
                    diffviz_review::engines::review_engine::ExportScope::SingleFile(path) => {
                        format!("instructions-{}.json", path.replace('/', "-"))
                    }
                    diffviz_review::engines::review_engine::ExportScope::SingleInstruction(_) => {
                        "instructions-single.json".to_string()
                    }
                    diffviz_review::engines::review_engine::ExportScope::All => {
                        "instructions-all.json".to_string()
                    }
                };

                // Return commands for file write and message
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

            // Not implemented yet
            BusinessEvent::EditContent { .. }
            | BusinessEvent::SaveSession
            | BusinessEvent::LoadSession { .. } => Ok(Command::None),
        }
    }

    fn navigate_to_next_reviewable_diff(&mut self) {
        // Navigate to next chunk in tree
        self.ui_state.decision_tree.navigate_next();
        self.ui_state.reset_scroll();
    }

    fn navigate_to_previous_reviewable_diff(&mut self) {
        // Navigate to previous chunk in tree
        self.ui_state.decision_tree.navigate_prev();
        self.ui_state.reset_scroll();
    }

    /// Get total number of lines in the currently displayed diff
    fn get_total_lines_in_current_diff(&self) -> usize {
        if let Some(reviewable_id) = self.ui_state.current_reviewable_id() {
            if let Some(diff) = self
                .review_engine
                .get_renderable_diff_object(&reviewable_id)
            {
                return diff.lines.len();
            }
        }
        0
    }
}
