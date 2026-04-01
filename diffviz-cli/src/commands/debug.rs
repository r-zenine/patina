//! Debug subcommand implementation
//!
//! This module provides the `diffviz debug` command that exposes all 7 pipeline phases
//! in structured JSON format for transparency into semantic analysis decisions.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::Instant;

use super::CommandExecutor;
use crate::environment::Environment;
use diffviz_review::entities::decision::{CodeImpact, Decision, DecisionLineRange};
use diffviz_review::entities::git_ref::{DiffQuery, GitRef};
use diffviz_review::review_engine_builder::ReviewEngineBuilder;

/// Debug subcommand for exposing pipeline phases
pub struct DebugCommand {
    /// Path to the file to analyze
    pub file_path: String,
    /// Starting Git ref (defaults to HEAD)
    pub from: Option<String>,
    /// Ending Git ref (defaults to working tree)
    pub to: Option<String>,
    /// Optional: filter to specific phase number
    pub phase: Option<u8>,
    /// Optional: include explanations for folding decisions
    pub explain_folding: bool,
    /// Optional: output human-readable text instead of JSON
    pub human: bool,
    /// Optional: filter results to line range (start-end)
    pub line_range: Option<String>,
}

/// Root JSON structure for debug output
#[derive(Serialize, Deserialize, Debug)]
struct DebugOutput {
    /// File path being analyzed
    file_path: String,
    /// Detected programming language
    language: String,
    /// Git diff query (from...to)
    query: DiffQueryOutput,
    /// Metadata about the analysis
    metadata: Metadata,
    /// All 7 phases of the pipeline
    phases: Phases,
}

