//! Generic agent-facing CLI for any ELMApp.
//!
//! `run_agent_cli` gives every app the same discovery and headless-test
//! surface, so agents (and scripts) never depend on per-app `main.rs`
//! plumbing:
//!
//! - `--describe`            print the app manifest (see [`crate::manifest`])
//! - `--test-input <seq>`    run a key sequence, print the final snapshot
//! - `--test-full <seq>`     run a key sequence, print state + visual per step
//! - `--agent-repl`          persistent NDJSON session (see [`crate::repl`])
//! - no flags                run the real TUI via [`crate::run_app`]

use std::fmt::Write as _;

use crate::{
    CombinedTestHarness, ELMApp, InputTestHarness, Result, TuiError, manifest::build_manifest,
    run_app,
};

/// Parsed agent CLI mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentMode {
    /// Print the app manifest.
    Describe,
    /// Run the sequence headlessly, print the final snapshot JSON.
    TestInput(String),
    /// Run the sequence headlessly, print state + visual at each step.
    TestFull(String),
    /// Persistent NDJSON session on stdin/stdout.
    Repl,
    /// Run the interactive TUI.
    Run,
}

/// Parse agent CLI arguments (excluding the program name).
///
/// Unknown flags are errors so agents get a loud signal instead of a
/// silently-ignored option.
pub fn parse_agent_args<I: IntoIterator<Item = String>>(args: I) -> Result<AgentMode> {
    let args: Vec<String> = args.into_iter().collect();

    match args.first().map(String::as_str) {
        None => Ok(AgentMode::Run),
        Some("--describe") => reject_extra(&args, 1, AgentMode::Describe),
        Some("--agent-repl") => reject_extra(&args, 1, AgentMode::Repl),
        Some("--test-input") => {
            let seq = sequence_arg(&args, "--test-input")?;
            reject_extra(&args, 2, AgentMode::TestInput(seq))
        }
        Some("--test-full") => {
            let seq = sequence_arg(&args, "--test-full")?;
            reject_extra(&args, 2, AgentMode::TestFull(seq))
        }
        Some(other) => Err(TuiError::Usage(format!(
            "unknown argument: {other} (expected --describe, --agent-repl, --test-input <seq>, or --test-full <seq>)"
        ))),
    }
}

fn sequence_arg(args: &[String], flag: &str) -> Result<String> {
    args.get(1)
        .cloned()
        .ok_or_else(|| TuiError::Usage(format!("{flag} requires an input sequence argument")))
}

fn reject_extra(args: &[String], expected_len: usize, mode: AgentMode) -> Result<AgentMode> {
    if args.len() > expected_len {
        return Err(TuiError::Usage(format!(
            "unexpected extra arguments: {:?}",
            &args[expected_len..]
        )));
    }
    Ok(mode)
}

/// Render the `--describe` output: the pretty-printed app manifest.
pub fn describe_output<M: ELMApp>(app: &M) -> Result<String> {
    let manifest = build_manifest(app);
    serde_json::to_string_pretty(&manifest).map_err(|e| TuiError::App(Box::new(e)))
}

/// Render the `--test-input` output: the bare pretty-printed final snapshot.
pub fn input_test_output<M: ELMApp>(app: M, sequence: &str) -> Result<String> {
    let mut harness = InputTestHarness::new(app);
    let snapshot = harness.run_sequence_final_state(sequence)?;
    serde_json::to_string_pretty(&snapshot).map_err(|e| TuiError::App(Box::new(e)))
}

/// Render the `--test-full` output: per-step state snapshot + visual.
pub fn full_test_output<M: ELMApp>(app: M, sequence: &str) -> Result<String> {
    let mut harness = CombinedTestHarness::new(app);
    let results = harness.run_sequence_with_renders(sequence)?;

    let mut output = String::new();
    for (i, result) in results.iter().enumerate() {
        let state_json =
            serde_json::to_string_pretty(&result.state).map_err(|e| TuiError::App(Box::new(e)))?;
        let affordances_json = serde_json::to_string_pretty(&result.affordances)
            .map_err(|e| TuiError::App(Box::new(e)))?;
        writeln!(output, "=== Step {i} ===").expect("writing to String cannot fail");
        writeln!(output, "State:").expect("writing to String cannot fail");
        writeln!(output, "{state_json}").expect("writing to String cannot fail");
        writeln!(output, "\nVisual:").expect("writing to String cannot fail");
        writeln!(output, "{}", result.visual).expect("writing to String cannot fail");
        writeln!(output, "\nAffordances:").expect("writing to String cannot fail");
        writeln!(output, "{affordances_json}").expect("writing to String cannot fail");
        writeln!(output).expect("writing to String cannot fail");
    }
    Ok(output)
}

/// Run the agent CLI: dispatch on parsed args, print to stdout, or run the
/// interactive TUI when no test flag is given.
pub fn run_agent_cli<M: ELMApp>(mut app: M, args: impl IntoIterator<Item = String>) -> Result<()> {
    match parse_agent_args(args)? {
        AgentMode::Describe => {
            println!("{}", describe_output(&app)?);
            Ok(())
        }
        AgentMode::TestInput(seq) => {
            println!("{}", input_test_output(app, &seq)?);
            Ok(())
        }
        AgentMode::TestFull(seq) => {
            print!("{}", full_test_output(app, &seq)?);
            Ok(())
        }
        AgentMode::Repl => crate::repl::run_repl(app),
        AgentMode::Run => run_app(&mut app),
    }
}
