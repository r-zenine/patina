//! Pure UI state management for the TUI
//!
//! This module contains presentation-layer state that tracks navigation,
//! focus, and input modes. Business logic state is handled by ReviewEngine.

use crate::decision_navigation::DecisionNavigationTree;
use diffviz_review::ReviewableDiffId;
use std::collections::HashSet;
use std::time::{Duration, Instant};

/// Timeout duration for leader key mode (seconds)
const LEADER_TIMEOUT: Duration = Duration::from_secs(2);

/// Number of lines to scroll per page operation
const PAGE_SCROLL_STEP: usize = 10;

/// Default view height used when total lines is unknown (for cursor positioning)
const CURSOR_VIEW_HEIGHT_FALLBACK: usize = 20;

/// Which panel currently has focus
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusPanel {
    FileList,
    DiffView,
}

/// Per-file view state within a drilled decision, retained while cycling
/// sibling files with h/l so a round-trip doesn't lose the reviewer's place.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FileView {
    /// j/k cursor within the file's chunk list (chunk-granular, no line cursor).
    pub(crate) cursor: usize,
    /// Chunks with expanded code context (Tab toggles).
    pub(crate) expanded: HashSet<usize>,
    /// Chunks with expanded note text (i toggles).
    pub(crate) expanded_notes: HashSet<usize>,
    /// Drill viewport page offset (Ctrl-d/u) for reading inside chunks
    /// taller than the screen.
    pub(crate) page_offset: usize,
}

/// State machine for the DrillNav pattern.
///
/// `Browse` — exploring top-level decisions as cards.
/// `Drill`  — inside a decision: its header is pinned, chunk cards scroll below.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrillNavState {
    Browse {
        cursor: usize,
    },
    Drill {
        /// Which decision we're inside (Enter drills in, Esc backs out).
        decision_idx: usize,
        /// h/l cycling among sibling files within the decision.
        file_idx: usize,
        /// One view state per sibling file, index-aligned with the
        /// decision's sorted file list.
        views: Vec<FileView>,
    },
}

impl Default for DrillNavState {
    fn default() -> Self {
        DrillNavState::Browse { cursor: 0 }
    }
}

/// Input mode for handling different types of text input
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    Navigation,
    Instruction { reviewable_id: ReviewableDiffId },
    DecisionInstruction { decision_number: u32 },
}

impl InputMode {
    pub fn is_input_active(&self) -> bool {
        !matches!(self, InputMode::Navigation)
    }
}

/// Pure UI navigation and display state
#[derive(Clone)]
pub struct UiState {
    /// Which panel has focus
    pub focused_panel: FocusPanel,

    /// Current input mode
    pub input_mode: InputMode,

    /// Vertical scroll offset in the diff view
    pub scroll_offset: usize,

    /// Input buffer for text entry
    pub input_buffer: String,

    /// Cursor position within input buffer
    pub input_cursor: usize,

    /// Whether to show all context lines or hide irrelevant ones
    pub show_all_context: bool,

    /// Application should quit
    pub should_quit: bool,

    /// Whether to highlight semantic changes
    pub highlight_semantics: bool,

    /// Current cursor position in the diff view (line index, not line number)
    pub cursor_index: usize,

    /// Anchor position for range selection (cursor index when 'v' first pressed)
    pub selection_anchor: Option<usize>,

    /// Current selection range (line numbers: start, end)
    pub selection_range: Option<(usize, usize)>,

    /// Whether to show instructions overlay
    pub show_instructions: bool,

    /// Whether to show inline reasoning annotations in the diff view
    pub show_reasoning: bool,

    /// Leader key state tracking
    pub leader_active: bool,
    pub leader_pressed_at: Option<Instant>,
    pub leader_submenu: Option<char>,

    /// Whether to show help overlay
    pub show_help: bool,

    /// Decision-based navigation tree (primary navigation pattern)
    pub decision_tree: DecisionNavigationTree,

    /// DrillNav state machine (replaces the two-panel navigation model).
    /// Private: accessed only through UiState methods (V4 encapsulation).
    drill_nav: DrillNavState,

    /// One-shot error line for the status bar; cleared on the next keypress.
    /// Private: accessed only through UiState methods (V4 encapsulation).
    status_message: Option<String>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            focused_panel: FocusPanel::FileList,
            input_mode: InputMode::Navigation,
            scroll_offset: 0,
            input_buffer: String::new(),
            input_cursor: 0,
            show_all_context: true,
            should_quit: false,
            highlight_semantics: true,
            cursor_index: 0,
            selection_anchor: None,
            selection_range: None,
            show_instructions: false,
            show_reasoning: false,
            leader_active: false,
            leader_pressed_at: None,
            leader_submenu: None,
            show_help: false,
            decision_tree: DecisionNavigationTree::new(),
            drill_nav: DrillNavState::default(),
            status_message: None,
        }
    }
}

