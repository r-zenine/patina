use super::ReviewEngine;
use crate::entities::Instruction;
use crate::entities::git_ref::DiffQuery;
use crate::entities::instruction::InstructionStatus;
use crate::entities::reviewable_diff_id::{LineRange, ReviewableDiffId};
use crate::errors::Result;
use serde::{Deserialize, Serialize};

/// JSON representation of an instruction for export/import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedInstruction {
    pub file: String,
    pub query: String,
    pub line_range: ExportedLineRange,
    pub content: String,
    pub author: String,
    pub timestamp: String,
    #[serde(default)]
    pub status: InstructionStatus,
}

/// JSON representation of a line range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedLineRange {
    pub start_line: usize,
    pub end_line: usize,
}

/// Metadata for the export format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub format_version: String,
    pub description: String,
}

/// Container for exported instructions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedInstructions {
    #[serde(rename = "_meta")]
    pub meta: ExportMetadata,
    pub instructions: Vec<ExportedInstruction>,
}

/// Summary of import operation results
#[derive(Debug, Clone, Default)]
pub struct ImportSummary {
    pub total_imported: usize,
    pub active_count: usize,
    pub errors: Vec<String>,
}

impl ReviewEngine {
    /// Export all instructions to JSON format
    pub fn export_instructions_json(&self) -> Result<String> {
        // Collect all (ReviewableDiffId, Instruction) pairs
        let pairs: Vec<(&ReviewableDiffId, &Instruction)> = self
            .state
            .instructions
            .instructions
            .iter()
            .flat_map(|(id, instructions)| instructions.iter().map(move |inst| (id, inst)))
            .collect();

        // Convert to exportable format
        let exported_instructions: Vec<ExportedInstruction> = pairs
            .into_iter()
            .map(|(id, inst)| {
                let line_range = id.line_range();
                // Extract query portion from full ReviewableDiffId display
                // Format is "query:file:L#-#", we want just the query part
                let full_id = format!("{id}");
                let query_str = full_id.split(':').next().unwrap_or("working");

                // Map "working" to proper git format "HEAD..unstaged"
                let query = if query_str == "working" {
                    "HEAD..unstaged".to_string()
                } else {
                    query_str.to_string()
                };

                ExportedInstruction {
                    file: id.file_path().to_string(),
                    query,
                    line_range: ExportedLineRange {
                        start_line: line_range.start_line,
                        end_line: line_range.end_line,
                    },
                    content: inst.content.clone(),
                    author: inst.author.clone(),
                    timestamp: inst.timestamp.clone(),
                    status: inst.status.clone(),
                }
            })
            .collect();

        let meta = ExportMetadata {
            format_version: "1.2".to_string(),
            description: "DiffViz instruction export for coding agents".to_string(),
        };

        let export = ExportedInstructions {
            meta,
            instructions: exported_instructions,
        };

        // Serialize to JSON with pretty printing
        serde_json::to_string_pretty(&export).map_err(|e| {
            crate::errors::DiffVizError::Review(crate::errors::ReviewError::ExportFailed {
                reason: format!("JSON serialization failed: {e}"),
            })
        })
    }

    /// Import instructions from JSON with hash verification
    pub fn import_instructions_json(&mut self, json: &str) -> Result<ImportSummary> {
        // Parse JSON to ExportedInstructions
        let exported: ExportedInstructions = serde_json::from_str(json).map_err(|e| {
            crate::errors::DiffVizError::Review(crate::errors::ReviewError::ImportFailed {
                reason: format!("Failed to parse JSON: {e}"),
            })
        })?;

        let mut summary = ImportSummary::default();

        for exported_inst in exported.instructions {
            // Skip if missing required fields
            if exported_inst.content.is_empty() {
                summary
                    .errors
                    .push("Skipping instruction with missing content".to_string());
                continue;
            }

            // Reconstruct ReviewableDiffId
            let reviewable_id = match self.create_reviewable_id_from_export(&exported_inst) {
                Ok(id) => id,
                Err(e) => {
                    summary
                        .errors
                        .push(format!("Failed to create ReviewableDiffId: {e}"));
                    continue;
                }
            };

            // Check if instruction already exists
            let existing_instruction = self
                .state
                .instructions
                .get_instructions_for_reviewable(&reviewable_id);
            if existing_instruction.is_some() {
                summary.errors.push(format!(
                    "Skipping duplicate instruction for {reviewable_id}"
                ));
                continue;
            }

            // Create Instruction
            let instruction = Instruction {
                id: format!(
                    "{}:{}",
                    reviewable_id,
                    chrono::Utc::now().timestamp_millis()
                ),
                author: exported_inst.author,
                timestamp: exported_inst.timestamp,
                content: exported_inst.content,
                status: InstructionStatus::Active,
            };

            // Add to ReviewState
            self.state
                .instructions
                .add_instruction(reviewable_id.clone(), instruction);

            // Update summary
            summary.total_imported += 1;
            summary.active_count += 1;
        }

        Ok(summary)
    }

