//! Review subcommand implementation
//!
//! This module implements the TUI review interface, launching an interactive
//! terminal UI for reviewing code diffs with semantic analysis.

use anyhow::Result;
use diffviz_review::{DiffQuery, GitRef, ReviewEngineBuilder};
use diffviz_review_tui::ReviewTuiApp;

use super::CommandExecutor;
use crate::environment::Environment;

/// Review subcommand for launching the interactive TUI
pub struct ReviewCommand {
    /// Optional filter to review specific files only
    #[allow(dead_code)]
    file_filter: Option<String>,
    /// Compare specific commits instead of working directory
    from_commit: Option<String>,
    to_commit: Option<String>,
}

impl ReviewCommand {
    /// Create a new Review command
    pub fn new(
        file_filter: Option<String>,
        from_commit: Option<String>,
        to_commit: Option<String>,
    ) -> Self {
        Self {
            file_filter,
            from_commit,
            to_commit,
        }
    }

    /// Create DiffQuery based on the command parameters
    fn create_diff_query(&self) -> DiffQuery {
        match (self.from_commit.as_ref(), self.to_commit.as_ref()) {
            (Some(from), Some(to)) => {
                DiffQuery::new(GitRef::Commit(from.clone()), GitRef::Commit(to.clone()))
            }
            (Some(from), None) => DiffQuery::new(GitRef::Commit(from.clone()), GitRef::Head),
            (None, Some(to)) => DiffQuery::new(GitRef::Head, GitRef::Commit(to.clone())),
            // Default: show working directory changes (HEAD to unstaged)
            (None, None) => DiffQuery::new(GitRef::Head, GitRef::Unstaged),
        }
    }
}

impl CommandExecutor for ReviewCommand {
    fn execute(&self, environment: Environment) -> Result<()> {
        // Take ownership of GitRepository since it implements DiffProvider
        let git_repo = environment.into_git_repository();
        let diff_provider = Box::new(git_repo);

        // Build ReviewEngine with proper dependencies
        let review_engine_builder =
            ReviewEngineBuilder::new(diff_provider, "review-user".to_string());

        let diff_query = self.create_diff_query();
        let review_engine = review_engine_builder
            .build(diff_query)
            .map_err(|e| anyhow::anyhow!("Failed to build ReviewEngine: {}", e))?;

        // Check if we have any reviewable diffs
        let reviewable_count = review_engine.get_ordered_reviewable_ids().len();

        if reviewable_count == 0 {
            return Err(anyhow::anyhow!(
                "No reviewable diffs found in the current working directory. Please make some changes to review."
            ));
        }

        // Create and run the TUI application
        let mut app = ReviewTuiApp::new(review_engine)?;
        app.run()?;

        Ok(())
    }
}
