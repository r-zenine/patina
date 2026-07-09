//! TDD test-design (red) for Phase 11 — Detector 6: near-duplicate data
//! structures (`.plans/plan-patina-detect/implementation-roadmap.md`,
//! Phase 11; `docs/patina-detect/spec.md:194-210`).
//!
//! Written against
//! `patina_detect::detectors::near_duplicate_structs::run_near_duplicate_structs`,
//! whose body is currently `unimplemented!()` — this is this phase's "red"
//! state; the assertions below are what the implementation contribution
//! must satisfy to go green.
//!
//! Like `dead_exports` (Phase 8), this detector's conversion-evidence gate
//! is a real `references()` call through `lspkit::LspClient` against a real
//! `rust-analyzer` process — no meaningful pure-unit-test slice exists for
//! "do these two structs have real conversion code between them" without
//! one. Per the Test Pyramid this collapses to a single, comprehensive
//! integration test (few E2E) rather than many small units; `#[ignore]`d
//! since CI may not have `rust-analyzer` on `PATH`. Run explicitly with:
//!
//! ```sh
//! cargo test -p patina-detect --test near_duplicate_structs_detection_tests -- --ignored
//! ```

use patina_detect::detectors::near_duplicate_structs::run_near_duplicate_structs;
use patina_detect::entities::{Evidence, Symptom};
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/near_duplicate_structs_fixture")
}

/// `rust-analyzer` answers `initialize` before it has finished indexing the
/// crate graph (see `dead_exports_detection_tests.rs`'s identical concern):
/// retry the whole detector run until the known-positive pair
/// (`UserRequest`/`UserRecord`) shows up, then trust every other assertion
/// in that same run.
fn run_with_retry(root: &std::path::Path) -> Vec<Symptom> {
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let outcome = run_near_duplicate_structs(root);
        let past_deadline = Instant::now() >= deadline;
        match outcome {
            Ok(symptoms) if has_pair(&symptoms, "UserRequest", "UserRecord") => return symptoms,
            Ok(_) if past_deadline => {
                panic!(
                    "UserRequest/UserRecord was never reported before the deadline \
                     (indexing never settled?)"
                )
            }
            Err(err) if past_deadline => {
                panic!("run_near_duplicate_structs never succeeded before the deadline: {err}")
            }
            _ => std::thread::sleep(Duration::from_millis(500)),
        }
    }
}

fn pair_named<'a>(symptoms: &'a [Symptom], a: &str, b: &str) -> Option<&'a Symptom> {
    symptoms.iter().find(|s| {
        matches!(&s.evidence, Evidence::NearDuplicateStructs { struct_a, struct_b, .. }
            if (struct_a.ends_with(a) && struct_b.ends_with(b))
                || (struct_a.ends_with(b) && struct_b.ends_with(a)))
    })
}

fn has_pair(symptoms: &[Symptom], a: &str, b: &str) -> bool {
    pair_named(symptoms, a, b).is_some()
}

#[test]
#[ignore = "requires rust-analyzer on PATH; run with `cargo test -- --ignored`"]
fn near_duplicate_structs_detector_finds_and_excludes_correctly() {
    let root = fixture_root();
    let symptoms = run_with_retry(&root);

    // Case 1: >= 4 shared fields (Jaccard 0.8) AND a real `impl From` —
    // both gates pass, must be reported with evidence populated.
    let reported = pair_named(&symptoms, "UserRequest", "UserRecord")
        .expect("UserRequest/UserRecord must be reported (overlap + conversion evidence)");
    match &reported.evidence {
        Evidence::NearDuplicateStructs {
            shared_field_count,
            total_field_count,
            overlap_percent,
            conversion_sites,
            footprint_file_count,
            ..
        } => {
            assert_eq!(*shared_field_count, 4);
            assert_eq!(*total_field_count, 5);
            assert!(
                *overlap_percent >= 70,
                "expected >= 70% overlap, got {overlap_percent}"
            );
            assert!(
                !conversion_sites.is_empty(),
                "the impl From<UserRequest> for UserRecord must surface as a conversion site"
            );
            assert!(*footprint_file_count >= 1);
        }
        other => panic!("expected Evidence::NearDuplicateStructs, got {other:?}"),
    }

    // Case 2: same overlap shape, zero conversion code — the
    // conversion-evidence gate must exclude it.
    assert!(
        !has_pair(&symptoms, "ProfileDraft", "ProfileSnapshot"),
        "ProfileDraft/ProfileSnapshot has no conversion code and must not be reported, \
         found: {symptoms:#?}"
    );

    // Case 3: a real `impl From` exists, but only 2 shared fields — below
    // the >= 4 shared fields minimum, the overlap gate must exclude it
    // regardless of the conversion evidence.
    assert!(
        !has_pair(&symptoms, "Small", "Other"),
        "Small/Other has only 2 shared fields (below the >= 4 minimum) and must not be \
         reported even though a conversion impl exists, found: {symptoms:#?}"
    );
}

#[test]
#[ignore = "requires rust-analyzer on PATH; run with `cargo test -- --ignored`"]
fn rerunning_with_no_code_change_produces_identical_symptom_ids() {
    let root = fixture_root();
    let first = run_with_retry(&root);
    let second = run_with_retry(&root);

    let first_ids: Vec<_> = first.iter().map(|s| s.id.to_string()).collect();
    let second_ids: Vec<_> = second.iter().map(|s| s.id.to_string()).collect();
    assert_eq!(first_ids, second_ids);
}
