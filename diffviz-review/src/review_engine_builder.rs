//! ReviewEngine builder for git-to-ReviewEngine integration at the CLI layer
//!
//! This module provides the ReviewEngineBuilder that orchestrates between the git,
//! core, and review layers to create fully populated ReviewEngine instances.

use crate::engines::ReviewEngine;
use crate::entities::decision::{Decision, ReviewDecisions};
use crate::entities::git_ref::DiffQuery;
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
                let new_source_str = self
                    .diff_provider
                    .get_source_code(file_path, &query.to)
                    .ok();
                let old_source_str = self
                    .diff_provider
                    .get_source_code(file_path, &query.from)
                    .ok();

                if new_source_str.is_none() && old_source_str.is_none() {
                    continue;
                }

                for range in &code_impact.line_ranges {
                    let new_source_str = match &new_source_str {
                        Some(s) => s.clone(),
                        None => continue, // file deleted — skip this range
                    };

                    // Create providers for the sources
                    let new_provider = Box::new(SourceCode::new(new_source_str.clone()))
                        as Box<dyn diffviz_core::ast_diff::SourceProvider>;
                    let old_provider = old_source_str.as_ref().map(|src| {
                        Box::new(SourceCode::new(src.clone()))
                            as Box<dyn diffviz_core::ast_diff::SourceProvider>
                    });

                    // Call decision-based diff creation (returns one or more diffs)
                    let core_diffs = match create_reviewable_diff_from_range(
                        file_path,
                        range.start,
                        range.end,
                        old_provider.as_deref(),
                        new_provider.as_ref(),
                        language,
                        parser.as_ref(),
                    ) {
                        Ok(diffs) => diffs,
                        Err(
                            diffviz_core::decision_based_diff::DecisionDiffError::NoUnitsInRange {
                                ..
                            },
                        ) => {
                            eprintln!(
                                "Skipping range {}:{}-{} in decision {}: no semantic units found",
                                file_path, range.start, range.end, decision.number
                            );
                            continue;
                        }
                        Err(e) => {
                            return Err(crate::errors::DiffVizError::ProcessingFailed(format!(
                                "Failed to create diff for {} (decision {}): {}",
                                file_path, decision.number, e
                            )));
                        }
                    };

                    let new_source_provider = SourceCode::new(new_source_str);
                    let old_source_provider =
                        old_source_str.as_ref().map(|s| SourceCode::new(s.clone()));

                    for core_diff in core_diffs {
                        let line_range = extract_line_range_from_core_diff(
                            &core_diff,
                            old_source_provider.as_ref().unwrap_or(&new_source_provider),
                            &new_source_provider,
                        )
                        .ok_or_else(|| {
                            crate::errors::DiffVizError::ProcessingFailed(
                                "Failed to extract line range from diff".to_string(),
                            )
                        })?;

                        // Create review-layer ReviewableDiff with the actual file path
                        let reviewable_id =
                            ReviewableDiffId::new(query.clone(), file_path.to_string(), line_range);

                        // Populate decision_index directly — no post-hoc overlap detection needed
                        review_decisions
                            .decision_index
                            .entry(reviewable_id.clone())
                            .or_default()
                            .push(decision.number);

                        let reviewable_diff =
                            ReviewableDiff::new(reviewable_id, core_diff, file_path.to_string());
                        all_reviewable_diffs.push(reviewable_diff);
                    }
                }
            }
        }

        // Create engine with ReviewableDiffs
        let mut engine = ReviewEngine::new(all_reviewable_diffs, self.author);

        // Set decisions directly — index was already populated during construction
        engine.set_decisions(review_decisions);

        Ok(engine)
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

#[cfg(test)]
#[path = "review_engine_builder_tests.rs"]
mod tests;

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
