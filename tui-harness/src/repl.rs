//! Agent REPL: a persistent NDJSON session over stdio.
//!
//! Protocol (plan-tui-harness-agent-discovery, contribution 006):
//! one JSON request per stdin line, exactly one compact JSON response line
//! per request. Observations are uniform — successful `keys`/`render`
//! responses carry `state` + `visual` + `affordances` so an agent acts and
//! sees its legal moves in one round trip. Errors never end the session
//! (accumulated app state is the asset the REPL exists to protect); only
//! `quit` or EOF do.

use std::io::{BufRead, Write};

use serde::Deserialize;
use serde_json::json;

use crate::{
    Result, TuiError, input_parser::parse_input_sequence, manifest::build_manifest,
    render_test::RenderTestHarness, traits::ELMApp,
};

/// Protocol version carried as `"v"` in every response.
pub const REPL_PROTOCOL_VERSION: u32 = 1;

#[derive(Debug, Deserialize)]
#[serde(tag = "cmd", rename_all = "lowercase", deny_unknown_fields)]
enum ReplCommand {
    Keys { input: String },
    Render,
    Describe,
    Quit,
}

/// Run the REPL over stdin/stdout.
pub fn run_repl<M: ELMApp>(app: M) -> Result<()> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    run_repl_io(app, stdin.lock(), stdout.lock())
}

/// Run the REPL over arbitrary streams (in-process testable).
pub fn run_repl_io<M: ELMApp>(
    mut app: M,
    input: impl BufRead,
    mut output: impl Write,
) -> Result<()> {
    let render_harness = RenderTestHarness::new();

    for line in input.lines() {
        let line = line.map_err(TuiError::Terminal)?;
        if line.trim().is_empty() {
            continue;
        }

        let (response, quit) = handle_line(&mut app, &render_harness, &line);
        writeln!(output, "{response}").map_err(TuiError::Terminal)?;
        if quit {
            break;
        }
    }

    Ok(())
}

/// Process one request line; returns (response, end_session).
fn handle_line<M: ELMApp>(
    app: &mut M,
    render_harness: &RenderTestHarness,
    line: &str,
) -> (serde_json::Value, bool) {
    let command: ReplCommand = match serde_json::from_str(line) {
        Ok(command) => command,
        Err(e) => return (error_response(format!("invalid request: {e}"), None), false),
    };

    match command {
        ReplCommand::Quit => (json!({"v": REPL_PROTOCOL_VERSION, "ok": true}), true),
        ReplCommand::Describe => (
            json!({
                "v": REPL_PROTOCOL_VERSION,
                "ok": true,
                "manifest": build_manifest(app),
            }),
            false,
        ),
        ReplCommand::Render => (observation(app, render_harness), false),
        ReplCommand::Keys { input } => {
            let steps = match parse_input_sequence(&input) {
                Ok(steps) => steps,
                Err(e) => return (error_response(e.to_string(), Some(0)), false),
            };

            for (applied, step) in steps.into_iter().enumerate() {
                if let Err(e) = step.apply(app) {
                    return (error_response(e.to_string(), Some(applied)), false);
                }
            }

            (observation(app, render_harness), false)
        }
    }
}

/// The uniform observation envelope: state + visual + affordances.
fn observation<M: ELMApp>(app: &M, render_harness: &RenderTestHarness) -> serde_json::Value {
    let visual = match render_harness.render(app) {
        Ok(visual) => visual,
        Err(e) => return error_response(format!("render failed: {e}"), None),
    };
    let state = match serde_json::to_value(app.snapshot()) {
        Ok(state) => state,
        Err(e) => return error_response(format!("snapshot failed: {e}"), None),
    };

    json!({
        "v": REPL_PROTOCOL_VERSION,
        "ok": true,
        "state": state,
        "visual": visual,
        "affordances": app.affordances(),
    })
}

fn error_response(message: String, applied: Option<usize>) -> serde_json::Value {
    let mut response = json!({
        "v": REPL_PROTOCOL_VERSION,
        "ok": false,
        "error": message,
    });
    if let Some(applied) = applied {
        response["applied"] = json!(applied);
    }
    response
}
