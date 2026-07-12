//! Bug: dead-exports labels a shared integration-test helper "Dead" when it
//! is used by sibling test crates.
//!
//! Expected: `helpers_common::drive` — defined in
//! `tests/helpers_common/mod.rs` (a module declared by BOTH sibling test
//! crates via `mod helpers_common;`) and called from a `#[test]` fn in
//! `second_tests.rs` — is reported at worst as a Test-only export
//! (`test_only: true`, `reference_count > 0`).
//!
//! Actual: reported as a Dead export (`reference_count: 0`,
//! `test_only: false`). rust-analyzer attributes a multi-included file to
//! exactly one canonical owner crate; each other including crate gets its
//! own distinct copy of the module, so a call from a non-owner crate is a
//! reference to a *different* symbol. When the owner crate happens not to
//! call a given helper (here: `first_tests` uses only `park`),
//! `references()` returns zero for it and the detector trusts that count
//! absolutely — while the helper right beside it gets the correct label.
//! Real-world instance: diffviz-review-tui's `tests/drillnav_common/mod.rs`
//! (included by six test crates): `drive_app`, `create_empty_engine`, and
//! `chunks_for_file` were labeled "Dead" in the 2026-07 precision audit
//! despite live callers in `drillnav_note_tests.rs`/
//! `drillnav_approval_tests.rs`.
//!
//! Run explicitly with:
//!
//! ```sh
//! cargo test -p patina-detect --test bug_dead_exports_shared_test_helper_mislabeled_dead -- --ignored
//! ```

use patina_detect::detectors::dead_exports::run_dead_exports;
use patina_detect::entities::{Evidence, Symptom};
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/bug_dead_exports_shared_helper_fixture")
}

/// Same indexing-warmup retry as `dead_exports_detection_tests`:
/// `dead_anchor` is the fixture's known-dead, always-detectable candidate.
fn run_with_retry(root: &std::path::Path) -> Vec<Symptom> {
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let outcome = run_dead_exports(root);
        let past_deadline = Instant::now() >= deadline;
        match outcome {
            Ok(symptoms) if find_export(&symptoms, "dead_anchor").is_some() => return symptoms,
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

fn find_export<'a>(symptoms: &'a [Symptom], qualified_name_suffix: &str) -> Option<&'a Symptom> {
    symptoms.iter().find(|s| {
        matches!(&s.evidence, Evidence::DeadExport { qualified_name, .. }
            if qualified_name.ends_with(qualified_name_suffix))
    })
}

#[test]
#[ignore = "FILED BUG, expected to fail until fixed (also requires rust-analyzer on PATH)"]
fn a_shared_test_helper_used_by_two_test_crates_is_not_labeled_dead() {
    let symptoms = run_with_retry(&fixture_root());

    let drive = find_export(&symptoms, "drive")
        .expect("drive should surface as a finding (it is test-only at best)");

    match &drive.evidence {
        Evidence::DeadExport {
            reference_count,
            test_only,
            ..
        } => {
            assert!(
                *test_only && *reference_count > 0,
                "drive is called from #[test] fns in first_tests.rs and \
                 second_tests.rs — it must be labeled test-only, not Dead \
                 (got reference_count: {reference_count}, test_only: {test_only})"
            );
        }
        other => panic!("expected Evidence::DeadExport, got {other:?}"),
    }
}
