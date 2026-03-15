mod commands;
mod environment;

use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::path::Path;

use commands::{
    CommandExecutor, diagnose::DiagnoseCommand, review::ReviewCommand, show::ShowCommand,
};
use diffviz_core::{
    ast_diff::SourceCode,
    common::{LanguageParser, ProgrammingLanguage},
    decision_based_diff::create_reviewable_diff_from_range,
    parsers::{
        CParser, CppParser, GoParser, JavaParser, JavaScriptParser, PythonParser, RustParser,
        TypeScriptParser,
    },
    reviewable_diff::NodeChangeStatus,
};
use diffviz_git::GitRepository;
use diffviz_review::{
    Approval, DecisionApproval, DecisionApprovals, DecisionLog, DiffQuery, ReviewApprovals,
    ReviewEngineBuilder, engines::review_engine::ExportScope, providers::DiffProvider,
};
use diffviz_review_tui::ReviewTuiApp;
use environment::{EnvironmentBuilder, TerminalBackend};

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
    /// Debug how a decision's line range expands to a semantic boundary
    DebugExpansion {
        /// Folder containing decision-log.yaml
        folder: String,
        /// Decision number to inspect
        decision_number: u32,
        /// File path to inspect (as listed in the code impact)
        file_path: String,
    },
}

/// Persisted review state: chunk approvals + instructions + decision approvals in one file.
/// Approvals stored as Vec because HashMap<ReviewableDiffId, Approval> keys are structs,
/// not strings, which serde_json rejects as map keys.
#[derive(Serialize, Deserialize)]
struct ReviewStateFile {
    approvals: Vec<Approval>,
    instructions: serde_json::Value,
    #[serde(default)]
    decision_approvals: Vec<DecisionApproval>,
}

fn load_review_state(folder: &Path, engine: &mut diffviz_review::ReviewEngine) -> Result<()> {
    let path = folder.join("review-state.json");
    if !path.exists() {
        return Ok(());
    }
    let json = std::fs::read_to_string(&path)?;
    let state_file: ReviewStateFile = serde_json::from_str(&json)?;
    let mut approvals = ReviewApprovals::new();
    for approval in state_file.approvals {
        approvals
            .approvals
            .insert(approval.reviewable_id.clone(), approval);
    }
    engine.load_approvals(approvals);
    let mut decision_approvals = DecisionApprovals::new();
    for da in state_file.decision_approvals {
        decision_approvals.approvals.insert(da.decision_number, da);
    }
    engine.load_decision_approvals(decision_approvals);
    let instructions_str = serde_json::to_string(&state_file.instructions)?;
    engine
        .import_instructions_json(&instructions_str)
        .map_err(|e| anyhow::anyhow!("Failed to import instructions: {}", e))?;
    Ok(())
}

