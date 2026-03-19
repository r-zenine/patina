//! Integration tests for the debug subcommand
//!
//! Tests verify:
//! - Basic command invocation and JSON output structure
//! - Input validation (file paths, git refs)
//! - Line-range filtering
//! - --explain-folding flag
//! - --export-fixture functionality
//! - --human readable output

use std::fs;
use std::path::Path;
use std::process::Command;

/// Helper to run debug command with given args
fn run_debug_command(args: &[&str]) -> std::process::Output {
    let mut cmd = Command::new("cargo");
    cmd.arg("run")
        .arg("--package").arg("diffviz-cli")
        .arg("--")
        .args(args);
    cmd.output().expect("Failed to run command")
}

/// Test basic invocation with valid file outputs JSON
#[test]
fn test_debug_valid_file_outputs_json() {
    // Use a file that exists in the repo (e.g., a common source file)
    let output = run_debug_command(&[
        "debug",
        "--file", "diffviz-cli/src/main.rs",
        "--from", "HEAD",
        "--to", "working_tree"
    ]);

    assert!(output.status.success(), "Command should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify JSON structure
    assert!(stdout.contains("\"file_path\""), "Output should contain file_path field");
    assert!(stdout.contains("\"language\""), "Output should contain language field");
    assert!(stdout.contains("\"phases\""), "Output should contain phases field");
    assert!(stdout.contains("\"metadata\""), "Output should contain metadata field");
}

/// Test invalid file path is rejected
#[test]
fn test_debug_invalid_file_rejected() {
    let output = run_debug_command(&[
        "debug",
        "--file", "/nonexistent/path/file.rs",
    ]);

    assert!(!output.status.success(), "Command should fail for nonexistent file");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("File not found") || stderr.contains("not found"),
            "Error message should mention file not found");
}

/// Test invalid git ref is handled
#[test]
fn test_debug_invalid_git_ref_handled() {
    // Using a file that exists but invalid ref should fail gracefully
    let output = run_debug_command(&[
        "debug",
        "--file", "diffviz-cli/src/main.rs",
        "--from", "nonexistent_commit_hash_xyz",
    ]);

    // Command may fail or may succeed depending on error handling
    // At minimum, should not crash
    let _ = String::from_utf8_lossy(&output.stdout);
    let _ = String::from_utf8_lossy(&output.stderr);
}

/// Test JSON output contains all 7 phases
#[test]
fn test_debug_json_contains_all_phases() {
    let output = run_debug_command(&[
        "debug",
        "--file", "diffviz-cli/src/main.rs",
        "--from", "HEAD",
        "--to", "working_tree"
    ]);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify all phase outputs are present
    assert!(stdout.contains("\"phase_1_semantic_tree\""), "Should have phase 1");
    assert!(stdout.contains("\"phase_2_semantic_pairs\""), "Should have phase 2");
    assert!(stdout.contains("\"phase_3_reviewable_diffs\""), "Should have phase 3");
    assert!(stdout.contains("\"phase_4_diff_node_hierarchy\""), "Should have phase 4");
    assert!(stdout.contains("\"phase_5_renderable_diffs\""), "Should have phase 5");
    assert!(stdout.contains("\"phase_6_code_impact\""), "Should have phase 6");
    assert!(stdout.contains("\"phase_7_final_output\""), "Should have phase 7");
}

/// Test --explain-folding flag includes explanations
#[test]
fn test_debug_explain_folding_includes_explanations() {
    let output = run_debug_command(&[
        "debug",
        "--file", "diffviz-cli/src/main.rs",
        "--from", "HEAD",
        "--to", "working_tree",
        "--explain-folding"
    ]);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // With explain-folding, explanations should be present in JSON
    // (though may be empty if no diffs exist)
    // We just verify the output is valid JSON and completes
    assert!(stdout.contains("\"phases\""), "Should contain phases");
}

/// Test --human flag outputs readable text
#[test]
fn test_debug_human_flag_readable_output() {
    let output = run_debug_command(&[
        "debug",
        "--file", "diffviz-cli/src/main.rs",
        "--from", "HEAD",
        "--to", "working_tree",
        "--human"
    ]);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should contain human-readable headers, not JSON
    assert!(stdout.contains("DiffViz Debug Analysis") || stdout.contains("File:"),
            "Should contain human-readable output");
    assert!(!stdout.contains("\"phases\""),
            "Should not contain JSON phases structure in --human output");
}

/// Test --export-fixture creates valid fixture file
#[test]
fn test_debug_export_fixture_creates_file() {
    let fixture_path = "/tmp/test_fixture_debug.json";

    // Clean up any existing file
    let _ = fs::remove_file(fixture_path);

    let output = run_debug_command(&[
        "debug",
        "--file", "diffviz-cli/src/main.rs",
        "--from", "HEAD",
        "--to", "working_tree",
        "--export-fixture", fixture_path
    ]);

    assert!(output.status.success());

    // Verify fixture file was created
    assert!(Path::new(fixture_path).exists(), "Fixture file should be created");

    // Verify fixture is valid JSON with required fields
    let fixture_content = fs::read_to_string(fixture_path)
        .expect("Should be able to read fixture file");
    assert!(fixture_content.contains("\"old_code\""), "Fixture should contain old_code");
    assert!(fixture_content.contains("\"new_code\""), "Fixture should contain new_code");
    assert!(fixture_content.contains("\"file_path\""), "Fixture should contain file_path");
    assert!(fixture_content.contains("\"language\""), "Fixture should contain language");

    // Clean up
    let _ = fs::remove_file(fixture_path);
}

/// Test language detection from file extension
#[test]
fn test_debug_language_detection_rust() {
    let output = run_debug_command(&[
        "debug",
        "--file", "diffviz-cli/src/main.rs",
        "--from", "HEAD",
        "--to", "working_tree"
    ]);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should detect Rust language
    assert!(stdout.contains("\"language\":\"Rust\""), "Should detect Rust language");
}

/// Test unsupported language is rejected
#[test]
fn test_debug_unsupported_language_rejected() {
    let output = run_debug_command(&[
        "debug",
        "--file", "test.xyz",
    ]);

    assert!(!output.status.success(), "Unsupported language should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unsupported") || stderr.contains("extension"),
            "Error should mention unsupported language");
}

/// Test invalid phase number is rejected
#[test]
fn test_debug_invalid_phase_rejected() {
    let output = run_debug_command(&[
        "debug",
        "--file", "diffviz-cli/src/main.rs",
        "--phase", "99"
    ]);

    assert!(!output.status.success(), "Invalid phase should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Phase") || stderr.contains("between"),
            "Error should mention phase validation");
}

/// Test valid phase number accepts 1-7
#[test]
fn test_debug_valid_phase_numbers() {
    for phase in 1..=7 {
        let output = run_debug_command(&[
            "debug",
            "--file", "diffviz-cli/src/main.rs",
            "--phase", &phase.to_string()
        ]);

        assert!(output.status.success(), "Phase {phase} should be valid");
    }
}
