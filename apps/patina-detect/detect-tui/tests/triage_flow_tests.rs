//! End-to-end triage flow: browse -> drill -> dismiss, verified against real
//! house-rules (Phase 2) findings from `libs/diffviz-core`, and against the
//! persisted baseline surviving a fresh detector run (roadmap Phase 3
//! acceptance criterion).

use std::path::PathBuf;

use patina_detect::detectors::house_rules::run_house_rules;
use patina_detect::engines::DetectorEngine;
use patina_detect::entities::Symptom;
use patina_detect::persistence::Baseline;
use patina_detect_tui::{TriageApp, TriageData};
use tempfile::NamedTempFile;
use tui_harness::InputTestHarness;

fn diffviz_core_dir() -> PathBuf {
    // Runs from apps/patina-detect/detect-tui, so climb to the repo root.
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../libs/diffviz-core")
}

fn real_symptoms() -> Vec<Symptom> {
    let symptoms = run_house_rules(&diffviz_core_dir()).expect("house-rules detector run failed");
    assert!(
        !symptoms.is_empty(),
        "expected at least one real house-rules symptom to drive this test"
    );
    symptoms
}

#[test]
fn browse_drill_dismiss_persists_and_excludes_on_next_run() {
    let symptoms = real_symptoms();
    let dismissed_id = symptoms[0].id.clone();

    let baseline_file = NamedTempFile::new().expect("failed to create temp baseline file");
    let baseline_path = baseline_file.path().to_path_buf();

    let baseline = Baseline::open(&baseline_path).expect("failed to open baseline");
    let untriaged = DetectorEngine::new(&baseline)
        .run(symptoms.clone())
        .expect("filtering against a fresh baseline should not fail");
    assert_eq!(
        untriaged.len(),
        symptoms.len(),
        "a fresh baseline should not exclude anything yet"
    );

    let data = TriageData::from_symptoms(untriaged).expect("rendering symptoms failed");
    let app = TriageApp::new(data, baseline);

    // Browse -> Enter drills into the first (cursor 0) symptom -> Space
    // opens the leader menu -> d enters the Dismiss submenu -> f records
    // DismissFalsePositive.
    let mut harness = InputTestHarness::new(app);
    let snapshot = harness
        .run_sequence_final_state("<Enter><Space>df")
        .expect("running the dismiss sequence failed");

    assert!(!snapshot.should_quit);
    assert_eq!(
        snapshot.nav_mode, "Browse",
        "recording a verdict should return to Browse"
    );
    assert!(
        snapshot.status_message.is_none(),
        "a successful dismiss should not leave an error status message, got: {:?}",
        snapshot.status_message
    );

    // Re-run the detector against a freshly reopened baseline (simulates
    // "next detect run") — the dismissed symptom must no longer surface.
    let reopened_baseline = Baseline::open(&baseline_path).expect("failed to reopen baseline");
    let still_untriaged = DetectorEngine::new(&reopened_baseline)
        .run(symptoms)
        .expect("filtering against the updated baseline should not fail");

    assert!(
        still_untriaged.iter().all(|s| s.id != dismissed_id),
        "the dismissed symptom should be excluded from the next detect run"
    );
}
