//! Phase 2 contract tests: the review-tui binary serves the generic agent CLI.
//!
//! These drive the real binary (via `CARGO_BIN_EXE_review-tui`) so they pin
//! exactly what an agent sees on the command line — discovery manifest,
//! headless test modes, and loud failures on unknown flags.

use std::process::Command;

fn review_tui(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_review-tui"))
        .args(args)
        .output()
        .expect("failed to spawn review-tui")
}

fn stdout_json(args: &[&str]) -> serde_json::Value {
    let output = review_tui(args);
    assert!(
        output.status.success(),
        "review-tui {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "review-tui {args:?} stdout is not JSON ({e}):\n{}",
            String::from_utf8_lossy(&output.stdout)
        )
    })
}

// All 17 StateSnapshot fields — the schema must describe every one.
const SNAPSHOT_FIELDS: &[&str] = &[
    "input_mode",
    "input_buffer",
    "input_cursor",
    "should_quit",
    "leader_active",
    "leader_submenu",
    "show_help",
    "show_reasoning",
    "nav_mode",
    "browse_cursor",
    "drill_decision",
    "drill_file",
    "drill_chunk",
    "drill_context_expanded",
    "drill_note_expanded",
    "drill_page_offset",
    "status_message",
];

#[test]
fn describe_serves_a_manifest_with_the_real_snapshot_schema() {
    let manifest = stdout_json(&["--describe"]);

    assert_eq!(manifest["manifest_version"], 1);
    assert_eq!(manifest["described"], true);
    assert_eq!(manifest["app"], "diffviz-review-tui");
    assert!(!manifest["version"].as_str().unwrap().is_empty());

    let properties = manifest["snapshot_schema"]["properties"]
        .as_object()
        .expect("snapshot_schema must have properties");
    for field in SNAPSHOT_FIELDS {
        assert!(properties.contains_key(*field), "schema missing {field}");
    }
    // Doc comments on StateSnapshot flow into schema descriptions.
    assert!(
        properties["input_mode"]["description"]
            .as_str()
            .unwrap()
            .contains("Navigation"),
        "field doc comments should surface as schema descriptions"
    );
}

#[test]
fn describe_lists_the_input_modes() {
    let manifest = stdout_json(&["--describe"]);
    let names: Vec<&str> = manifest["modes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["name"].as_str().unwrap())
        .collect();
    for expected in ["Navigation", "Instruction", "DecisionInstruction"] {
        assert!(names.contains(&expected), "missing mode {expected}");
    }
}

#[test]
fn describe_bindings_come_from_the_registry() {
    // Flipped intentionally in Phase 3: bindings are generated from the
    // keybinding registry (events/bindings.rs), never hand-written.
    let manifest = stdout_json(&["--describe"]);
    let bindings = manifest["bindings"].as_array().unwrap();

    assert!(
        bindings.len() >= 30,
        "expected the full registry (+ catch-alls), got {}",
        bindings.len()
    );

    // Spot-check one row per scope, including an alias pair and a catch-all.
    let find = |mode: &str, event: &str| {
        bindings
            .iter()
            .find(|b| b["mode"] == mode && b["event"] == event)
            .unwrap_or_else(|| panic!("no binding for mode={mode} event={event}"))
    };
    assert_eq!(
        find("Navigation", "NavigateDown")["keys"],
        serde_json::json!(["j", "<Down>"])
    );
    assert_eq!(
        find("Leader", "EnterLeaderSubmenu('a')")["keys"],
        serde_json::json!(["a"])
    );
    assert_eq!(
        find("Leader:a", "ApproveFile")["keys"],
        serde_json::json!(["f"])
    );
    assert_eq!(
        find("Input", "InputChar")["keys"],
        serde_json::json!(["<any character>"])
    );

    for binding in bindings {
        assert!(
            !binding["description"].as_str().unwrap().is_empty(),
            "undocumented binding: {binding}"
        );
    }
}

#[test]
fn test_input_prints_the_bare_final_snapshot() {
    let snapshot = stdout_json(&["--test-input", "j"]);

    // Same shape as before the migration (pre-migration golden: a bare
    // StateSnapshot with browse cursor moved down once).
    assert_eq!(snapshot["nav_mode"], "Browse");
    assert_eq!(snapshot["browse_cursor"], 1);
    assert_eq!(snapshot["should_quit"], false);
    let object = snapshot.as_object().unwrap();
    for field in SNAPSHOT_FIELDS {
        assert!(object.contains_key(*field), "snapshot missing {field}");
    }
}

#[test]
fn test_full_prints_step_sections_with_state_and_visual() {
    let output = review_tui(&["--test-full", "j"]);
    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);

    for header in ["=== Step 0 ===", "=== Step 1 ==="] {
        assert!(text.contains(header), "missing {header}");
    }
    assert!(!text.contains("=== Step 2 ==="));
    assert!(text.contains("State:"));
    assert!(text.contains("Visual:"));
}

