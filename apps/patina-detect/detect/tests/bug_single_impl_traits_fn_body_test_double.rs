//! Bug: single-impl-traits reports a DI trait whose test double is
//! declared inside a `#[test]` fn body.
//!
//! Expected: `saver_core::Saver` is excluded — it has one production impl
//! (`saver_cli::RealSaver`) plus a `MockSaver` impl in test code, the
//! Environment/DI pattern the detector's own spec (spec.md:255-257) says
//! must never be reported.
//!
//! Actual: `Saver` is reported as a single-impl trait. The mock's `impl`
//! is a block-local item inside the `#[test]` fn, and rust-analyzer's
//! `implementations()` does not surface impls declared inside function
//! bodies — so `partition_by_test_context` sees only the production impl
//! and the DI-exclusion cannot fire. A mod-level `#[cfg(test)]` mock (the
//! existing `single_impl_traits_fixture`'s `Clock`/`FakeClock`) IS
//! returned and excluded correctly; the fn-body-local mock is the
//! trigger. Real-world instance: sam-core's `SessionSaver`, whose
//! `MockSessionSaver` lives inside
//! `tests::normal_execution_does_not_save_to_session` — flagged in the
//! 2026-07 precision audit.
//!
//! Run explicitly with:
//!
//! ```sh
//! cargo test -p patina-detect --test bug_single_impl_traits_fn_body_test_double -- --ignored
//! ```

use patina_detect::detectors::single_impl_traits::run_single_impl_traits;
use patina_detect::entities::{Evidence, Symptom};
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/bug_single_impl_cross_crate_fixture")
}

/// Same indexing-warmup retry as `single_impl_traits_detection_tests`:
/// `Beacon` is the fixture's always-reportable positive; once it shows up,
/// the same run's other results are trustworthy.
fn run_with_retry(root: &std::path::Path) -> Vec<Symptom> {
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let outcome = run_single_impl_traits(root);
        let past_deadline = Instant::now() >= deadline;
        match outcome {
            Ok(symptoms) if has_trait(&symptoms, "Beacon") => return symptoms,
            Ok(_) if past_deadline => {
                panic!("Beacon was never reported before the deadline (indexing never settled?)")
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
#[ignore = "FILED BUG, expected to fail until fixed (also requires rust-analyzer on PATH)"]
fn a_di_trait_whose_mock_lives_inside_a_test_fn_is_excluded() {
    let symptoms = run_with_retry(&fixture_root());

    assert!(
        !has_trait(&symptoms, "Saver"),
        "Saver has a #[cfg(test)] MockSaver impl alongside its one \
         production impl (RealSaver in the sibling cli crate) — the DI \
         pattern, must not be reported; found: {symptoms:#?}"
    );
}
