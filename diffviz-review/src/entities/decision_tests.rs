use super::*;
use crate::entities::git_ref::DiffQuery;
use crate::entities::reviewable_diff_id::LineRange;
use crate::state::ReviewableDiff;

#[test]
fn test_decision_log_parse_deserializes_correctly() {
    let yaml = r#"
commit: "abc123"
decisions:
  - number: 1
    title: "Use Core-then-Integrate strategy"
    rationale: "Three independent layers with a clear dependency order."
    code_impacts:
      - file: "diffviz-review/src/entities/decision.rs"
        reasoning: "Add DecisionLog struct here"
        line_ranges:
          - start: 1
            end: 36
  - number: 2
    title: "DecisionLog lives in diffviz-review"
    code_impacts: []
"#;
    let log = DecisionLog::parse(yaml).unwrap();

    assert_eq!(log.commit, "abc123");
    assert_eq!(log.decisions.len(), 2);
    assert_eq!(log.decisions[0].number, 1);
    assert_eq!(log.decisions[0].title, "Use Core-then-Integrate strategy");
    assert_eq!(
        log.decisions[0].rationale,
        Some("Three independent layers with a clear dependency order.".to_string())
    );
    assert_eq!(log.decisions[0].code_impacts.len(), 1);
    assert_eq!(
        log.decisions[0].code_impacts[0].file,
        "diffviz-review/src/entities/decision.rs"
    );
    assert_eq!(log.decisions[0].code_impacts[0].line_ranges[0].start, 1);
    assert_eq!(log.decisions[0].code_impacts[0].line_ranges[0].end, 36);
    assert_eq!(log.decisions[1].number, 2);
    assert_eq!(log.decisions[1].rationale, None);
    assert!(log.decisions[1].code_impacts.is_empty());
}

#[test]
fn test_decision_log_parse_invalid_yaml_returns_err() {
    let result = DecisionLog::parse("not: valid: yaml: [{{");
    assert!(result.is_err());
}
use diffviz_core::ast_diff::{OwnedNodeData, SourceCode};
use diffviz_core::common::{ProgrammingLanguage, SemanticNodeKind};
use diffviz_core::reviewable_diff::{
    DiffMetadata, DiffNode, NodeChangeStatus, ReviewableDiff as CoreReviewableDiff,
};
use std::collections::HashMap;

fn create_test_decision() -> Decision {
    Decision {
        number: 1,
        title: "Refactor authentication module".to_string(),
        rationale: Some("Extract auth logic into separate module for clarity".to_string()),
        code_impacts: vec![CodeImpact {
            file: "src/auth.rs".to_string(),
            line_ranges: vec![DecisionLineRange { start: 10, end: 50 }],
            reasoning: "New authentication module implementation".to_string(),
        }],
    }
}

fn create_test_reviewable_diff(
    file_path: &str,
    start_line: usize,
    end_line: usize,
) -> ReviewableDiff {
    let reviewable_id = ReviewableDiffId::new(
        DiffQuery::head_to_unstaged(),
        file_path.to_string(),
        LineRange {
            start_line,
            end_line,
            start_column: 0,
            end_column: 0,
        },
    );

    let placeholder_content = format!("content for {file_path}");
    let old_source = Box::new(SourceCode::new(placeholder_content.clone()));
    let new_source = Box::new(SourceCode::new(placeholder_content.clone()));

    let core_diff = CoreReviewableDiff {
        language: ProgrammingLanguage::Rust,
        boundary: DiffNode {
            node_type: "test".to_string(),
            semantic_kind: SemanticNodeKind::Other("test".to_string()),
            change_status: NodeChangeStatus::Unchanged {
                node: OwnedNodeData {
                    start_byte: 0,
                    end_byte: placeholder_content.len(),
                    kind: "test".to_string(),
                },
            },
            relevance: 0,
            children: vec![],
        },
        old_source,
        new_source,
        metadata: DiffMetadata {
            total_changes: 1,
            change_summary: HashMap::new(),
            essential_node_count: 1,
            analysis_duration_ms: 0,
        },
    };

    ReviewableDiff {
        id: reviewable_id,
        core_diff,
        file_path: file_path.to_string(),
    }
}

#[test]
fn test_review_decisions_add_and_get() {
    let mut decisions = ReviewDecisions::new();
    let decision = create_test_decision();

    decisions.add_decision(decision.clone());

    assert_eq!(decisions.get_decision(1), Some(&decision));
}

