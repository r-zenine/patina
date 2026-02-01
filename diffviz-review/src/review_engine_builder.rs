//! ReviewEngine builder for git-to-ReviewEngine integration at the CLI layer
//!
//! This module provides the ReviewEngineBuilder that orchestrates between the git,
//! core, and review layers to create fully populated ReviewEngine instances.

use crate::engines::ReviewEngine;
use crate::entities::decision::{Decision, ReviewDecisions};
use crate::entities::git_ref::{DiffQuery, GitRef};
use crate::entities::reviewable_diff_id::{LineRange, ReviewableDiffId};
use crate::providers::DiffProvider;
use crate::state::ReviewableDiff;

use diffviz_core::parsers::{CParser, CppParser, JavaParser, JavaScriptParser, TypeScriptParser};
// Import diffviz-core for semantic analysis
use diffviz_core::{
    ast_diff::SourceCode,
    common::{LanguageParser, ProgrammingLanguage},
    decision_based_diff::create_reviewable_diff_from_range,
    parsers::{GoParser, PythonParser, RustParser},
    reviewable_diff::ReviewableDiff as CoreReviewableDiff,
    reviewable_diff_from_semantic::semantic_pairs_to_reviewable_diffs,
    semantic_ast::build_semantic_pairs,
};

/// Builder for creating ReviewEngine instances from git queries
///
/// This sits at the CLI layer and orchestrates the complete pipeline:
/// 1. Execute git queries through DiffProvider (git layer)  
/// 2. Parse and analyze code using diffviz-core (core layer)
/// 3. Create ReviewEngine with ReviewableDiffs (review layer)
pub struct ReviewEngineBuilder {
    diff_provider: Box<dyn DiffProvider>,
    author: String,
}

impl ReviewEngineBuilder {
    /// Create a new ReviewEngineBuilder with a DiffProvider
    pub fn new(diff_provider: Box<dyn DiffProvider>, author: String) -> Self {
        Self {
            diff_provider,
            author,
        }
    }

    /// Build a ReviewEngine from a DiffQuery
    ///
    /// This executes the complete pipeline to transform git diffs into
    /// a fully populated ReviewEngine ready for TUI usage.
    pub fn build(self, query: DiffQuery) -> Result<ReviewEngine, crate::errors::DiffVizError> {
        // Step 1: Get changed files from git
        let changed_files = self.diff_provider.get_changed_files(&query).map_err(|e| {
            crate::errors::DiffVizError::Git(format!("Failed to get changed files from git: {e}"))
        })?;

        if changed_files.is_empty() {
            return Ok(ReviewEngine::new(vec![], self.author, self.diff_provider));
        }

        // Step 2: Filter and process only supported files to create ReviewableDiffs
        let mut all_reviewable_diffs = Vec::new();

        for (file_path, file_status) in &changed_files {
            // Skip unsupported file types
            if !is_supported_file(file_path) {
                eprintln!("Skipping unsupported file type: {file_path}");
                continue;
            }

            // Skip deleted files for now - semantic analysis requires both old and new content
            // TODO: In the future, we could show deleted files with special handling
            if matches!(file_status, crate::providers::FileStatus::Deleted) {
                eprintln!("Skipping deleted file: {file_path}");
                continue;
            }

            // For added/untracked files, use empty string as old content
            // For other statuses, get actual content from git
            let reviewable_diffs = if matches!(
                file_status,
                crate::providers::FileStatus::Added | crate::providers::FileStatus::Untracked
            ) {
                self.create_semantic_reviewable_diffs_for_added_file(file_path, &query)?
            } else {
                self.create_semantic_reviewable_diffs(file_path, &query)?
            };
            all_reviewable_diffs.extend(reviewable_diffs);
        }

        // Step 3: Create ReviewEngine with all ReviewableDiffs
        Ok(ReviewEngine::new(
            all_reviewable_diffs,
            self.author,
            self.diff_provider,
        ))
    }