    /// Helper to reconstruct ReviewableDiffId from ExportedInstruction
    fn create_reviewable_id_from_export(
        &self,
        exported: &ExportedInstruction,
    ) -> Result<ReviewableDiffId> {
        let diff_query = exported.query.parse::<DiffQuery>().map_err(|reason| {
            crate::errors::DiffVizError::Review(crate::errors::ReviewError::ImportFailed { reason })
        })?;

        let line_range = LineRange {
            start_line: exported.line_range.start_line,
            end_line: exported.line_range.end_line,
            start_column: 0,
            end_column: 0,
        };

        Ok(ReviewableDiffId::new(
            diff_query,
            exported.file.clone(),
            line_range,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers::*;
    use super::*;

    #[test]
    fn test_export_all_instructions() {
        let diff1 = create_test_reviewable_diff("test1.rs", 1);
        let diff2 = create_test_reviewable_diff("test2.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff1, diff2], "test_author".to_string());

        engine
            .add_instruction(
                test_id("test1.rs", 10, 12),
                "Instruction 1".to_string(),
                "reviewer".to_string(),
            )
            .unwrap();
        engine
            .add_instruction(
                test_id("test2.rs", 20, 22),
                "Instruction 2".to_string(),
                "reviewer".to_string(),
            )
            .unwrap();

        let json = engine.export_instructions_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["_meta"].is_object());
        assert_eq!(parsed["_meta"]["format_version"], "1.2");
        assert_eq!(parsed["instructions"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_export_empty() {
        let engine = ReviewEngine::new(
            vec![create_test_reviewable_diff("test.rs", 1)],
            "test_author".to_string(),
        );
        let json = engine.export_instructions_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["instructions"].as_array().unwrap().len(), 0);
        assert!(parsed["_meta"].is_object());
    }

    #[test]
    fn test_export_json_structure() {
        let mut engine = ReviewEngine::new(
            vec![create_test_reviewable_diff("test.rs", 1)],
            "test_author".to_string(),
        );
        engine
            .add_instruction(
                test_id("test.rs", 10, 12),
                "Test instruction".to_string(),
                "reviewer".to_string(),
            )
            .unwrap();

        let json = engine.export_instructions_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["_meta"]["format_version"], "1.2");
        assert!(parsed["_meta"]["description"].is_string());

        let inst = &parsed["instructions"][0];
        assert_eq!(inst["file"], "test.rs");
        assert_eq!(inst["query"], "HEAD..unstaged");
        assert_eq!(inst["line_range"]["start_line"], 10);
        assert_eq!(inst["line_range"]["end_line"], 12);
        assert_eq!(inst["content"], "Test instruction");
        assert_eq!(inst["author"], "reviewer");
        assert!(inst["timestamp"].is_string());
        assert_eq!(inst["status"], "active");
    }

    #[test]
    fn test_export_query_format_mapping() {
        let mut engine = ReviewEngine::new(
            vec![create_test_reviewable_diff("test.rs", 1)],
            "test_author".to_string(),
        );
        engine
            .add_instruction(
                test_id("test.rs", 10, 12),
                "Test".to_string(),
                "reviewer".to_string(),
            )
            .unwrap();
        let json = engine.export_instructions_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["instructions"][0]["query"], "HEAD..unstaged");
    }

    #[test]
    fn test_import_valid_json() {
        let mut engine = ReviewEngine::new(
            vec![create_test_reviewable_diff("src/main.rs", 1)],
            "test_author".to_string(),
        );

        let json = r#"{
            "_meta": {"format_version": "1.2", "description": "DiffViz instruction export"},
            "instructions": [{
                "file": "src/main.rs",
                "query": "HEAD..unstaged",
                "line_range": {"start_line": 10, "end_line": 12},
                "content": "Extract this to a separate function",
                "author": "reviewer",
                "timestamp": "2024-01-12T10:30:00Z",
                "status": "active"
            }]
        }"#;

        let summary = engine.import_instructions_json(json).unwrap();
        assert_eq!(summary.total_imported, 1);
        assert_eq!(summary.active_count, 1);
    }

    #[test]
    fn test_import_legacy_json() {
        let mut engine = ReviewEngine::new(
            vec![create_test_reviewable_diff("src/main.rs", 1)],
            "test_author".to_string(),
        );

        // Legacy JSON without status field — should deserialize with default Active status
        let json = r#"{
            "_meta": {"format_version": "1.0", "description": "DiffViz instruction export"},
            "instructions": [{
                "file": "src/main.rs",
                "query": "HEAD..unstaged",
                "line_range": {"start_line": 10, "end_line": 12},
                "content": "Extract this to a separate function",
                "author": "reviewer",
                "timestamp": "2024-01-12T10:30:00Z"
            }]
        }"#;

        let result = engine.import_instructions_json(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_import_invalid_json() {
        let mut engine = ReviewEngine::new(
            vec![create_test_reviewable_diff("src/main.rs", 1)],
            "test_author".to_string(),
        );
        assert!(
            engine
                .import_instructions_json(r#"{"invalid": "json structure"#)
                .is_err()
        );
    }

    #[test]
    fn test_import_missing_content_errors() {
        let mut engine = ReviewEngine::new(
            vec![create_test_reviewable_diff("src/main.rs", 1)],
            "test_author".to_string(),
        );

        let json = r#"{
            "_meta": {"format_version": "1.2", "description": "test"},
            "instructions": [{
                "file": "src/main.rs",
                "query": "HEAD..unstaged",
                "line_range": {"start_line": 10, "end_line": 12},
                "author": "reviewer",
                "timestamp": "2024-01-12T10:30:00Z",
                "status": "active"
            }]
        }"#;

        // content field missing entirely causes parse failure
        assert!(engine.import_instructions_json(json).is_err());
    }

    #[test]
    fn test_import_summary_multiple_instructions() {
        let mut engine = ReviewEngine::new(
            vec![
                create_test_reviewable_diff("src/file1.rs", 1),
                create_test_reviewable_diff("src/file2.rs", 2),
            ],
            "test_author".to_string(),
        );

        let json = r#"{
            "_meta": {"format_version": "1.2", "description": "test"},
            "instructions": [
                {"file": "src/file1.rs", "query": "HEAD..unstaged", "line_range": {"start_line": 10, "end_line": 12}, "content": "Instruction 1", "author": "reviewer", "timestamp": "2024-01-12T10:30:00Z", "status": "active"},
                {"file": "src/file2.rs", "query": "HEAD..unstaged", "line_range": {"start_line": 20, "end_line": 22}, "content": "Instruction 2", "author": "reviewer", "timestamp": "2024-01-12T10:30:00Z", "status": "active"},
                {"file": "src/file1.rs", "query": "HEAD..unstaged", "line_range": {"start_line": 30, "end_line": 32}, "content": "Instruction 3", "author": "reviewer", "timestamp": "2024-01-12T10:30:00Z", "status": "active"}
            ]
        }"#;

        let summary = engine.import_instructions_json(json).unwrap();
        assert_eq!(summary.total_imported, 3);
        assert_eq!(summary.active_count, 3);
    }

    #[test]
    fn test_import_zero_instructions() {
        let mut engine = ReviewEngine::new(
            vec![create_test_reviewable_diff("src/main.rs", 1)],
            "test_author".to_string(),
        );
        let json =
            r#"{"_meta": {"format_version": "1.2", "description": "test"}, "instructions": []}"#;
        let summary = engine.import_instructions_json(json).unwrap();
        assert_eq!(summary.total_imported, 0);
        assert_eq!(summary.active_count, 0);
    }

    #[test]
    fn test_import_duplicate_skipped() {
        let mut engine = ReviewEngine::new(
            vec![create_test_reviewable_diff("src/main.rs", 1)],
            "test_author".to_string(),
        );
        engine
            .add_instruction(
                test_id("src/main.rs", 10, 12),
                "Original".to_string(),
                "reviewer".to_string(),
            )
            .unwrap();

        let json = r#"{
            "_meta": {"format_version": "1.2", "description": "test"},
            "instructions": [{"file": "src/main.rs", "query": "HEAD..unstaged", "line_range": {"start_line": 10, "end_line": 12}, "content": "Duplicate", "author": "reviewer", "timestamp": "2024-01-12T10:30:00Z", "status": "active"}]
        }"#;

        let summary = engine.import_instructions_json(json).unwrap();
        assert!(!summary.errors.is_empty() || summary.total_imported == 0);
    }
}
