use crossterm::event::KeyEvent;
use ratatui::Frame;
use serde::Serialize;
use tui_harness::ELMApp;

use tui_harness::TuiError;
use tui_harness::{Affordance, AppDescription, KeyBindingDoc, ModeDoc};

use super::state::{ExecutionState, OptionToggle, Value, ViewMode, ViewState};
use super::theme::UITheme;
use super::ui_insert_mode::{ListItems, UIInsertMode};
use super::ui_options_mode::UIOptionsMode;
use super::view::key_transformer;

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct SamSnapshot {
    pub current_mode: String,
    pub cursor: usize,
    pub filter_query: String,
    pub item_count: usize,
    pub marked_count: usize,
}

/// One hand-written discovery row (sam is registry-free by design — see
/// plan-tui-harness-agent-discovery D006). This single list feeds BOTH
/// `describe()` and `affordances()` so sam's two discovery surfaces cannot
/// disagree with each other; it must be kept in sync with `key_transformer`.
struct SamBinding {
    keys: &'static [&'static str],
    event: &'static str,
    description: &'static str,
    requires_options: bool,
    requires_multi_select: bool,
}

const SAM_BINDINGS: &[SamBinding] = &[
    SamBinding {
        keys: &["j", "<Down>", "<C-n>"],
        event: "Down",
        description: "Move the selection down",
        requires_options: false,
        requires_multi_select: false,
    },
    SamBinding {
        keys: &["k", "<Up>", "<C-p>"],
        event: "Up",
        description: "Move the selection up",
        requires_options: false,
        requires_multi_select: false,
    },
    SamBinding {
        keys: &["<Enter>"],
        event: "Entr",
        description: "Confirm the highlighted selection",
        requires_options: false,
        requires_multi_select: false,
    },
    SamBinding {
        keys: &["<Backspace>", "<Delete>"],
        event: "Backspace",
        description: "Delete the last fuzzy-filter character",
        requires_options: false,
        requires_multi_select: false,
    },
    SamBinding {
        keys: &["<C-c>"],
        event: "AppClosed",
        description: "Cancel and close the modal",
        requires_options: false,
        requires_multi_select: false,
    },
    SamBinding {
        keys: &["<Esc>"],
        event: "ToggleViewMode",
        description: "Switch between the list and the options view",
        requires_options: true,
        requires_multi_select: false,
    },
    SamBinding {
        keys: &["<C-s>"],
        event: "Mark",
        description: "Mark/unmark the highlighted item",
        requires_options: false,
        requires_multi_select: true,
    },
    SamBinding {
        keys: &["<C-a>"],
        event: "MarkAll",
        description: "Mark all displayed items",
        requires_options: false,
        requires_multi_select: true,
    },
];

/// Fuzzy typing is a catch-all, not a finite row (`j`/`k` stay navigation).
const SAM_INPUT_CATCH_ALL: (&str, &str, &str) = (
    "<any other character>",
    "InputChar",
    "Append the character to the fuzzy filter",
);

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

    /// Rows active under this view's construction config.
    fn active_bindings(&self) -> impl Iterator<Item = &'static SamBinding> + '_ {
        SAM_BINDINGS.iter().filter(move |b| {
            (!b.requires_options || self.has_options)
                && (!b.requires_multi_select || self.allow_multi_select)
        })
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

    fn describe(&self) -> Option<AppDescription> {
        let mut bindings: Vec<KeyBindingDoc> = self
            .active_bindings()
            .map(|b| KeyBindingDoc {
                // key_transformer does not branch on ViewMode; every
                // binding applies in both modes.
                mode: "*".to_owned(),
                keys: b.keys.iter().map(|k| k.to_string()).collect(),
                event: b.event.to_owned(),
                description: b.description.to_owned(),
            })
            .collect();
        let (keys_label, event, description) = SAM_INPUT_CATCH_ALL;
        bindings.push(KeyBindingDoc {
            mode: "*".to_owned(),
            keys: vec![keys_label.to_owned()],
            event: event.to_owned(),
            description: description.to_owned(),
        });

        Some(AppDescription {
            app: "sam-tui-modal".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            modes: vec![
                ModeDoc {
                    name: "InsertMode".to_owned(),
                    description: "Fuzzy-filter and select from the list".to_owned(),
                },
                ModeDoc {
                    name: "OptionsMode".to_owned(),
                    description: "Toggle option flags (reached with Esc when options exist)"
                        .to_owned(),
                },
            ],
            bindings,
        })
    }

    fn affordances(&self) -> Vec<Affordance> {
        let mut affordances: Vec<Affordance> = self
            .active_bindings()
            .map(|b| Affordance {
                keys: b.keys.iter().map(|k| k.to_string()).collect(),
                event: b.event.to_owned(),
                description: b.description.to_owned(),
            })
            .collect();
        let (keys_label, event, description) = SAM_INPUT_CATCH_ALL;
        affordances.push(Affordance {
            keys: vec![keys_label.to_owned()],
            event: event.to_owned(),
            description: description.to_owned(),
        });
        affordances
    }
}