    /// Build a ReviewEngine from decisions using the decision-based diff pipeline
    ///
    /// This new pipeline (Phase 2.1) uses architectural decisions as the primary input
    /// instead of discovering changes from git. For each decision and its code impacts:
    /// 1. Fetch old/new source via DiffProvider
    /// 2. Call create_reviewable_diff_from_range() from diffviz-core
    /// 3. Wrap result in review-layer ReviewableDiff with ReviewableDiffId
    ///
    /// The decision-to-diff relationship is established at creation time, eliminating
    /// the need for post-hoc overlap detection via build_index_from_review_state().
    pub fn build_from_decisions(
        self,
        decisions: Vec<Decision>,
        query: DiffQuery,
    ) -> Result<ReviewEngine, crate::errors::DiffVizError> {
        let mut all_reviewable_diffs = Vec::new();
        let mut review_decisions = ReviewDecisions::new();

        // Process each decision to create ReviewableDiffs
        for decision in decisions {
            review_decisions.add_decision(decision.clone());

            for code_impact in &decision.code_impacts {
                let file_path = &code_impact.file;

                // Skip unsupported files
                if !is_supported_file(file_path) {
                    eprintln!(
                        "Skipping unsupported file in decision {}: {}",
                        decision.number, file_path
                    );
                    continue;
                }

                // Get language parser for this file
                let (parser, language) = get_language_parser_for_file(file_path)?;

                // Process each line range in the code impact
                for range in &code_impact.line_ranges {
                    // Get source code for old and new versions
                    let new_source_str = self
                        .diff_provider
                        .get_source_code(file_path, &query.to)
                        .map_err(|e| {
                            crate::errors::DiffVizError::Git(format!(
                                "Failed to get new source for {file_path}: {e}"
                            ))
                        })?;

                    let old_source_str = self
                        .diff_provider
                        .get_source_code(file_path, &query.from)
                        .ok();

                    // Create providers for the sources
                    let new_provider = Box::new(SourceCode::new(new_source_str.clone()))
                        as Box<dyn diffviz_core::ast_diff::FullSourceProvider>;
                    let old_provider = old_source_str.as_ref().map(|src| {
                        Box::new(SourceCode::new(src.clone()))
                            as Box<dyn diffviz_core::ast_diff::FullSourceProvider>
                    });

                    // Call decision-based diff creation
                    let core_diff = create_reviewable_diff_from_range(
                        file_path,
                        range.start,
                        range.end,
                        old_provider.as_deref(),
                        new_provider.as_ref(),
                        language,
                        parser.as_ref(),
                    )
                    .map_err(|e| {
                        crate::errors::DiffVizError::ProcessingFailed(format!(
                            "Failed to create diff for {} (decision {}): {}",
                            file_path, decision.number, e
                        ))
                    })?;

                    // Extract line range from the core diff
                    let new_source_provider = SourceCode::new(new_source_str);
                    let old_source_provider = old_source_str.map(SourceCode::new);
                    let line_range = extract_line_range_from_core_diff(
                        &core_diff,
                        &new_source_provider,
                        old_source_provider.as_ref().unwrap_or(&new_source_provider),
                    )
                    .ok_or_else(|| {
                        crate::errors::DiffVizError::ProcessingFailed(
                            "Failed to extract line range from diff".to_string(),
                        )
                    })?;

                    // Create review-layer ReviewableDiff
                    let reviewable_id = ReviewableDiffId::new(
                        query.clone(),
                        format!("{file_path}#d{}", decision.number),
                        line_range,
                    );

                    let reviewable_diff =
                        ReviewableDiff::new(reviewable_id, core_diff, file_path.to_string());
                    all_reviewable_diffs.push(reviewable_diff);
                }
            }
        }

        // Create engine with ReviewableDiffs
        let mut engine = ReviewEngine::new(all_reviewable_diffs, self.author, self.diff_provider);

        // Set decisions with index (maps ReviewableDiffIds to decision numbers)
        engine.set_decisions_with_index(review_decisions);

        Ok(engine)
    }