#[test]
fn test_review_decisions_all_decisions() {
    let mut decisions = ReviewDecisions::new();

    decisions.add_decision(Decision {
        number: 2,
        title: "Second decision".to_string(),
        rationale: None,
        code_impacts: vec![],
    });

    decisions.add_decision(Decision {
        number: 1,
        title: "First decision".to_string(),
        rationale: None,
        code_impacts: vec![],
    });

    let all = decisions.all_decisions();
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].number, 1);
    assert_eq!(all[1].number, 2);
}

#[test]
fn test_build_index_exact_overlap() {
    let mut decisions = ReviewDecisions::new();
    decisions.add_decision(Decision {
        number: 1,
        title: "Decision 1".to_string(),
        rationale: None,
        code_impacts: vec![CodeImpact {
            file: "src/auth.rs".to_string(),
            line_ranges: vec![DecisionLineRange { start: 10, end: 20 }],
            reasoning: "Auth changes".to_string(),
        }],
    });

    let diff = create_test_reviewable_diff("src/auth.rs", 10, 20);
    let review_state = ReviewState::new(vec![diff.clone()], "author".to_string());

    decisions.build_index_from_review_state(&review_state);

    let decision_list = decisions.get_decisions_for_diff(&diff.id);
    assert_eq!(decision_list.len(), 1);
    assert_eq!(decision_list[0].number, 1);
}

#[test]
fn test_build_index_partial_overlap() {
    let mut decisions = ReviewDecisions::new();
    decisions.add_decision(Decision {
        number: 1,
        title: "Decision 1".to_string(),
        rationale: None,
        code_impacts: vec![CodeImpact {
            file: "src/auth.rs".to_string(),
            line_ranges: vec![DecisionLineRange { start: 10, end: 30 }],
            reasoning: "Auth changes".to_string(),
        }],
    });

    // Diff that partially overlaps: 15-25 overlaps with decision range 10-30
    let diff = create_test_reviewable_diff("src/auth.rs", 15, 25);
    let review_state = ReviewState::new(vec![diff.clone()], "author".to_string());

    decisions.build_index_from_review_state(&review_state);

    let decision_list = decisions.get_decisions_for_diff(&diff.id);
    assert_eq!(decision_list.len(), 1);
    assert_eq!(decision_list[0].number, 1);
}

#[test]
fn test_build_index_no_overlap() {
    let mut decisions = ReviewDecisions::new();
    decisions.add_decision(Decision {
        number: 1,
        title: "Decision 1".to_string(),
        rationale: None,
        code_impacts: vec![CodeImpact {
            file: "src/auth.rs".to_string(),
            line_ranges: vec![DecisionLineRange { start: 10, end: 20 }],
            reasoning: "Auth changes".to_string(),
        }],
    });

    // Diff that doesn't overlap: 30-40 doesn't overlap with decision range 10-20
    let diff = create_test_reviewable_diff("src/auth.rs", 30, 40);
    let review_state = ReviewState::new(vec![diff.clone()], "author".to_string());

    decisions.build_index_from_review_state(&review_state);

    let decision_list = decisions.get_decisions_for_diff(&diff.id);
    assert_eq!(decision_list.len(), 0);
}

#[test]
fn test_build_index_different_file_no_match() {
    let mut decisions = ReviewDecisions::new();
    decisions.add_decision(Decision {
        number: 1,
        title: "Decision 1".to_string(),
        rationale: None,
        code_impacts: vec![CodeImpact {
            file: "src/auth.rs".to_string(),
            line_ranges: vec![DecisionLineRange { start: 10, end: 20 }],
            reasoning: "Auth changes".to_string(),
        }],
    });

    // Diff in different file should not match
    let diff = create_test_reviewable_diff("src/other.rs", 10, 20);
    let review_state = ReviewState::new(vec![diff.clone()], "author".to_string());

    decisions.build_index_from_review_state(&review_state);

    let decision_list = decisions.get_decisions_for_diff(&diff.id);
    assert_eq!(decision_list.len(), 0);
}

#[test]
fn test_build_index_multiple_decisions() {
    let mut decisions = ReviewDecisions::new();
    decisions.add_decision(Decision {
        number: 1,
        title: "Decision 1".to_string(),
        rationale: None,
        code_impacts: vec![CodeImpact {
            file: "src/auth.rs".to_string(),
            line_ranges: vec![DecisionLineRange { start: 10, end: 30 }],
            reasoning: "Auth changes".to_string(),
        }],
    });

    decisions.add_decision(Decision {
        number: 2,
        title: "Decision 2".to_string(),
        rationale: None,
        code_impacts: vec![CodeImpact {
            file: "src/auth.rs".to_string(),
            line_ranges: vec![DecisionLineRange { start: 15, end: 25 }],
            reasoning: "More auth changes".to_string(),
        }],
    });

    // Diff in range that overlaps both decisions
    let diff = create_test_reviewable_diff("src/auth.rs", 18, 22);
    let review_state = ReviewState::new(vec![diff.clone()], "author".to_string());

    decisions.build_index_from_review_state(&review_state);

    let decision_list = decisions.get_decisions_for_diff(&diff.id);
    assert_eq!(decision_list.len(), 2);
    assert_eq!(decision_list[0].number, 1);
    assert_eq!(decision_list[1].number, 2);
}