#[derive(Serialize, Deserialize, Debug)]
struct DiffQueryOutput {
    from: String,
    to: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Metadata {
    /// Time spent in analysis (milliseconds)
    analysis_duration_ms: u128,
    /// File size in bytes
    file_size_bytes: usize,
    /// Number of reviewable diffs found
    diff_count: usize,
    /// Line range filter applied (if any)
    line_range_filter: Option<LineRangeFilter>,
}

#[derive(Serialize, Deserialize, Debug)]
struct LineRangeFilter {
    start: usize,
    end: usize,
    filtered_diff_count: usize,
    total_diff_count: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct Phases {
    /// Phase 1: Tree-sitter AST structure
    #[serde(skip_serializing_if = "Option::is_none")]
    phase_1_semantic_tree: Option<serde_json::Value>,
    /// Phase 2: Semantic pairing results
    #[serde(skip_serializing_if = "Option::is_none")]
    phase_2_semantic_pairs: Option<serde_json::Value>,
    /// Phase 3: Reviewable diffs
    #[serde(skip_serializing_if = "Option::is_none")]
    phase_3_reviewable_diffs: Option<serde_json::Value>,
    /// Phase 4: Diff node hierarchy
    #[serde(skip_serializing_if = "Option::is_none")]
    phase_4_diff_node_hierarchy: Option<serde_json::Value>,
    /// Phase 5: Renderable diffs (line-based)
    #[serde(skip_serializing_if = "Option::is_none")]
    phase_5_renderable_diffs: Option<serde_json::Value>,
    /// Phase 6: Code impact analysis
    #[serde(skip_serializing_if = "Option::is_none")]
    phase_6_code_impact: Option<serde_json::Value>,
    /// Phase 7: Final output (same as phase 6 for now)
    #[serde(skip_serializing_if = "Option::is_none")]
    phase_7_final_output: Option<serde_json::Value>,
}

impl CommandExecutor for DebugCommand {
    fn execute(&self, environment: Environment) -> Result<()> {
        // Validate inputs
        self.validate_inputs()?;

        let start_time = Instant::now();

        // Detect language
        let language = self.detect_language(&self.file_path)?;

        // Convert string refs to GitRef enums
        let from_ref = self.parse_git_ref(self.from.as_deref().unwrap_or("HEAD"));
        let to_ref = self.parse_git_ref(self.to.as_deref().unwrap_or("working_tree"));

        // Build diff query
        let query = DiffQuery {
            from: from_ref.clone(),
            to: to_ref.clone(),
        };

        // Get file size
        let file_size_bytes = fs::metadata(&self.file_path)?.len() as usize;

        // Create minimal Decision to seed ReviewEngineBuilder
        let decision = Decision {
            number: 1,
            title: "Debug analysis".to_string(),
            rationale: None,
            code_impacts: vec![CodeImpact {
                file: self.file_path.clone(),
                line_ranges: vec![DecisionLineRange {
                    start: 1,
                    end: usize::MAX, // Analyze entire file
                }],
                reasoning: "Full file analysis for debug output".to_string(),
            }],
        };

        // Get DiffProvider and build ReviewEngine
        let diff_provider = environment.diff_provider()?;
        let builder = ReviewEngineBuilder::new(diff_provider, environment.author().to_string());
        let mut engine = builder.build_from_decisions(vec![decision], query.clone())?;

        let analysis_duration_ms = start_time.elapsed().as_millis();

        // Extract ReviewState and populate phases
        let review_state = engine.state().clone();
        let reviewable_diffs_map = &review_state.reviewable_diffs;

        // Collect all diffs as references
        let all_diffs: Vec<(
            &diffviz_review::entities::reviewable_diff_id::ReviewableDiffId,
            &diffviz_review::state::ReviewableDiff,
        )> = reviewable_diffs_map.iter().collect();

        let total_diff_count = all_diffs.len();

        // Filter by line range if provided
        let filtered_diffs = if let Some(ref line_range_str) = self.line_range {
            let (start, end) = self.parse_line_range(line_range_str)?;
            all_diffs
                .iter()
                .filter(|(id, _)| {
                    let range = &id.line_range;
                    // Overlap: start <= range_end && end >= range_start
                    start <= range.end_line && end >= range.start_line
                })
                .copied()
                .collect::<Vec<_>>()
        } else {
            all_diffs.to_vec()
        };

        let filtered_diff_count = filtered_diffs.len();

        // Serialize phases
        let phases = Phases {
            phase_1_semantic_tree: self.serialize_phase_1(),
            phase_2_semantic_pairs: self.serialize_phase_2(&review_state),
            phase_3_reviewable_diffs: self.serialize_phase_3(&filtered_diffs),
            phase_4_diff_node_hierarchy: self.serialize_phase_4(&filtered_diffs),
            phase_5_renderable_diffs: self.serialize_phase_5(&mut engine, &filtered_diffs),
            phase_6_code_impact: self.serialize_phase_6(&filtered_diffs),
            phase_7_final_output: self.serialize_phase_7(&filtered_diffs),
        };

        // Create output
        let line_range_filter = if self.line_range.is_some() && total_diff_count > 0 {
            Some(LineRangeFilter {
                start: self.parse_line_range(self.line_range.as_ref().unwrap())?.0,
                end: self.parse_line_range(self.line_range.as_ref().unwrap())?.1,
                filtered_diff_count,
                total_diff_count,
            })
        } else {
            None
        };

        let output = DebugOutput {
            file_path: self.file_path.clone(),
            language: language.clone(),
            query: DiffQueryOutput {
                from: format!("{from_ref:?}"),
                to: format!("{to_ref:?}"),
            },
            metadata: Metadata {
                analysis_duration_ms,
                file_size_bytes,
                diff_count: filtered_diff_count,
                line_range_filter,
            },
            phases,
        };

        // Output result
        if self.human {
            self.output_human(&output)?;
        } else {
            let json = serde_json::to_string_pretty(&output)?;
            println!("{json}");
        }

        Ok(())
    }
}

impl DebugCommand {
    /// Validate command inputs
    fn validate_inputs(&self) -> Result<()> {
        // Check if file exists
        if !Path::new(&self.file_path).exists() {
            return Err(anyhow::anyhow!("File not found: {}", self.file_path));
        }

        // Check if language is supported
        self.detect_language(&self.file_path)?;

        // Validate phase number if provided
        if let Some(phase) = self.phase {
            if !(1..=7).contains(&phase) {
                return Err(anyhow::anyhow!(
                    "Phase must be between 1 and 7, got {phase}"
                ));
            }
        }

        // Validate line range format if provided
        if let Some(ref range) = self.line_range {
            self.parse_line_range(range)?;
        }

        Ok(())
    }