    /// Create ReviewableDiffs using semantic analysis pipeline
    ///
    /// Implementation of the full semantic analysis pipeline:
    /// 1. Get file content from DiffProvider  
    /// 2. Use diffviz-core for AST parsing and change detection
    /// 3. Use diffviz-core for semantic context expansion
    /// 4. Convert to review layer ReviewableDiff format
    fn create_semantic_reviewable_diffs(
        &self,
        file_path: &str,
        query: &DiffQuery,
    ) -> Result<Vec<ReviewableDiff>, crate::errors::DiffVizError> {
        // Get the appropriate language parser for this file
        let (parser, language) = get_language_parser_for_file(file_path)?;

        // Step 1: Get file content from git via DiffProvider
        let old_content = self
            .diff_provider
            .get_source_code(file_path, &query.from)
            .map_err(|e| {
                crate::errors::DiffVizError::Git(format!("Failed to get old content: {e}"))
            })?;
        let new_content = self
            .diff_provider
            .get_source_code(file_path, &query.to)
            .map_err(|e| {
                crate::errors::DiffVizError::Git(format!("Failed to get new content: {e}"))
            })?;

        // Step 2: Create SourceCode objects
        let old_source = SourceCode::new(old_content.clone());
        let new_source = SourceCode::new(new_content.clone());

        // Step 3: Parse AST trees
        let old_tree = parser.try_parse(&old_content).map_err(|e| {
            crate::errors::DiffVizError::ProcessingFailed(format!("Failed to parse old AST: {e}"))
        })?;
        let new_tree = parser.try_parse(&new_content).map_err(|e| {
            crate::errors::DiffVizError::ProcessingFailed(format!("Failed to parse new AST: {e}"))
        })?;

        // Step 4: Build semantic trees
        let old_semantic = parser
            .build_semantic_tree(&old_tree, &old_content)
            .map_err(|e| {
                crate::errors::DiffVizError::ProcessingFailed(format!(
                    "Failed to build old semantic tree: {e}"
                ))
            })?;
        let new_semantic = parser
            .build_semantic_tree(&new_tree, &new_content)
            .map_err(|e| {
                crate::errors::DiffVizError::ProcessingFailed(format!(
                    "Failed to build new semantic tree: {e}"
                ))
            })?;

        // Step 5: Build semantic pairs
        let semantic_pairs = build_semantic_pairs(
            &old_semantic,
            &new_semantic,
            &old_source,
            &new_source,
            parser.as_ref(),
        )
        .map_err(|e| {
            crate::errors::DiffVizError::ProcessingFailed(format!(
                "Failed to build semantic pairs: {e}"
            ))
        })?;

        // Step 6: Convert to core ReviewableDiffs with context expansion (Phase 6)
        let core_reviewable_diffs = semantic_pairs_to_reviewable_diffs(
            &semantic_pairs,
            language,
            &old_source,
            &new_source,
            parser.as_ref(),
        );

        // Step 7: Convert core ReviewableDiffs to review layer ReviewableDiffs
        let review_layer_diffs: Vec<ReviewableDiff> = core_reviewable_diffs
            .into_iter()
            .enumerate()
            .map(|(index, core_diff)| {
                // Create ReviewableDiff ID based on the core diff boundary
                // Extract line range from the core diff boundary
                let line_range = if let Some(line_range) =
                    extract_line_range_from_core_diff(&core_diff, &old_source, &new_source)
                {
                    line_range
                } else {
                    // Fallback line range
                    LineRange {
                        start_line: 1,
                        end_line: 10,
                        start_column: 0,
                        end_column: 0,
                    }
                };

                let reviewable_id = ReviewableDiffId::new(
                    query.clone(),
                    format!("{file_path}#{index}"), // Make unique with index
                    line_range,
                );

                ReviewableDiff::new(reviewable_id, core_diff, file_path.to_string())
            })
            .collect();

        Ok(review_layer_diffs)
    }

