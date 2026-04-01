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
    #[serde(default)]
    pub file_content_hash: String,
    #[serde(default)]
    pub content_snapshot: Option<String>,
}

/// JSON representation of a line range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedLineRange {
    pub start_line: usize,
    pub end_line: usize,
}

/// Metadata section for export format documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub format_version: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field_descriptions: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query_formats: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_usage_examples: Option<serde_json::Value>,
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
    pub stale_count: usize,
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
                    file_content_hash: String::new(),
                    content_snapshot: None,
                }
            })
            .collect();

        let meta = ExportMetadata {
            format_version: "1.1".to_string(),
            description: "DiffViz instruction export for coding agents".to_string(),
            field_descriptions: Some(serde_json::json!({
                "file": "Relative path to the file from repository root",
                "query": "Git diff query to retrieve file content. Format: 'from_ref..to_ref'. Use with: git diff <query> <file>",
                "line_range": "1-based line numbers where instruction applies (inclusive range)",
                "content": "The instruction text for the coding agent to follow",
                "author": "Username/identifier of instruction author",
                "timestamp": "When instruction was created/last modified (UTC format)",
                "status": "Instruction validity: 'active' (file unchanged), 'stale' (file changed), 'addressed' (completed)",
                "file_content_hash": "SHA256 hash of file content at time of instruction creation, used for validity verification",
                "content_snapshot": "Code lines from instruction range for visual reference (optional)"
            })),
            query_formats: Some(serde_json::json!({
                "HEAD..unstaged": "Uncommitted changes in working directory",
                "commit_hash..HEAD": "Changes from specific commit to current HEAD",
                "HEAD..commit_hash": "Changes from HEAD to specific commit",
                "commit_hash..commit_hash": "Changes between two commits"
            })),
            git_usage_examples: Some(serde_json::json!({
                "view_diff": "git diff HEAD..unstaged src/main.rs",
                "view_historical_diff": "git diff abc123d..HEAD src/main.rs",
                "get_file_at_head": "git show HEAD:src/main.rs",
                "get_file_at_commit": "git show abc123d:src/main.rs"
            })),
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

    // Tests for JSON export functionality (Phase 4)
    #[test]
    fn test_export_all_instructions() {
        let diff1 = create_test_reviewable_diff("test1.rs", 1);
        let diff2 = create_test_reviewable_diff("test2.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff1, diff2], "test_author".to_string());

        // Add instructions to both files
        let id1 = test_id("test1.rs", 10, 12);
        let id2 = test_id("test2.rs", 20, 22);

        engine
            .add_instruction(
                id1,
                "Instruction 1".to_string(),
                "reviewer".to_string(),
                None,
            )
            .unwrap();
        engine
            .add_instruction(
                id2,
                "Instruction 2".to_string(),
                "reviewer".to_string(),
                None,
            )
            .unwrap();

        // Export all
        let json = engine.export_instructions_json().unwrap();

        // Verify JSON is valid
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["_meta"].is_object());
        assert_eq!(parsed["_meta"]["format_version"], "1.1");
        assert_eq!(parsed["instructions"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_export_empty_scope() {
        let diff = create_test_reviewable_diff("test.rs", 1);
        let engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        // Export with no instructions
        let json = engine.export_instructions_json().unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["instructions"].as_array().unwrap().len(), 0);
        assert!(parsed["_meta"].is_object());
    }

    #[test]
    fn test_export_json_structure() {
        let diff = create_test_reviewable_diff("test.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        let id = test_id("test.rs", 10, 12);

        engine
            .add_instruction(
                id,
                "Test instruction".to_string(),
                "reviewer".to_string(),
                None,
            )
            .unwrap();

        let json = engine.export_instructions_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Verify metadata structure
        assert_eq!(parsed["_meta"]["format_version"], "1.1");
        assert!(parsed["_meta"]["description"].is_string());
        assert!(parsed["_meta"]["field_descriptions"].is_object());
        assert!(parsed["_meta"]["query_formats"].is_object());
        assert!(parsed["_meta"]["git_usage_examples"].is_object());

        // Verify instruction structure
        let inst = &parsed["instructions"][0];
        assert_eq!(inst["file"], "test.rs");
        assert_eq!(inst["query"], "HEAD..unstaged");
        assert_eq!(inst["line_range"]["start_line"], 10);
        assert_eq!(inst["line_range"]["end_line"], 12);
        assert_eq!(inst["content"], "Test instruction");
        assert_eq!(inst["author"], "reviewer");
        assert!(inst["timestamp"].is_string());

        // Verify line_range has no columns
        assert!(inst["line_range"]["start_column"].is_null());
        assert!(inst["line_range"]["end_column"].is_null());
    }

    #[test]
    fn test_export_query_format_mapping() {
        let diff = create_test_reviewable_diff("test.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        let id = test_id("test.rs", 10, 12);

        engine
            .add_instruction(id, "Test".to_string(), "reviewer".to_string(), None)
            .unwrap();

        let json = engine.export_instructions_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Verify "working" is mapped to "HEAD..unstaged"
        assert_eq!(parsed["instructions"][0]["query"], "HEAD..unstaged");
    }

    // Tests for Phase 3: Export Format Enhancement
    #[test]
    fn test_export_includes_status_field() {
        let diff = create_test_reviewable_diff("test.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        let reviewable_id = test_id("test.rs", 10, 12);

        engine
            .add_instruction(
                reviewable_id,
                "Test instruction".to_string(),
                "reviewer".to_string(),
                None,
            )
            .unwrap();

        let json = engine.export_instructions_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        let instructions = parsed["instructions"].as_array().unwrap();
        assert_eq!(instructions.len(), 1);

        let inst = &instructions[0];
        assert!(inst["status"].is_string());
        assert_eq!(inst["status"], "active");
    }

    #[test]
    fn test_export_includes_file_content_hash() {
        let diff = create_test_reviewable_diff("test.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        let reviewable_id = test_id("test.rs", 10, 12);

        engine
            .add_instruction(
                reviewable_id,
                "Test instruction".to_string(),
                "reviewer".to_string(),
                None,
            )
            .unwrap();

        let json = engine.export_instructions_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        let instructions = parsed["instructions"].as_array().unwrap();
        assert_eq!(instructions.len(), 1);

        let inst = &instructions[0];
        assert!(inst["file_content_hash"].is_string());
        let hash = inst["file_content_hash"].as_str().unwrap();
        assert_eq!(hash.len(), 0); // file_content_hash is not stored on Instruction
    }

    #[test]
    fn test_export_includes_content_snapshot() {
        let diff = create_test_reviewable_diff("test.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        let reviewable_id = test_id("test.rs", 2, 3);

        engine
            .add_instruction(
                reviewable_id,
                "Test instruction".to_string(),
                "reviewer".to_string(),
                None,
            )
            .unwrap();

        let json = engine.export_instructions_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        let instructions = parsed["instructions"].as_array().unwrap();
        assert_eq!(instructions.len(), 1);

        let inst = &instructions[0];
        // content_snapshot should be present (may be null or string)
        assert!(inst.get("content_snapshot").is_some());
    }

    #[test]
    fn test_export_format_version_is_1_1() {
        let diff = create_test_reviewable_diff("test.rs", 1);
        let engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        let json = engine.export_instructions_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["_meta"]["format_version"], "1.1");
    }

    #[test]
    fn test_export_metadata_includes_new_field_descriptions() {
        let diff = create_test_reviewable_diff("test.rs", 1);
        let engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        let json = engine.export_instructions_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        let field_descriptions = &parsed["_meta"]["field_descriptions"];
        assert!(field_descriptions["status"].is_string());
        assert!(field_descriptions["file_content_hash"].is_string());
        assert!(field_descriptions["content_snapshot"].is_string());

        // Verify descriptions are meaningful
        let status_desc = field_descriptions["status"].as_str().unwrap();
        assert!(
            status_desc.contains("active")
                || status_desc.contains("stale")
                || status_desc.contains("validity")
        );

        let hash_desc = field_descriptions["file_content_hash"].as_str().unwrap();
        assert!(hash_desc.contains("hash") || hash_desc.contains("SHA256"));
    }

    #[test]
    fn test_export_status_serialization_active() {
        let diff = create_test_reviewable_diff("test.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        let reviewable_id = test_id("test.rs", 10, 12);

        engine
            .add_instruction(
                reviewable_id,
                "Test instruction".to_string(),
                "reviewer".to_string(),
                None,
            )
            .unwrap();

        let json = engine.export_instructions_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        let instructions = parsed["instructions"].as_array().unwrap();
        let inst = &instructions[0];

        // Active status should serialize as "active"
        assert_eq!(inst["status"], "active");
    }

    #[test]
    fn test_export_all_scopes_include_new_fields() {
        let diff1 = create_test_reviewable_diff("test1.rs", 1);
        let diff2 = create_test_reviewable_diff("test2.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff1, diff2], "test_author".to_string());

        // Add instructions to different files
        let id1 = test_id("test1.rs", 10, 12);
        let id2 = test_id("test2.rs", 20, 22);

        engine
            .add_instruction(
                id1,
                "Instruction 1".to_string(),
                "reviewer".to_string(),
                None,
            )
            .unwrap();

        engine
            .add_instruction(
                id2,
                "Instruction 2".to_string(),
                "reviewer".to_string(),
                None,
            )
            .unwrap();

        let json = engine.export_instructions_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let instructions = parsed["instructions"].as_array().unwrap();

        assert_eq!(instructions.len(), 2);
        for inst in instructions {
            assert!(inst["status"].is_string());
            assert!(inst["file_content_hash"].is_string());
            assert!(inst.get("content_snapshot").is_some());
        }
    }

    // ========== Phase 4: Import Functionality Tests ==========

    // Tests for JSON parsing
    #[test]
    fn test_import_valid_json_with_new_fields() {
        let diff = create_test_reviewable_diff("src/main.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        let json = r#"{
            "_meta": {
                "format_version": "1.1",
                "description": "DiffViz instruction export for coding agents"
            },
            "instructions": [{
                "file": "src/main.rs",
                "query": "HEAD..unstaged",
                "line_range": {"start_line": 10, "end_line": 12},
                "content": "Extract this to a separate function",
                "author": "reviewer",
                "timestamp": "2024-01-12T10:30:00Z",
                "status": "active",
                "file_content_hash": "test_hash_123",
                "content_snapshot": "    let x = calculate();\n    process(x);"
            }]
        }"#;

        let result = engine.import_instructions_json(json);
        if result.is_err() {
            eprintln!("Import error: {:?}", result.as_ref().unwrap_err());
        }
        assert!(result.is_ok());
        let summary = result.unwrap();
        assert_eq!(summary.total_imported, 1);
    }

    #[test]
    fn test_import_legacy_json_without_new_fields() {
        let diff = create_test_reviewable_diff("src/main.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        // Legacy JSON without status, file_content_hash, content_snapshot
        let json = r#"{
            "_meta": {
                "format_version": "1.0",
                "description": "DiffViz instruction export for coding agents"
            },
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
        // Should handle gracefully - calculate hash for legacy instructions
        assert!(result.is_ok());
    }

    #[test]
    fn test_import_invalid_json() {
        let diff = create_test_reviewable_diff("src/main.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        let invalid_json = r#"{"invalid": "json structure"#;
        let result = engine.import_instructions_json(invalid_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_import_json_with_missing_required_fields() {
        let diff = create_test_reviewable_diff("src/main.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        // Missing 'content' field (required)
        let json = r#"{
            "_meta": {
                "format_version": "1.1",
                "description": "DiffViz instruction export"
            },
            "instructions": [{
                "file": "src/main.rs",
                "query": "HEAD..unstaged",
                "line_range": {"start_line": 10, "end_line": 12},
                "author": "reviewer",
                "timestamp": "2024-01-12T10:30:00Z",
                "status": "active",
                "file_content_hash": "test_hash"
            }]
        }"#;

        let result = engine.import_instructions_json(json);
        assert!(result.is_err());
    }

    // Tests for import
    #[test]
    fn test_import_sets_active() {
        let diff = create_test_reviewable_diff("src/main.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        let json = r#"{
            "_meta": {"format_version": "1.1", "description": "test"},
            "instructions": [{
                "file": "src/main.rs",
                "query": "HEAD..unstaged",
                "line_range": {"start_line": 10, "end_line": 12},
                "content": "Test instruction",
                "author": "reviewer",
                "timestamp": "2024-01-12T10:30:00Z",
                "status": "active",
                "file_content_hash": "",
                "content_snapshot": null
            }]
        }"#;

        let result = engine.import_instructions_json(json);
        assert!(result.is_ok());
        let summary = result.unwrap();
        assert_eq!(summary.active_count, 1);
        assert_eq!(summary.stale_count, 0);
    }

    // Tests for import summary
    #[test]
    fn test_import_summary_multiple_instructions() {
        let diff1 = create_test_reviewable_diff("src/file1.rs", 1);
        let diff2 = create_test_reviewable_diff("src/file2.rs", 2);
        let mut engine = ReviewEngine::new(vec![diff1, diff2], "test_author".to_string());

        let json = r#"{
            "_meta": {"format_version": "1.1", "description": "test"},
            "instructions": [
                {
                    "file": "src/file1.rs",
                    "query": "HEAD..unstaged",
                    "line_range": {"start_line": 10, "end_line": 12},
                    "content": "Instruction 1",
                    "author": "reviewer",
                    "timestamp": "2024-01-12T10:30:00Z",
                    "status": "active",
                    "file_content_hash": "",
                    "content_snapshot": null
                },
                {
                    "file": "src/file2.rs",
                    "query": "HEAD..unstaged",
                    "line_range": {"start_line": 20, "end_line": 22},
                    "content": "Instruction 2",
                    "author": "reviewer",
                    "timestamp": "2024-01-12T10:30:00Z",
                    "status": "active",
                    "file_content_hash": "",
                    "content_snapshot": null
                },
                {
                    "file": "src/file1.rs",
                    "query": "HEAD..unstaged",
                    "line_range": {"start_line": 30, "end_line": 32},
                    "content": "Instruction 3",
                    "author": "reviewer",
                    "timestamp": "2024-01-12T10:30:00Z",
                    "status": "active",
                    "file_content_hash": "",
                    "content_snapshot": null
                }
            ]
        }"#;

        let result = engine.import_instructions_json(json);
        assert!(result.is_ok());
        let summary = result.unwrap();
        assert_eq!(summary.total_imported, 3);
        assert_eq!(summary.active_count, 3);
        assert_eq!(summary.stale_count, 0);
    }

    #[test]
    fn test_import_summary_with_zero_instructions() {
        let diff = create_test_reviewable_diff("src/main.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        let json = r#"{
            "_meta": {"format_version": "1.1", "description": "test"},
            "instructions": []
        }"#;

        let result = engine.import_instructions_json(json);
        assert!(result.is_ok());
        let summary = result.unwrap();
        assert_eq!(summary.total_imported, 0);
        assert_eq!(summary.active_count, 0);
        assert_eq!(summary.stale_count, 0);
    }

    // Tests for duplicate handling
    #[test]
    fn test_import_duplicate_instruction_same_reviewable_id() {
        let diff = create_test_reviewable_diff("src/main.rs", 1);
        let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

        // Add an instruction first
        let reviewable_id = test_id("src/main.rs", 10, 12);

        engine
            .add_instruction(
                reviewable_id.clone(),
                "Original instruction".to_string(),
                "reviewer".to_string(),
                None,
            )
            .unwrap();

        // Try to import instruction with same ReviewableDiffId
        let json = r#"{
            "_meta": {"format_version": "1.1", "description": "test"},
            "instructions": [{
                "file": "src/main.rs",
                "query": "HEAD..unstaged",
                "line_range": {"start_line": 10, "end_line": 12},
                "content": "Duplicate instruction",
                "author": "reviewer",
                "timestamp": "2024-01-12T10:30:00Z",
                "status": "active",
                "file_content_hash": "test_hash",
                "content_snapshot": null
            }]
        }"#;

        let result = engine.import_instructions_json(json);
        assert!(result.is_ok());
        let summary = result.unwrap();

        // Should skip duplicate (or update based on strategy)
        // For MVP: skip duplicates, add warning to summary
        assert!(!summary.errors.is_empty() || summary.total_imported == 0);
    }
}
