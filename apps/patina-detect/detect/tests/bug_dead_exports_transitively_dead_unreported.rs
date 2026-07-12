//! Bug: dead-exports misses transitively dead exports.
//!
//! Expected: `used_only_by_dead` — whose only caller is `dead_entry`,
//! itself reported dead in the same run — is unreachable from any live
//! code and should be reported (both symbols go when the dead entry is
//! deleted).
//!
//! Actual: only `dead_entry` is reported. The analysis is one-hop: a
//! single reference from anywhere (including from other dead code) counts
//! as "used", so deleting `dead_entry` would immediately re-surface
//! `used_only_by_dead` as a new finding — the detector chases the frontier
//! one deletion at a time instead of reporting the whole dead subgraph.
//! Real-world instance: tui-design's `tokens.rs::surface2`, reachable only
//! from the separately-flagged-dead `stylesheet::selection` (2026-07
//! precision audit).
//!
//! Run explicitly with:
//!
//! ```sh
//! cargo test -p patina-detect --test bug_dead_exports_transitively_dead_unreported -- --ignored
//! ```

use patina_detect::detectors::dead_exports::run_dead_exports;
use patina_detect::entities::{Evidence, Symptom};
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/bug_dead_exports_transitive_fixture")
}

/// Same indexing-warmup retry as `dead_exports_detection_tests`:
/// `dead_entry` is the fixture's known-dead, always-detectable candidate —
/// and the root of the dead subgraph.
fn run_with_retry(root: &std::path::Path) -> Vec<Symptom> {
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let outcome = run_dead_exports(root);
        let past_deadline = Instant::now() >= deadline;
        match outcome {
            Ok(symptoms) if has_export(&symptoms, "dead_entry") => return symptoms,
            Ok(_) if past_deadline => {
                panic!(
                    "dead_entry was never reported before the deadline (indexing never settled?)"
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
fn an_export_reachable_only_from_dead_code_is_reported() {
    let symptoms = run_with_retry(&fixture_root());

    assert!(
        has_export(&symptoms, "used_only_by_dead"),
        "used_only_by_dead's only caller is dead_entry, itself dead — the \
         whole subgraph is unreachable and must be reported together; \
         found: {symptoms:#?}"
    );
}
