use crossterm::event::KeyEvent;
use ratatui::Frame;
use serde::Serialize;
use tui_harness::ELMApp;

use tui_harness::TuiError;

use super::state::{ExecutionState, OptionToggle, Value, ViewMode, ViewState};
use super::theme::UITheme;
use super::ui_insert_mode::{ListItems, UIInsertMode};
use super::ui_options_mode::UIOptionsMode;
use super::view::key_transformer;

#[derive(Debug, Serialize)]
pub struct SamSnapshot {
    pub current_mode: String,
    pub cursor: usize,
    pub filter_query: String,
    pub item_count: usize,
    pub marked_count: usize,
}

pub struct HeadlessModalView<V: Value> {
    pub(super) state: ViewState<V>,
    has_options: bool,
    allow_multi_select: bool,
    should_quit: bool,
    cancelled: bool,
    theme: UITheme,
}

impl<V: Value> HeadlessModalView<V> {
    pub fn new(list: Vec<V>, options: Vec<OptionToggle>, allow_multi_select: bool) -> Self {
        let has_options = !options.is_empty();
        let state = ViewState::new(list, options);
        HeadlessModalView {
            state,
            has_options,
            allow_multi_select,
            should_quit: false,
            cancelled: false,
            theme: UITheme::default(),
        }
    }

    pub fn cancelled(&self) -> bool {
        self.cancelled
    }
}

impl<V: Value> ELMApp for HeadlessModalView<V> {
    type Snapshot = SamSnapshot;
    type Error = TuiError;

    fn dispatch_key(&mut self, key: KeyEvent) -> std::result::Result<(), Self::Error> {
        if let Some(event) = key_transformer(key, self.has_options, self.allow_multi_select) {
            let status = self.state.update(&event);
            match status {
                ExecutionState::Keep => {}
                ExecutionState::ExitSuccess => self.should_quit = true,
                ExecutionState::Cancelled => {
                    self.should_quit = true;
                    self.cancelled = true;
                }
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        match self.state.current_mod {
            ViewMode::OptionsMode => {
                let options_mode_view = UIOptionsMode::new(&self.theme);
                options_mode_view.draw(frame, &self.state.options)
            }
            ViewMode::InsertMode => {
                let preview = self.state.preview().unwrap_or_default();
                let insert_mode_view = UIInsertMode::new(frame.area(), &self.theme);
                insert_mode_view.draw(
                    frame,
                    ListItems::from(&self.state),
                    self.state.search_filter(),
                    &preview,
                )
            }
        }
    }

    fn should_quit(&self) -> bool {
        self.should_quit
    }

    fn snapshot(&self) -> SamSnapshot {
        let cursor = self.state.list.highlighted_line.unwrap_or(0);
        let filter_query = self.state.list.search_filter().to_owned();
        let item_count = self.state.list.current_displayed_values.len();
        let marked_count = self.state.list.marked_values().len();
        let current_mode = match self.state.current_mod {
            ViewMode::InsertMode => "InsertMode".to_owned(),
            ViewMode::OptionsMode => "OptionsMode".to_owned(),
        };
        SamSnapshot {
            current_mode,
            cursor,
            filter_query,
            item_count,
            marked_count,
        }
    }
}
