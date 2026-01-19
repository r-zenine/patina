//! Navigation logic for collapsible file list

use diffviz_review::{engines::ReviewEngine, ReviewableDiffId};
use std::collections::BTreeMap;

use crate::state::UiState;

pub struct NavigationState {
    pub selection_map: Vec<SelectionItem>,
    pub current_index: usize,
}

#[derive(Clone, Debug)]
pub enum SelectionItem {
    FileHeader {
        path: String,
        first_diff_id: ReviewableDiffId,
    },
    DiffItem {
        path: String,
        diff_id: ReviewableDiffId,
    },
}

impl NavigationState {
    pub fn build(ui_state: &UiState, review_engine: &ReviewEngine) -> Self {
        let reviewable_ids = review_engine.get_ordered_reviewable_ids();

        // Group ReviewableDiffs by file path
        let mut files: BTreeMap<String, Vec<ReviewableDiffId>> = BTreeMap::new();
        for &id in &reviewable_ids {
            if let Some(diff) = review_engine.get_reviewable_diff(id) {
                files
                    .entry(diff.file_path.clone())
                    .or_default()
                    .push(id.clone());
            }
        }

        // Build selection map
        let mut selection_map = Vec::new();
        for (file_path, diff_ids) in files {
            if let Some(first_id) = diff_ids.first() {
                // Add file header
                selection_map.push(SelectionItem::FileHeader {
                    path: file_path.clone(),
                    first_diff_id: first_id.clone(),
                });

                // Add diff items if expanded
                if ui_state.is_file_expanded(&file_path) {
                    for diff_id in diff_ids {
                        selection_map.push(SelectionItem::DiffItem {
                            path: file_path.clone(),
                            diff_id: diff_id.clone(),
                        });
                    }
                }
            }
        }

        NavigationState {
            selection_map,
            current_index: ui_state.file_list_selection,
        }
    }

    pub fn get_current_selection(&self) -> Option<&SelectionItem> {
        self.selection_map.get(self.current_index)
    }

    pub fn move_up(&mut self) -> usize {
        if self.current_index > 0 {
            self.current_index -= 1;
        }
        self.current_index
    }

    pub fn move_down(&mut self) -> usize {
        if self.current_index < self.selection_map.len().saturating_sub(1) {
            self.current_index += 1;
        }
        self.current_index
    }

    pub fn move_to_first(&mut self) -> usize {
        self.current_index = 0;
        self.current_index
    }

    pub fn move_to_last(&mut self) -> usize {
        self.current_index = self.selection_map.len().saturating_sub(1);
        self.current_index
    }
}

/// Handle navigation in the file list
pub fn navigate_file_list(
    ui_state: &mut UiState,
    review_engine: &ReviewEngine,
    direction: NavigationDirection,
) {
    let mut nav_state = NavigationState::build(ui_state, review_engine);

    let new_index = match direction {
        NavigationDirection::Up => nav_state.move_up(),
        NavigationDirection::Down => nav_state.move_down(),
        NavigationDirection::First => nav_state.move_to_first(),
        NavigationDirection::Last => nav_state.move_to_last(),
    };

    ui_state.file_list_selection = new_index;

    // Update current reviewable if on a diff item
    if let Some(selection) = nav_state.get_current_selection() {
        match selection {
            SelectionItem::DiffItem { diff_id, .. } => {
                ui_state.current_reviewable_id = Some(diff_id.clone());
            }
            SelectionItem::FileHeader { first_diff_id, .. } => {
                // When on a file header, show the first diff
                ui_state.current_reviewable_id = Some(first_diff_id.clone());
            }
        }
    }
}

/// Handle enter key on current selection
pub fn handle_selection_action(ui_state: &mut UiState, review_engine: &ReviewEngine) {
    let nav_state = NavigationState::build(ui_state, review_engine);

    if let Some(selection) = nav_state.get_current_selection() {
        match selection {
            SelectionItem::FileHeader { path, .. } => {
                // Toggle expansion of the file
                ui_state.toggle_file_expansion(path);
            }
            SelectionItem::DiffItem { diff_id, .. } => {
                // Navigate to the diff
                ui_state.current_reviewable_id = Some(diff_id.clone());
                // Optionally switch to diff view
                ui_state.focused_panel = crate::state::FocusPanel::DiffView;
            }
        }
    }
}

#[derive(Debug)]
pub enum NavigationDirection {
    Up,
    Down,
    First,
    Last,
}
