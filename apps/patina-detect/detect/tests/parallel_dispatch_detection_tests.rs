//! TDD test-design (red) for Phase 13 — Detector 7: parallel dispatch
//! (`.plans/plan-patina-detect/implementation-roadmap.md`, Phase 13;
//! `docs/patina-detect/spec.md:212-224`).
//!
//! Like `dead_exports`, this detector's whole reason to exist is real
//! `hover()`/`definition()` calls through `lspkit::LspClient` against a real
//! `rust-analyzer` process, so this collapses to a single, comprehensive
//! integration test rather than many small units (those live alongside the
//! tree-sitter-only parts in `detector.rs`'s own `#[cfg(test)]` module).
//! `#[ignore]`d since CI may not have `rust-analyzer` on `PATH`. Run
//! explicitly with:
//!
//! ```sh
//! cargo test -p patina-detect --test parallel_dispatch_detection_tests -- --ignored
//! ```

use patina_detect::detectors::parallel_dispatch::run_parallel_dispatch;
use patina_detect::entities::{Evidence, Symptom};
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/parallel_dispatch_fixture")
}

/// Same indexing-warmup concern as `dead_exports_detection_tests`'s
/// `run_dead_exports_with_retry` — retry the whole detector run until the
/// fixture's known-positive (`Shape`) shows up, since a run right after
/// spawn can under-report while `rust-analyzer` is still indexing.
fn run_parallel_dispatch_with_retry(root: &std::path::Path) -> Vec<Symptom> {
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let outcome = run_parallel_dispatch(root);
        let past_deadline = Instant::now() >= deadline;
        match outcome {
            Ok(symptoms) if has_enum(&symptoms, "Shape") => return symptoms,
            Ok(_) if past_deadline => {
                panic!("Shape was never reported before the deadline (indexing never settled?)")
            }
            Err(err) if past_deadline => {
                panic!("run_parallel_dispatch never succeeded before the deadline: {err}")
            }
            _ => std::thread::sleep(Duration::from_millis(500)),
        }
    }
}

fn has_enum(symptoms: &[Symptom], enum_name_suffix: &str) -> bool {
    enum_symptom_ending_with(symptoms, enum_name_suffix).is_some()
}

fn enum_symptom_ending_with<'a>(
    symptoms: &'a [Symptom],
    enum_name_suffix: &str,
) -> Option<&'a Symptom> {
    symptoms.iter().find(|s| {
        matches!(&s.evidence, Evidence::ParallelDispatch { enum_name, .. }
            if enum_name.ends_with(enum_name_suffix))
    })
}

#[test]
#[ignore = "requires rust-analyzer on PATH; run with `cargo test -- --ignored`"]
fn parallel_dispatch_detector_finds_and_excludes_correctly() {
    let root = fixture_root();
    let symptoms = run_parallel_dispatch_with_retry(&root);

    // `Shape` is matched in 3 sites across 2 files (describe, area_hint in
    // lib.rs; icon_for in other.rs) — must be reported.
    let shape = enum_symptom_ending_with(&symptoms, "Shape")
        .expect("Shape (3 sites, 2 files) must be reported");
    match &shape.evidence {
        Evidence::ParallelDispatch {
            site_count,
            file_count,
            arm_counts,
            ..
        } => {
            assert_eq!(*site_count, 3);
            assert_eq!(*file_count, 2);
            assert_eq!(arm_counts.len(), 3);
            assert!(arm_counts.iter().all(|&n| n == 3));
        }
        other => panic!("expected Evidence::ParallelDispatch, got {other:?}"),
    }
    assert_eq!(
        shape.sites.len(),
        3,
        "one Site per match site, found: {:#?}",
        shape.sites
    );

    // `Mode` is matched twice, both in the same file (lib.rs) — under both
    // thresholds, must not be reported.
    assert!(
        !has_enum(&symptoms, "Mode"),
        "Mode is only matched twice in one file, must not be reported, found: {symptoms:#?}"
    );

    // `Option` is a std type matched repeatedly — must never be grouped,
    // regardless of how many sites/files it appears across.
    assert!(
        symptoms.iter().all(|s| !matches!(
            &s.evidence,
            Evidence::ParallelDispatch { enum_name, .. } if enum_name.contains("Option")
        )),
        "Option must be excluded by the std-type denylist, found: {symptoms:#?}"
    );
}

#[test]
#[ignore = "requires rust-analyzer on PATH; run with `cargo test -- --ignored`"]
fn rerunning_with_no_code_change_produces_identical_symptom_ids() {
    let root = fixture_root();
    let first = run_parallel_dispatch_with_retry(&root);
    let second = run_parallel_dispatch_with_retry(&root);

    let first_ids: Vec<_> = first.iter().map(|s| s.id.to_string()).collect();
    let second_ids: Vec<_> = second.iter().map(|s| s.id.to_string()).collect();
    assert_eq!(first_ids, second_ids);
}
