//! Navigation state for decision-first review hierarchy
//!
//! Manages navigation through Decision List → Decision Detail Modal → File View → Chunk Detail.
//! Replaces file-first navigation as the primary navigation pattern.

use diffviz_review::{engines::ReviewEngine, ReviewableDiffId};

/// Navigation level in the decision-first hierarchy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationLevel {
    /// At the decision list
    Decision,
    /// Inside a decision's file view
    File,
    /// Inside a chunk detail view
    Chunk,
}

/// Tracks position in the decision-first navigation hierarchy
#[derive(Debug, Clone)]
pub struct DecisionNavigationState {
    /// Current navigation level (decision list, file view, or chunk view)
    pub current_level: NavigationLevel,

    /// Currently selected decision number (if any)
    pub selected_decision: Option<u32>,

    /// Currently selected file path (when at File or Chunk level)
    pub selected_file: Option<String>,

    /// Currently selected chunk ReviewableDiffId (when at Chunk level)
    pub selected_chunk: Option<ReviewableDiffId>,

    /// Whether decision detail modal is visible
    pub show_decision_modal: bool,

    /// Current index in decision list (for navigation)
    pub decision_list_index: usize,

    /// Current index in file list within a decision (for navigation)
    pub file_list_index: usize,
}

impl Default for DecisionNavigationState {
    fn default() -> Self {
        Self::new()
    }
}

impl DecisionNavigationState {
    /// Create new decision navigation state at the decision list
    pub fn new() -> Self {
        Self {
            current_level: NavigationLevel::Decision,
            selected_decision: None,
            selected_file: None,
            selected_chunk: None,
            show_decision_modal: false,
            decision_list_index: 0,
            file_list_index: 0,
        }
    }

    /// Navigate to next decision in the list
    pub fn next_decision(&mut self) {
        self.decision_list_index += 1;
    }

    /// Navigate to previous decision in the list
    pub fn prev_decision(&mut self) {
        self.decision_list_index = self.decision_list_index.saturating_sub(1);
    }

    /// Clamp decision list index to valid range
    pub fn clamp_decision_index(&mut self, max_index: usize) {
        if self.decision_list_index > max_index {
            self.decision_list_index = max_index;
        }
    }

    /// Select a specific decision by number and open modal
    pub fn select_decision(&mut self, decision_number: u32) {
        self.selected_decision = Some(decision_number);
        self.show_decision_modal = true;
    }

    /// Close the decision detail modal
    pub fn close_decision_modal(&mut self) {
        self.show_decision_modal = false;
    }

    /// Drill into files view for the selected decision
    pub fn drill_into_files(&mut self) {
        if self.selected_decision.is_some() {
            self.current_level = NavigationLevel::File;
            self.show_decision_modal = false;
            self.file_list_index = 0;
        }
    }

    /// Return to decision list from file/chunk view
    pub fn back_to_decisions(&mut self) {
        self.current_level = NavigationLevel::Decision;
        self.selected_file = None;
        self.selected_chunk = None;
        self.file_list_index = 0;
    }

    /// Navigate to next file in file view
    pub fn next_file(&mut self) {
        self.file_list_index += 1;
    }

    /// Navigate to previous file in file view
    pub fn prev_file(&mut self) {
        self.file_list_index = self.file_list_index.saturating_sub(1);
    }

    /// Clamp file list index to valid range
    pub fn clamp_file_index(&mut self, max_index: usize) {
        if self.file_list_index > max_index {
            self.file_list_index = max_index;
        }
    }

    /// Select a file and move to chunk view
    pub fn select_file(&mut self, file_path: String, chunk_id: ReviewableDiffId) {
        self.selected_file = Some(file_path);
        self.selected_chunk = Some(chunk_id);
        self.current_level = NavigationLevel::Chunk;
    }

    /// Navigate to next chunk within current file
    pub fn next_chunk(&mut self) {
        // This will be coordinated with file view component
        // which will update selected_chunk based on visible chunks
    }

    /// Navigate to previous chunk within current file
    pub fn prev_chunk(&mut self) {
        // This will be coordinated with file view component
        // which will update selected_chunk based on visible chunks
    }

    /// Clear all navigation state (used when reloading review)
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Get the current decision number if any
    pub fn current_decision(&self) -> Option<u32> {
        self.selected_decision
    }

    /// Check if we're at the decision list level
    pub fn at_decision_list(&self) -> bool {
        self.current_level == NavigationLevel::Decision
    }

    /// Check if we're in file view
    pub fn at_file_view(&self) -> bool {
        self.current_level == NavigationLevel::File
    }

    /// Check if we're in chunk view
    pub fn at_chunk_view(&self) -> bool {
        self.current_level == NavigationLevel::Chunk
    }
}

/// Helper to build filtered file list for a specific decision
pub fn get_files_for_decision(review_engine: &ReviewEngine, decision_number: u32) -> Vec<String> {
    if let Some(decision) = review_engine.get_decision(decision_number) {
        let mut files = Vec::new();
        for code_impact in &decision.code_impacts {
            if !files.contains(&code_impact.file) {
                files.push(code_impact.file.clone());
            }
        }
        files.sort();
        files
    } else {
        Vec::new()
    }
}