#[test]
fn test_build_index_nested_range() {
    let mut decisions = ReviewDecisions::new();
    decisions.add_decision(Decision {
        number: 1,
        title: "Decision 1".to_string(),
        rationale: None,
        code_impacts: vec![CodeImpact {
            file: "src/auth.rs".to_string(),
            line_ranges: vec![DecisionLineRange { start: 10, end: 50 }],
            reasoning: "Large refactor".to_string(),
        }],
    });

    // Diff nested inside decision range
    let diff = create_test_reviewable_diff("src/auth.rs", 20, 30);
    let review_state = ReviewState::new(vec![diff.clone()], "author".to_string());

    decisions.build_index_from_review_state(&review_state);

    let decision_list = decisions.get_decisions_for_diff(&diff.id);
    assert_eq!(decision_list.len(), 1);
    assert_eq!(decision_list[0].number, 1);
}

#[test]
fn test_decision_serialization() {
    let decision = create_test_decision();
    let json = serde_json::to_string(&decision).unwrap();
    let deserialized: Decision = serde_json::from_str(&json).unwrap();
    assert_eq!(decision, deserialized);
}

#[test]
fn test_create_unmapped_decision_with_unmapped_diffs() {
    let mut decisions = ReviewDecisions::new();
    decisions.add_decision(Decision {
        number: 1,
        title: "Decision 1".to_string(),
        rationale: None,
        code_impacts: vec![CodeImpact {
            file: "src/auth.rs".to_string(),
            line_ranges: vec![DecisionLineRange { start: 10, end: 20 }],
            reasoning: "Auth changes".to_string(),
        }],
    });

    // Create two diffs: one mapped, one unmapped
    let mapped_diff = create_test_reviewable_diff("src/auth.rs", 10, 20);
    let unmapped_diff = create_test_reviewable_diff("src/other.rs", 5, 15);

    let review_state = ReviewState::new(
        vec![mapped_diff.clone(), unmapped_diff.clone()],
        "author".to_string(),
    );

    // Build index first to map Decision 1
    decisions.build_index_from_review_state(&review_state);

    // Create Decision 0 for unmapped
    decisions.create_unmapped_decision(&review_state);

    // Verify Decision 0 exists
    let decision_0 = decisions.get_decision(0);
    assert!(decision_0.is_some());
    assert_eq!(decision_0.unwrap().title, "Unmapped Changes");

    // Verify unmapped diff is mapped to Decision 0
    let decisions_for_unmapped = decisions.get_decisions_for_diff(&unmapped_diff.id);
    assert_eq!(decisions_for_unmapped.len(), 1);
    assert_eq!(decisions_for_unmapped[0].number, 0);

    // Verify mapped diff is still mapped to Decision 1 (not Decision 0)
    let decisions_for_mapped = decisions.get_decisions_for_diff(&mapped_diff.id);
    assert_eq!(decisions_for_mapped.len(), 1);
    assert_eq!(decisions_for_mapped[0].number, 1);
}

#[test]
fn test_create_unmapped_decision_with_no_unmapped_diffs() {
    let mut decisions = ReviewDecisions::new();
    decisions.add_decision(Decision {
        number: 1,
        title: "Decision 1".to_string(),
        rationale: None,
        code_impacts: vec![CodeImpact {
            file: "src/auth.rs".to_string(),
            line_ranges: vec![DecisionLineRange { start: 10, end: 20 }],
            reasoning: "Auth changes".to_string(),
        }],
    });

    // Create only mapped diffs
    let diff1 = create_test_reviewable_diff("src/auth.rs", 10, 20);
    let review_state = ReviewState::new(vec![diff1], "author".to_string());

    decisions.build_index_from_review_state(&review_state);
    decisions.create_unmapped_decision(&review_state);

    // Verify Decision 0 was NOT created
    let decision_0 = decisions.get_decision(0);
    assert!(decision_0.is_none());

    // Verify we still only have Decision 1
    let all = decisions.all_decisions();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].number, 1);
}