impl UiState {
    /// Create new UI state
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset scroll when navigating (called after tree navigation updates selection)
    pub fn reset_scroll(&mut self) {
        self.scroll_offset = 0;
    }

    /// Get currently selected ReviewableDiff ID (computed from tree)
    pub fn current_reviewable_id(&self) -> Option<ReviewableDiffId> {
        self.decision_tree.selected_chunk_id()
    }

    /// Get currently selected file path (extracted from chunk_id when at chunk depth)
    pub fn current_file_path(&self) -> Option<String> {
        self.decision_tree
            .selected_chunk_id()
            .map(|chunk_id| chunk_id.file_path.clone())
    }

    /// Get currently selected decision number when at depth 0 (decision level)
    pub fn current_decision_number(&self) -> Option<u32> {
        if self.decision_tree.selected_path.depth() == 0 {
            self.decision_tree.selected_decision_number()
        } else {
            None
        }
    }

    /// Switch focus between panels
    pub fn toggle_focus(&mut self) {
        self.focused_panel = match self.focused_panel {
            FocusPanel::FileList => FocusPanel::DiffView,
            FocusPanel::DiffView => FocusPanel::FileList,
        };
    }

    fn enter_input_mode(&mut self, mode: InputMode) {
        self.input_mode = mode;
        self.input_buffer.clear();
        self.input_cursor = 0;
    }

    /// Enter instruction input mode for a chunk
    pub fn start_instruction_input(&mut self, reviewable_id: ReviewableDiffId) {
        self.enter_input_mode(InputMode::Instruction { reviewable_id });
    }

    /// Enter instruction input mode for a decision
    pub fn start_decision_instruction_input(&mut self, decision_number: u32) {
        self.enter_input_mode(InputMode::DecisionInstruction { decision_number });
    }

    /// Exit input mode and return to navigation
    pub fn exit_input_mode(&mut self) {
        self.enter_input_mode(InputMode::Navigation);
    }

    /// Check if currently in any input mode
    pub fn is_in_input_mode(&self) -> bool {
        self.input_mode.is_input_active()
    }

    /// Add character to input buffer
    pub fn input_char(&mut self, c: char) {
        if self.input_mode.is_input_active() {
            self.input_buffer.insert(self.input_cursor, c);
            self.input_cursor += 1;
        }
    }

    /// Delete character before cursor
    pub fn delete_char(&mut self) {
        if self.input_mode.is_input_active() && self.input_cursor > 0 {
            self.input_cursor -= 1;
            self.input_buffer.remove(self.input_cursor);
        }
    }

    /// Move cursor left
    pub fn move_cursor_left(&mut self) {
        if self.input_cursor > 0 {
            self.input_cursor -= 1;
        }
    }

    /// Move cursor right
    pub fn move_cursor_right(&mut self) {
        if self.input_cursor < self.input_buffer.len() {
            self.input_cursor += 1;
        }
    }

    /// Move cursor to beginning
    pub fn move_cursor_home(&mut self) {
        self.input_cursor = 0;
    }

    /// Move cursor to end
    pub fn move_cursor_end(&mut self) {
        self.input_cursor = self.input_buffer.len();
    }

    /// Scroll diff view up
    pub fn scroll_up(&mut self, lines: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
    }

    /// Scroll diff view down
    pub fn scroll_down(&mut self, lines: usize) {
        self.scroll_offset += lines;
    }

    /// Page up in diff view
    pub fn page_up(&mut self) {
        self.scroll_up(PAGE_SCROLL_STEP);
    }

    /// Page down in diff view
    pub fn page_down(&mut self) {
        self.scroll_down(PAGE_SCROLL_STEP);
    }

    /// Move cursor and page up
    pub fn cursor_page_up(&mut self, _total_lines: usize) {
        self.cursor_index = self.cursor_index.saturating_sub(PAGE_SCROLL_STEP);
        self.update_selection_range();
        self.page_up();
        if self.cursor_index < self.scroll_offset {
            self.scroll_offset = self.cursor_index;
        }
    }

