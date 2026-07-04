//! Pure UI state management for the TUI
//!
//! This module contains presentation-layer state that tracks navigation,
//! focus, and input modes. Business logic state is handled by ReviewEngine.

use diffviz_review::ReviewableDiffId;
use std::collections::HashSet;
use std::time::{Duration, Instant};

/// Timeout duration for leader key mode (seconds)
const LEADER_TIMEOUT: Duration = Duration::from_secs(2);

/// Number of lines to scroll per page operation (drill viewport paging)
const PAGE_SCROLL_STEP: usize = 10;

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

/// Precomputed decision→files→chunks index (plan-drillnav-main-tui D6).
///
/// Built once at startup from the engine so navigation bounds and
/// approve/note target resolution never query the engine per keystroke.
/// Decisions follow engine order; files are sorted lexicographically by
/// path; chunks are sorted by ascending start line.
#[derive(Debug, Default, Clone)]
pub(crate) struct DrillIndex {
    pub(crate) decisions: Vec<DrillDecision>,
}

#[derive(Debug, Clone)]
pub(crate) struct DrillDecision {
    pub(crate) number: u32,
    pub(crate) files: Vec<DrillFile>,
}

#[derive(Debug, Clone)]
pub(crate) struct DrillFile {
    pub(crate) path: String,
    pub(crate) chunks: Vec<ReviewableDiffId>,
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
    /// Current input mode
    pub input_mode: InputMode,

    /// Input buffer for text entry
    pub input_buffer: String,

    /// Cursor position within input buffer
    pub input_cursor: usize,

    /// Application should quit
    pub should_quit: bool,

    /// Whether to show inline reasoning annotations in the diff view
    pub show_reasoning: bool,

    /// Leader key state tracking
    pub leader_active: bool,
    pub leader_pressed_at: Option<Instant>,
    pub leader_submenu: Option<char>,

    /// Whether to show help overlay
    pub show_help: bool,

    /// DrillNav state machine (replaces the two-panel navigation model).
    /// Private: accessed only through UiState methods (V4 encapsulation).
    drill_nav: DrillNavState,

    /// Precomputed decision→files→chunks index (D6), set at startup.
    drill_index: DrillIndex,

    /// One-shot error line for the status bar; cleared on the next keypress.
    /// Private: accessed only through UiState methods (V4 encapsulation).
    status_message: Option<String>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            input_mode: InputMode::Navigation,
            input_buffer: String::new(),
            input_cursor: 0,
            should_quit: false,
            show_reasoning: false,
            leader_active: false,
            leader_pressed_at: None,
            leader_submenu: None,
            show_help: false,
            drill_nav: DrillNavState::default(),
            drill_index: DrillIndex::default(),
            status_message: None,
        }
    }
}

impl UiState {
    /// Create new UI state
    pub fn new() -> Self {
        Self::default()
    }

    /// The focused chunk's ReviewableDiff ID (Drill mode); None while browsing.
    pub fn current_reviewable_id(&self) -> Option<ReviewableDiffId> {
        let (decision_idx, file_idx, cursor) = self.drill_position()?;
        Some(self.drill_index.decisions[decision_idx].files[file_idx].chunks[cursor].clone())
    }

    /// The focused sibling file's path (Drill mode); None while browsing.
    pub fn current_file_path(&self) -> Option<String> {
        let (decision_idx, file_idx, _) = self.drill_position()?;
        Some(
            self.drill_index.decisions[decision_idx].files[file_idx]
                .path
                .clone(),
        )
    }

    /// The decision number under the browse cursor; None while drilled in
    /// or when the review has no decisions.
    pub fn current_decision_number(&self) -> Option<u32> {
        let cursor = self.browse_cursor()?;
        self.drill_index.decisions.get(cursor).map(|d| d.number)
    }

    fn enter_input_mode(&mut self, mode: InputMode, initial_content: Option<String>) {
        self.input_mode = mode;
        self.input_buffer = initial_content.unwrap_or_default();
        self.input_cursor = self.input_buffer.len();
    }

