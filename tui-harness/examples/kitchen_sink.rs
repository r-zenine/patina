//! Kitchen-sink example: a toy counter ELM app demonstrating all harness modes.
//!
//! Usage:
//!   cargo run --example kitchen_sink                    # run interactive TUI
//!   cargo run --example kitchen_sink -- --test-input "jjjkr"   # input harness
//!   cargo run --example kitchen_sink -- --test-render           # render harness
//!   cargo run --example kitchen_sink -- --test-full "jq"        # combined harness

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use serde::Serialize;

use tui_harness::{CombinedTestHarness, ELMApp, InputTestHarness, RenderTestHarness, run_app};

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

#[derive(Debug, Serialize)]
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
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if let Some(pos) = args.iter().position(|a| a == "--test-input") {
        let seq = args.get(pos + 1).expect("--test-input requires a sequence");
        let mut harness = InputTestHarness::new(CounterApp::new());
        let snapshots = harness.run_sequence(seq).expect("input harness failed");
        for snap in &snapshots {
            println!("{}", serde_json::to_string_pretty(snap).unwrap());
        }
    } else if args.iter().any(|a| a == "--test-render") {
        let harness = RenderTestHarness::new();
        let app = CounterApp::new();
        let output = harness.render(&app).expect("render harness failed");
        println!("{output}");
    } else if let Some(pos) = args.iter().position(|a| a == "--test-full") {
        let seq = args.get(pos + 1).expect("--test-full requires a sequence");
        let mut harness = CombinedTestHarness::new(CounterApp::new());
        let results = harness
            .run_sequence_with_renders(seq)
            .expect("combined harness failed");
        for (i, result) in results.iter().enumerate() {
            println!("--- step {i} ---");
            println!("{}", serde_json::to_string_pretty(&result.state).unwrap());
            println!("{}", result.visual);
        }
    } else {
        run_app(&mut CounterApp::new()).expect("TUI runtime failed");
    }
}