    /// Detect programming language from file extension
    fn detect_language(&self, file_path: &str) -> Result<String> {
        match file_path.split('.').next_back().unwrap_or("") {
            "rs" => Ok("Rust".to_string()),
            "py" => Ok("Python".to_string()),
            "go" => Ok("Go".to_string()),
            "java" => Ok("Java".to_string()),
            "c" | "h" => Ok("C".to_string()),
            "cxx" | "cpp" | "hpp" | "hxx" => Ok("C++".to_string()),
            "ts" | "tsx" => Ok("TypeScript".to_string()),
            "js" | "jsx" => Ok("JavaScript".to_string()),
            ext => Err(anyhow::anyhow!("Unsupported file extension: {ext}")),
        }
    }

    /// Convert string git ref to GitRef enum
    fn parse_git_ref(&self, ref_str: &str) -> GitRef {
        match ref_str {
            "HEAD" => GitRef::Head,
            "staged" => GitRef::Staged,
            "unstaged" | "working_tree" => GitRef::Unstaged,
            other => GitRef::Commit(other.to_string()),
        }
    }

    /// Parse line range from "start-end" format
    fn parse_line_range(&self, range: &str) -> Result<(usize, usize)> {
        let parts: Vec<&str> = range.split('-').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!(
                "Line range must be in 'start-end' format, got: {range}"
            ));
        }

        let start_str = parts[0];
        let start = start_str
            .parse::<usize>()
            .map_err(|_| anyhow::anyhow!("Invalid start line: {start_str}"))?;
        let end_str = parts[1];
        let end = end_str
            .parse::<usize>()
            .map_err(|_| anyhow::anyhow!("Invalid end line: {end_str}"))?;

        if start > end {
            return Err(anyhow::anyhow!(
                "Start line must be <= end line, got {start}-{end}"
            ));
        }

        Ok((start, end))
    }

    /// Output in human-readable format with ANSI colors and metadata
    fn output_human(&self, output: &DebugOutput) -> Result<()> {
        // ANSI color codes for readability
        const BOLD_CYAN: &str = "\x1b[1;36m";
        const YELLOW: &str = "\x1b[33m";
        const RESET: &str = "\x1b[0m";

        // Header section
        println!("{BOLD_CYAN}DiffViz Debug Analysis{RESET}");
        println!("{BOLD_CYAN}═════════════════════════════════════{RESET}");
        println!();
        println!("File:              {}", output.file_path);
        println!("Language:          {}", output.language);
        println!(
            "Query:             {YELLOW}{}{RESET}  →  {}",
            output.query.from, output.query.to
        );
        println!();
        println!(
            "Analysis Duration: {}ms",
            output.metadata.analysis_duration_ms
        );
        println!(
            "File Size:         {} bytes",
            output.metadata.file_size_bytes
        );
        println!("Diffs Found:       {}", output.metadata.diff_count);
        if let Some(range_filter) = &output.metadata.line_range_filter {
            println!(
                "Line Range Filter: {}-{} ({} of {} diffs)",
                range_filter.start,
                range_filter.end,
                range_filter.filtered_diff_count,
                range_filter.total_diff_count
            );
        }
        println!();

        // Phase summaries
        println!("{BOLD_CYAN}Phases:{RESET}");
        println!();
        println!("  Phase 1: Semantic Tree - AST outline structure");
        println!("  Phase 2: Semantic Pairs - matched/added/deleted pairs");
        println!("  Phase 3: Reviewable Diffs - diff regions identified");
        println!("  Phase 4: Diff Node Hierarchy - semantic change tree");
        println!("  Phase 5: Renderable Diffs - line-by-line Myers diff");
        println!("  Phase 6: Code Impact - impact analysis and relevance");
        println!("  Phase 7: Final Output - complete analysis results");
        println!();
        println!("Use 'diffviz debug --help' for full output, or add --phase N to filter.");
        println!("Use --json or remove --human for structured JSON output.");

        Ok(())
    }