#[test]
fn test_create_unmapped_decision_with_all_unmapped() {
    let mut decisions = ReviewDecisions::new();

    // Create only unmapped diffs
    let diff1 = create_test_reviewable_diff("src/auth.rs", 10, 20);
    let diff2 = create_test_reviewable_diff("src/other.rs", 5, 15);
    let review_state = ReviewState::new(vec![diff1.clone(), diff2.clone()], "author".to_string());

    decisions.build_index_from_review_state(&review_state);
    decisions.create_unmapped_decision(&review_state);

    // Verify Decision 0 exists
    let decision_0 = decisions.get_decision(0);
    assert!(decision_0.is_some());

    // Verify both diffs are mapped to Decision 0
    let decisions_for_diff1 = decisions.get_decisions_for_diff(&diff1.id);
    assert_eq!(decisions_for_diff1.len(), 1);
    assert_eq!(decisions_for_diff1[0].number, 0);

    let decisions_for_diff2 = decisions.get_decisions_for_diff(&diff2.id);
    assert_eq!(decisions_for_diff2.len(), 1);
    assert_eq!(decisions_for_diff2[0].number, 0);

    // Verify Decision 0 has correct code impacts
    let code_impacts = &decision_0.unwrap().code_impacts;
    assert_eq!(code_impacts.len(), 2);

    // Verify line ranges are captured correctly
    let auth_impact = code_impacts.iter().find(|c| c.file == "src/auth.rs");
    assert!(auth_impact.is_some());
    assert_eq!(auth_impact.unwrap().line_ranges[0].start, 10);
    assert_eq!(auth_impact.unwrap().line_ranges[0].end, 20);

    let other_impact = code_impacts.iter().find(|c| c.file == "src/other.rs");
    assert!(other_impact.is_some());
    assert_eq!(other_impact.unwrap().line_ranges[0].start, 5);
    assert_eq!(other_impact.unwrap().line_ranges[0].end, 15);
}

#[test]
fn test_create_unmapped_decision_preserves_existing_decisions() {
    let mut decisions = ReviewDecisions::new();

    // Add Decision 1 and 2
    decisions.add_decision(Decision {
        number: 1,
        title: "Decision 1".to_string(),
        rationale: None,
        code_impacts: vec![CodeImpact {
            file: "src/file1.rs".to_string(),
            line_ranges: vec![DecisionLineRange { start: 1, end: 10 }],
            reasoning: "Change 1".to_string(),
        }],
    });

    decisions.add_decision(Decision {
        number: 2,
        title: "Decision 2".to_string(),
        rationale: None,
        code_impacts: vec![CodeImpact {
            file: "src/file2.rs".to_string(),
            line_ranges: vec![DecisionLineRange { start: 20, end: 30 }],
            reasoning: "Change 2".to_string(),
        }],
    });

    let diff1 = create_test_reviewable_diff("src/file1.rs", 1, 10);
    let diff2 = create_test_reviewable_diff("src/file2.rs", 20, 30);
    let unmapped_diff = create_test_reviewable_diff("src/unmapped.rs", 50, 60);

    let review_state = ReviewState::new(
        vec![diff1.clone(), diff2.clone(), unmapped_diff.clone()],
        "author".to_string(),
    );

    decisions.build_index_from_review_state(&review_state);
    decisions.create_unmapped_decision(&review_state);

    // Verify all decisions still exist
    let all = decisions.all_decisions();
    assert_eq!(all.len(), 3);
    assert_eq!(all[0].number, 0);
    assert_eq!(all[1].number, 1);
    assert_eq!(all[2].number, 2);

    // Verify correct mappings
    assert_eq!(decisions.get_decisions_for_diff(&diff1.id).len(), 1);
    assert_eq!(decisions.get_decisions_for_diff(&diff1.id)[0].number, 1);

    assert_eq!(decisions.get_decisions_for_diff(&diff2.id).len(), 1);
    assert_eq!(decisions.get_decisions_for_diff(&diff2.id)[0].number, 2);

    assert_eq!(decisions.get_decisions_for_diff(&unmapped_diff.id).len(), 1);
    assert_eq!(
        decisions.get_decisions_for_diff(&unmapped_diff.id)[0].number,
        0
    );
}

#[test]
fn test_decision_approval_lifecycle() {
    let mut approvals = DecisionApprovals::new();
    let decision_number = 1;

    // Initially not approved
    assert!(!approvals.is_approved(decision_number));
    assert_eq!(approvals.total_approved(), 0);

    // Approve it
    approvals.approve(
        decision_number,
        "reviewer".to_string(),
        "2023-01-01T00:00:00Z".to_string(),
    );
    assert!(approvals.is_approved(decision_number));
    assert_eq!(approvals.total_approved(), 1);

    // Verify approval details
    let approval = approvals.get_approval(decision_number).unwrap();
    assert_eq!(approval.approved_by, "reviewer");
    assert_eq!(approval.approval_timestamp, "2023-01-01T00:00:00Z");

    // Unapprove it
    approvals.unapprove(decision_number);
    assert!(!approvals.is_approved(decision_number));
    assert_eq!(approvals.total_approved(), 0);
}

