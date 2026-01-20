//! Main TUI application state and business logic integration
//!
//! This module contains the core ReviewTuiApp that coordinates between
//! the ReviewEngine (business logic) and the UI state (presentation).

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;

use diffviz_review::engines::ReviewEngine;

use crate::{
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

        let mut ui_state = UiState::new();

        // Initialize with first ReviewableDiff if available
        let reviewable_ids = review_engine.get_ordered_reviewable_ids();
        if let Some(first_id) = reviewable_ids.first() {
            ui_state.navigate_to((*first_id).clone());
            ui_state.current_file_path = Some(first_id.file_path.clone());
        }

        Ok(Self {
            review_engine,
            ui_state,
            terminal,
        })
    }

    /// Run the main application loop
    pub fn run(&mut self) -> Result<()> {
        loop {
            // Render UI
            self.render()?;

            // Handle events
            if self.handle_events()? {
                break; // Should quit
            }
        }

        Ok(())
    }

    /// Render the current UI state
    fn render(&mut self) -> Result<()> {
        self.terminal.draw(|f| {
            ui::draw(f, &mut self.ui_state, &self.review_engine);
        })?;
        Ok(())
    }

    /// Handle input events and return true if should quit
    fn handle_events(&mut self) -> Result<bool> {
        // Check leader timeout first
        if self.ui_state.leader_active && self.ui_state.is_leader_timed_out() {
            self.ui_state.deactivate_leader();
        }

        // Poll for input events with timeout
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
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
                    if let Some(business_event) =
                        ui_event_to_business_event(&ui_event, &self.ui_state)
                    {
                        self.handle_business_event(business_event)?;

                        // Exit input mode after successful business operation
                        if matches!(ui_event, UiEvent::SubmitInput) {
                            self.ui_state.exit_input_mode();
                        }
                    }
                }
            }
        }

        Ok(self.ui_state.should_quit)
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
                use crate::decision_navigation::NavigationLevel;
                // Handle decision navigation at decision level
                if self.ui_state.decision_nav.current_level == NavigationLevel::Decision {
                    self.ui_state.decision_nav.prev_decision();
                    let decision_count = self.review_engine.get_all_decisions().len();
                    self.ui_state.decision_nav.clamp_decision_index(decision_count.saturating_sub(1));
                } else {
                    // Handle traditional file/chunk navigation
                    match self.ui_state.focused_panel {
                        crate::state::FocusPanel::FileList => {
                            crate::navigation::navigate_file_list(
                                &mut self.ui_state,
                                &self.review_engine,
                                crate::navigation::NavigationDirection::Up,
                            );
                        }
                        crate::state::FocusPanel::DiffView => {
                            self.ui_state.cursor_up();
                        }
                    }
                }
            }

            UiEvent::NavigateDown => {
                use crate::decision_navigation::NavigationLevel;
                // Handle decision navigation at decision level
                if self.ui_state.decision_nav.current_level == NavigationLevel::Decision {
                    self.ui_state.decision_nav.next_decision();
                    let decision_count = self.review_engine.get_all_decisions().len();
                    self.ui_state.decision_nav.clamp_decision_index(decision_count.saturating_sub(1));
                } else {
                    // Handle traditional file/chunk navigation
                    match self.ui_state.focused_panel {
                        crate::state::FocusPanel::FileList => {
                            crate::navigation::navigate_file_list(
                                &mut self.ui_state,
                                &self.review_engine,
                                crate::navigation::NavigationDirection::Down,
                            );
                        }
                        crate::state::FocusPanel::DiffView => {
                            self.ui_state.cursor_down(total_lines);
                        }
                    }
                }
            }

            UiEvent::NavigateLeft => {
                use crate::decision_navigation::NavigationLevel;
                // Handle decision navigation - go back to decisions from file view
                if self.ui_state.decision_nav.current_level == NavigationLevel::File {
                    self.ui_state.decision_nav.back_to_decisions();
                } else {
                    // Handle traditional file/chunk navigation
                    match self.ui_state.focused_panel {
                        crate::state::FocusPanel::FileList => {
                            self.navigate_to_previous_file();
                        }
                        crate::state::FocusPanel::DiffView => {
                            self.ui_state.toggle_focus();
                        }
                    }
                }
            }

            UiEvent::NavigateRight => {
                use crate::decision_navigation::NavigationLevel;
                // Handle decision navigation - drill into files from decision level
                if self.ui_state.decision_nav.current_level == NavigationLevel::Decision {
                    self.ui_state.decision_nav.drill_into_files();
                } else {
                    // Handle traditional file/chunk navigation
                    match self.ui_state.focused_panel {
                        crate::state::FocusPanel::FileList => {
                            self.ui_state.toggle_focus();
                        }
                        crate::state::FocusPanel::DiffView => {
                            self.navigate_to_next_file();
                        }
                    }
                }
            }

            UiEvent::NavigateToTop => match self.ui_state.focused_panel {
                crate::state::FocusPanel::FileList => {
                    crate::navigation::navigate_file_list(
                        &mut self.ui_state,
                        &self.review_engine,
                        crate::navigation::NavigationDirection::First,
                    );
                }
                crate::state::FocusPanel::DiffView => {
                    self.ui_state.cursor_to_top();
                }
            },

            UiEvent::NavigateToBottom => match self.ui_state.focused_panel {
                crate::state::FocusPanel::FileList => {
                    crate::navigation::navigate_file_list(
                        &mut self.ui_state,
                        &self.review_engine,
                        crate::navigation::NavigationDirection::Last,
                    );
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
                if let Some(reviewable_id) = &self.ui_state.current_reviewable_id {
                    self.ui_state.start_instruction_input(reviewable_id.clone());
                }
                // Deactivate leader after executing complete action
                self.ui_state.deactivate_leader();
            }

            UiEvent::EnterEditMode => {
                if let Some(reviewable_id) = &self.ui_state.current_reviewable_id {
                    self.ui_state.start_edit_mode(reviewable_id.clone());
                }
                // Deactivate leader after executing complete action
                self.ui_state.deactivate_leader();
            }

            UiEvent::ExitInputMode | UiEvent::CancelInput => {
                // Close decision modal if open
                if self.ui_state.decision_nav.show_decision_modal {
                    self.ui_state.decision_nav.close_decision_modal();
                } else {
                    self.ui_state.exit_input_mode();
                }
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
                use crate::decision_navigation::NavigationLevel;
                // Handle selection based on navigation level
                if self.ui_state.decision_nav.current_level == NavigationLevel::Decision {
                    // At decision level - open decision modal
                    let all_decisions = self.review_engine.get_all_decisions();
                    if !all_decisions.is_empty() && self.ui_state.decision_nav.decision_list_index < all_decisions.len() {
                        let selected_decision = all_decisions[self.ui_state.decision_nav.decision_list_index];
                        self.ui_state.decision_nav.select_decision(selected_decision.number);
                    }
                } else {
                    // Handle traditional file/chunk selection
                    match self.ui_state.focused_panel {
                        crate::state::FocusPanel::FileList => {
                            crate::navigation::handle_selection_action(
                                &mut self.ui_state,
                                &self.review_engine,
                            );
                        }
                        crate::state::FocusPanel::DiffView => {
                            // In diff view, could toggle approval or other action
                        }
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

    /// Handle business events that require ReviewEngine operations
    fn handle_business_event(&mut self, event: BusinessEvent) -> Result<()> {
        let author = self.review_engine.author().to_string();

        match event {
            BusinessEvent::ToggleApprove { reviewable_id } => {
                if self.review_engine.state().is_approved(&reviewable_id) {
                    self.review_engine.reject(reviewable_id, None)?;
                } else {
                    self.review_engine.approve(reviewable_id, author, None)?;
                }
            }

            BusinessEvent::ApproveFile { file_path } => {
                self.review_engine
                    .approve_all_in_file(&file_path, author, None)?;
            }

            BusinessEvent::AddInstruction {
                reviewable_id,
                content,
            } => {
                if !content.trim().is_empty() {
                    self.review_engine
                        .add_instruction(reviewable_id, content, author, None)?;
                }
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

                // Write to file
                std::fs::write(&filename, json)
                    .map_err(diffviz_review::errors::DiffVizError::Io)?;

                // Show success message (will be visible in status bar)
                // TODO: Add proper success notification UI
                eprintln!("Exported instructions to {filename}");
            }

            // Not implemented yet
            BusinessEvent::EditContent { .. }
            | BusinessEvent::SaveSession
            | BusinessEvent::LoadSession { .. } => {}
        }

        Ok(())
    }

    // Navigation helper methods

    fn navigate_to_next_reviewable_diff(&mut self) {
        let reviewable_ids = self.review_engine.get_ordered_reviewable_ids();
        if let Some(current_id) = &self.ui_state.current_reviewable_id {
            if let Some(current_index) = reviewable_ids.iter().position(|id| *id == current_id) {
                if current_index + 1 < reviewable_ids.len() {
                    let next_id = reviewable_ids[current_index + 1];
                    self.ui_state.navigate_to(next_id.clone());
                    self.ui_state.current_file_path = Some(next_id.file_path.clone());
                }
            }
        }
    }

    fn navigate_to_previous_reviewable_diff(&mut self) {
        let reviewable_ids = self.review_engine.get_ordered_reviewable_ids();
        if let Some(current_id) = &self.ui_state.current_reviewable_id {
            if let Some(current_index) = reviewable_ids.iter().position(|id| *id == current_id) {
                if current_index > 0 {
                    let prev_id = reviewable_ids[current_index - 1];
                    self.ui_state.navigate_to(prev_id.clone());
                    self.ui_state.current_file_path = Some(prev_id.file_path.clone());
                }
            }
        }
    }

    fn navigate_to_previous_file(&mut self) {
        let file_paths = self.review_engine.get_file_paths();
        if let Some(current_file) = &self.ui_state.current_file_path {
            if let Some(current_index) = file_paths.iter().position(|path| path == current_file) {
                if current_index > 0 {
                    let prev_file = &file_paths[current_index - 1];
                    self.navigate_to_first_diff_in_file(prev_file);
                }
            }
        }
    }

    fn navigate_to_next_file(&mut self) {
        let file_paths = self.review_engine.get_file_paths();
        if let Some(current_file) = &self.ui_state.current_file_path {
            if let Some(current_index) = file_paths.iter().position(|path| path == current_file) {
                if current_index + 1 < file_paths.len() {
                    let next_file = &file_paths[current_index + 1];
                    self.navigate_to_first_diff_in_file(next_file);
                }
            }
        }
    }

    fn navigate_to_first_diff_in_file(&mut self, file_path: &str) {
        let reviewable_ids = self.review_engine.get_ordered_reviewable_ids();
        if let Some(first_in_file) = reviewable_ids.iter().find(|id| id.file_path == file_path) {
            self.ui_state.navigate_to((*first_in_file).clone());
            self.ui_state.current_file_path = Some(file_path.to_string());
            // Reset cursor when navigating to a new diff
            self.ui_state.cursor_index = 0;
        }
    }

    /// Get total number of lines in the currently displayed diff
    fn get_total_lines_in_current_diff(&self) -> usize {
        if let Some(reviewable_id) = &self.ui_state.current_reviewable_id {
            if let Some(diff) = self.review_engine.get_renderable_diff_object(reviewable_id) {
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
