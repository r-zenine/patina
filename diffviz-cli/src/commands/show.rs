//! Show subcommand implementation
//!
//! This module implements the file diff display functionality,
//! using diffviz-core's semantic analysis and formatting capabilities.

use anyhow::Result;
use diffviz_core::{
    ast_diff::SourceCode, common::ProgrammingLanguage, parsers::*, renderable_diff::RenderableDiff,
    reviewable_diff_from_semantic::semantic_pairs_to_reviewable_diffs,
    semantic_ast::build_semantic_pairs,
};
use diffviz_review::{DiffProvider, DiffQuery, FileStats, GitRef};
use tree_sitter::Parser;

use super::{CommandExecutor, formatter};
use crate::environment::Environment;

/// Show subcommand for displaying file diffs with semantic analysis
pub struct ShowCommand {
    file_path: String,
    from_commit: Option<String>,
    to_commit: Option<String>,
    staged: bool,
    unstaged: bool,
}

impl ShowCommand {
    /// Create a new Show command with the specified parameters
    pub fn new(
        file_path: String,
        from_commit: Option<String>,
        to_commit: Option<String>,
        staged: bool,
        unstaged: bool,
    ) -> Self {
        Self {
            file_path,
            from_commit,
            to_commit,
            staged,
            unstaged,
        }
    }

    /// Create DiffQuery based on the command parameters
    fn create_diff_query(&self) -> DiffQuery {
        // Handle explicit stage flags first (these override commit parameters)
        if self.staged {
            return DiffQuery::head_to_staged();
        }
        if self.unstaged {
            return DiffQuery::staged_to_unstaged();
        }

        // Handle commit-based queries
        match (self.from_commit.as_ref(), self.to_commit.as_ref()) {
            (Some(from), Some(to)) => {
                DiffQuery::new(GitRef::Commit(from.clone()), GitRef::Commit(to.clone()))
            }
            (Some(from), None) => DiffQuery::new(GitRef::Commit(from.clone()), GitRef::Head),
            (None, Some(to)) => DiffQuery::new(GitRef::Head, GitRef::Commit(to.clone())),
            // Default: show working directory changes (HEAD to unstaged)
            (None, None) => DiffQuery::head_to_unstaged(),
        }
    }

    /// Display file statistics in a clean format
    fn display_file_stats(&self, stats: &FileStats) {
        use formatter::Colors;

        println!("{}📊 File Statistics:{}", Colors::HEADER, Colors::RESET);
        println!(
            "{}   • Additions: +{}{}",
            Colors::RESET,
            stats.additions,
            Colors::RESET
        );
        println!(
            "{}   • Deletions: -{}{}",
            Colors::RESET,
            stats.deletions,
            Colors::RESET
        );
        println!(
            "{}   • Total changes: {}{}",
            Colors::RESET,
            stats.total_changes,
            Colors::RESET
        );

        if stats.is_creation() {
            println!(
                "{}   • File type: New file{}",
                Colors::HEADER,
                Colors::RESET
            );
        } else if stats.is_deletion() {
            println!(
                "{}   • File type: Deleted file{}",
                Colors::HEADER,
                Colors::RESET
            );
        } else if stats.is_modification() {
            println!(
                "{}   • File type: Modified file{}",
                Colors::HEADER,
                Colors::RESET
            );
        }

        println!();
    }

    /// Create parser for the given language
    fn create_parser(
        language: ProgrammingLanguage,
    ) -> Result<(Parser, Box<dyn diffviz_core::common::LanguageParser>)> {
        let lang_parser: Box<dyn diffviz_core::common::LanguageParser> = match language {
            ProgrammingLanguage::Rust => Box::new(RustParser::new()),
            ProgrammingLanguage::Python => Box::new(PythonParser::new()),
            ProgrammingLanguage::TypeScript => Box::new(TypeScriptParser::new()),
            ProgrammingLanguage::Go => Box::new(GoParser::new()),
            _ => return Err(anyhow::anyhow!("Unsupported language: {:?}", language)),
        };

        let mut parser = Parser::new();
        parser.set_language(lang_parser.get_language())?;

        Ok((parser, lang_parser))
    }