    /// Enter instruction input mode for a chunk, preloading any existing
    /// note's content so the reviewer can see and edit it rather than typing
    /// blind into an empty box.
    pub fn start_instruction_input(
        &mut self,
        reviewable_id: ReviewableDiffId,
        existing_content: Option<String>,
    ) {
        self.enter_input_mode(InputMode::Instruction { reviewable_id }, existing_content);
    }

    /// Enter instruction input mode for a decision, preloading any existing
    /// note's content so the reviewer can see and edit it rather than typing
    /// blind into an empty box.
    pub fn start_decision_instruction_input(
        &mut self,
        decision_number: u32,
        existing_content: Option<String>,
    ) {
        self.enter_input_mode(
            InputMode::DecisionInstruction { decision_number },
            existing_content,
        );
    }

    /// Exit input mode and return to navigation
    pub fn exit_input_mode(&mut self) {
        self.enter_input_mode(InputMode::Navigation, None);
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

    /// Mark application to quit
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Toggle inline reasoning annotations visibility
    pub fn toggle_reasoning(&mut self) {
        self.show_reasoning = !self.show_reasoning;
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

    // ── DrillNav state machine ────────────────────────────────────────────

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

    /// Whether the given chunk in the focused file has expanded code context.
    /// False while browsing (the drill view only asks when drilled in).
    pub fn drill_chunk_expanded(&self, chunk: usize) -> bool {
        self.current_file_view()
            .is_some_and(|v| v.expanded.contains(&chunk))
    }

    /// Whether the given chunk in the focused file has its note expanded.
    /// False while browsing (the drill view only asks when drilled in).
    pub fn drill_chunk_note_expanded(&self, chunk: usize) -> bool {
        self.current_file_view()
            .is_some_and(|v| v.expanded_notes.contains(&chunk))
    }

    /// Read-only view of the precomputed decision→files→chunks index (D6),
    /// for views needing file/chunk metadata.
    pub(crate) fn drill_index(&self) -> &DrillIndex {
        &self.drill_index
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

    fn current_file_view_mut(&mut self) -> Option<&mut FileView> {
        match &mut self.drill_nav {
            DrillNavState::Browse { .. } => None,
            DrillNavState::Drill {
                file_idx, views, ..
            } => Some(&mut views[*file_idx]),
        }
    }

    /// Install the precomputed decision→files→chunks index (startup only).
    pub(crate) fn set_drill_index(&mut self, index: DrillIndex) {
        self.drill_index = index;
    }

    /// Set the one-shot status-bar error message.
    pub fn set_status_message(&mut self, message: String) {
        self.status_message = Some(message);
    }

    /// Clear the one-shot status-bar message (on the next keypress).
    pub fn clear_status_message(&mut self) {
        self.status_message = None;
    }

    /// Move the cursor up: previous decision card (Browse) or previous chunk (Drill).
    pub fn navigate_up(&mut self) {
        match &mut self.drill_nav {
            DrillNavState::Browse { cursor } => *cursor = cursor.saturating_sub(1),
            DrillNavState::Drill {
                file_idx, views, ..
            } => {
                let view = &mut views[*file_idx];
                if view.cursor > 0 {
                    view.cursor -= 1;
                    view.page_offset = 0;
                }
            }
        }
    }

    /// Move the cursor down: next decision card (Browse) or next chunk (Drill).
    pub fn navigate_down(&mut self) {
        match &mut self.drill_nav {
            DrillNavState::Browse { cursor } => {
                if *cursor + 1 < self.drill_index.decisions.len() {
                    *cursor += 1;
                }
            }
            DrillNavState::Drill {
                decision_idx,
                file_idx,
                views,
            } => {
                let n_chunks = self.drill_index.decisions[*decision_idx].files[*file_idx]
                    .chunks
                    .len();
                let view = &mut views[*file_idx];
                if view.cursor + 1 < n_chunks {
                    view.cursor += 1;
                    view.page_offset = 0;
                }
            }
        }
    }

    /// Cycle to the previous sibling file (Drill only, wraps around).
    pub fn navigate_left(&mut self) {
        if let DrillNavState::Drill {
            file_idx, views, ..
        } = &mut self.drill_nav
        {
            let n = views.len();
            *file_idx = file_idx.checked_sub(1).unwrap_or(n - 1);
        }
    }

    /// Cycle to the next sibling file (Drill only, wraps around).
    pub fn navigate_right(&mut self) {
        if let DrillNavState::Drill {
            file_idx, views, ..
        } = &mut self.drill_nav
        {
            *file_idx = (*file_idx + 1) % views.len();
        }
    }

    /// Enter the decision under the browse cursor.
    pub fn drill_in(&mut self) {
        if let DrillNavState::Browse { cursor } = &self.drill_nav {
            let cursor = *cursor;
            if let Some(decision) = self.drill_index.decisions.get(cursor)
                && !decision.files.is_empty()
            {
                self.drill_nav = DrillNavState::Drill {
                    decision_idx: cursor,
                    file_idx: 0,
                    views: vec![FileView::default(); decision.files.len()],
                };
            }
        }
    }

    /// Back out of a drilled decision, restoring the browse cursor to it.
    pub fn back(&mut self) {
        if let DrillNavState::Drill { decision_idx, .. } = &self.drill_nav {
            self.drill_nav = DrillNavState::Browse {
                cursor: *decision_idx,
            };
        }
    }

    /// Toggle expanded code context on the focused chunk (Drill only).
    pub fn toggle_context(&mut self) {
        if let Some(view) = self.current_file_view_mut() {
            let chunk = view.cursor;
            if !view.expanded.remove(&chunk) {
                view.expanded.insert(chunk);
            }
        }
    }

    /// Toggle note expansion on the focused chunk (Drill only).
    pub fn toggle_note_expansion(&mut self) {
        if let Some(view) = self.current_file_view_mut() {
            let chunk = view.cursor;
            if !view.expanded_notes.remove(&chunk) {
                view.expanded_notes.insert(chunk);
            }
        }
    }

    /// Page the drill viewport up (Ctrl-u).
    pub fn drill_page_up(&mut self) {
        if let Some(view) = self.current_file_view_mut() {
            view.page_offset = view.page_offset.saturating_sub(PAGE_SCROLL_STEP);
        }
    }

    /// Page the drill viewport down (Ctrl-d).
    ///
    /// The offset is unclamped at the state level; Phase 2 rendering combines
    /// it with `scroll_into_view` against the real content height.
    pub fn drill_page_down(&mut self) {
        if let Some(view) = self.current_file_view_mut() {
            view.page_offset += PAGE_SCROLL_STEP;
        }
    }

    /// Jump to the first decision card (Browse) or first chunk (Drill).
    pub fn to_top(&mut self) {
        match &mut self.drill_nav {
            DrillNavState::Browse { cursor } => *cursor = 0,
            DrillNavState::Drill {
                file_idx, views, ..
            } => {
                let view = &mut views[*file_idx];
                view.cursor = 0;
                view.page_offset = 0;
            }
        }
    }

    /// Jump to the last decision card (Browse) or last chunk (Drill).
    pub fn to_bottom(&mut self) {
        match &mut self.drill_nav {
            DrillNavState::Browse { cursor } => {
                *cursor = self.drill_index.decisions.len().saturating_sub(1);
            }
            DrillNavState::Drill {
                decision_idx,
                file_idx,
                views,
            } => {
                let n_chunks = self.drill_index.decisions[*decision_idx].files[*file_idx]
                    .chunks
                    .len();
                views[*file_idx].cursor = n_chunks.saturating_sub(1);
                views[*file_idx].page_offset = 0;
            }
        }
    }
}
