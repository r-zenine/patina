//! `triage-tui` — interactive triage for `patina-detect` findings.
//!
//! Scans `libs/diffviz-core` with the house-rules detector (Phase 2),
//! filters against the triage baseline (`patina-detect-baseline.json`, both
//! relative to the current working directory — run from the repo root), and
//! opens the triage TUI on whatever's left untriaged.
//!
//! The agent CLI surface comes from tui-harness:
//!   --describe             machine-readable app manifest
//!   --test-input <seq>     run keys headlessly, print the final snapshot
//!   --test-full <seq>      run keys headlessly, print state + visual per step
//!   --agent-repl           persistent NDJSON session
//!   (no flags)             interactive TUI

use anyhow::{Context, Result};
use patina_detect::detectors::house_rules::run_house_rules;
use patina_detect::engines::DetectorEngine;
use patina_detect::persistence::Baseline;
use patina_detect_tui::{TriageApp, TriageData};
use std::env;
use std::path::Path;

const SCAN_PATH: &str = "libs/diffviz-core";
const BASELINE_PATH: &str = "patina-detect-baseline.json";

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let app = build_app(Path::new(SCAN_PATH), Path::new(BASELINE_PATH))?;
    tui_harness::run_agent_cli(app, env::args().skip(1))?;

    Ok(())
}

fn build_app(scan_path: &Path, baseline_path: &Path) -> Result<TriageApp> {
    let symptoms = run_house_rules(scan_path).with_context(|| {
        format!(
            "running house-rules detector against {}",
            scan_path.display()
        )
    })?;

    let baseline = Baseline::open(baseline_path)
        .with_context(|| format!("opening triage baseline at {}", baseline_path.display()))?;
    let untriaged = DetectorEngine::new(&baseline)
        .run(symptoms)
        .context("filtering symptoms against the triage baseline")?;

    let data = TriageData::from_symptoms(untriaged)
        .context("rendering detector findings for the triage TUI")?;

    Ok(TriageApp::new(data, baseline))
}