    /// Create ReviewableDiffs for added files (new files)
    ///
    /// For added files, the old content is empty and the new content is from the working directory
    fn create_semantic_reviewable_diffs_for_added_file(
        &self,
        file_path: &str,
        query: &DiffQuery,
    ) -> Result<Vec<ReviewableDiff>, crate::errors::DiffVizError> {
        // Get the appropriate language parser for this file
        let (parser, language) = get_language_parser_for_file(file_path)?;

        // For added files, old content is empty
        let old_content = String::new();
        let new_content = self
            .diff_provider
            .get_source_code(file_path, &query.to)
            .map_err(|e| {
                crate::errors::DiffVizError::Git(format!(
                    "Failed to get new content for added file: {e}"
                ))
            })?;

        // Step 2: Create SourceCode objects
        let old_source = SourceCode::new(old_content.clone());
        let new_source = SourceCode::new(new_content.clone());

        // Step 3: Parse AST trees (old tree will be empty)
        let old_tree = parser.try_parse(&old_content).map_err(|e| {
            crate::errors::DiffVizError::ProcessingFailed(format!("Failed to parse old AST: {e}"))
        })?;
        let new_tree = parser.try_parse(&new_content).map_err(|e| {
            crate::errors::DiffVizError::ProcessingFailed(format!("Failed to parse new AST: {e}"))
        })?;

        // Step 4: Build semantic trees
        let old_semantic = parser
            .build_semantic_tree(&old_tree, &old_content)
            .map_err(|e| {
                crate::errors::DiffVizError::ProcessingFailed(format!(
                    "Failed to build old semantic tree: {e}"
                ))
            })?;
        let new_semantic = parser
            .build_semantic_tree(&new_tree, &new_content)
            .map_err(|e| {
                crate::errors::DiffVizError::ProcessingFailed(format!(
                    "Failed to build new semantic tree: {e}"
                ))
            })?;

        // Step 5: Build semantic pairs and convert to ReviewableDiffs
        let semantic_pairs = build_semantic_pairs(
            &old_semantic,
            &new_semantic,
            &old_source,
            &new_source,
            parser.as_ref(),
        )
        .map_err(|e| {
            crate::errors::DiffVizError::ProcessingFailed(format!(
                "Failed to build semantic pairs: {e}"
            ))
        })?;

        // Step 6: Convert to core ReviewableDiffs with context expansion (Phase 6)
        let core_reviewable_diffs = semantic_pairs_to_reviewable_diffs(
            &semantic_pairs,
            language,
            &old_source,
            &new_source,
            parser.as_ref(),
        );

        // Step 7: Convert core ReviewableDiffs to review layer format
        let review_layer_diffs: Vec<ReviewableDiff> = core_reviewable_diffs
            .into_iter()
            .enumerate()
            .map(|(index, core_diff)| {
                let line_range =
                    extract_line_range_from_core_diff(&core_diff, &old_source, &new_source)
                        .unwrap_or(LineRange {
                            start_line: 1,
                            end_line: 10,
                            start_column: 0,
                            end_column: 0,
                        });

                let reviewable_id = ReviewableDiffId::new(
                    query.clone(),
                    format!("{file_path}#{index}"),
                    line_range,
                );

                ReviewableDiff::new(reviewable_id, core_diff, file_path.to_string())
            })
            .collect();

        Ok(review_layer_diffs)
    }

    /// Convenience method: Build ReviewEngine for working directory changes
    #[cfg(test)]
    pub fn from_working_directory(
        diff_provider: Box<dyn DiffProvider>,
        author: String,
    ) -> Result<ReviewEngine, crate::errors::DiffVizError> {
        let builder = Self::new(diff_provider, author);
        builder.build(DiffQuery::new(GitRef::Head, GitRef::Unstaged))
    }

    /// Convenience method: Build ReviewEngine for commit comparison
    #[cfg(test)]
    pub fn from_commit_comparison(
        diff_provider: Box<dyn DiffProvider>,
        author: String,
        from_commit: String,
        to_commit: String,
    ) -> Result<ReviewEngine, crate::errors::DiffVizError> {
        let builder = Self::new(diff_provider, author);
        builder.build(DiffQuery::new(
            GitRef::Commit(from_commit),
            GitRef::Commit(to_commit),
        ))
    }

    /// Convenience method: Build ReviewEngine for commit to HEAD comparison
    #[allow(dead_code)]
    pub fn from_commit_to_head(
        diff_provider: Box<dyn DiffProvider>,
        author: String,
        from_commit: String,
    ) -> Result<ReviewEngine, crate::errors::DiffVizError> {
        let builder = Self::new(diff_provider, author);
        builder.build(DiffQuery::new(GitRef::Commit(from_commit), GitRef::Head))
    }

