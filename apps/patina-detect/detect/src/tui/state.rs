//! Pure UI state management for the triage TUI.
//!
//! This module tracks navigation, focus, and input modes only. The symptom
//! list itself (and its baseline) live on `TriageApp`, mirroring how
//! `diffviz-review-tui`'s `UiState` holds cursors/indices while
//! `ReviewEngine` owns the domain data.

use std::time::Duration;
use tui_elm::LeaderState;

use crate::entities::SymptomId;

/// Timeout duration for leader key mode (seconds).
const LEADER_TIMEOUT: Duration = Duration::from_secs(2);

/// Number of lines to scroll per page operation (drill viewport paging).
const PAGE_SCROLL_STEP: usize = 10;

/// Per-symptom view state while drilled in, retained so paging position
/// survives navigating back to Browse and drilling in again is not
/// required (unlike `diffviz-review-tui`, there is no sibling-file cycling
/// here — `Symptom -> Site` is a flat 2-level hierarchy, so `siblings: None`
/// is always passed to `render_drill`).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SiteView {
    /// j/k cursor within the symptom's site list.
    pub(crate) cursor: usize,
    /// Drill viewport page offset (Ctrl-d/u) for reading inside sites taller
    /// than the screen.
    pub(crate) page_offset: usize,
}

/// State machine for the DrillNav pattern.
///
/// `Browse` — exploring untriaged symptoms as cards.
/// `Drill`  — inside a symptom: its header is pinned, site cards scroll below.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrillNavState {
    Browse {
        cursor: usize,
    },
    Drill {
        /// Index into `TriageApp::symptoms` — which symptom we're inside.
        symptom_idx: usize,
        view: SiteView,
    },
}

impl Default for DrillNavState {
    fn default() -> Self {
        DrillNavState::Browse { cursor: 0 }
    }
}

/// Input mode for handling different types of text input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    Navigation,
    /// Free-text "how to fix this" instruction for the symptom under focus.
    FixInstruction {
        symptom_id: SymptomId,
    },
}

impl InputMode {
    pub fn is_input_active(&self) -> bool {
        !matches!(self, InputMode::Navigation)
    }
}

/// Pure UI navigation and display state.
#[derive(Clone)]
pub struct UiState {
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub input_cursor: usize,
    pub should_quit: bool,
    pub leader: LeaderState,
    pub show_help: bool,

    /// DrillNav state machine. Private: accessed only through UiState
    /// methods (V4 encapsulation).
    drill_nav: DrillNavState,

    /// Total number of symptoms currently loaded, needed to bound Browse
    /// cursor movement without reaching into `TriageApp`.
    symptom_count: usize,

    /// One-shot status-bar message; cleared on the next keypress.
    status_message: Option<String>,
}

impl UiState {
    pub fn new(symptom_count: usize) -> Self {
        Self {
            input_mode: InputMode::Navigation,
            input_buffer: String::new(),
            input_cursor: 0,
            should_quit: false,
            leader: LeaderState::new(LEADER_TIMEOUT),
            show_help: false,
            drill_nav: DrillNavState::default(),
            symptom_count,
            status_message: None,
        }
    }

    /// The index of the symptom under focus, whether browsing (cursor) or
    /// drilled in (symptom_idx) — both cases resolve to one symptom. `None`
    /// only when the list is empty.
    pub fn current_symptom_index(&self) -> Option<usize> {
        if self.symptom_count == 0 {
            return None;
        }
        match self.drill_nav {
            DrillNavState::Browse { cursor } => Some(cursor),
            DrillNavState::Drill { symptom_idx, .. } => Some(symptom_idx),
        }
    }

