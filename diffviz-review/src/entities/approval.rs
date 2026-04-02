//! Approval system for code review workflow

use crate::entities::reviewable_diff_id::ReviewableDiffId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;

/// Approval metadata stored per key in an ApprovalMap
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalRecord {
    pub approved: bool,
    pub approved_by: String,
    pub approval_timestamp: String,
}

/// Generic collection of approvals keyed by any hashable type.
///
/// Used for both chunk-level approvals (`K = ReviewableDiffId`) and
/// decision-level approvals (`K = u32`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(bound(
    serialize = "K: Serialize + Hash + Eq + Clone",
    deserialize = "K: for<'de2> Deserialize<'de2> + Hash + Eq + Clone"
))]
pub struct ApprovalMap<K: Hash + Eq + Clone> {
    pub approvals: HashMap<K, ApprovalRecord>,
}

impl<K: Hash + Eq + Clone> ApprovalMap<K> {
    pub fn new() -> Self {
        Self {
            approvals: HashMap::new(),
        }
    }

    pub fn approve(&mut self, key: K, approved_by: String, timestamp: String) {
        self.approvals.insert(
            key,
            ApprovalRecord {
                approved: true,
                approved_by,
                approval_timestamp: timestamp,
            },
        );
    }

    pub fn unapprove(&mut self, key: &K) {
        self.approvals.remove(key);
    }

    pub fn is_approved(&self, key: &K) -> bool {
        self.approvals
            .get(key)
            .is_some_and(|record| record.approved)
    }

    pub fn get_approval(&self, key: &K) -> Option<&ApprovalRecord> {
        self.approvals.get(key)
    }

    pub fn total_approved(&self) -> usize {
        self.approvals.values().filter(|r| r.approved).count()
    }

    pub fn approval_percentage(&self, total: usize) -> f32 {
        if total == 0 {
            0.0
        } else {
            (self.total_approved() as f32 / total as f32) * 100.0
        }
    }
}

/// Approvals keyed by ReviewableDiffId (chunk-level)
pub type ReviewApprovals = ApprovalMap<ReviewableDiffId>;

/// Approvals keyed by decision number
pub type DecisionApprovals = ApprovalMap<u32>;

// Keep Approval as a public type alias for backward compatibility in tests
pub type Approval = ApprovalRecord;

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

        assert!(!approvals.is_approved(&reviewable_id));
        assert_eq!(approvals.total_approved(), 0);

        approvals.approve(
            reviewable_id.clone(),
            "reviewer".to_string(),
            "2023-01-01T00:00:00Z".to_string(),
        );
        assert!(approvals.is_approved(&reviewable_id));
        assert_eq!(approvals.total_approved(), 1);

        let approval = approvals.get_approval(&reviewable_id).unwrap();
        assert_eq!(approval.approved_by, "reviewer");
        assert_eq!(approval.approval_timestamp, "2023-01-01T00:00:00Z");

        approvals.unapprove(&reviewable_id);
        assert!(!approvals.is_approved(&reviewable_id));
        assert_eq!(approvals.total_approved(), 0);
    }

    #[test]
    fn test_approval_percentage() {
        let mut approvals = ReviewApprovals::new();

        assert_eq!(approvals.approval_percentage(0), 0.0);

        let id1 = create_test_reviewable_id();
        approvals.approve(
            id1,
            "reviewer".to_string(),
            "2023-01-01T00:00:00Z".to_string(),
        );

        assert_eq!(approvals.approval_percentage(2), 50.0);
        assert_eq!(approvals.approval_percentage(1), 100.0);
    }

    #[test]
    fn test_decision_approvals() {
        let mut approvals = DecisionApprovals::new();

        assert!(!approvals.is_approved(&1));
        approvals.approve(
            1,
            "reviewer".to_string(),
            "2023-01-01T00:00:00Z".to_string(),
        );
        assert!(approvals.is_approved(&1));
        approvals.unapprove(&1);
        assert!(!approvals.is_approved(&1));
    }
}
