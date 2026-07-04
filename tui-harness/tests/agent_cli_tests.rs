//! Phase 1 contract tests for the generic agent CLI (plan-tui-harness-agent-discovery).
//!
//! The discovery surface (`--describe`) and the hoisted test modes
//! (`--test-input`, `--test-full`) are exercised in-process against fixture
//! apps: one that provides a full `AppDescription`, one that relies on the
//! trait defaults.

use crossterm::event::{KeyCode, KeyEvent};
use schemars::JsonSchema;
use serde::Serialize;

use tui_harness::{
    Affordance, AgentMode, AppDescription, ELMApp, InputTestHarness, KeyBindingDoc,
    MANIFEST_VERSION, ModeDoc, describe_output, full_test_output, input_test_output,
    parse_agent_args,
};

// ---------------------------------------------------------------------------
// Fixture: a described counter app
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, JsonSchema)]
struct FixtureSnapshot {
    count: i32,
    editing: bool,
}

struct FixtureApp {
    count: i32,
    editing: bool,
    quit: bool,
}

impl FixtureApp {
    fn new() -> Self {
        Self {
            count: 0,
            editing: false,
            quit: false,
        }
    }
}

impl ELMApp for FixtureApp {
    type Snapshot = FixtureSnapshot;
    type Error = std::convert::Infallible;

    fn dispatch_key(&mut self, key: KeyEvent) -> Result<(), Self::Error> {
        match key.code {
            KeyCode::Char('j') => self.count += 1,
            KeyCode::Char('i') => self.editing = true,
            KeyCode::Esc => self.editing = false,
            KeyCode::Char('q') => self.quit = true,
            _ => {}
        }
        Ok(())
    }

    fn draw(&self, frame: &mut ratatui::Frame) {
        let text = format!("count: {}", self.count);
        frame.render_widget(ratatui::widgets::Paragraph::new(text), frame.area());
    }

    fn should_quit(&self) -> bool {
        self.quit
    }

    fn snapshot(&self) -> FixtureSnapshot {
        FixtureSnapshot {
            count: self.count,
            editing: self.editing,
        }
    }

    fn describe(&self) -> Option<AppDescription> {
        Some(AppDescription {
            app: "fixture".to_string(),
            version: "0.0.1".to_string(),
            modes: vec![ModeDoc {
                name: "Normal".to_string(),
                description: "Counting mode".to_string(),
            }],
            bindings: vec![KeyBindingDoc {
                mode: "Normal".to_string(),
                keys: vec!["j".to_string()],
                event: "Increment".to_string(),
                description: "Increment the counter".to_string(),
            }],
        })
    }

    fn affordances(&self) -> Vec<Affordance> {
        if self.editing {
            vec![Affordance {
                keys: vec!["<Esc>".to_string()],
                event: "ExitEditing".to_string(),
                description: "Leave editing".to_string(),
            }]
        } else {
            vec![Affordance {
                keys: vec!["j".to_string()],
                event: "Increment".to_string(),
                description: "Increment the counter".to_string(),
            }]
        }
    }
}

// ---------------------------------------------------------------------------
// Fixture: an app using only trait defaults
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, JsonSchema)]
struct BareSnapshot {
    ticks: u32,
}

struct BareApp;

impl ELMApp for BareApp {
    type Snapshot = BareSnapshot;
    type Error = std::convert::Infallible;

    fn dispatch_key(&mut self, _key: KeyEvent) -> Result<(), Self::Error> {
        Ok(())
    }
    fn draw(&self, _frame: &mut ratatui::Frame) {}
    fn should_quit(&self) -> bool {
        false
    }
    fn snapshot(&self) -> BareSnapshot {
        BareSnapshot { ticks: 0 }
    }
}

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

// ---------------------------------------------------------------------------
// Argument parsing
// ---------------------------------------------------------------------------

#[test]
fn no_args_means_run() {
    assert!(matches!(
        parse_agent_args(args(&[])).unwrap(),
        AgentMode::Run
    ));
}

#[test]
fn describe_flag_parses() {
    assert!(matches!(
        parse_agent_args(args(&["--describe"])).unwrap(),
        AgentMode::Describe
    ));
}

#[test]
fn test_input_flag_takes_sequence() {
    match parse_agent_args(args(&["--test-input", "jjk"])).unwrap() {
        AgentMode::TestInput(seq) => assert_eq!(seq, "jjk"),
        other => panic!("expected TestInput, got {other:?}"),
    }
}

