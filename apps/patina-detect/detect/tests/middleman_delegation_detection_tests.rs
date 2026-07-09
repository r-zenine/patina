//! TDD test-design (red) for Phase 10 — Detector 4: middleman delegation
//! chains (`.plans/plan-patina-detect/implementation-roadmap.md`, Phase 10;
//! `docs/patina-detect/spec.md:165-177`).
//!
//! Like `dead_exports`, this detector's reason to exist is a real
//! `incoming_calls` call through `lspkit::LspClient` against a real
//! `rust-analyzer` process — there is no meaningful pure-unit-test slice of
//! "does this function have exactly one caller" without one. `#[ignore]`d
//! since CI may not have `rust-analyzer` on `PATH`. Run explicitly with:
//!
//! ```sh
//! cargo test -p patina-detect --test middleman_delegation_detection_tests -- --ignored
//! ```

use patina_detect::detectors::middleman_delegation::run_middleman_delegation;
use patina_detect::entities::{Evidence, Symptom};
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/middleman_delegation_fixture")
}

/// Same rust-analyzer indexing race as `dead_exports_detection_tests`'s
/// retry helper — a detector run immediately after spawn can under-report
/// callers transiently. Retrying the whole run until the known 2-link chain
/// shows up is the cheapest signal that indexing has settled.
fn run_middleman_delegation_with_retry(root: &std::path::Path) -> Vec<Symptom> {
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let outcome = run_middleman_delegation(root);
        let past_deadline = Instant::now() >= deadline;
        match outcome {
            Ok(symptoms) if has_chain_containing(&symptoms, "facade") => return symptoms,
            Ok(_) if past_deadline => {
                panic!("the facade/inner_helper chain was never reported before the deadline")
            }
            Err(err) if past_deadline => {
                panic!("run_middleman_delegation never succeeded before the deadline: {err}")
            }
            _ => std::thread::sleep(Duration::from_millis(500)),
        }
    }
}

fn has_chain_containing(symptoms: &[Symptom], qualified_name_suffix: &str) -> bool {
    chain_containing(symptoms, qualified_name_suffix).is_some()
}

fn chain_containing<'a>(
    symptoms: &'a [Symptom],
    qualified_name_suffix: &str,
) -> Option<&'a Symptom> {
    symptoms.iter().find(|s| {
        matches!(&s.evidence, Evidence::MiddlemanChain { chain, .. }
            if chain.iter().any(|name| name.ends_with(qualified_name_suffix)))
    })
}

#[test]
#[ignore = "requires rust-analyzer on PATH; run with `cargo test -- --ignored`"]
fn middleman_delegation_detector_finds_and_excludes_correctly() {
    let root = fixture_root();
    let symptoms = run_middleman_delegation_with_retry(&root);

    // `facade` (called only by `entry_point`) delegates to `inner_helper`
    // (called only by `facade`), which delegates to `core_logic` (not a
    // single-delegating-call body) — a 2-link composed chain, reported as
    // one symptom.
    let chain = chain_containing(&symptoms, "facade")
        .expect("the facade -> inner_helper chain must be reported");
    match &chain.evidence {
        Evidence::MiddlemanChain {
            chain,
            caller_count,
            ..
        } => {
            assert_eq!(*caller_count, 1);
            assert!(
                chain.iter().any(|n| n.ends_with("facade"))
                    && chain.iter().any(|n| n.ends_with("inner_helper")),
                "expected both facade and inner_helper composed into one chain, got {chain:?}"
            );
            assert!(
                !chain.iter().any(|n| n.ends_with("core_logic")),
                "core_logic does real work, not a single delegating call, and must not be \
                 counted as a middleman link, got {chain:?}"
            );
        }
        other => panic!("expected Evidence::MiddlemanChain, got {other:?}"),
    }

    // `shared_helper` has the same single-delegating-call body shape as
    // `facade`, but is called from two places (`caller_one`/`caller_two`) —
    // the exactly-one-caller gate must exclude it.
    assert!(
        !has_chain_containing(&symptoms, "shared_helper"),
        "shared_helper has 2 callers and must not be reported, found: {symptoms:#?}"
    );

    // `Thing::greet` is a trait-impl method with the same single-delegating-
    // call body shape and ends up with exactly one caller (`use_greeter`),
    // but must be excluded outright — it may be satisfying the `Greeter`
    // interface, not just a pointless wrapper.
    assert!(
        !has_chain_containing(&symptoms, "greet"),
        "Thing::greet is a trait-impl method and must be excluded even with exactly one \
         caller, found: {symptoms:#?}"
    );
}

#[test]
#[ignore = "requires rust-analyzer on PATH; run with `cargo test -- --ignored`"]
fn rerunning_with_no_code_change_produces_identical_symptom_ids() {
    let root = fixture_root();
    let first = run_middleman_delegation_with_retry(&root);
    let second = run_middleman_delegation_with_retry(&root);

    let first_ids: Vec<_> = first.iter().map(|s| s.id.to_string()).collect();
    let second_ids: Vec<_> = second.iter().map(|s| s.id.to_string()).collect();
    assert_eq!(first_ids, second_ids);
}
