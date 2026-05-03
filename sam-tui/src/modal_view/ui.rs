use std::marker::PhantomData;

use ratatui::Frame;

use super::state::Value;
use super::state::ViewMode;
use super::state::ViewState;
use super::theme::UITheme;
use super::ui_insert_mode::{ListItems, UIInsertMode};
use super::ui_options_mode::UIOptionsMode;

pub struct UIModal<V: Value> {
    pub(super) theme: UITheme,
    _marker: PhantomData<V>,
}

impl<V: Value> UIModal<V> {
    pub fn new() -> Self {
        UIModal {
            theme: UITheme::default(),
            _marker: PhantomData,
        }
    }

    pub(super) fn draw(&self, frame: &mut Frame, state: &ViewState<V>) {
        match state.current_mod {
            ViewMode::OptionsMode => {
                let options_mode_view = UIOptionsMode::new(&self.theme);
                options_mode_view.draw(frame, &state.options)
            }
            ViewMode::InsertMode => {
                let preview = state.preview().unwrap_or_default();
                let insert_mode_view = UIInsertMode::new(frame.area(), &self.theme);
                insert_mode_view.draw(
                    frame,
                    ListItems::from(state),
                    state.search_filter(),
                    &preview,
                )
            }
        }
    }
}