    /// Display semantic diff analysis with full formatting
    fn display_semantic_diff(&self, environment: &Environment, query: &DiffQuery) -> Result<()> {
        use formatter::Colors;

        let git_repo = environment.git_repository();
        let provider: &dyn DiffProvider = git_repo;

        // Get file statistics
        let stats = provider
            .get_file_stats(&self.file_path, query)
            .map_err(|e| anyhow::anyhow!("Failed to get file stats: {}", e))?;

        // Display header
        println!(
            "{}📄 File: {}{}",
            Colors::HEADER,
            self.file_path,
            Colors::RESET
        );

        // Determine language
        let language = ProgrammingLanguage::from_file_path(&self.file_path);
        println!(
            "{}🔤 Language: {:?}{}",
            Colors::HEADER,
            language,
            Colors::RESET
        );

        self.display_file_stats(&stats);

        // If no changes, show info and return
        if stats.total_changes == 0 {
            println!(
                "{}📝 No changes detected in this file{}",
                Colors::HEADER,
                Colors::RESET
            );
            return Ok(());
        }

        // Get source code for both versions
        let old_source_content = provider
            .get_source_code(&self.file_path, &query.from)
            .map_err(|e| anyhow::anyhow!("Failed to get old source: {}", e))?;

        let new_source_content = provider
            .get_source_code(&self.file_path, &query.to)
            .map_err(|e| anyhow::anyhow!("Failed to get new source: {}", e))?;

        // Note: Debug file dumping removed for cleaner output

        // Create parser and perform semantic analysis
        let (mut parser, lang_parser) = Self::create_parser(language)?;

        let old_source = SourceCode::new(&old_source_content);
        let new_source = SourceCode::new(&new_source_content);

        // Parse AST trees
        let old_tree = parser
            .parse(&old_source_content, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse old source"))?;
        let new_tree = parser
            .parse(&new_source_content, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse new source"))?;

        // Build semantic trees using the new algorithm
        println!(
            "{}🔍 Analyzing semantic changes...{}",
            Colors::HEADER,
            Colors::RESET
        );

        let old_semantic_tree = lang_parser
            .build_semantic_tree(&old_tree, &old_source_content)
            .map_err(|e| anyhow::anyhow!("Failed to build old semantic tree: {}", e))?;
        let new_semantic_tree = lang_parser
            .build_semantic_tree(&new_tree, &new_source_content)
            .map_err(|e| anyhow::anyhow!("Failed to build new semantic tree: {}", e))?;

        // Build semantic pairs and convert to reviewable diffs
        let semantic_pairs = build_semantic_pairs(
            &old_semantic_tree,
            &new_semantic_tree,
            &old_source,
            &new_source,
            &*lang_parser,
        )
        .map_err(|e| anyhow::anyhow!("Failed to build semantic pairs: {}", e))?;

        let reviewable_diffs =
            semantic_pairs_to_reviewable_diffs(&semantic_pairs, language, &old_source, &new_source, &*lang_parser);

        // Core already filters out diffs with no visible content
        if reviewable_diffs.is_empty() {
            println!(
                "{}📝 No visible semantic changes detected{}",
                Colors::HEADER,
                Colors::RESET
            );
            return Ok(());
        }

        println!(
            "{}📊 Found {} semantic boundaries with changes{}",
            Colors::HEADER,
            reviewable_diffs.len(),
            Colors::RESET
        );
        println!(
            "{}────────────────────────────────────────────────────────────{}",
            Colors::UNCHANGED,
            Colors::RESET
        );

        // Format and display each reviewable diff using RenderableDiff
        for reviewable_diff in reviewable_diffs.iter() {
            // Convert to RenderableDiff for display (gets our line-by-line improvements)
            let renderable: RenderableDiff = RenderableDiff::from(reviewable_diff);

            let changed_lines = renderable
                .lines
                .iter()
                .filter(|line| line.has_changes())
                .count();
            let _hidden_lines = renderable
                .lines
                .iter()
                .filter(|line| line.should_fold())
                .count();

            println!(
                "\n{}📦 {} ({:?}) - {} changes merged{}",
                Colors::HEADER,
                renderable.metadata.boundary_name,
                language,
                changed_lines,
                Colors::RESET
            );
            println!(
                "{}────────────────────────────────────────────────────────────{}",
                Colors::UNCHANGED,
                Colors::RESET
            );

            // Display source code with our line-by-line diff improvements
            let mut hidden_count = 0;
            for line in &renderable.lines {
                if line.should_fold() {
                    hidden_count += 1;
                    continue;
                }

                if hidden_count > 0 {
                    println!(
                        "{}  ... {} lines hidden ...{}",
                        Colors::UNCHANGED,
                        hidden_count,
                        Colors::RESET
                    );
                    hidden_count = 0;
                }

                // Get line prefix and color based on change type
                if let Some(change_type) = line.primary_change_type() {
                    let (prefix, color) = match change_type {
                        diffviz_core::renderable_diff::ChangeType::Added => ("+", Colors::ADDED),
                        diffviz_core::renderable_diff::ChangeType::Deleted => {
                            ("-", Colors::DELETED)
                        }
                        diffviz_core::renderable_diff::ChangeType::Modified => {
                            ("~", Colors::MODIFIED)
                        }
                        _ => (" ", Colors::UNCHANGED),
                    };
                    println!("{}{} {}{}", color, prefix, line.content, Colors::RESET);
                } else {
                    println!("  {}", line.content);
                }
            }

            if hidden_count > 0 {
                println!(
                    "{}  ... {} lines hidden ...{}",
                    Colors::UNCHANGED,
                    hidden_count,
                    Colors::RESET
                );
            }
        }

        Ok(())
    }
}

impl CommandExecutor for ShowCommand {
    fn execute(&self, environment: Environment) -> Result<()> {
        // Validate command parameters
        if self.staged && self.unstaged {
            return Err(anyhow::anyhow!(
                "Cannot specify both --staged and --unstaged flags. Use one or neither."
            ));
        }

        if (self.staged || self.unstaged)
            && (self.from_commit.is_some() || self.to_commit.is_some())
        {
            return Err(anyhow::anyhow!(
                "Cannot combine --staged/--unstaged flags with explicit commit parameters"
            ));
        }

        // Check if file is tracked in git
        let git_repo = environment.git_repository();
        let provider: &dyn DiffProvider = git_repo;
        let query = self.create_diff_query();

        // Verify file exists in the diff
        let changed_files = provider
            .get_changed_files(&query)
            .map_err(|e| anyhow::anyhow!("Failed to get changed files: {}", e))?;

        if !changed_files
            .iter()
            .any(|(path, _)| path == &self.file_path)
        {
            return Err(anyhow::anyhow!(
                "File '{}' not found in the current diff. Available files:\n{}",
                self.file_path,
                changed_files
                    .iter()
                    .map(|(path, status)| format!("  • {path} ({status:?})"))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }

        // Display the semantic diff
        self.display_semantic_diff(&environment, &query)
    }
}
