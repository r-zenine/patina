//! Phase 4 contract tests for the agent REPL (plan-tui-harness-agent-discovery).
//!
//! The protocol (006 design doc): NDJSON request/response over stdio, one
//! compact response line per command, sessions survive every error class.
//! Tested in-process by running `run_repl_io` over byte buffers.

use crossterm::event::{KeyCode, KeyEvent};
use schemars::JsonSchema;
use serde::Serialize;
use thiserror::Error;

use tui_harness::{Affordance, ELMApp, run_repl_io};

// ---------------------------------------------------------------------------
// Fixture: counter that ticks, errors on 'x', quits on 'q'
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
#[error("boom on x")]
struct BoomError;

#[derive(Debug, Serialize, JsonSchema)]
struct ReplSnapshot {
    count: i32,
    ticks: u32,
    should_quit: bool,
}

struct ReplApp {
    count: i32,
    ticks: u32,
    quit: bool,
}

impl ReplApp {
    fn new() -> Self {
        Self {
            count: 0,
            ticks: 0,
            quit: false,
        }
    }
}

impl ELMApp for ReplApp {
    type Snapshot = ReplSnapshot;
    type Error = BoomError;

    fn dispatch_key(&mut self, key: KeyEvent) -> Result<(), BoomError> {
        match key.code {
            KeyCode::Char('j') => self.count += 1,
            KeyCode::Char('x') => return Err(BoomError),
            KeyCode::Char('q') => self.quit = true,
            _ => {}
        }
        Ok(())
    }

    fn draw(&self, frame: &mut ratatui::Frame) {
        frame.render_widget(
            ratatui::widgets::Paragraph::new(format!("count: {}", self.count)),
            frame.area(),
        );
    }

    fn should_quit(&self) -> bool {
        self.quit
    }

    fn snapshot(&self) -> ReplSnapshot {
        ReplSnapshot {
            count: self.count,
            ticks: self.ticks,
            should_quit: self.quit,
        }
    }

    fn on_tick(&mut self) {
        self.ticks += 1;
    }

    fn affordances(&self) -> Vec<Affordance> {
        vec![Affordance {
            keys: vec!["j".to_string()],
            event: "Increment".to_string(),
            description: "Increment".to_string(),
        }]
    }
}

/// Run a scripted session; return one parsed JSON value per response line.
fn session(script: &str) -> Vec<serde_json::Value> {
    let mut out: Vec<u8> = Vec::new();
    run_repl_io(ReplApp::new(), script.as_bytes(), &mut out).expect("repl failed");
    let text = String::from_utf8(out).expect("responses are UTF-8");
    text.lines()
        .map(|line| {
            serde_json::from_str(line)
                .unwrap_or_else(|e| panic!("response is not one JSON line ({e}): {line}"))
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Happy paths
// ---------------------------------------------------------------------------

#[test]
fn keys_returns_state_visual_and_affordances() {
    let responses = session(r#"{"cmd":"keys","input":"jjj"}"#);

    assert_eq!(responses.len(), 1);
    let r = &responses[0];
    assert_eq!(r["v"], 1);
    assert_eq!(r["ok"], true);
    assert_eq!(r["state"]["count"], 3);
    assert!(r["visual"].as_str().unwrap().contains("count: 3"));
    assert_eq!(r["affordances"][0]["keys"][0], "j");
}

#[test]
fn state_persists_across_commands() {
    let responses = session(concat!(
        r#"{"cmd":"keys","input":"jj"}"#,
        "\n",
        r#"{"cmd":"keys","input":"j"}"#,
        "\n",
        r#"{"cmd":"render"}"#,
    ));

    assert_eq!(responses[0]["state"]["count"], 2);
    assert_eq!(responses[1]["state"]["count"], 3, "no prefix replay needed");
    assert_eq!(responses[2]["state"]["count"], 3);
    assert!(
        responses[2]["visual"]
            .as_str()
            .unwrap()
            .contains("count: 3")
    );
}

#[test]
fn describe_returns_the_manifest() {
    let responses = session(r#"{"cmd":"describe"}"#);
    let r = &responses[0];
    assert_eq!(r["ok"], true);
    assert_eq!(r["manifest"]["manifest_version"], 1);
    assert!(
        r["manifest"]["snapshot_schema"]["properties"]
            .get("count")
            .is_some()
    );
}

#[test]
fn wait_steps_run_on_tick_inside_the_session() {
    let responses = session(r#"{"cmd":"keys","input":"j<Wait:20>j"}"#);
    assert_eq!(responses[0]["state"]["count"], 2);
    assert_eq!(responses[0]["state"]["ticks"], 1);
}

#[test]
fn quit_acknowledges_and_ends_the_session() {
    let responses = session(concat!(
        r#"{"cmd":"quit"}"#,
        "\n",
        r#"{"cmd":"keys","input":"j"}"#,
    ));

    assert_eq!(responses.len(), 1, "nothing is processed after quit");
    assert_eq!(responses[0]["ok"], true);
}

#[test]
fn eof_ends_the_session_cleanly() {
    let responses = session("");
    assert!(responses.is_empty());
}

#[test]
fn should_quit_state_does_not_end_the_session() {
    let responses = session(concat!(
        r#"{"cmd":"keys","input":"q"}"#,
        "\n",
        r#"{"cmd":"render"}"#,
    ));

    assert_eq!(responses[0]["state"]["should_quit"], true);
    assert_eq!(responses[1]["ok"], true, "session outlives should_quit");
}

// ---------------------------------------------------------------------------
// Error classes — the session always survives
// ---------------------------------------------------------------------------

#[test]
fn malformed_json_line_is_an_error_and_session_survives() {
    let responses = session(concat!(
        "this is not json\n",
        r#"{"cmd":"keys","input":"j"}"#
    ));

    assert_eq!(responses[0]["ok"], false);
    assert!(!responses[0]["error"].as_str().unwrap().is_empty());
    assert_eq!(responses[1]["ok"], true);
    assert_eq!(responses[1]["state"]["count"], 1);
}

#[test]
fn unknown_command_is_an_error_and_session_survives() {
    let responses = session(concat!(r#"{"cmd":"bogus"}"#, "\n", r#"{"cmd":"render"}"#,));

    assert_eq!(responses[0]["ok"], false);
    assert_eq!(responses[1]["ok"], true);
}

#[test]
fn bad_key_sequence_reports_zero_applied() {
    let responses = session(r#"{"cmd":"keys","input":"<Bogus>"}"#);
    assert_eq!(responses[0]["ok"], false);
    assert_eq!(responses[0]["applied"], 0);
}

#[test]
fn app_error_mid_sequence_reports_applied_count_and_session_survives() {
    let responses = session(concat!(
        r#"{"cmd":"keys","input":"jjxj"}"#,
        "\n",
        r#"{"cmd":"render"}"#,
    ));

    let error = &responses[0];
    assert_eq!(error["ok"], false);
    assert_eq!(error["applied"], 2, "two j's landed before the boom");
    assert!(error["error"].as_str().unwrap().contains("boom"));

    // Partial state is observable afterwards.
    assert_eq!(responses[1]["state"]["count"], 2);
}

#[test]
fn empty_lines_are_ignored() {
    let responses = session(concat!("\n\n", r#"{"cmd":"render"}"#, "\n\n"));
    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0]["ok"], true);
}
