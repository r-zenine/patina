mod commands;
mod environment;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::Path;

use commands::{
    CommandExecutor, debug::DebugCommand, display::DisplayReviewCommand, review::ReviewCommand,
    templates::TemplatesCommand, validate::ValidateCommand,
};
use diffviz_review::adapters::git::GitRepository;
use diffviz_review::{
    DecisionLog, DiffQuery, ReviewEngineBuilder, load_review_state, save_review_state,
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
    /// Display information about a contribution review
    Display {
        #[command(subcommand)]
        subcommand: DisplayCommands,
    },
}

#[derive(Subcommand)]
enum DisplayCommands {
    /// Show YAML summary of a contribution's review state
    Review {
        /// Path to a contribution folder containing decision-log.yaml
        folder: String,
        /// Include approved decisions and summary stats
        #[arg(long)]
        full: bool,
    },
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
            // Templates, Validate, and Display do not require a git repository
            if let Commands::Templates { ref artifact } = command {
                return TemplatesCommand::new(artifact.clone()).run();
            }
            if let Commands::Validate {
                ref artifact,
                ref file,
            } = command
            {
                return ValidateCommand::new(artifact.clone(), file.clone()).run();
            }
            if let Commands::Display {
                subcommand: DisplayCommands::Review { ref folder, full },
            } = command
            {
                return DisplayReviewCommand::new(folder.clone(), full).run();
            }

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
                Commands::Templates { .. }
                | Commands::Validate { .. }
                | Commands::Display { .. } => unreachable!(),
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
