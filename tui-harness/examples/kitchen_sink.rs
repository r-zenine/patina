//! Kitchen-sink example: a toy counter ELM app demonstrating the agent CLI.
//!
//! Usage:
//!   cargo run --example kitchen_sink                             # interactive TUI
//!   cargo run --example kitchen_sink -- --describe               # app manifest
//!   cargo run --example kitchen_sink -- --test-input "jjjkr"     # final snapshot
//!   cargo run --example kitchen_sink -- --test-full "jq"         # state + visual per step

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use schemars::JsonSchema;
use serde::Serialize;

use tui_harness::{Affordance, AppDescription, ELMApp, KeyBindingDoc, ModeDoc, run_agent_cli};

// ---------------------------------------------------------------------------
// App state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
enum Mode {
    Normal,
    LabelEdit,
}

struct CounterApp {
    count: i32,
    mode: Mode,
    label: String,
    quit: bool,
}

impl CounterApp {
    fn new() -> Self {
        Self {
            count: 0,
            mode: Mode::Normal,
            label: String::from("counter"),
            quit: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Snapshot
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, JsonSchema)]
struct CounterSnapshot {
    count: i32,
    mode: String,
    label: String,
}

// ---------------------------------------------------------------------------
// ELMApp impl
// ---------------------------------------------------------------------------

impl ELMApp for CounterApp {
    type Snapshot = CounterSnapshot;
    type Error = std::convert::Infallible;

    fn dispatch_key(&mut self, key: KeyEvent) -> Result<(), Self::Error> {
        match self.mode {
            Mode::Normal => match key.code {
                KeyCode::Char('j') => self.count += 1,
                KeyCode::Char('k') => self.count -= 1,
                KeyCode::Char('r') => self.count = 0,
                KeyCode::Char('i') => self.mode = Mode::LabelEdit,
                KeyCode::Char('q') => self.quit = true,
                _ => {}
            },
            Mode::LabelEdit => match key.code {
                KeyCode::Esc => self.mode = Mode::Normal,
                KeyCode::Backspace => {
                    self.label.pop();
                }
                KeyCode::Char(c) => self.label.push(c),
                _ => {}
            },
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(area);

        let mode_str = match self.mode {
            Mode::Normal => "NORMAL",
            Mode::LabelEdit => "LABEL EDIT",
        };

        let title = format!(" {} — {} ", self.label, mode_str);
        let count_text = format!("{}", self.count);

        let count_para = Paragraph::new(Line::from(vec![Span::styled(
            count_text,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title(title));

        let help = Paragraph::new("j=+1  k=-1  r=reset  i=edit-label  q=quit")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Help "));

        frame.render_widget(count_para, chunks[0]);
        frame.render_widget(help, chunks[1]);
    }

    fn should_quit(&self) -> bool {
        self.quit
    }

    fn snapshot(&self) -> Self::Snapshot {
        CounterSnapshot {
            count: self.count,
            mode: match self.mode {
                Mode::Normal => "Normal".to_string(),
                Mode::LabelEdit => "LabelEdit".to_string(),
            },
            label: self.label.clone(),
        }
    }

    fn describe(&self) -> Option<AppDescription> {
        let binding = |mode: &str, keys: &[&str], event: &str, description: &str| KeyBindingDoc {
            mode: mode.to_string(),
            keys: keys.iter().map(|k| k.to_string()).collect(),
            event: event.to_string(),
            description: description.to_string(),
        };
        Some(AppDescription {
            app: "kitchen_sink".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            modes: vec![
                ModeDoc {
                    name: "Normal".to_string(),
                    description: "Counting mode".to_string(),
                },
                ModeDoc {
                    name: "LabelEdit".to_string(),
                    description: "Editing the counter label".to_string(),
                },
            ],
            bindings: vec![
                binding("Normal", &["j"], "Increment", "Increase the count by 1"),
                binding("Normal", &["k"], "Decrement", "Decrease the count by 1"),
                binding("Normal", &["r"], "Reset", "Reset the count to 0"),
                binding("Normal", &["i"], "EditLabel", "Enter label-edit mode"),
                binding("Normal", &["q"], "Quit", "Quit the app"),
                binding("LabelEdit", &["<Esc>"], "ExitEdit", "Back to normal mode"),
                binding(
                    "LabelEdit",
                    &["<Backspace>"],
                    "DeleteChar",
                    "Delete last label char",
                ),
            ],
        })
    }

    fn affordances(&self) -> Vec<Affordance> {
        let description = self.describe().expect("kitchen_sink is always described");
        let active_mode = match self.mode {
            Mode::Normal => "Normal",
            Mode::LabelEdit => "LabelEdit",
        };
        description
            .bindings
            .into_iter()
            .filter(|b| b.mode == active_mode)
            .map(|b| Affordance {
                keys: b.keys,
                event: b.event,
                description: b.description,
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    run_agent_cli(CounterApp::new(), std::env::args().skip(1)).expect("agent CLI failed");
}