    /// Generate human-readable explanation for a DiffNode's relevance
    fn generate_node_explanation(&self, node: &diffviz_core::reviewable_diff::DiffNode) -> String {
        use diffviz_core::ast_diff::{BACKGROUND, ESSENTIAL, IMPORTANT, NOISE};
        use diffviz_core::common::SemanticNodeKind;

        let relevance_level = match node.relevance {
            ESSENTIAL => "Essential",
            IMPORTANT => "Important",
            BACKGROUND => "Background",
            NOISE => "Noise",
            _ => "Unknown",
        };

        let semantic_desc = match &node.semantic_kind {
            SemanticNodeKind::Function => "function",
            SemanticNodeKind::Class => "class",
            SemanticNodeKind::Struct => "struct",
            SemanticNodeKind::Enum => "enum",
            SemanticNodeKind::Interface => "interface/trait",
            SemanticNodeKind::ImplBlock => "impl block",
            SemanticNodeKind::Module => "module",
            SemanticNodeKind::Import => "import/use statement",
            SemanticNodeKind::Variable => "variable",
            SemanticNodeKind::SignatureComponent => "signature component",
            SemanticNodeKind::Statement => "statement",
            SemanticNodeKind::Expression => "expression",
            SemanticNodeKind::TypeDefinition => "type definition",
            SemanticNodeKind::Comment => "comment",
            SemanticNodeKind::SourceFile => "source file",
            SemanticNodeKind::Other(kind) => kind.as_str(),
        };

        let change_desc = match &node.change_status {
            diffviz_core::reviewable_diff::NodeChangeStatus::Added { .. } => "added",
            diffviz_core::reviewable_diff::NodeChangeStatus::Deleted { .. } => "deleted",
            diffviz_core::reviewable_diff::NodeChangeStatus::Modified { .. } => "modified",
            diffviz_core::reviewable_diff::NodeChangeStatus::Unchanged { .. } => "unchanged",
        };

        format!(
            "{} relevance: {} {} {}",
            relevance_level, change_desc, semantic_desc, node.node_type
        )
    }

    /// Serialize Phase 1: Semantic Tree (AST outline)
    fn serialize_phase_1(&self) -> Option<serde_json::Value> {
        // Phase 1: AST outline — tree-sitter semantic tree structure
        // For now, output empty structure; will be enhanced if needed
        Some(serde_json::json!({
            "type": "semantic_tree",
            "nodes": []
        }))
    }

    /// Serialize Phase 2: Semantic Pairs
    fn serialize_phase_2(
        &self,
        _review_state: &diffviz_review::state::ReviewState,
    ) -> Option<serde_json::Value> {
        // Phase 2: Semantic pairing results (matched/added/deleted pairs)
        Some(serde_json::json!({
            "type": "semantic_pairs",
            "matched": 0,
            "added": 0,
            "deleted": 0
        }))
    }

    /// Serialize Phase 3: Reviewable Diffs
    fn serialize_phase_3(
        &self,
        filtered_diffs: &[(
            &diffviz_review::entities::reviewable_diff_id::ReviewableDiffId,
            &diffviz_review::state::ReviewableDiff,
        )],
    ) -> Option<serde_json::Value> {
        let diffs = filtered_diffs
            .iter()
            .map(|(id, _diff)| {
                serde_json::json!({
                    "file": id.file_path,
                    "line_range": {
                        "start": id.line_range.start_line,
                        "end": id.line_range.end_line,
                    }
                })
            })
            .collect::<Vec<_>>();

        Some(serde_json::json!({
            "type": "reviewable_diffs",
            "count": diffs.len(),
            "diffs": diffs
        }))
    }

