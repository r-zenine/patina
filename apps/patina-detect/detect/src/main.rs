use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use patina_detect::detectors::cognitive_complexity::run_cognitive_complexity;
use patina_detect::detectors::data_clumps::run_data_clumps_refined;
use patina_detect::detectors::dead_exports::run_dead_exports;
use patina_detect::detectors::near_duplicate_structs::run_near_duplicate_structs;
use patina_detect::detectors::parallel_dispatch::run_parallel_dispatch;
use patina_detect::detectors::single_impl_traits::run_single_impl_traits;
use patina_detect::detectors::type2_clones::run_type2_clones;
use patina_detect::engines::DetectorEngine;
use patina_detect::export_symptom_log;
use patina_detect::persistence::Baseline;
use patina_detect::tui::{TriageApp, TriageData};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "patina-detect")]
#[command(about = "Deterministic, LLM-free detectors for agent-generated code")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run all detectors against a path and print untriaged symptoms
    Detect {
        /// Directory to scan (defaults to libs/diffviz-core)
        #[arg(long, default_value = "libs/diffviz-core")]
        path: PathBuf,

        /// Triage baseline file (created if missing)
        #[arg(long, default_value = "patina-detect-baseline.json")]
        baseline: PathBuf,

        /// Print all findings, ignoring the baseline
        #[arg(long)]
        audit: bool,
    },
    /// Run all detectors and open the interactive triage TUI
    Triage {
        /// Directory to scan (defaults to libs/diffviz-core)
        #[arg(long, default_value = "libs/diffviz-core")]
        path: PathBuf,

        /// Triage baseline file (created if missing)
        #[arg(long, default_value = "patina-detect-baseline.json")]
        baseline: PathBuf,

        /// tui-harness agent CLI flags (--describe, --test-input, --test-full,
        /// --agent-repl), passed through after `--`
        #[arg(last = true)]
        harness_args: Vec<String>,
    },
    /// Run all detectors and print decision-log-shaped YAML
    Export {
        /// Directory to scan (defaults to libs/diffviz-core)
        #[arg(long, default_value = "libs/diffviz-core")]
        path: PathBuf,

        /// Triage baseline file (created if missing)
        #[arg(long, default_value = "patina-detect-baseline.json")]
        baseline: PathBuf,

        /// Print all findings, ignoring the baseline
        #[arg(long)]
        audit: bool,

        /// Commit label to stamp on the export (audit runs have no single commit)
        #[arg(long, default_value = "[audit]")]
        commit: String,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    match cli.command {
        Commands::Detect {
            path,
            baseline,
            audit,
        } => {
            let symptoms = detect_symptoms(&path, &baseline, audit)?;
            if symptoms.is_empty() {
                println!("No untriaged symptoms found.");
            }
            for symptom in &symptoms {
                println!("[{}] {}", symptom.detector, symptom.title);
                for site in &symptom.sites {
                    for range in &site.line_ranges {
                        println!(
                            "  {}:{}-{}{}",
                            site.file.display(),
                            range.start,
                            range.end,
                            role_label(&site.role)
                        );
                    }
                }
            }
        }
        Commands::Triage {
            path,
            baseline,
            harness_args,
        } => {
            let symptoms = detect_symptoms(&path, &baseline, false)?;
            let baseline = Baseline::open(&baseline)
                .with_context(|| format!("opening triage baseline at {}", baseline.display()))?;
            let data = TriageData::from_symptoms(&path, symptoms)
                .context("rendering detector findings for the triage TUI")?;
            let app = TriageApp::new(data, baseline);
            tui_harness::run_agent_cli(app, harness_args).context("running the triage TUI")?;
        }
        Commands::Export {
            path,
            baseline,
            audit,
            commit,
        } => {
            let symptoms = detect_symptoms(&path, &baseline, audit)?;
            println!("{}", export_symptom_log(commit, &symptoms));
        }
    }
    Ok(())
}

/// Suffix distinguishing supporting-evidence sites from the locations that
/// themselves are the finding — without it, e.g. a data clump's forwarding
/// call site prints identically to the clump's occurrences, making a
/// "3 occurrences" symptom appear to list four.
fn role_label(role: &patina_detect::entities::SiteRole) -> &'static str {
    use patina_detect::entities::SiteRole;
    match role {
        SiteRole::MatchSite | SiteRole::CloneMember => "",
        SiteRole::ForwardingSite => " [forwarding call]",
        SiteRole::Definition => " [definition]",
        SiteRole::Caller => " [caller]",
        SiteRole::ConversionSite => " [conversion site]",
        _ => " [supporting site]",
    }
}

fn detect_symptoms(
    path: &Path,
    baseline_path: &Path,
    audit: bool,
) -> Result<Vec<patina_detect::entities::Symptom>> {
    let mut symptoms = run_type2_clones(path)
        .with_context(|| format!("running type2-clones detector against {}", path.display()))?;
    symptoms.extend(run_cognitive_complexity(path).with_context(|| {
        format!(
            "running cognitive-complexity detector against {}",
            path.display()
        )
    })?);
    symptoms.extend(
        run_data_clumps_refined(path)
            .with_context(|| format!("running data-clumps detector against {}", path.display()))?,
    );
    symptoms
        .extend(run_dead_exports(path).with_context(|| {
            format!("running dead-exports detector against {}", path.display())
        })?);
    symptoms.extend(run_near_duplicate_structs(path).with_context(|| {
        format!(
            "running near-duplicate-structs detector against {}",
            path.display()
        )
    })?);
    symptoms.extend(run_parallel_dispatch(path).with_context(|| {
        format!(
            "running parallel-dispatch detector against {}",
            path.display()
        )
    })?);
    symptoms.extend(run_single_impl_traits(path).with_context(|| {
        format!(
            "running single-impl-traits detector against {}",
            path.display()
        )
    })?);

    if audit {
        return Ok(symptoms);
    }

    let baseline = Baseline::open(baseline_path)
        .with_context(|| format!("opening triage baseline at {}", baseline_path.display()))?;
    let engine = DetectorEngine::new(&baseline);
    let untriaged = engine
        .run(symptoms)
        .context("filtering symptoms against the triage baseline")?;
    Ok(untriaged)
}