    /// Move cursor and page down
    pub fn cursor_page_down(&mut self, total_lines: usize) {
        self.cursor_index =
            (self.cursor_index + PAGE_SCROLL_STEP).min(total_lines.saturating_sub(1));
        self.update_selection_range();
        self.page_down();
    }

    /// Move cursor to top
    pub fn cursor_to_top(&mut self) {
        self.cursor_index = 0;
        self.scroll_offset = 0;
        self.update_selection_range();
    }

    /// Move cursor to bottom
    pub fn cursor_to_bottom(&mut self, total_lines: usize) {
        if total_lines > 0 {
            self.cursor_index = total_lines - 1;
            self.update_selection_range();
            self.scroll_offset = total_lines.saturating_sub(CURSOR_VIEW_HEIGHT_FALLBACK);
        }
    }

    /// Move cursor up one line
    pub fn cursor_up(&mut self) {
        if self.cursor_index > 0 {
            self.cursor_index -= 1;
            self.update_selection_range();
            if self.cursor_index < self.scroll_offset {
                self.scroll_up(1);
            }
        }
    }

    /// Move cursor down one line
    pub fn cursor_down(&mut self, total_lines: usize) {
        if self.cursor_index + 1 < total_lines {
            self.cursor_index += 1;
            self.update_selection_range();
            if self.cursor_index >= self.scroll_offset + CURSOR_VIEW_HEIGHT_FALLBACK {
                self.scroll_down(1);
            }
        }
    }

    /// Toggle showing all context vs hiding irrelevant context
    pub fn toggle_context_display(&mut self) {
        self.show_all_context = !self.show_all_context;
    }

    /// Mark application to quit
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Toggle semantic highlighting on/off
    pub fn toggle_semantic_highlight(&mut self) {
        self.highlight_semantics = !self.highlight_semantics;
    }

    /// Toggle instructions visibility
    pub fn toggle_instructions(&mut self) {
        self.show_instructions = !self.show_instructions;
    }

    /// Toggle inline reasoning annotations visibility
    pub fn toggle_reasoning(&mut self) {
        self.show_reasoning = !self.show_reasoning;
    }

    /// Toggle range selection - cycles through states (vim-like visual select)
    /// State cycle: none → selecting (anchor set) → finalized → clear
    pub fn toggle_range_selection(&mut self, total_lines: usize) {
        if total_lines == 0 {
            return;
        }

        // Ensure cursor is within bounds
        if self.cursor_index >= total_lines {
            self.cursor_index = total_lines - 1;
        }

        if self.selection_anchor.is_some() {
            // Finalize selection, clear anchor (keep the range)
            self.selection_anchor = None;
        } else if self.selection_range.is_some() {
            // Clear selection entirely
            self.selection_range = None;
        } else {
            // Start new selection at current cursor position
            self.selection_anchor = Some(self.cursor_index);
            // Create initial single-line selection (line numbers are 1-indexed)
            let line_num = self.cursor_index + 1;
            self.selection_range = Some((line_num, line_num));
        }
    }

    /// Update selection range based on cursor movement (when anchor is set)
    /// This extends the visual selection as the user navigates
    pub fn update_selection_range(&mut self) {
        if let Some(anchor) = self.selection_anchor {
            // Calculate range (line numbers are 1-indexed)
            let start = anchor.min(self.cursor_index) + 1;
            let end = anchor.max(self.cursor_index) + 1;
            self.selection_range = Some((start, end));
        }
    }

    /// Activate leader key mode
    pub fn activate_leader(&mut self) {
        self.leader_active = true;
        self.leader_pressed_at = Some(Instant::now());
        self.leader_submenu = None;
    }

    /// Enter a leader submenu (a, c, i, t, etc.)
    pub fn enter_leader_submenu(&mut self, submenu: char) {
        self.leader_submenu = Some(submenu);
        // Reset timeout
        self.leader_pressed_at = Some(Instant::now());
    }

    /// Deactivate leader key mode
    pub fn deactivate_leader(&mut self) {
        self.leader_active = false;
        self.leader_pressed_at = None;
        self.leader_submenu = None;
    }

    /// Check if leader key has timed out
    pub fn is_leader_timed_out(&self) -> bool {
        if let Some(pressed_at) = self.leader_pressed_at {
            pressed_at.elapsed() > LEADER_TIMEOUT
        } else {
            false
        }
    }