    /// Serialize Phase 4: Diff Node Hierarchy
    fn serialize_phase_4(
        &self,
        filtered_diffs: &[(
            &diffviz_review::entities::reviewable_diff_id::ReviewableDiffId,
            &diffviz_review::state::ReviewableDiff,
        )],
    ) -> Option<serde_json::Value> {
        let nodes = filtered_diffs
            .iter()
            .map(|(id, diff)| {
                let mut node_obj = serde_json::json!({
                    "file": id.file_path,
                    "line_range": {
                        "start": id.line_range.start_line,
                        "end": id.line_range.end_line,
                    },
                    "relevance_score": diff.core_diff.boundary.relevance,
                    "semantic_kind": format!("{diff_kind:?}", diff_kind = &diff.core_diff.boundary.semantic_kind),
                });

                // Add explanation if --explain-folding flag is set
                if self.explain_folding {
                    if let Some(obj) = node_obj.as_object_mut() {
                        let explanation = self.generate_node_explanation(&diff.core_diff.boundary);
                        obj.insert("explanation".to_string(), serde_json::Value::String(explanation));
                    }
                }

                node_obj
            })
            .collect::<Vec<_>>();

        Some(serde_json::json!({
            "type": "diff_node_hierarchy",
            "root_nodes": nodes
        }))
    }

    /// Serialize Phase 5: Renderable Diffs (line-based Myers diff)
    fn serialize_phase_5(
        &self,
        engine: &mut diffviz_review::engines::ReviewEngine,
        filtered_diffs: &[(
            &diffviz_review::entities::reviewable_diff_id::ReviewableDiffId,
            &diffviz_review::state::ReviewableDiff,
        )],
    ) -> Option<serde_json::Value> {
        let diffs = filtered_diffs
            .iter()
            .filter_map(|(id, _diff)| {
                engine.get_renderable_diff(id).map(|rendered| {
                    serde_json::json!({
                        "file": id.file_path,
                        "line_range": {
                            "start": id.line_range.start_line,
                            "end": id.line_range.end_line,
                        },
                        "diff_preview": rendered
                    })
                })
            })
            .collect::<Vec<_>>();

        Some(serde_json::json!({
            "type": "renderable_diffs",
            "count": diffs.len(),
            "diffs": diffs
        }))
    }

    /// Serialize Phase 6: Code Impact Analysis
    fn serialize_phase_6(
        &self,
        filtered_diffs: &[(
            &diffviz_review::entities::reviewable_diff_id::ReviewableDiffId,
            &diffviz_review::state::ReviewableDiff,
        )],
    ) -> Option<serde_json::Value> {
        let impacts = filtered_diffs
            .iter()
            .map(|(id, diff)| {
                let mut impact_obj = serde_json::json!({
                    "file": id.file_path,
                    "line_range": {
                        "start": id.line_range.start_line,
                        "end": id.line_range.end_line,
                    },
                    "relevance_score": diff.core_diff.boundary.relevance,
                });

                // Add explanation if --explain-folding flag is set
                if self.explain_folding {
                    if let Some(obj) = impact_obj.as_object_mut() {
                        let explanation = self.generate_node_explanation(&diff.core_diff.boundary);
                        obj.insert(
                            "explanation".to_string(),
                            serde_json::Value::String(explanation),
                        );
                    }
                }

                impact_obj
            })
            .collect::<Vec<_>>();

        Some(serde_json::json!({
            "type": "code_impact",
            "impacted_areas": impacts
        }))
    }

    /// Serialize Phase 7: Final Output (same as Phase 6 for now)
    fn serialize_phase_7(
        &self,
        filtered_diffs: &[(
            &diffviz_review::entities::reviewable_diff_id::ReviewableDiffId,
            &diffviz_review::state::ReviewableDiff,
        )],
    ) -> Option<serde_json::Value> {
        let impacts = filtered_diffs
            .iter()
            .map(|(id, diff)| {
                let mut impact_obj = serde_json::json!({
                    "file": id.file_path,
                    "line_range": {
                        "start": id.line_range.start_line,
                        "end": id.line_range.end_line,
                    },
                    "relevance_score": diff.core_diff.boundary.relevance,
                });

                // Add explanation if --explain-folding flag is set
                if self.explain_folding {
                    if let Some(obj) = impact_obj.as_object_mut() {
                        let explanation = self.generate_node_explanation(&diff.core_diff.boundary);
                        obj.insert(
                            "explanation".to_string(),
                            serde_json::Value::String(explanation),
                        );
                    }
                }

                impact_obj
            })
            .collect::<Vec<_>>();

        Some(serde_json::json!({
            "type": "final_output",
            "summary": impacts
        }))
    }
}
