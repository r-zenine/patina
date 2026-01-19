//! Approval system for code review workflow
//!
//! This module contains the approval entities used in the ReviewableDiff-based
//! review system, allowing reviewers to approve or reject code changes.

use crate::entities::reviewable_diff_id::ReviewableDiffId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An approval record for a reviewable diff
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Approval {
    pub reviewable_id: ReviewableDiffId,
    pub approved: bool,
    pub approved_by: String,
    pub approval_timestamp: String,
}

/// Collection of approvals organized by ReviewableDiffId
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReviewApprovals {
    pub approvals: HashMap<ReviewableDiffId, Approval>,
}

impl ReviewApprovals {
    pub fn new() -> Self {
        Self {
            approvals: HashMap::new(),
        }
    }

    pub fn approve(
        &mut self,
        reviewable_id: ReviewableDiffId,
        approved_by: String,
        timestamp: String,
    ) {
        let approval = Approval {
            reviewable_id: reviewable_id.clone(),
            approved: true,
            approved_by,
            approval_timestamp: timestamp,
        };
        self.approvals.insert(reviewable_id, approval);
    }

    pub fn unapprove(&mut self, reviewable_id: &ReviewableDiffId) {
        self.approvals.remove(reviewable_id);
    }

    pub fn is_approved(&self, reviewable_id: &ReviewableDiffId) -> bool {
        self.approvals
            .get(reviewable_id)
            .is_some_and(|approval| approval.approved)
    }

    pub fn get_approval(&self, reviewable_id: &ReviewableDiffId) -> Option<&Approval> {
        self.approvals.get(reviewable_id)
    }

    pub fn total_approved(&self) -> usize {
        self.approvals
            .values()
            .filter(|approval| approval.approved)
            .count()
    }

    pub fn approval_percentage(&self, total_reviewable_diffs: usize) -> f32 {
        if total_reviewable_diffs == 0 {
            0.0
        } else {
            (self.total_approved() as f32 / total_reviewable_diffs as f32) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::git_ref::DiffQuery;
    use crate::entities::reviewable_diff_id::LineRange;

    fn create_test_reviewable_id() -> ReviewableDiffId {
        ReviewableDiffId::new(
            DiffQuery::head_to_unstaged(),
            "test.rs".to_string(),
            LineRange {
                start_line: 1,
                end_line: 10,
                start_column: 0,
                end_column: 0,
            },
        )
    }

    #[test]
    fn test_approval_system() {
        let mut approvals = ReviewApprovals::new();
        let reviewable_id = create_test_reviewable_id();

        // Initially not approved
        assert!(!approvals.is_approved(&reviewable_id));
        assert_eq!(approvals.total_approved(), 0);

        // Approve it
        approvals.approve(
            reviewable_id.clone(),
            "reviewer".to_string(),
            "2023-01-01T00:00:00Z".to_string(),
        );
        assert!(approvals.is_approved(&reviewable_id));
        assert_eq!(approvals.total_approved(), 1);

        // Verify approval details
        let approval = approvals.get_approval(&reviewable_id).unwrap();
        assert_eq!(approval.approved_by, "reviewer");
        assert_eq!(approval.approval_timestamp, "2023-01-01T00:00:00Z");

        // Unapprove it
        approvals.unapprove(&reviewable_id);
        assert!(!approvals.is_approved(&reviewable_id));
        assert_eq!(approvals.total_approved(), 0);
    }

    #[test]
    fn test_approval_percentage() {
        let mut approvals = ReviewApprovals::new();

        // Test with no diffs
        assert_eq!(approvals.approval_percentage(0), 0.0);

        // Test with some approved
        let id1 = create_test_reviewable_id();
        approvals.approve(
            id1,
            "reviewer".to_string(),
            "2023-01-01T00:00:00Z".to_string(),
        );

        assert_eq!(approvals.approval_percentage(2), 50.0);
        assert_eq!(approvals.approval_percentage(1), 100.0);
    }
}