#[test]
fn test_full_flag_takes_sequence() {
    match parse_agent_args(args(&["--test-full", "<Space>a"])).unwrap() {
        AgentMode::TestFull(seq) => assert_eq!(seq, "<Space>a"),
        other => panic!("expected TestFull, got {other:?}"),
    }
}

#[test]
fn unknown_flag_is_an_error() {
    assert!(parse_agent_args(args(&["--bogus"])).is_err());
}

#[test]
fn missing_sequence_is_an_error() {
    assert!(parse_agent_args(args(&["--test-input"])).is_err());
    assert!(parse_agent_args(args(&["--test-full"])).is_err());
}

// ---------------------------------------------------------------------------
// --describe
// ---------------------------------------------------------------------------

#[test]
fn describe_emits_versioned_manifest_with_schema() {
    let output = describe_output(&FixtureApp::new()).unwrap();
    let manifest: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(manifest["manifest_version"], MANIFEST_VERSION);
    assert_eq!(manifest["described"], true);
    assert_eq!(manifest["app"], "fixture");
    assert_eq!(manifest["version"], "0.0.1");
    assert_eq!(manifest["bindings"][0]["keys"][0], "j");
    assert_eq!(manifest["modes"][0]["name"], "Normal");

    // The snapshot schema is generated from the real Snapshot type.
    let properties = &manifest["snapshot_schema"]["properties"];
    assert!(
        properties.get("count").is_some() && properties.get("editing").is_some(),
        "snapshot_schema must describe FixtureSnapshot fields, got: {properties}"
    );
}

#[test]
fn describe_documents_the_input_notation() {
    let output = describe_output(&FixtureApp::new()).unwrap();
    let manifest: serde_json::Value = serde_json::from_str(&output).unwrap();

    let notation = &manifest["notation"];
    let special = notation["special_keys"].as_array().unwrap();
    let names: Vec<&str> = special.iter().filter_map(|v| v.as_str()).collect();
    for expected in ["Space", "Enter", "Esc", "Up", "F12"] {
        assert!(names.contains(&expected), "missing special key {expected}");
    }
    assert!(notation["wait"].as_str().unwrap().contains("Wait"));
    assert!(!notation["modifiers"].as_array().unwrap().is_empty());
}

#[test]
fn default_describe_is_explicitly_marked_undescribed() {
    let output = describe_output(&BareApp).unwrap();
    let manifest: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(manifest["manifest_version"], MANIFEST_VERSION);
    assert_eq!(manifest["described"], false);
    assert_eq!(manifest["bindings"].as_array().unwrap().len(), 0);
    // Schema is still real even without a description.
    assert!(
        manifest["snapshot_schema"]["properties"]
            .get("ticks")
            .is_some()
    );
}

// ---------------------------------------------------------------------------
// --test-input / --test-full output contracts
// ---------------------------------------------------------------------------

#[test]
fn input_test_output_is_bare_pretty_final_snapshot() {
    let output = input_test_output(FixtureApp::new(), "jjj").unwrap();

    let mut reference = InputTestHarness::new(FixtureApp::new());
    let expected =
        serde_json::to_string_pretty(&reference.run_sequence_final_state("jjj").unwrap()).unwrap();

    assert_eq!(output, expected, "--test-input output format drifted");
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["count"], 3);
}

#[test]
fn full_test_output_uses_step_sections() {
    let output = full_test_output(FixtureApp::new(), "jj").unwrap();

    // Initial state + 2 steps.
    for header in ["=== Step 0 ===", "=== Step 1 ===", "=== Step 2 ==="] {
        assert!(output.contains(header), "missing {header} in:\n{output}");
    }
    assert!(!output.contains("=== Step 3 ==="));
    assert!(output.contains("State:"));
    assert!(output.contains("Visual:"));
    assert!(
        output.contains("count: 2"),
        "final visual should render count"
    );
}

// ---------------------------------------------------------------------------
// Affordances
// ---------------------------------------------------------------------------

#[test]
fn default_affordances_are_empty() {
    assert!(BareApp.affordances().is_empty());
}

#[test]
fn affordances_follow_app_state() {
    let mut app = FixtureApp::new();
    assert_eq!(app.affordances()[0].keys, vec!["j".to_string()]);

    app.dispatch_key(KeyEvent::new(
        KeyCode::Char('i'),
        crossterm::event::KeyModifiers::NONE,
    ))
    .unwrap();

    assert_eq!(app.affordances()[0].keys, vec!["<Esc>".to_string()]);
}