fn save_review_state(folder: &Path, engine: &diffviz_review::ReviewEngine) -> Result<()> {
    let instructions_str = engine
        .export_instructions_json(ExportScope::All)
        .map_err(|e| anyhow::anyhow!("Failed to export instructions: {}", e))?;
    let instructions: serde_json::Value = serde_json::from_str(&instructions_str)?;
    let approvals_vec: Vec<Approval> = engine
        .state()
        .approvals
        .approvals
        .values()
        .cloned()
        .collect();
    let decision_approvals_vec: Vec<DecisionApproval> = engine
        .state()
        .decision_approvals
        .approvals
        .values()
        .cloned()
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

fn run_contribution_review(folder: &str, repo_path: &str, author: &str) -> Result<()> {
    let folder_path = Path::new(folder);
    let content = std::fs::read_to_string(folder_path.join("decision-log.yaml"))?;
    let log = DecisionLog::parse(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse decision-log.yaml: {}", e))?;
    let query = log
        .base_commit
        .clone()
        .map(DiffQuery::commit_to_head)
        .unwrap_or_else(DiffQuery::head_to_unstaged);

    let git_repo = GitRepository::open(repo_path)
        .map_err(|e| anyhow::anyhow!("Failed to open repository: {}", e))?;

    let mut engine = ReviewEngineBuilder::new(Box::new(git_repo), author.to_string())
        .build_from_decisions(log.decisions, query)
        .map_err(|e| anyhow::anyhow!("Failed to build review engine: {}", e))?;

    load_review_state(folder_path, &mut engine)?;

    let mut app =
        ReviewTuiApp::new(engine).map_err(|e| anyhow::anyhow!("Failed to launch TUI: {}", e))?;
    app.run().map_err(|e| anyhow::anyhow!("TUI error: {}", e))?;
    let engine = app.into_review_engine();

    save_review_state(folder_path, &engine)?;
    Ok(())
}

fn byte_to_line(source: &str, byte_offset: usize) -> usize {
    source[..byte_offset.min(source.len())]
        .bytes()
        .filter(|&b| b == b'\n')
        .count()
        + 1
}

fn parser_for_file(file_path: &str) -> Result<(Box<dyn LanguageParser>, ProgrammingLanguage)> {
    match file_path.split('.').next_back().unwrap_or("") {
        "rs" => Ok((Box::new(RustParser::new()), ProgrammingLanguage::Rust)),
        "py" => Ok((Box::new(PythonParser::new()), ProgrammingLanguage::Python)),
        "go" => Ok((Box::new(GoParser::new()), ProgrammingLanguage::Go)),
        "java" => Ok((Box::new(JavaParser::new()), ProgrammingLanguage::Java)),
        "c" | "h" => Ok((Box::new(CParser::new()), ProgrammingLanguage::C)),
        "cxx" | "cpp" | "hpp" | "hxx" => Ok((Box::new(CppParser::new()), ProgrammingLanguage::Cpp)),
        "ts" | "tsx" => Ok((
            Box::new(TypeScriptParser::new()),
            ProgrammingLanguage::TypeScript,
        )),
        "js" | "jsx" => Ok((
            Box::new(JavaScriptParser::new()),
            ProgrammingLanguage::JavaScript,
        )),
        ext => Err(anyhow::anyhow!("Unsupported file extension: {ext}")),
    }
}

fn run_debug_expansion(
    folder: &str,
    decision_number: u32,
    file_path: &str,
    repo_path: &str,
) -> Result<()> {
    let folder_path = Path::new(folder);
    let content = std::fs::read_to_string(folder_path.join("decision-log.yaml"))?;
    let log = DecisionLog::parse(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse decision-log.yaml: {}", e))?;
    let query = log
        .base_commit
        .clone()
        .map(DiffQuery::commit_to_head)
        .unwrap_or_else(DiffQuery::head_to_unstaged);

    let decision = log
        .decisions
        .iter()
        .find(|d| d.number == decision_number)
        .ok_or_else(|| anyhow::anyhow!("Decision #{decision_number} not found"))?;

    let code_impact = decision
        .code_impacts
        .iter()
        .find(|c| c.file == file_path)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "File '{file_path}' not found in decision #{}",
                decision_number
            )
        })?;

    let git_repo = GitRepository::open(repo_path)
        .map_err(|e| anyhow::anyhow!("Failed to open repository: {}", e))?;

    let new_source = git_repo
        .get_source_code(file_path, &query.to)
        .map_err(|e| anyhow::anyhow!("Failed to get new source for {file_path}: {e}"))?;
    let old_source = git_repo.get_source_code(file_path, &query.from).ok();

    let (parser, language) = parser_for_file(file_path)?;

    println!("Decision #{}: \"{}\"", decision.number, decision.title);
    println!("File: {file_path}");

    for range in &code_impact.line_ranges {
        println!();
        println!("  Input range : lines {}-{}", range.start, range.end);

        let new_provider = Box::new(SourceCode::new(new_source.clone()))
            as Box<dyn diffviz_core::ast_diff::FullSourceProvider>;
        let old_provider = old_source.as_ref().map(|src| {
            Box::new(SourceCode::new(src.clone()))
                as Box<dyn diffviz_core::ast_diff::FullSourceProvider>
        });

        match create_reviewable_diff_from_range(
            file_path,
            range.start,
            range.end,
            old_provider.as_deref(),
            new_provider.as_ref(),
            language,
            parser.as_ref(),
        ) {
            Ok(diffs) => {
                let mode = if diffs.len() == 1 {
                    "Expand"
                } else {
                    "Decompose"
                };
                println!("  Mode        : {mode} ({} unit(s))", diffs.len());
                for (i, diff) in diffs.iter().enumerate() {
                    let (start_byte, end_byte) = match &diff.boundary.change_status {
                        NodeChangeStatus::Unchanged { node } => (node.start_byte, node.end_byte),
                        NodeChangeStatus::Added { node } => (node.start_byte, node.end_byte),
                        NodeChangeStatus::Deleted { node } => (node.start_byte, node.end_byte),
                        NodeChangeStatus::Modified { new_node, .. } => {
                            (new_node.start_byte, new_node.end_byte)
                        }
                        NodeChangeStatus::Moved { new_node, .. } => {
                            (new_node.start_byte, new_node.end_byte)
                        }
                        NodeChangeStatus::Reordered { new_node, .. } => {
                            (new_node.start_byte, new_node.end_byte)
                        }
                    };
                    let expanded_start = byte_to_line(&new_source, start_byte);
                    let expanded_end = byte_to_line(&new_source, end_byte);
                    let status_name = match &diff.boundary.change_status {
                        NodeChangeStatus::Unchanged { .. } => "Unchanged",
                        NodeChangeStatus::Added { .. } => "Added",
                        NodeChangeStatus::Deleted { .. } => "Deleted",
                        NodeChangeStatus::Modified { .. } => "Modified",
                        NodeChangeStatus::Moved { .. } => "Moved",
                        NodeChangeStatus::Reordered { .. } => "Reordered",
                    };
                    let prefix = if diffs.len() > 1 {
                        format!("  [{}/{}]", i + 1, diffs.len())
                    } else {
                        "  ".to_string()
                    };
                    println!("{prefix} Unit type   : {}", diff.boundary.node_type);
                    println!("{prefix} Status      : {status_name}");
                    println!(
                        "{prefix} Expanded to : lines {expanded_start}-{expanded_end}  ({} lines)",
                        expanded_end - expanded_start + 1
                    );
                    let expansion_factor = (expanded_end - expanded_start + 1) as f64
                        / (range.end - range.start + 1) as f64;
                    println!("{prefix} Expansion   : {expansion_factor:.1}x");
                }
            }
            Err(e) => {
                println!("  ERROR: {e}");
            }
        }
    }

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
                .verbose(cli.verbose)
                .terminal_backend(TerminalBackend::Crossterm);

            if let Some(ref a) = cli.author {
                env_builder = env_builder.author(a.clone());
            }

            let environment = env_builder
                .build()
                .map_err(|e| anyhow::anyhow!("Failed to create environment: {}", e))?;

            match command {
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
                Commands::DebugExpansion {
                    folder,
                    decision_number,
                    file_path,
                } => run_debug_expansion(&folder, decision_number, &file_path, &cli.repo_path),
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
