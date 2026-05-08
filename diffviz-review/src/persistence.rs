use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::entities::instruction::InstructionStatus;
use crate::summary::{
    ApprovedDecisionEntry, InstructionEntry, ReviewSummary, ReviewSummaryDecisions,
    ReviewSummaryInstructions, ReviewSummaryStats, UnapprovedDecisionEntry,
};
use crate::{
    ApprovalRecord, DecisionApprovals, DecisionLog, ReviewApprovals, ReviewEngine, ReviewableDiffId,
};

#[derive(Debug, thiserror::Error)]
pub enum PersistenceError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Import failed: {0}")]
    Import(String),
    #[error("Missing required file: {0}")]
    MissingFile(String),
}

#[derive(Serialize, Deserialize, Clone)]
struct PersistedApproval {
    reviewable_id: ReviewableDiffId,
    approved: bool,
    approved_by: String,
    approval_timestamp: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct PersistedDecisionApproval {
    decision_number: u32,
    approved: bool,
    approved_by: String,
    approval_timestamp: String,
}

#[derive(Serialize, Deserialize)]
struct ReviewStateFile {
    approvals: Vec<PersistedApproval>,
    instructions: serde_json::Value,
    #[serde(default)]
    decision_approvals: Vec<PersistedDecisionApproval>,
}

pub fn load_review_state(folder: &Path, engine: &mut ReviewEngine) -> Result<(), PersistenceError> {
    let path = folder.join("review-state.json");
    if !path.exists() {
        return Ok(());
    }
    let json = std::fs::read_to_string(&path)?;
    let state_file: ReviewStateFile = serde_json::from_str(&json)?;
    let mut approvals = ReviewApprovals::new();
    for a in state_file.approvals {
        approvals.approvals.insert(
            a.reviewable_id,
            ApprovalRecord {
                approved: a.approved,
                approved_by: a.approved_by,
                approval_timestamp: a.approval_timestamp,
            },
        );
    }
    engine.load_approvals(approvals);
    let mut decision_approvals = DecisionApprovals::new();
    for da in state_file.decision_approvals {
        decision_approvals.approvals.insert(
            da.decision_number,
            ApprovalRecord {
                approved: da.approved,
                approved_by: da.approved_by,
                approval_timestamp: da.approval_timestamp,
            },
        );
    }
    engine.load_decision_approvals(decision_approvals);
    let instructions_str = serde_json::to_string(&state_file.instructions)?;
    engine
        .import_instructions_json(&instructions_str)
        .map_err(|e| PersistenceError::Import(e.to_string()))?;
    Ok(())
}

/// Minimal JSON shapes needed to deserialize review-state.json for summarization
#[derive(Deserialize)]
struct SummaryDecisionApproval {
    decision_number: u32,
    approved_by: String,
    approval_timestamp: String,
}

#[derive(Deserialize)]
struct SummaryExportedLineRange {
    start_line: usize,
    end_line: usize,
}

#[derive(Deserialize)]
struct SummaryExportedInstruction {
    file: String,
    line_range: SummaryExportedLineRange,
    content: String,
    author: String,
    timestamp: String,
    #[serde(default)]
    status: InstructionStatus,
}

#[derive(Deserialize)]
struct SummaryExportedInstructions {
    instructions: Vec<SummaryExportedInstruction>,
}

#[derive(Deserialize)]
struct SummaryReviewStateFile {
    #[serde(default)]
    decision_approvals: Vec<SummaryDecisionApproval>,
    instructions: SummaryExportedInstructions,
}

pub fn summarize_review_state(folder: &Path) -> Result<ReviewSummary, PersistenceError> {
    let decision_log_path = folder.join("decision-log.yaml");
    if !decision_log_path.exists() {
        return Err(PersistenceError::MissingFile(
            decision_log_path.display().to_string(),
        ));
    }
    let yaml_content = std::fs::read_to_string(&decision_log_path)?;
    let log =
        DecisionLog::parse(&yaml_content).map_err(|e| PersistenceError::Parse(e.to_string()))?;

    let review_state_path = folder.join("review-state.json");
    let (decision_approvals, exported_instructions) = if review_state_path.exists() {
        let json = std::fs::read_to_string(&review_state_path)?;
        let state: SummaryReviewStateFile = serde_json::from_str(&json)?;
        (state.decision_approvals, state.instructions.instructions)
    } else {
        (vec![], vec![])
    };

    let approval_map: std::collections::HashMap<u32, &SummaryDecisionApproval> = decision_approvals
        .iter()
        .map(|a| (a.decision_number, a))
        .collect();

    let mut approved_decisions = Vec::new();
    let mut unapproved_decisions = Vec::new();
    for decision in &log.decisions {
        if let Some(approval) = approval_map.get(&decision.number) {
            approved_decisions.push(ApprovedDecisionEntry {
                number: decision.number,
                title: decision.title.clone(),
                approved_by: approval.approved_by.clone(),
                approval_timestamp: approval.approval_timestamp.clone(),
            });
        } else {
            unapproved_decisions.push(UnapprovedDecisionEntry {
                number: decision.number,
                title: decision.title.clone(),
                rationale: decision.rationale.clone(),
                code_impacts: decision.code_impacts.clone(),
            });
        }
    }

    let mut active_instructions = Vec::new();
    let mut addressed_instructions = Vec::new();
    for inst in exported_instructions {
        let entry = InstructionEntry {
            file: inst.file,
            lines: format!(
                "{}-{}",
                inst.line_range.start_line, inst.line_range.end_line
            ),
            content: inst.content,
            author: inst.author,
            timestamp: inst.timestamp,
        };
        match inst.status {
            InstructionStatus::Active => active_instructions.push(entry),
            InstructionStatus::Addressed => addressed_instructions.push(entry),
        }
    }

    let total_decisions = log.decisions.len();
    let approved_count = approved_decisions.len();
    let unapproved_count = unapproved_decisions.len();
    let total_instructions = active_instructions.len() + addressed_instructions.len();
    let active_count = active_instructions.len();

    Ok(ReviewSummary {
        commit: log.commit,
        contribution_folder: folder.display().to_string(),
        decisions: ReviewSummaryDecisions {
            approved: approved_decisions,
            unapproved: unapproved_decisions,
        },
        instructions: ReviewSummaryInstructions {
            active: active_instructions,
            addressed: addressed_instructions,
        },
        summary: ReviewSummaryStats {
            total_decisions,
            approved_decisions: approved_count,
            unapproved_decisions: unapproved_count,
            total_instructions,
            active_instructions: active_count,
        },
    })
}

pub fn save_review_state(folder: &Path, engine: &ReviewEngine) -> Result<(), PersistenceError> {
    let instructions_str = engine
        .export_instructions_json()
        .map_err(|e| PersistenceError::Import(e.to_string()))?;
    let instructions: serde_json::Value = serde_json::from_str(&instructions_str)?;
    let approvals_vec: Vec<PersistedApproval> = engine
        .state()
        .approvals
        .approvals
        .iter()
        .map(|(id, r)| PersistedApproval {
            reviewable_id: id.clone(),
            approved: r.approved,
            approved_by: r.approved_by.clone(),
            approval_timestamp: r.approval_timestamp.clone(),
        })
        .collect();
    let decision_approvals_vec: Vec<PersistedDecisionApproval> = engine
        .state()
        .decision_approvals
        .approvals
        .iter()
        .map(|(num, r)| PersistedDecisionApproval {
            decision_number: *num,
            approved: r.approved,
            approved_by: r.approved_by.clone(),
            approval_timestamp: r.approval_timestamp.clone(),
        })
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
