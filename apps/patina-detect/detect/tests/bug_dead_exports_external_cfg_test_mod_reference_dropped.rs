//! Bug: dead-exports silently drops a test-only symbol whose reference sits
//! in a file included via an external `#[cfg(test)] mod` declaration.
//!
//! Expected: `make_thing` — referenced ONLY from `src/external_tests.rs`, a
//! file compiled solely through `#[cfg(test)] #[path = "external_tests.rs"]
//! mod tests;` in `lib.rs` — is reported as a Test-only export.
//!
//! Actual: not reported at all. The test-context classifier
//! (`is_reference_test_only`) walks the *reference file's own AST* looking
//! for a preceding `#[test]`/`#[cfg(test)]` attribute; the gating attribute
//! lives on the `mod` declaration in `lib.rs`, and the referencing helper
//! `setup()` carries no test attribute itself, so the reference is judged
//! production and the candidate is dropped as "used". Real-world instance:
//! gitkit's `TestRepo::{new,path,commit_file}`, silently unreported in the
//! 2026-07 precision audit while `delete_file`/`get_commit_hash` in the
//! same file (called only from real `#[test]` fns) were flagged — a
//! same-file inconsistency.
//!
//! Run explicitly with:
//!
//! ```sh
//! cargo test -p patina-detect --test bug_dead_exports_external_cfg_test_mod_reference_dropped -- --ignored
//! ```

use patina_detect::detectors::dead_exports::run_dead_exports;
use patina_detect::entities::{Evidence, Symptom};
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/bug_dead_exports_external_test_mod_fixture")
}

/// Same indexing-warmup retry as `dead_exports_detection_tests`:
/// `dead_anchor` is the fixture's known-dead, always-detectable candidate.
fn run_with_retry(root: &std::path::Path) -> Vec<Symptom> {
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let outcome = run_dead_exports(root);
        let past_deadline = Instant::now() >= deadline;
        match outcome {
            Ok(symptoms) if has_export(&symptoms, "dead_anchor") => return symptoms,
            Ok(_) if past_deadline => {
                panic!(
                    "dead_anchor was never reported before the deadline (indexing never settled?)"
                )
            }
            Err(err) if past_deadline => {
                panic!("run_dead_exports never succeeded before the deadline: {err}")
            }
            _ => std::thread::sleep(Duration::from_millis(500)),
        }
    }
}

fn has_export(symptoms: &[Symptom], qualified_name_suffix: &str) -> bool {
    symptoms.iter().any(|s| {
        matches!(&s.evidence, Evidence::DeadExport { qualified_name, .. }
            if qualified_name.ends_with(qualified_name_suffix))
    })
}

#[test]
#[ignore = "FILED BUG, expected to fail until fixed (also requires rust-analyzer on PATH)"]
fn a_symbol_used_only_via_an_external_cfg_test_mod_is_reported_test_only() {
    let symptoms = run_with_retry(&fixture_root());

    assert!(
        has_export(&symptoms, "make_thing"),
        "make_thing's only reference is inside external_tests.rs, compiled \
         solely under the #[cfg(test)] mod declaration in lib.rs — it is \
         test-only and must be reported, not silently dropped as \
         production-used; found: {symptoms:#?}"
    );
}
