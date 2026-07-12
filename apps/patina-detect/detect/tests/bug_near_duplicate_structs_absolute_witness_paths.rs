//! Bug: near-duplicate-structs leaks absolute paths in its conversion-site
//! witnesses.
//!
//! Expected: every `Site.file` on a symptom uses the same root-relative
//! convention — the two struct-definition sites already do
//! (`strip_prefix(root)` in `run_near_duplicate_structs`).
//!
//! Actual: the conversion-site witnesses are pushed with the raw absolute
//! path straight from rust-analyzer's `references()` response
//! (`ConversionSite.file = reference.path`, never relativized), so one
//! symptom mixes `src/lib.rs` with `/Users/.../src/lib.rs`. Observed in the
//! 2026-07 precision audit output (detect-full.txt lines 617-662).
//!
//! Run explicitly with:
//!
//! ```sh
//! cargo test -p patina-detect --test bug_near_duplicate_structs_absolute_witness_paths -- --ignored
//! ```

use patina_detect::detectors::near_duplicate_structs::run_near_duplicate_structs;
use patina_detect::entities::{Evidence, Symptom};
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/near_duplicate_structs_fixture")
}

/// Same indexing-warmup retry as `near_duplicate_structs_detection_tests`:
/// `UserRequest`/`UserRecord` is the fixture's known-positive pair.
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

fn has_pair(symptoms: &[Symptom], a: &str, b: &str) -> bool {
    symptoms.iter().any(|s| {
        matches!(&s.evidence, Evidence::NearDuplicateStructs { struct_a, struct_b, .. }
            if (struct_a.ends_with(a) && struct_b.ends_with(b))
                || (struct_a.ends_with(b) && struct_b.ends_with(a)))
    })
}

#[test]
#[ignore = "FILED BUG, expected to fail until fixed (also requires rust-analyzer on PATH)"]
fn every_site_path_is_root_relative() {
    let symptoms = run_with_retry(&fixture_root());

    for symptom in &symptoms {
        for site in &symptom.sites {
            assert!(
                site.file.is_relative(),
                "site paths must be root-relative like the definition sites, \
                 but '{}' (note: {}) on symptom '{}' is absolute",
                site.file.display(),
                site.note,
                symptom.title
            );
        }
    }
}