fn last_step_of(output: &str) -> &str {
    output
        .split("=== Step ")
        .last()
        .expect("output has step sections")
}

#[test]
fn affordances_in_navigation_list_movement_not_submenu_actions() {
    let output = review_tui(&["--test-full", ""]);
    let text = String::from_utf8_lossy(&output.stdout);
    let step = last_step_of(&text);

    assert!(step.contains("Affordances:"));
    assert!(step.contains("\"event\": \"NavigateDown\""));
    assert!(
        !step.contains("\"event\": \"ApproveFile\""),
        "submenu-only action leaked into navigation affordances"
    );
}

#[test]
fn affordances_follow_the_leader_submenu() {
    let output = review_tui(&["--test-full", "<Space>a"]);
    let text = String::from_utf8_lossy(&output.stdout);
    let step = last_step_of(&text);

    assert!(step.contains("\"event\": \"ApproveFile\""));
    assert!(step.contains("\"event\": \"ToggleApprove\""));
    assert!(
        step.contains("<any other key>"),
        "leader catch-all must be advertised"
    );
    assert!(
        !step.contains("\"event\": \"NavigateDown\""),
        "navigation bindings leaked into leader affordances"
    );
}

#[test]
fn affordances_in_input_mode_advertise_text_entry() {
    let output = review_tui(&["--test-full", "<Enter>n"]);
    let text = String::from_utf8_lossy(&output.stdout);
    let step = last_step_of(&text);

    assert!(step.contains("\"event\": \"SubmitInput\""));
    assert!(step.contains("<any character>"));
    assert!(!step.contains("\"event\": \"NavigateDown\""));
}

#[test]
fn which_key_overlay_renders_from_the_registry() {
    let output = review_tui(&["--test-full", "<Space>"]);
    let text = String::from_utf8_lossy(&output.stdout);
    let step = last_step_of(&text);

    assert!(
        step.contains("Actions"),
        "which-key must show submenu hints"
    );
    assert!(
        step.contains("Toggles"),
        "which-key must show submenu hints"
    );
}

#[test]
fn unknown_flags_fail_loudly_instead_of_launching_the_tui() {
    // Pre-migration, a typo fell through to the interactive TUI — fatal for
    // a headless agent session. Now it must exit non-zero with a message.
    let output = review_tui(&["--decribe"]);
    assert!(
        !output.status.success(),
        "typo'd flag must not fall through to the interactive TUI"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--decribe"),
        "error should name the offending flag, got: {stderr}"
    );
}

#[test]
fn test_input_without_sequence_fails_loudly() {
    let output = review_tui(&["--test-input"]);
    assert!(!output.status.success());
}

#[test]
fn agent_repl_session_persists_state_across_commands() {
    // The Phase 4 gate: drill in with one command, navigate with the next,
    // observe with a third — cursor state must persist with no prefix replay.
    use std::io::Write;

    let mut child = Command::new(env!("CARGO_BIN_EXE_review-tui"))
        .arg("--agent-repl")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn review-tui --agent-repl");

    child
        .stdin
        .take()
        .expect("stdin piped")
        .write_all(
            concat!(
                r#"{"cmd":"keys","input":"<Enter>"}"#, // drill into decision 1
                "\n",
                r#"{"cmd":"keys","input":"j"}"#, // next chunk — builds on prior command
                "\n",
                r#"{"cmd":"render"}"#,
                "\n",
                r#"{"cmd":"quit"}"#,
                "\n",
            )
            .as_bytes(),
        )
        .expect("write script");

    let output = child.wait_with_output().expect("repl run");
    assert!(output.status.success());

    let responses: Vec<serde_json::Value> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|l| serde_json::from_str(l).expect("one JSON response per line"))
        .collect();

    assert_eq!(responses.len(), 4);
    assert_eq!(responses[0]["v"], 1);
    assert_eq!(responses[0]["state"]["nav_mode"], "Drill");
    assert_eq!(responses[0]["state"]["drill_chunk"], 0);
    assert_eq!(
        responses[1]["state"]["drill_chunk"], 1,
        "the j built on the previous command's drill state — no prefix replay"
    );
    assert_eq!(responses[2]["state"]["drill_chunk"], 1);
    assert!(
        responses[2]["affordances"]
            .as_array()
            .unwrap()
            .iter()
            .any(|a| a["event"] == "NavigateDown"),
        "observation carries legal moves"
    );
    assert_eq!(responses[3]["ok"], true);
}
