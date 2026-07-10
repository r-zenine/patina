//! Integration test for Phase 15 — Detector 9: single-impl traits
//! (`.plans/plan-patina-detect/implementation-roadmap.md`, Phase 15;
//! `docs/patina-detect/spec.md:250-259`).
//!
//! Like `dead_exports`/`near_duplicate_structs`, this detector's whole
//! reason to exist is a real `implementations()` call through
//! `lspkit::LspClient` against a real `rust-analyzer` process — there is no
//! meaningful pure-unit-test slice of "does this trait have exactly one
//! production implementor" without one. The marker/sealed/test-declared
//! exclusions are covered by pure unit tests in the detector module itself
//! (Test Pyramid: many units, one comprehensive integration test here).
//! `#[ignore]`d since CI may not have `rust-analyzer` on `PATH`. Run
//! explicitly with:
//!
//! ```sh
//! cargo test -p patina-detect --test single_impl_traits_detection_tests -- --ignored
//! ```

use patina_detect::detectors::single_impl_traits::run_single_impl_traits;
use patina_detect::entities::{Evidence, Symptom};
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/single_impl_traits_fixture")
}

/// Same indexing-warmup concern as `dead_exports_detection_tests`: retry
/// the whole detector run until `Greeter` — the fixture's always-reportable
/// positive — shows up, since a transiently-empty `implementations()`
/// result before indexing settles must not be trusted.
fn run_with_retry(root: &std::path::Path) -> Vec<Symptom> {
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let outcome = run_single_impl_traits(root);
        let past_deadline = Instant::now() >= deadline;
        match outcome {
            Ok(symptoms) if has_trait(&symptoms, "Greeter") => return symptoms,
            Ok(_) if past_deadline => {
                panic!("Greeter was never reported before the deadline (indexing never settled?)")
            }
            Err(err) if past_deadline => {
                panic!("run_single_impl_traits never succeeded before the deadline: {err}")
            }
            _ => std::thread::sleep(Duration::from_millis(500)),
        }
    }
}

fn has_trait(symptoms: &[Symptom], name_suffix: &str) -> bool {
    symptoms.iter().any(|s| match &s.evidence {
        Evidence::SingleImplTrait { trait_name, .. } => trait_name.ends_with(name_suffix),
        _ => false,
    })
}

#[test]
#[ignore = "requires a real rust-analyzer process on PATH"]
fn a_genuinely_single_impl_trait_is_reported() {
    let symptoms = run_with_retry(&fixture_root());

    let greeter = symptoms
        .iter()
        .find(|s| has_trait(std::slice::from_ref(s), "Greeter"))
        .expect("Greeter should be reported as a single-impl trait");

    match &greeter.evidence {
        Evidence::SingleImplTrait {
            impl_count,
            test_doubles_present,
            ..
        } => {
            assert_eq!(*impl_count, 1);
            assert!(!test_doubles_present);
        }
        other => panic!("expected SingleImplTrait evidence, got {other:?}"),
    }
}

#[test]
#[ignore = "requires a real rust-analyzer process on PATH"]
fn a_trait_with_a_test_double_alongside_its_one_production_impl_is_excluded() {
    let symptoms = run_with_retry(&fixture_root());

    assert!(
        !has_trait(&symptoms, "Clock"),
        "Clock has a test-double impl (FakeClock) alongside SystemClock — the DI pattern, must not be reported: {symptoms:?}"
    );
}