    /// Convenience method: Build ReviewEngine for HEAD to commit comparison  
    #[allow(dead_code)]
    pub fn from_head_to_commit(
        diff_provider: Box<dyn DiffProvider>,
        author: String,
        to_commit: String,
    ) -> Result<ReviewEngine, crate::errors::DiffVizError> {
        let builder = Self::new(diff_provider, author);
        builder.build(DiffQuery::new(GitRef::Head, GitRef::Commit(to_commit)))
    }
}

/// Extract line range from a core ReviewableDiff boundary
fn extract_line_range_from_core_diff(
    core_diff: &CoreReviewableDiff,
    old_source: &dyn diffviz_core::ast_diff::SourceProvider,
    new_source: &dyn diffviz_core::ast_diff::SourceProvider,
) -> Option<LineRange> {
    // Try to extract line range from the boundary node's change status
    use diffviz_core::reviewable_diff::NodeChangeStatus;

    let (node_data, source_provider) = match &core_diff.boundary.change_status {
        NodeChangeStatus::Unchanged { node } => (Some(node), new_source),
        NodeChangeStatus::Added { node } => (Some(node), new_source),
        NodeChangeStatus::Deleted { node } => (Some(node), old_source),
        NodeChangeStatus::Modified { new_node, .. } => (Some(new_node), new_source),
        NodeChangeStatus::Moved { new_node, .. } => (Some(new_node), new_source),
        NodeChangeStatus::Reordered { new_node, .. } => (Some(new_node), new_source),
    };

    if let Some(node) = node_data {
        // Use the SourceProvider's line_range method which handles both TreeSitter and owned nodes
        let core_line_range = source_provider.line_range(node);

        // Convert from diffviz_core::LineRange to review layer LineRange
        Some(LineRange {
            start_line: core_line_range.start_line,
            end_line: core_line_range.end_line,
            start_column: core_line_range.start_column,
            end_column: core_line_range.end_column,
        })
    } else {
        None
    }
}

/// Check if a file is supported for semantic analysis based on its extension
/// Only includes languages that are fully implemented with semantic tree building
fn is_supported_file(file_path: &str) -> bool {
    matches!(
        file_path.split('.').next_back().unwrap_or(""),
        "rs" | "py"
            | "go"
            | "java"
            | "ts"
            | "tsx"
            | "js"
            | "jsx"
            | "c"
            | "h"
            | "cxx"
            | "cpp"
            | "hpp"
            | "hxx"
    )
}

/// Get the appropriate language parser for a file based on its extension
/// Only supports languages that are fully implemented with semantic tree building
fn get_language_parser_for_file(
    file_path: &str,
) -> Result<(Box<dyn LanguageParser>, ProgrammingLanguage), crate::errors::DiffVizError> {
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
        ext => Err(crate::errors::DiffVizError::ProcessingFailed(format!(
            "Unsupported file extension: {ext}"
        ))),
    }
}

