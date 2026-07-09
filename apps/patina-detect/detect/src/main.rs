use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use patina_detect::detectors::cognitive_complexity::run_cognitive_complexity;
use patina_detect::detectors::data_clumps::run_data_clumps;
use patina_detect::detectors::dead_exports::run_dead_exports;
use patina_detect::detectors::house_rules::run_house_rules;
use patina_detect::detectors::middleman_delegation::run_middleman_delegation;
use patina_detect::detectors::type2_clones::run_type2_clones;
use patina_detect::engines::DetectorEngine;
use patina_detect::export_symptom_log;
use patina_detect::persistence::Baseline;
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
                        println!("  {}:{}-{}", site.file.display(), range.start, range.end);
                    }
                }
            }
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

fn detect_symptoms(
    path: &Path,
    baseline_path: &Path,
    audit: bool,
) -> Result<Vec<patina_detect::entities::Symptom>> {
    let mut symptoms = run_house_rules(path)
        .with_context(|| format!("running house-rules detector against {}", path.display()))?;
    symptoms
        .extend(run_type2_clones(path).with_context(|| {
            format!("running type2-clones detector against {}", path.display())
        })?);
    symptoms.extend(run_cognitive_complexity(path).with_context(|| {
        format!(
            "running cognitive-complexity detector against {}",
            path.display()
        )
    })?);
    symptoms.extend(
        run_data_clumps(path)
            .with_context(|| format!("running data-clumps detector against {}", path.display()))?,
    );
    symptoms
        .extend(run_dead_exports(path).with_context(|| {
            format!("running dead-exports detector against {}", path.display())
        })?);
    symptoms.extend(run_middleman_delegation(path).with_context(|| {
        format!(
            "running middleman-delegation detector against {}",
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