    /// Get remaining timeout duration for display
    pub fn leader_timeout_remaining(&self) -> Option<Duration> {
        if let Some(pressed_at) = self.leader_pressed_at {
            let timeout = LEADER_TIMEOUT;
            let elapsed = pressed_at.elapsed();
            if elapsed < timeout {
                Some(timeout - elapsed)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Navigate to the first item in the decision tree
    pub fn navigate_to_first_in_tree(&mut self) {
        let flattened = self.decision_tree.flatten();
        if let Some(first) = flattened.first() {
            self.decision_tree.selected_path = first.path.clone();
            self.reset_scroll();
        }
    }

    /// Navigate to the last item in the decision tree
    pub fn navigate_to_last_in_tree(&mut self) {
        let flattened = self.decision_tree.flatten();
        if let Some(last) = flattened.last() {
            self.decision_tree.selected_path = last.path.clone();
            self.reset_scroll();
        }
    }

    // ── DrillNav state machine ────────────────────────────────────────────
    // Phase 0 of plan-drillnav-main-tui: accessors are real, mutators are
    // inert stubs — the state machine logic lands in Phase 1.

    /// DrillNav mode name for snapshots: "Browse" or "Drill".
    pub fn nav_mode(&self) -> &'static str {
        match self.drill_nav {
            DrillNavState::Browse { .. } => "Browse",
            DrillNavState::Drill { .. } => "Drill",
        }
    }

    /// Cursor over decision cards while browsing; None when drilled in.
    pub fn browse_cursor(&self) -> Option<usize> {
        match self.drill_nav {
            DrillNavState::Browse { cursor } => Some(cursor),
            DrillNavState::Drill { .. } => None,
        }
    }

    /// (decision_idx, file_idx, chunk_cursor) when drilled in; None while browsing.
    pub fn drill_position(&self) -> Option<(usize, usize, usize)> {
        match &self.drill_nav {
            DrillNavState::Browse { .. } => None,
            DrillNavState::Drill {
                decision_idx,
                file_idx,
                views,
            } => Some((*decision_idx, *file_idx, views[*file_idx].cursor)),
        }
    }

    /// Whether the focused chunk has expanded code context; None while browsing.
    pub fn drill_context_expanded(&self) -> Option<bool> {
        self.current_file_view()
            .map(|v| v.expanded.contains(&v.cursor))
    }

    /// Whether the focused chunk has its note expanded; None while browsing.
    pub fn drill_note_expanded(&self) -> Option<bool> {
        self.current_file_view()
            .map(|v| v.expanded_notes.contains(&v.cursor))
    }

    /// Drill viewport page offset for the focused file; None while browsing.
    pub fn drill_page_offset(&self) -> Option<usize> {
        self.current_file_view().map(|v| v.page_offset)
    }

    /// One-shot status-bar error message, if any.
    pub fn status_message(&self) -> Option<&str> {
        self.status_message.as_deref()
    }

    fn current_file_view(&self) -> Option<&FileView> {
        match &self.drill_nav {
            DrillNavState::Browse { .. } => None,
            DrillNavState::Drill {
                file_idx, views, ..
            } => Some(&views[*file_idx]),
        }
    }

    /// Set the one-shot status-bar error message.
    pub fn set_status_message(&mut self, _message: String) {}

    /// Clear the one-shot status-bar message (on the next keypress).
    pub fn clear_status_message(&mut self) {}

    /// Move the cursor up: previous decision card (Browse) or previous chunk (Drill).
    pub fn navigate_up(&mut self) {}

    /// Move the cursor down: next decision card (Browse) or next chunk (Drill).
    pub fn navigate_down(&mut self) {}

    /// Cycle to the previous sibling file (Drill only, wraps around).
    pub fn navigate_left(&mut self) {}

    /// Cycle to the next sibling file (Drill only, wraps around).
    pub fn navigate_right(&mut self) {}

    /// Enter the decision under the browse cursor.
    pub fn drill_in(&mut self) {}

    /// Back out of a drilled decision, restoring the browse cursor to it.
    pub fn back(&mut self) {}

    /// Toggle expanded code context on the focused chunk (Drill only).
    pub fn toggle_context(&mut self) {}

    /// Toggle note expansion on the focused chunk (Drill only).
    pub fn toggle_note_expansion(&mut self) {}

    /// Page the drill viewport up (Ctrl-u).
    pub fn drill_page_up(&mut self) {}

    /// Page the drill viewport down (Ctrl-d).
    pub fn drill_page_down(&mut self) {}

    /// Jump to the first decision card (Browse) or first chunk (Drill).
    pub fn to_top(&mut self) {}

    /// Jump to the last decision card (Browse) or last chunk (Drill).
    pub fn to_bottom(&mut self) {}
}
