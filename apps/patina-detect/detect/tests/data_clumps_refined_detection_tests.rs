//! Integration test for Phase 16 — Detector 8 revision: call-hierarchy-based
//! forwarding-gate refinement (`.plans/plan-patina-detect/implementation-roadmap.md`,
//! Phase 16; decision D011).
//!
//! Like `middleman_delegation`/`single_impl_traits`, the closed-cluster
//! check's whole reason to exist is a real `incoming_calls` call through
//! `lspkit::LspClient` against a real `rust-analyzer` process — there is no
//! meaningful pure-unit-test slice of "is this clump's call graph closed"
//! without one. The tree-sitter-only promotion gates (member/occurrence/
//! forwarding thresholds) are already covered by
//! `data_clumps_detection_tests.rs` against the unrefined `run_data_clumps`.
//! `#[ignore]`d since CI may not have `rust-analyzer` on `PATH`. Run
//! explicitly with:
//!
//! ```sh
//! cargo test -p patina-detect --test data_clumps_refined_detection_tests -- --ignored
//! ```

use patina_detect::detectors::data_clumps::run_data_clumps_refined;
use patina_detect::entities::{Evidence, Symptom};
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/data_clumps_refined_fixture")
}

/// Same rust-analyzer indexing race as the other lspkit-backed detectors'
/// integration tests: retry the whole detector run until the known
/// always-kept clump (`contact_a`/`contact_b`/`contact_c`, called from two
/// distinct places) shows up, since a transiently-incomplete index can
/// under- or over-report before it settles.
fn run_with_retry(root: &std::path::Path) -> Vec<Symptom> {
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let outcome = run_data_clumps_refined(root);
        let past_deadline = Instant::now() >= deadline;
        match outcome {
            Ok(symptoms) if has_clump(&symptoms, "id") => return symptoms,
            Ok(_) if past_deadline => {
                panic!("the contact_a/b/c clump was never reported before the deadline")
            }
            Err(err) if past_deadline => {
                panic!("run_data_clumps_refined never succeeded before the deadline: {err}")
            }
            _ => std::thread::sleep(Duration::from_millis(500)),
        }
    }
}

fn has_clump(symptoms: &[Symptom], member_name: &str) -> bool {
    symptoms.iter().any(|s| match &s.evidence {
        Evidence::DataClump { members, .. } => members.iter().any(|(name, _)| name == member_name),
        _ => false,
    })
}

#[test]
#[ignore = "requires a real rust-analyzer process on PATH"]
fn a_closed_recursive_helper_family_with_a_single_entry_point_is_excluded() {
    let symptoms = run_with_retry(&fixture_root());

    assert!(
        !has_clump(&symptoms, "value"),
        "visit_node/visit_branch/visit_leaf form a closed family reached only \
         through run_visitor's single entry-point call — must not be \
         reported, found: {symptoms:?}"
    );
}

#[test]
#[ignore = "requires a real rust-analyzer process on PATH"]
fn a_clump_reached_from_two_distinct_call_sites_sharing_a_file_is_still_reported() {
    let symptoms = run_with_retry(&fixture_root());

    let clump = symptoms
        .iter()
        .find(|s| has_clump(std::slice::from_ref(s), "id"))
        .expect(
            "contact_a/contact_b/contact_c is reached from caller_one and \
             caller_two, two genuinely distinct call sites that happen to \
             share a file — must still be reported",
        );

    match &clump.evidence {
        Evidence::DataClump {
            occurrence_count, ..
        } => assert_eq!(*occurrence_count, 3),
        other => panic!("expected DataClump evidence, got {other:?}"),
    }
}