    /// DrillNav mode name for snapshots: "Browse" or "Drill".
    pub fn nav_mode(&self) -> &'static str {
        match self.drill_nav {
            DrillNavState::Browse { .. } => "Browse",
            DrillNavState::Drill { .. } => "Drill",
        }
    }

    /// Cursor over symptom cards while browsing; None when drilled in.
    pub fn browse_cursor(&self) -> Option<usize> {
        match self.drill_nav {
            DrillNavState::Browse { cursor } => Some(cursor),
            DrillNavState::Drill { .. } => None,
        }
    }

    /// (symptom_idx, site_cursor) when drilled in; None while browsing.
    pub fn drill_position(&self) -> Option<(usize, usize)> {
        match &self.drill_nav {
            DrillNavState::Browse { .. } => None,
            DrillNavState::Drill { symptom_idx, view } => Some((*symptom_idx, view.cursor)),
        }
    }

    /// Drill viewport page offset for the focused symptom; None while
    /// browsing.
    pub fn drill_page_offset(&self) -> Option<usize> {
        match &self.drill_nav {
            DrillNavState::Browse { .. } => None,
            DrillNavState::Drill { view, .. } => Some(view.page_offset),
        }
    }

    /// One-shot status-bar message, if any.
    pub fn status_message(&self) -> Option<&str> {
        self.status_message.as_deref()
    }

    pub fn set_status_message(&mut self, message: String) {
        self.status_message = Some(message);
    }

    pub fn clear_status_message(&mut self) {
        self.status_message = None;
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn activate_leader(&mut self) {
        self.leader.activate();
    }

    pub fn enter_leader_submenu(&mut self, submenu: char) {
        self.leader.enter_submenu(submenu);
    }

    pub fn deactivate_leader(&mut self) {
        self.leader.deactivate();
    }

    pub fn is_leader_timed_out(&self) -> bool {
        self.leader.is_timed_out()
    }

    fn enter_input_mode(&mut self, mode: InputMode, initial_content: Option<String>) {
        self.input_mode = mode;
        self.input_buffer = initial_content.unwrap_or_default();
        self.input_cursor = self.input_buffer.len();
    }

    pub fn start_fix_instruction_input(&mut self, symptom_id: SymptomId) {
        self.enter_input_mode(InputMode::FixInstruction { symptom_id }, None);
    }

    pub fn exit_input_mode(&mut self) {
        self.enter_input_mode(InputMode::Navigation, None);
    }

    pub fn is_in_input_mode(&self) -> bool {
        self.input_mode.is_input_active()
    }

    pub fn input_char(&mut self, c: char) {
        if self.input_mode.is_input_active() {
            self.input_buffer.insert(self.input_cursor, c);
            self.input_cursor += 1;
        }
    }

    pub fn delete_char(&mut self) {
        if self.input_mode.is_input_active() && self.input_cursor > 0 {
            self.input_cursor -= 1;
            self.input_buffer.remove(self.input_cursor);
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.input_cursor > 0 {
            self.input_cursor -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.input_cursor < self.input_buffer.len() {
            self.input_cursor += 1;
        }
    }

    pub fn move_cursor_home(&mut self) {
        self.input_cursor = 0;
    }

    pub fn move_cursor_end(&mut self) {
        self.input_cursor = self.input_buffer.len();
    }

    // ── DrillNav state machine ────────────────────────────────────────────

    pub fn navigate_up(&mut self) {
        match &mut self.drill_nav {
            DrillNavState::Browse { cursor } => *cursor = cursor.saturating_sub(1),
            DrillNavState::Drill { view, .. } => {
                if view.cursor > 0 {
                    view.cursor -= 1;
                    view.page_offset = 0;
                }
            }
        }
    }

    /// `n_sites` is the site count of the currently-drilled symptom; the
    /// caller (event handler, which has `TriageApp` access) supplies it.
    pub fn navigate_down(&mut self, n_sites: usize) {
        match &mut self.drill_nav {
            DrillNavState::Browse { cursor } => {
                if *cursor + 1 < self.symptom_count {
                    *cursor += 1;
                }
            }
            DrillNavState::Drill { view, .. } => {
                if view.cursor + 1 < n_sites {
                    view.cursor += 1;
                    view.page_offset = 0;
                }
            }
        }
    }

    /// Enter the symptom under the browse cursor.
    pub fn drill_in(&mut self) {
        if let DrillNavState::Browse { cursor } = &self.drill_nav {
            self.drill_nav = DrillNavState::Drill {
                symptom_idx: *cursor,
                view: SiteView::default(),
            };
        }
    }

    /// Back out of a drilled symptom, restoring the browse cursor to it.
    pub fn back(&mut self) {
        if let DrillNavState::Drill { symptom_idx, .. } = &self.drill_nav {
            self.drill_nav = DrillNavState::Browse {
                cursor: *symptom_idx,
            };
        }
    }

    /// Return to Browse after a verdict is recorded on the drilled symptom
    /// (dismiss/fix both remove the symptom's relevance to this session).
    pub fn back_to_browse_after_verdict(&mut self) {
        if let DrillNavState::Drill { symptom_idx, .. } = &self.drill_nav {
            self.drill_nav = DrillNavState::Browse {
                cursor: *symptom_idx,
            };
        }
    }

    pub fn drill_page_up(&mut self) {
        if let DrillNavState::Drill { view, .. } = &mut self.drill_nav {
            view.page_offset = view.page_offset.saturating_sub(PAGE_SCROLL_STEP);
        }
    }

    pub fn drill_page_down(&mut self) {
        if let DrillNavState::Drill { view, .. } = &mut self.drill_nav {
            view.page_offset += PAGE_SCROLL_STEP;
        }
    }
}