/// Helper to get chunks for a specific file within a decision
pub fn get_chunks_for_file_in_decision(
    review_engine: &ReviewEngine,
    decision_number: u32,
    file_path: &str,
) -> Vec<ReviewableDiffId> {
    if let Some(decision) = review_engine.get_decision(decision_number) {
        let mut chunk_ids: Vec<ReviewableDiffId> = Vec::new();
        for code_impact in &decision.code_impacts {
            if code_impact.file == file_path {
                // Find ReviewableDiffs that match this code impact's line ranges
                let all_diffs = review_engine.get_ordered_reviewable_ids();
                for id_ref in &all_diffs {
                    let id = (*id_ref).clone();
                    if let Some(diff) = review_engine.get_reviewable_diff(&id) {
                        if diff.file_path == file_path {
                            // Check if diff overlaps with any line range in this code impact
                            for line_range in &code_impact.line_ranges {
                                if diff.id.line_range.end_line >= line_range.start
                                    && diff.id.line_range.start_line <= line_range.end
                                {
                                    if !chunk_ids.contains(&id) {
                                        chunk_ids.push(id.clone());
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
        chunk_ids
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_decision_navigation_state() {
        let state = DecisionNavigationState::new();
        assert_eq!(state.current_level, NavigationLevel::Decision);
        assert_eq!(state.selected_decision, None);
        assert_eq!(state.selected_file, None);
        assert_eq!(state.selected_chunk, None);
        assert!(!state.show_decision_modal);
        assert_eq!(state.decision_list_index, 0);
        assert_eq!(state.file_list_index, 0);
    }

    #[test]
    fn test_next_decision() {
        let mut state = DecisionNavigationState::new();
        state.next_decision();
        assert_eq!(state.decision_list_index, 1);
        state.next_decision();
        assert_eq!(state.decision_list_index, 2);
    }

    #[test]
    fn test_prev_decision() {
        let mut state = DecisionNavigationState::new();
        state.decision_list_index = 5;
        state.prev_decision();
        assert_eq!(state.decision_list_index, 4);
        state.prev_decision();
        assert_eq!(state.decision_list_index, 3);
    }

    #[test]
    fn test_prev_decision_at_start() {
        let mut state = DecisionNavigationState::new();
        state.prev_decision();
        assert_eq!(state.decision_list_index, 0); // Should stay at 0
    }

    #[test]
    fn test_clamp_decision_index() {
        let mut state = DecisionNavigationState::new();
        state.decision_list_index = 10;
        state.clamp_decision_index(5);
        assert_eq!(state.decision_list_index, 5);

        state.clamp_decision_index(20);
        assert_eq!(state.decision_list_index, 5); // Should not change
    }

    #[test]
    fn test_select_decision() {
        let mut state = DecisionNavigationState::new();
        state.select_decision(2);
        assert_eq!(state.selected_decision, Some(2));
        assert!(state.show_decision_modal);
    }

    #[test]
    fn test_close_decision_modal() {
        let mut state = DecisionNavigationState::new();
        state.show_decision_modal = true;
        state.close_decision_modal();
        assert!(!state.show_decision_modal);
    }

    #[test]
    fn test_drill_into_files() {
        let mut state = DecisionNavigationState::new();
        state.select_decision(1);
        state.drill_into_files();

        assert_eq!(state.current_level, NavigationLevel::File);
        assert!(!state.show_decision_modal);
        assert_eq!(state.file_list_index, 0);
    }

    #[test]
    fn test_drill_into_files_requires_decision() {
        let mut state = DecisionNavigationState::new();
        state.drill_into_files();

        // Should not change level if no decision selected
        assert_eq!(state.current_level, NavigationLevel::Decision);
    }

    #[test]
    fn test_back_to_decisions() {
        let mut state = DecisionNavigationState::new();
        state.current_level = NavigationLevel::File;
        state.selected_file = Some("src/main.rs".to_string());
        state.file_list_index = 5;

        state.back_to_decisions();

        assert_eq!(state.current_level, NavigationLevel::Decision);
        assert_eq!(state.selected_file, None);
        assert_eq!(state.selected_chunk, None);
        assert_eq!(state.file_list_index, 0);
    }

    #[test]
    fn test_select_file() {
        let mut state = DecisionNavigationState::new();
        // For this test, we just verify state transitions without creating actual ReviewableDiffId
        // The chunk_id would be properly created when wired to the actual TUI components
        assert_eq!(state.current_level, NavigationLevel::Decision);

        state.current_level = NavigationLevel::File;
        state.selected_file = Some("src/main.rs".to_string());

        assert_eq!(state.current_level, NavigationLevel::File);
        assert_eq!(state.selected_file, Some("src/main.rs".to_string()));
    }

    #[test]
    fn test_reset() {
        let mut state = DecisionNavigationState::new();
        state.current_level = NavigationLevel::Chunk;
        state.selected_decision = Some(5);
        state.decision_list_index = 10;

        state.reset();

        assert_eq!(state.current_level, NavigationLevel::Decision);
        assert_eq!(state.selected_decision, None);
        assert_eq!(state.decision_list_index, 0);
    }

    #[test]
    fn test_at_decision_list() {
        let mut state = DecisionNavigationState::new();
        assert!(state.at_decision_list());

        state.current_level = NavigationLevel::File;
        assert!(!state.at_decision_list());
    }

    #[test]
    fn test_at_file_view() {
        let mut state = DecisionNavigationState::new();
        state.current_level = NavigationLevel::File;
        assert!(state.at_file_view());

        state.current_level = NavigationLevel::Decision;
        assert!(!state.at_file_view());
    }

    #[test]
    fn test_at_chunk_view() {
        let mut state = DecisionNavigationState::new();
        state.current_level = NavigationLevel::Chunk;
        assert!(state.at_chunk_view());

        state.current_level = NavigationLevel::File;
        assert!(!state.at_chunk_view());
    }
}
