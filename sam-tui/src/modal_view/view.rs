use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;
use tui_harness::ELMApp;

use tui_harness::TuiError;

use super::state::{
    Event as AppEvent, ExecutionState, OptionToggle, Value, ViewResponse, ViewState,
};
use super::ui::UIModal;

/// Map a crossterm KeyEvent to a sam domain Event.
///
/// j/k provide vim-style navigation in InsertMode (Down/Up).
/// Ctrl bindings and arrow keys follow the original termion mapping.
pub(super) fn key_transformer(
    key: KeyEvent,
    has_options: bool,
    allow_multi_select: bool,
) -> Option<AppEvent> {
    match key.code {
        KeyCode::Backspace | KeyCode::Delete => Some(AppEvent::Backspace),
        KeyCode::Esc if has_options => Some(AppEvent::ToggleViewMode),
        KeyCode::Up => Some(AppEvent::Up),
        KeyCode::Down => Some(AppEvent::Down),
        KeyCode::Enter => Some(AppEvent::Entr),
        KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(AppEvent::Up),
        KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(AppEvent::Down),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(AppEvent::AppClosed)
        }
        KeyCode::Char('s')
            if key.modifiers.contains(KeyModifiers::CONTROL) && allow_multi_select =>
        {
            Some(AppEvent::Mark)
        }
        KeyCode::Char('a')
            if key.modifiers.contains(KeyModifiers::CONTROL) && allow_multi_select =>
        {
            Some(AppEvent::MarkAll)
        }
        KeyCode::Char('j') if key.modifiers == KeyModifiers::NONE => Some(AppEvent::Down),
        KeyCode::Char('k') if key.modifiers == KeyModifiers::NONE => Some(AppEvent::Up),
        KeyCode::Char(c) if key.modifiers == KeyModifiers::NONE => Some(AppEvent::InputChar(c)),
        _ => None,
    }
}

pub struct ModalView<V: Value> {
    pub(super) state: ViewState<V>,
    ui: UIModal<V>,
    pub(super) has_options: bool,
    pub(super) allow_multi_select: bool,
    should_quit: bool,
    cancelled: bool,
}

impl<V: Value> ModalView<V> {
    pub fn new(list: Vec<V>, options: Vec<OptionToggle>, allow_multi_select: bool) -> Self {
        let has_options = !options.is_empty();
        let state = ViewState::new(list, options);
        let ui = UIModal::new();
        ModalView {
            state,
            ui,
            has_options,
            allow_multi_select,
            should_quit: false,
            cancelled: false,
        }
    }

    pub fn run(mut self) -> Option<ViewResponse<V>> {
        let _ = tui_harness::run_app(&mut self);
        if self.cancelled {
            None
        } else {
            Some(self.state.response())
        }
    }
}

impl<V: Value> ELMApp for ModalView<V> {
    type Snapshot = ();
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
        self.ui.draw(frame, &self.state);
    }

    fn should_quit(&self) -> bool {
        self.should_quit
    }

    fn snapshot(&self) {}
}