// TODO: Future implementation would include:
//
// fn perform_semantic_analysis(
//     old_content: &str,
//     new_content: &str,
//     language: ProgrammingLanguage,
//     query: &DiffQuery,
//     file_path: &str,
// ) -> Result<Vec<ReviewableDiff>> {
//     use diffviz_core::ast_diff::{diff_ast_trees_with_strategies, ChangeDetectionStrategies, SourceCode};
//     use diffviz_core::reviewable_diff::expand_changes_to_reviewable_diffs;
//     use diffviz_core::common::LanguageParser;
//     use tree_sitter::Parser;
//
//     // 1. Create SourceCode objects
//     let old_source = SourceCode::new(old_content);
//     let new_source = SourceCode::new(new_content);
//
//     // 2. Get appropriate parser for language
//     let parser_impl: Box<dyn LanguageParser> = get_parser_for_language(language)
//         .with_context(|| format!("Unsupported language for file: {}", file_path))?;
//     let mut ts_parser = Parser::new();
//     ts_parser.set_language(parser_impl.get_language())
//         .with_context(|| "Failed to set TreeSitter language")?;
//
//     // 3. Parse AST trees
//     let old_tree = ts_parser.parse(old_content, None)
//         .with_context(|| "Failed to parse old content")?;
//     let new_tree = ts_parser.parse(new_content, None)
//         .with_context(|| "Failed to parse new content")?;
//
//     // 4. Detect changes using strategies
//     let strategies = ChangeDetectionStrategies::default_strategies();
//     let ast_diff = diff_ast_trees_with_strategies(
//         &old_tree, &new_tree, old_content, new_content, strategies
//     );
//
//     // 5. Expand changes to ReviewableDiffs with context
//     let core_reviewable_diffs = expand_changes_to_reviewable_diffs(
//         &ast_diff.changes,
//         parser_impl.as_ref(),
//         &old_source,
//         &new_source,
//         language,
//     );
//
//     // 6. Convert diffviz-core ReviewableDiffs to review-layer ReviewableDiffs
//     let review_layer_diffs = core_reviewable_diffs.into_iter().map(|core_diff| {
//         convert_core_to_review_diff(core_diff, query, file_path)
//     }).collect::<Result<Vec<_>>>()?;
//
//     Ok(review_layer_diffs)
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::{FileStats, FileStatus};

    /// Mock DiffProvider for testing
    struct MockDiffProvider {
        files: Vec<(String, FileStatus)>,
    }

    impl DiffProvider for MockDiffProvider {
        fn get_changed_files(
            &self,
            _query: &DiffQuery,
        ) -> std::result::Result<Vec<(String, FileStatus)>, Box<dyn std::error::Error>> {
            Ok(self.files.clone())
        }

        fn get_file_stats(
            &self,
            _file_path: &str,
            _query: &DiffQuery,
        ) -> std::result::Result<FileStats, Box<dyn std::error::Error>> {
            Ok(FileStats::new(10, 5))
        }

        fn get_source_code(
            &self,
            _file_path: &str,
            git_ref: &crate::entities::git_ref::GitRef,
        ) -> Result<String, Box<dyn std::error::Error>> {
            // Return different content for old vs new to simulate a real diff
            match git_ref {
                crate::entities::git_ref::GitRef::Head => {
                    Ok("fn test() {\n    println!(\"old\");\n}".to_string())
                }
                crate::entities::git_ref::GitRef::Unstaged => {
                    Ok("fn test() {\n    println!(\"new\");\n}".to_string())
                }
                _ => Ok("test content".to_string()),
            }
        }
    }

    #[test]
    fn test_builder_with_empty_diff() {
        let provider = Box::new(MockDiffProvider { files: vec![] });
        let builder = ReviewEngineBuilder::new(provider, "test_author".to_string());

        let engine = builder
            .build(DiffQuery::new(GitRef::Head, GitRef::Unstaged))
            .unwrap();
        assert_eq!(engine.author(), "test_author");
        assert_eq!(engine.get_review_progress().total_reviewable_diffs, 0);
    }

    #[test]
    fn test_builder_with_single_file() {
        let provider = Box::new(MockDiffProvider {
            files: vec![("test.rs".to_string(), FileStatus::Modified)],
        });
        let builder = ReviewEngineBuilder::new(provider, "test_author".to_string());

        let engine = builder
            .build(DiffQuery::new(GitRef::Head, GitRef::Unstaged))
            .unwrap();
        assert_eq!(engine.author(), "test_author");
        assert_eq!(engine.get_review_progress().total_reviewable_diffs, 1);
        assert_eq!(engine.get_file_paths(), vec!["test.rs"]);
    }

    #[test]
    fn test_convenience_constructors() {
        let provider = Box::new(MockDiffProvider { files: vec![] });

        // Test working directory
        let engine =
            ReviewEngineBuilder::from_working_directory(provider, "author".to_string()).unwrap();
        assert_eq!(engine.author(), "author");

        let provider2 = Box::new(MockDiffProvider { files: vec![] });

        // Test commit comparison
        let engine2 = ReviewEngineBuilder::from_commit_comparison(
            provider2,
            "author".to_string(),
            "abc123".to_string(),
            "def456".to_string(),
        )
        .unwrap();
        assert_eq!(engine2.author(), "author");
    }

    #[test]
    fn test_file_filtering_mixed_supported_unsupported() {
        // Create a mix of supported and unsupported files
        // Only use languages that are fully implemented (Rust, Python, Go)
        let provider = Box::new(MockDiffProvider {
            files: vec![
                // Supported files (only use fully implemented languages)
                ("main.rs".to_string(), FileStatus::Modified),
                ("utils.py".to_string(), FileStatus::Modified),
                ("handler.go".to_string(), FileStatus::Modified),
                // Unsupported files
                ("README.md".to_string(), FileStatus::Modified),
                ("config.json".to_string(), FileStatus::Modified),
                ("styles.css".to_string(), FileStatus::Modified),
                ("data.xml".to_string(), FileStatus::Modified),
                ("binary.exe".to_string(), FileStatus::Modified),
            ],
        });

        let builder = ReviewEngineBuilder::new(provider, "test_author".to_string());
        let engine = builder
            .build(DiffQuery::new(GitRef::Head, GitRef::Unstaged))
            .unwrap();

        // Should only process the 3 supported files, not the 5 unsupported ones
        // Note: Some files might not generate reviewable diffs due to semantic analysis filtering
        let total_diffs = engine.get_review_progress().total_reviewable_diffs;
        assert!(
            total_diffs > 0 && total_diffs <= 3,
            "Expected 1-3 reviewable diffs, got {total_diffs}"
        );

        // Verify the file paths contain only supported files
        let file_paths = engine.get_file_paths();
        assert!(
            file_paths.len() <= 3,
            "Expected at most 3 file paths, got {}",
            file_paths.len()
        );

        // All file paths should be supported extensions
        for file_path in &file_paths {
            assert!(
                is_supported_file(file_path),
                "Unsupported file found: {file_path}"
            );
        }

        // Verify unsupported files are not included
        assert!(!file_paths.contains(&"README.md".to_string()));
        assert!(!file_paths.contains(&"config.json".to_string()));
        assert!(!file_paths.contains(&"styles.css".to_string()));
        assert!(!file_paths.contains(&"data.xml".to_string()));
        assert!(!file_paths.contains(&"binary.exe".to_string()));
    }

    #[test]
    fn test_file_filtering_all_unsupported() {
        // Test with only unsupported files
        let provider = Box::new(MockDiffProvider {
            files: vec![
                ("README.md".to_string(), FileStatus::Modified),
                ("config.json".to_string(), FileStatus::Modified),
                ("styles.css".to_string(), FileStatus::Modified),
                ("data.xml".to_string(), FileStatus::Modified),
            ],
        });

        let builder = ReviewEngineBuilder::new(provider, "test_author".to_string());
        let engine = builder
            .build(DiffQuery::new(GitRef::Head, GitRef::Unstaged))
            .unwrap();

        // Should process 0 files since all are unsupported
        assert_eq!(engine.get_review_progress().total_reviewable_diffs, 0);
        assert_eq!(engine.get_file_paths().len(), 0);
    }

    #[test]
    fn test_is_supported_file() {
        // Test supported extensions (fully implemented languages)
        assert!(is_supported_file("main.rs"));
        assert!(is_supported_file("script.py"));
        assert!(is_supported_file("handler.go"));
        assert!(is_supported_file("component.ts"));
        assert!(is_supported_file("component.tsx"));
        assert!(is_supported_file("script.js"));
        assert!(is_supported_file("component.jsx"));

        // Test unsupported extensions
        assert!(!is_supported_file("README.md"));
        assert!(!is_supported_file("config.json"));
        assert!(!is_supported_file("styles.css"));
        assert!(!is_supported_file("data.xml"));
        assert!(!is_supported_file("binary.exe"));
        assert!(!is_supported_file("Dockerfile"));
        assert!(!is_supported_file("makefile"));

        // Test edge cases
        assert!(!is_supported_file("no_extension"));
        assert!(!is_supported_file(""));
        assert!(!is_supported_file(".hidden"));
        assert!(is_supported_file("deeply/nested/path/file.rs"));
    }
}
