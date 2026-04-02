mod commands;
mod environment;

use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::path::Path;

use commands::{
    CommandExecutor, debug::DebugCommand, review::ReviewCommand, templates::TemplatesCommand,
    validate::ValidateCommand,
};
use diffviz_git::GitRepository;
use diffviz_review::{
    ApprovalRecord, DecisionApprovals, DecisionLog, DiffQuery, ReviewApprovals,
    ReviewEngineBuilder, ReviewableDiffId,
};
use diffviz_review_tui::ReviewTuiApp;
use environment::EnvironmentBuilder;

#[derive(Parser)]
#[command(name = "diffviz")]
#[command(about = "A TUI for reviewing code diffs with semantic analysis")]
#[command(version)]
struct Cli {
    /// Folder containing decision-log.yaml to review
    #[arg(index = 1)]
    folder: Option<String>,

    /// Repository path (defaults to current directory)
    #[arg(short, long, default_value = ".", global = true)]
    repo_path: String,

    /// Author name for reviews and approvals
    #[arg(
        short,
        long,
        help = "Author name for reviews (defaults to system username)",
        global = true
    )]
    author: Option<String>,

    /// Enable verbose logging
    #[arg(short, long, help = "Enable verbose logging", global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch interactive TUI for reviewing code diffs
    Review {
        /// Optional filter to review specific files only
        file_filter: Option<String>,
        /// From commit hash (optional - defaults to HEAD for working directory comparison)
        from_commit: Option<String>,
        /// To commit hash (optional - defaults to working directory)
        to_commit: Option<String>,
    },
    /// Debug the pipeline stages for a given file
    Debug {
        /// Path to the file to analyze
        #[arg(long, value_name = "FILE")]
        file: String,
        /// Starting Git ref (defaults to HEAD)
        #[arg(long, value_name = "REF")]
        from: Option<String>,
        /// Ending Git ref (defaults to working tree)
        #[arg(long, value_name = "REF")]
        to: Option<String>,
        /// Filter output to specific phase (1-7)
        #[arg(long, value_name = "NUM")]
        phase: Option<u8>,
        /// Include explanations for folding decisions
        #[arg(long)]
        explain_folding: bool,
        /// Output human-readable text instead of JSON
        #[arg(long)]
        human: bool,
        /// Filter results to line range (format: start-end)
        #[arg(long, value_name = "RANGE")]
        line_range: Option<String>,
    },
    /// Generate schema templates for artifacts
    Templates {
        /// Type of artifact (e.g., decision-log)
        artifact: String,
    },
    /// Validate an artifact file against its schema
    Validate {
        /// Type of artifact (e.g., decision-log)
        artifact: String,
        /// Path to the file to validate
        file: String,
    },
}

/// Persisted chunk approval: key (reviewable_id) embedded alongside approval data.
/// Stored as Vec because HashMap<ReviewableDiffId, _> keys are structs, not strings.
#[derive(Serialize, Deserialize, Clone)]
struct PersistedApproval {
    reviewable_id: ReviewableDiffId,
    approved: bool,
    approved_by: String,
    approval_timestamp: String,
}

/// Persisted decision approval: key (decision_number) embedded alongside approval data.
#[derive(Serialize, Deserialize, Clone)]
struct PersistedDecisionApproval {
    decision_number: u32,
    approved: bool,
    approved_by: String,
    approval_timestamp: String,
}

/// Persisted review state: chunk approvals + instructions + decision approvals in one file.
#[derive(Serialize, Deserialize)]
struct ReviewStateFile {
    approvals: Vec<PersistedApproval>,
    instructions: serde_json::Value,
    #[serde(default)]
    decision_approvals: Vec<PersistedDecisionApproval>,
}

fn load_review_state(folder: &Path, engine: &mut diffviz_review::ReviewEngine) -> Result<()> {
    let path = folder.join("review-state.json");
    if !path.exists() {
        return Ok(());
    }
    let json = std::fs::read_to_string(&path)?;
    let state_file: ReviewStateFile = serde_json::from_str(&json)?;
    let mut approvals = ReviewApprovals::new();
    for a in state_file.approvals {
        approvals.approvals.insert(
            a.reviewable_id,
            ApprovalRecord {
                approved: a.approved,
                approved_by: a.approved_by,
                approval_timestamp: a.approval_timestamp,
            },
        );
    }
    engine.load_approvals(approvals);
    let mut decision_approvals = DecisionApprovals::new();
    for da in state_file.decision_approvals {
        decision_approvals.approvals.insert(
            da.decision_number,
            ApprovalRecord {
                approved: da.approved,
                approved_by: da.approved_by,
                approval_timestamp: da.approval_timestamp,
            },
        );
    }
    engine.load_decision_approvals(decision_approvals);
    let instructions_str = serde_json::to_string(&state_file.instructions)?;
    engine
        .import_instructions_json(&instructions_str)
        .map_err(|e| anyhow::anyhow!("Failed to import instructions: {e}"))?;
    Ok(())
}