#[test]
fn test_decision_approval_percentage() {
    let mut approvals = DecisionApprovals::new();

    // Test with no decisions
    assert_eq!(approvals.approval_percentage(0), 0.0);

    // Test with some approved
    approvals.approve(
        1,
        "reviewer".to_string(),
        "2023-01-01T00:00:00Z".to_string(),
    );

    assert_eq!(approvals.approval_percentage(2), 50.0);
    assert_eq!(approvals.approval_percentage(1), 100.0);
}

#[test]
fn test_decision_approval_serialization() {
    let mut approvals = DecisionApprovals::new();
    approvals.approve(
        1,
        "reviewer".to_string(),
        "2023-01-01T00:00:00Z".to_string(),
    );
    approvals.approve(
        2,
        "reviewer2".to_string(),
        "2023-01-02T00:00:00Z".to_string(),
    );

    // Serialize
    let json = serde_json::to_string(&approvals).expect("Failed to serialize");

    // Deserialize
    let deserialized: DecisionApprovals =
        serde_json::from_str(&json).expect("Failed to deserialize");

    // Verify
    assert!(deserialized.is_approved(1));
    assert!(deserialized.is_approved(2));
    assert_eq!(deserialized.total_approved(), 2);
    assert_eq!(
        deserialized.get_approval(1).unwrap().approved_by,
        "reviewer"
    );
}

#[test]
fn test_decision_approval_multiple_approvals() {
    let mut approvals = DecisionApprovals::new();

    // Approve multiple decisions
    approvals.approve(
        1,
        "reviewer".to_string(),
        "2023-01-01T00:00:00Z".to_string(),
    );
    approvals.approve(
        2,
        "reviewer".to_string(),
        "2023-01-01T00:00:00Z".to_string(),
    );
    approvals.approve(
        3,
        "reviewer".to_string(),
        "2023-01-01T00:00:00Z".to_string(),
    );

    assert_eq!(approvals.total_approved(), 3);
    assert!((approvals.approval_percentage(5) - 60.0).abs() < 0.01);

    // Unapprove one
    approvals.unapprove(2);
    assert_eq!(approvals.total_approved(), 2);
    assert!(!approvals.is_approved(2));
    assert!(approvals.is_approved(1));
    assert!(approvals.is_approved(3));
}

#[test]
fn test_decision_approval_edge_cases() {
    let mut approvals = DecisionApprovals::new();

    // Unapproving non-existent decision should not panic
    approvals.unapprove(999);
    assert!(!approvals.is_approved(999));

    // Getting non-existent decision should return None
    assert_eq!(approvals.get_approval(999), None);

    // Approving twice should update the approval
    approvals.approve(
        1,
        "reviewer1".to_string(),
        "2023-01-01T00:00:00Z".to_string(),
    );
    approvals.approve(
        1,
        "reviewer2".to_string(),
        "2023-01-02T00:00:00Z".to_string(),
    );

    let approval = approvals.get_approval(1).unwrap();
    assert_eq!(approval.approved_by, "reviewer2");
    assert_eq!(approval.approval_timestamp, "2023-01-02T00:00:00Z");
    assert_eq!(approvals.total_approved(), 1); // Still only 1 approval
}

// --- Phase 1 TDD: new DecisionLog API ---

#[test]
fn test_decision_log_parse_returns_struct_with_decisions() {
    let yaml = r#"
commit: "abc123"
decisions:
  - number: 1
    title: "Some decision"
    code_impacts: []
"#;
    let log = DecisionLog::parse(yaml).unwrap();
    assert_eq!(log.commit, "abc123");
    assert_eq!(log.decisions.len(), 1);
    assert_eq!(log.decisions[0].number, 1);
}

#[test]
fn test_decision_log_parse_without_commit_returns_error() {
    let yaml = r#"
decisions:
  - number: 1
    title: "Some decision"
    code_impacts: []
"#;
    let result = DecisionLog::parse(yaml);
    assert!(result.is_err());
}

#[test]
fn test_decision_log_parse_with_commit() {
    let yaml = r#"
commit: "abc123def456"
decisions:
  - number: 1
    title: "Some decision"
    code_impacts: []
"#;
    let log = DecisionLog::parse(yaml).unwrap();
    assert_eq!(log.commit, "abc123def456");
}
