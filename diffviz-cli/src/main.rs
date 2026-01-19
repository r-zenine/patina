mod commands;
mod environment;

use anyhow::Result;
use clap::{Parser, Subcommand};

use commands::{
    CommandExecutor, diagnose::DiagnoseCommand, review::ReviewCommand, show::ShowCommand,
};
use environment::{EnvironmentBuilder, TerminalBackend};

#[derive(Parser)]
#[command(name = "diffviz")]
#[command(about = "A TUI for reviewing code diffs with semantic analysis")]
#[command(version)]
struct Cli {
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
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Display diff for a specific file with semantic analysis
    Show {
        /// File path to show diff for
        file_path: String,
        /// From commit hash (optional - defaults to HEAD for working directory comparison)
        from_commit: Option<String>,
        /// To commit hash (optional - defaults to working directory)
        to_commit: Option<String>,
        /// Show staged changes (HEAD to staged)
        #[arg(long, conflicts_with = "unstaged")]
        staged: bool,
        /// Show unstaged changes only (staged to unstaged)
        #[arg(long, conflicts_with = "staged")]
        unstaged: bool,
    },
    /// Launch interactive TUI for reviewing code diffs
    Review {
        /// Optional filter to review specific files only
        file_filter: Option<String>,
        /// From commit hash (optional - defaults to HEAD for working directory comparison)
        from_commit: Option<String>,
        /// To commit hash (optional - defaults to working directory)
        to_commit: Option<String>,
    },
    /// Run diagnostic modes for debugging the data pipeline
    Diagnose {
        /// Generate comprehensive debug context for a specific file
        file_path: Option<String>,
    },
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

    // Build environment with dependency injection
    let mut env_builder = EnvironmentBuilder::new()
        .repo_path(&cli.repo_path)
        .verbose(cli.verbose)
        .terminal_backend(TerminalBackend::Crossterm);

    if let Some(ref author) = cli.author {
        env_builder = env_builder.author(author.clone());
    }

    let environment = env_builder
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to create environment: {}", e))?;

    // Dispatch to subcommand handlers
    match cli.command {
        Commands::Show {
            file_path,
            from_commit,
            to_commit,
            staged,
            unstaged,
        } => {
            let show_command =
                ShowCommand::new(file_path, from_commit, to_commit, staged, unstaged);
            show_command.execute(environment)
        }
        Commands::Review {
            file_filter,
            from_commit,
            to_commit,
        } => {
            let review_command = ReviewCommand::new(file_filter, from_commit, to_commit);
            review_command.execute(environment)
        }
        Commands::Diagnose { file_path } => {
            let diagnose_command = DiagnoseCommand::new(file_path);
            diagnose_command.execute(environment)
        }
    }
}