fn save_review_state(folder: &Path, engine: &diffviz_review::ReviewEngine) -> Result<()> {
    let instructions_str = engine
        .export_instructions_json()
        .map_err(|e| anyhow::anyhow!("Failed to export instructions: {e}"))?;
    let instructions: serde_json::Value = serde_json::from_str(&instructions_str)?;
    let approvals_vec: Vec<PersistedApproval> = engine
        .state()
        .approvals
        .approvals
        .iter()
        .map(|(id, r)| PersistedApproval {
            reviewable_id: id.clone(),
            approved: r.approved,
            approved_by: r.approved_by.clone(),
            approval_timestamp: r.approval_timestamp.clone(),
        })
        .collect();
    let decision_approvals_vec: Vec<PersistedDecisionApproval> = engine
        .state()
        .decision_approvals
        .approvals
        .iter()
        .map(|(num, r)| PersistedDecisionApproval {
            decision_number: *num,
            approved: r.approved,
            approved_by: r.approved_by.clone(),
            approval_timestamp: r.approval_timestamp.clone(),
        })
        .collect();
    let state_file = ReviewStateFile {
        approvals: approvals_vec,
        instructions,
        decision_approvals: decision_approvals_vec,
    };
    let json = serde_json::to_string_pretty(&state_file)?;
    std::fs::write(folder.join("review-state.json"), json)?;
    Ok(())
}

fn resolve_commit_diff(git_repo: &GitRepository, commit_hash: String) -> Result<DiffQuery> {
    let parent_hash = git_repo
        .resolve_parent_commit(&commit_hash)
        .map_err(|e| anyhow::anyhow!("Failed to resolve parent of commit {commit_hash}: {e}"))?;
    Ok(DiffQuery::new(
        diffviz_review::GitRef::Commit(parent_hash),
        diffviz_review::GitRef::Commit(commit_hash),
    ))
}

fn run_contribution_review(folder: &str, repo_path: &str, author: &str) -> Result<()> {
    let folder_path = Path::new(folder);
    let content = std::fs::read_to_string(folder_path.join("decision-log.yaml"))?;
    let log = DecisionLog::parse(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse decision-log.yaml: {e}"))?;

    let git_repo = GitRepository::open(repo_path)
        .map_err(|e| anyhow::anyhow!("Failed to open repository: {e}"))?;

    let query = resolve_commit_diff(&git_repo, log.commit)?;

    let mut engine = ReviewEngineBuilder::new(Box::new(git_repo), author.to_string())
        .build_from_decisions(log.decisions, query)
        .map_err(|e| anyhow::anyhow!("Failed to build review engine: {e}"))?;

    load_review_state(folder_path, &mut engine)?;

    let mut app =
        ReviewTuiApp::new(engine).map_err(|e| anyhow::anyhow!("Failed to launch TUI: {e}"))?;
    app.run().map_err(|e| anyhow::anyhow!("TUI error: {e}"))?;
    let engine = app.into_review_engine();

    save_review_state(folder_path, &engine)?;
    Ok(())
}

fn main() -> Result<()> {
    // Initialize logging
    use log::LevelFilter;
    use log4rs::append::file::FileAppender;
    use log4rs::config::{Appender, Config, Root};
    use log4rs::encode::pattern::PatternEncoder;

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build("output.log")?;

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))?;

    log4rs::init_config(config)?;

    // Parse command line arguments
    let cli = Cli::parse();

    let author = cli.author.clone().unwrap_or_else(whoami::username);

    match (cli.folder, cli.command) {
        (Some(folder), None) => run_contribution_review(&folder, &cli.repo_path, &author),

        (None, Some(command)) => {
            let mut env_builder = EnvironmentBuilder::new()
                .repo_path(&cli.repo_path)
                .verbose(cli.verbose);

            if let Some(ref a) = cli.author {
                env_builder = env_builder.author(a.clone());
            }

            let environment = env_builder
                .build()
                .map_err(|e| anyhow::anyhow!("Failed to create environment: {e}"))?;

            match command {
                Commands::Review { .. } => {
                    let review_command = ReviewCommand::new();
                    review_command.execute(environment)
                }
                Commands::Debug {
                    file,
                    from,
                    to,
                    phase,
                    explain_folding,
                    human,
                    line_range,
                } => {
                    let debug_command = DebugCommand {
                        file_path: file,
                        from,
                        to,
                        phase,
                        explain_folding,
                        human,
                        line_range,
                    };
                    debug_command.execute(environment)
                }
                Commands::Templates { artifact } => {
                    let templates_command = TemplatesCommand::new(artifact);
                    templates_command.execute(environment)
                }
                Commands::Validate { artifact, file } => {
                    let validate_command = ValidateCommand::new(artifact, file);
                    validate_command.execute(environment)
                }
            }
        }

        (None, None) => {
            eprintln!("Usage: diffviz <folder> | diffviz <subcommand>");
            std::process::exit(1);
        }

        (Some(_), Some(_)) => {
            eprintln!("Error: cannot specify both a folder and a subcommand");
            std::process::exit(1);
        }
    }
}
