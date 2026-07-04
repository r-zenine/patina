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
fn describe_bindings_stay_empty_until_the_registry_exists() {
    // Phase 3 generates bindings from the registry; hand-writing them here
    // would recreate the drift this plan eliminates.
    let manifest = stdout_json(&["--describe"]);
    assert_eq!(manifest["bindings"].as_array().unwrap().len(), 0);
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
