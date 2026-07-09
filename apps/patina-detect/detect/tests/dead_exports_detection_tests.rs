//! TDD test-design (red) for Phase 8 — Detector 3: dead exports and
//! write-only code (`.plans/plan-patina-detect/implementation-roadmap.md`,
//! Phase 8; `docs/patina-detect/spec.md:150-163`).
//!
//! Written against `patina_detect::detectors::dead_exports::run_dead_exports`,
//! whose body is currently `unimplemented!()` — this is this phase's "red"
//! state; the assertions below are what the implementation contribution
//! must satisfy to go green.
//!
//! Unlike the tree-sitter-only detectors (Type-2 clones, cognitive
//! complexity, data clumps), this detector's whole reason to exist is a
//! real `references()` call through `lspkit::LspClient` against a real
//! `rust-analyzer` process — there is no meaningful pure-unit-test slice of
//! "is this symbol dead" without one. Per the Test Pyramid this collapses
//! to a single, comprehensive integration test (few E2E) rather than many
//! small units; `#[ignore]`d since CI may not have `rust-analyzer` on
//! `PATH`. Run explicitly with:
//!
//! ```sh
//! cargo test -p patina-detect --test dead_exports_detection_tests -- --ignored
//! ```

use patina_detect::detectors::dead_exports::run_dead_exports;
use patina_detect::entities::{Evidence, Symptom};
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/dead_exports_fixture")
}

/// `rust-analyzer` answers `initialize` before it has finished indexing the
/// crate graph (see `libs/lspkit/tests/references_integration.rs`'s
/// identical concern), so a detector run immediately after spawn can
/// under-report references transiently. Retrying the whole detector run
/// (not a single `references()` call, since this detector issues many)
/// until `dead_helper` — the fixture's known-dead, always-detectable
/// candidate — shows up is the cheapest signal that indexing has settled;
/// once it does, every other assertion in the same run is trustworthy.
fn run_dead_exports_with_retry(root: &std::path::Path) -> Vec<Symptom> {
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let outcome = run_dead_exports(root);
        let past_deadline = Instant::now() >= deadline;
        match outcome {
            Ok(symptoms) if has_dead_export(&symptoms, "dead_helper") => return symptoms,
            Ok(_) if past_deadline => {
                panic!(
                    "dead_helper was never reported before the deadline (indexing never settled?)"
                )
            }
            Err(err) if past_deadline => {
                panic!("run_dead_exports never succeeded before the deadline: {err}")
            }
            _ => std::thread::sleep(Duration::from_millis(500)),
        }
    }
}

fn has_dead_export(symptoms: &[Symptom], qualified_name_suffix: &str) -> bool {
    dead_export_ending_with(symptoms, qualified_name_suffix).is_some()
}

fn dead_export_ending_with<'a>(
    symptoms: &'a [Symptom],
    qualified_name_suffix: &str,
) -> Option<&'a Symptom> {
    symptoms.iter().find(|s| {
        matches!(&s.evidence, Evidence::DeadExport { qualified_name, .. }
            if qualified_name.ends_with(qualified_name_suffix))
    })
}

fn any_qualified_name_contains(symptoms: &[Symptom], needle: &str) -> bool {
    symptoms.iter().any(|s| {
        matches!(&s.evidence, Evidence::DeadExport { qualified_name, .. }
            if qualified_name.contains(needle))
    })
}

#[test]
#[ignore = "requires rust-analyzer on PATH; run with `cargo test -- --ignored`"]
fn dead_exports_detector_finds_and_excludes_correctly() {
    let root = fixture_root();
    let symptoms = run_dead_exports_with_retry(&root);

    // A genuinely dead `pub fn` with zero references anywhere is reported,
    // with a zero reference count and not tagged test-only.
    let dead = dead_export_ending_with(&symptoms, "dead_helper")
        .expect("dead_helper (zero references) must be reported");
    match &dead.evidence {
        Evidence::DeadExport {
            reference_count,
            test_only,
            ..
        } => {
            assert_eq!(*reference_count, 0);
            assert!(!test_only, "dead_helper is not test-only, it's just dead");
        }
        other => panic!("expected Evidence::DeadExport, got {other:?}"),
    }

    // A `pub fn` referenced elsewhere in the crate must not be reported.
    assert!(
        !any_qualified_name_contains(&symptoms, "used_helper"),
        "used_helper is called from caller() and must not be reported, found: {symptoms:#?}"
    );

    // A trait-impl method has the same zero-direct-reference shape as
    // dead_helper but must be excluded — it's referenced through the trait,
    // not directly (spec.md:160's exclusion list).
    assert!(
        !any_qualified_name_contains(&symptoms, "greet"),
        "Thing::greet is a trait-impl method and must be excluded even with zero \
         direct references, found: {symptoms:#?}"
    );

    // A field on a struct carrying a derive is excluded regardless of
    // whether it's ever read directly (the derive uses it invisibly).
    assert!(
        !any_qualified_name_contains(&symptoms, "Config"),
        "Config::name sits on a #[derive(Debug)] struct and must be excluded, \
         found: {symptoms:#?}"
    );

    // A bin entry point (`main`) is excluded even though nothing in source
    // calls it (the Rust runtime does).
    assert!(
        !any_qualified_name_contains(&symptoms, "main"),
        "the bin entry point must be excluded, found: {symptoms:#?}"
    );

    // A symbol referenced only from test code is reported (not dropped)
    // but tagged test_only, with a non-zero reference count.
    let test_only = dead_export_ending_with(&symptoms, "test_only_helper")
        .expect("test_only_helper must still be reported (test-only usage is its own finding)");
    match &test_only.evidence {
        Evidence::DeadExport {
            reference_count,
            test_only,
            ..
        } => {
            assert!(*reference_count > 0, "the test module does reference it");
            assert!(
                test_only,
                "test_only_helper's only reference is from #[cfg(test)] code"
            );
        }
        other => panic!("expected Evidence::DeadExport, got {other:?}"),
    }
}

#[test]
#[ignore = "requires rust-analyzer on PATH; run with `cargo test -- --ignored`"]
fn rerunning_with_no_code_change_produces_identical_symptom_ids() {
    let root = fixture_root();
    let first = run_dead_exports_with_retry(&root);
    let second = run_dead_exports_with_retry(&root);

    let first_ids: Vec<_> = first.iter().map(|s| s.id.to_string()).collect();
    let second_ids: Vec<_> = second.iter().map(|s| s.id.to_string()).collect();
    assert_eq!(first_ids, second_ids);
}
