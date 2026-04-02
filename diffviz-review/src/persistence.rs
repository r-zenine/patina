use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{ApprovalRecord, DecisionApprovals, ReviewApprovals, ReviewEngine, ReviewableDiffId};

#[derive(Debug, thiserror::Error)]
pub enum PersistenceError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Import failed: {0}")]
    Import(String),
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

pub fn load_review_state(
    folder: &Path,
    engine: &mut ReviewEngine,
) -> Result<(), PersistenceError> {
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
