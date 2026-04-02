use super::super::test_helpers::*;
use super::*;

/// Helper to set up a decision with multiple chunks
fn create_engine_with_decision_and_chunks() -> ReviewEngine {
    use crate::entities::decision::{CodeImpact, Decision, DecisionLineRange};

    // Create 3 chunks in the same file
    let chunks = vec![
        create_test_reviewable_diff("main.rs", 1),
        create_test_reviewable_diff("main.rs", 15),
        create_test_reviewable_diff("main.rs", 30),
    ];

    let mut engine = ReviewEngine::new(chunks, "test_author".to_string());

    // Add a decision that affects all 3 chunks
    let decision = Decision {
        number: 1,
        title: "Add authentication module".to_string(),
        rationale: Some("Implement user authentication".to_string()),
        code_impacts: vec![CodeImpact {
            file: "main.rs".to_string(),
            line_ranges: vec![DecisionLineRange { start: 1, end: 45 }],
            reasoning: "Affects main function and helpers".to_string(),
        }],
    };

    engine.state.decisions.add_decision(decision);
    {
        let state_copy = engine.state.clone();
        engine
            .state
            .decisions
            .build_index_from_review_state(&state_copy);
    }

    engine
}

fn create_engine_with_decision() -> ReviewEngine {
    use crate::entities::decision::Decision;

    let diff = create_test_reviewable_diff("test.rs", 1);
    let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

    let decision = Decision {
        number: 1,
        title: "Test decision".to_string(),
        rationale: None,
        code_impacts: vec![],
    };
    engine.state.decisions.add_decision(decision);
    engine
}

fn create_engine_with_two_decisions() -> ReviewEngine {
    use crate::entities::decision::Decision;

    let diff = create_test_reviewable_diff("test.rs", 1);
    let mut engine = ReviewEngine::new(vec![diff], "test_author".to_string());

    engine.state.decisions.add_decision(Decision {
        number: 1,
        title: "First decision".to_string(),
        rationale: None,
        code_impacts: vec![],
    });
    engine.state.decisions.add_decision(Decision {
        number: 2,
        title: "Second decision".to_string(),
        rationale: None,
        code_impacts: vec![],
    });
    engine
}

// ===== Decision Approval Integration Tests =====

#[test]
fn test_decision_approval_cascades_to_chunks() {
    let mut engine = create_engine_with_decision_and_chunks();

    // Approve decision 1
    let result = engine.approve_decision(1, "reviewer".to_string());
    assert!(result.is_ok());

    // Verify cascade result
    let cascade_result = result.unwrap();
    match cascade_result {
        CascadeResult::DecisionApproved {
            decision_number,
            chunks_affected,
        } => {
            assert_eq!(decision_number, 1);
            assert_eq!(chunks_affected, 3);
        }
        _ => panic!("Expected DecisionApproved result"),
    }

    // Verify decision is approved
    assert!(engine.is_decision_approved(1));

    // Verify all chunks are approved
    for reviewable_diff in engine.state.reviewable_diffs.values() {
        assert!(engine.state.is_approved(&reviewable_diff.id));
    }
}

#[test]
fn test_decision_rejection_cascades_to_chunks() {
    let mut engine = create_engine_with_decision_and_chunks();

    // First approve decision and chunks
    engine.approve_decision(1, "reviewer".to_string()).unwrap();
    assert!(engine.is_decision_approved(1));

    // Now reject the decision
    let result = engine.reject_decision(1);
    assert!(result.is_ok());

    // Verify cascade result
    let cascade_result = result.unwrap();
    match cascade_result {
        CascadeResult::DecisionUnapproved {
            decision_number,
            chunks_affected,
        } => {
            assert_eq!(decision_number, 1);
            assert_eq!(chunks_affected, 3);
        }
        _ => panic!("Expected DecisionUnapproved result"),
    }

    // Verify decision is unapproved
    assert!(!engine.is_decision_approved(1));

    // Verify all chunks are unapproved
    for reviewable_diff in engine.state.reviewable_diffs.values() {
        assert!(!engine.state.is_approved(&reviewable_diff.id));
    }
}

#[test]
fn test_reverse_cascade_all_chunks_approved() {
    let mut engine = create_engine_with_decision_and_chunks();

    // Get all chunk IDs
    let chunk_ids: Vec<_> = engine
        .state
        .reviewable_diffs
        .values()
        .map(|d| d.id.clone())
        .collect();

    // Approve chunks one by one
    for (i, chunk_id) in chunk_ids.iter().enumerate() {
        engine
            .approve(chunk_id.clone(), "reviewer".to_string())
            .unwrap();

        // Decision should still be unapproved until ALL chunks are approved
        if i < chunk_ids.len() - 1 {
            assert!(!engine.is_decision_approved(1));
        }
    }

    // After approving all chunks, decision should be auto-approved
    assert!(engine.is_decision_approved(1));
}

#[test]
fn test_reverse_cascade_reject_one_chunk_unapproves_decision() {
    let mut engine = create_engine_with_decision_and_chunks();

    // Approve all chunks
    let chunk_ids: Vec<_> = engine
        .state
        .reviewable_diffs
        .values()
        .map(|d| d.id.clone())
        .collect();
    for chunk_id in chunk_ids {
        engine.approve(chunk_id, "reviewer".to_string()).unwrap();
    }

    // Verify decision is auto-approved
    assert!(engine.is_decision_approved(1));

    // Reject one chunk
    let first_chunk_id = engine
        .state
        .reviewable_diffs
        .values()
        .next()
        .unwrap()
        .id
        .clone();

    engine.reject(first_chunk_id).unwrap();

    // Decision should be unapproved since not all chunks are approved anymore
    assert!(!engine.is_decision_approved(1));
}

#[test]
fn test_decision_progress_partial_approval() {
    let mut engine = create_engine_with_decision_and_chunks();

    // Get chunk IDs
    let chunk_ids: Vec<_> = engine
        .state
        .reviewable_diffs
        .values()
        .map(|d| d.id.clone())
        .collect();

    // Initially: no chunks approved
    let (approved, total) = engine.decision_approval_progress(1);
    assert_eq!(approved, 0);
    assert_eq!(total, 3);

    // Approve first chunk
    engine
        .approve(chunk_ids[0].clone(), "reviewer".to_string())
        .unwrap();
    let (approved, total) = engine.decision_approval_progress(1);
    assert_eq!(approved, 1);
    assert_eq!(total, 3);

    // Approve second chunk
    engine
        .approve(chunk_ids[1].clone(), "reviewer".to_string())
        .unwrap();
    let (approved, total) = engine.decision_approval_progress(1);
    assert_eq!(approved, 2);
    assert_eq!(total, 3);

    // Approve third chunk - should trigger reverse cascade
    engine
        .approve(chunk_ids[2].clone(), "reviewer".to_string())
        .unwrap();
    let (approved, total) = engine.decision_approval_progress(1);
    assert_eq!(approved, 3);
    assert_eq!(total, 3);

    // Decision should be auto-approved
    assert!(engine.is_decision_approved(1));
}

#[test]
fn test_multiple_decisions_independent() {
    use crate::entities::decision::{CodeImpact, Decision, DecisionLineRange};

    // Create 4 chunks: 2 for decision 1, 2 for decision 2
    let chunks = vec![
        create_test_reviewable_diff("module1.rs", 1),
        create_test_reviewable_diff("module1.rs", 50),
        create_test_reviewable_diff("module2.rs", 1),
        create_test_reviewable_diff("module2.rs", 50),
    ];

    let mut engine = ReviewEngine::new(chunks, "test_author".to_string());

    // Add decision 1 (affects module1.rs)
    let decision1 = Decision {
        number: 1,
        title: "Module 1 changes".to_string(),
        rationale: Some("Changes to module 1".to_string()),
        code_impacts: vec![CodeImpact {
            file: "module1.rs".to_string(),
            line_ranges: vec![DecisionLineRange { start: 1, end: 100 }],
            reasoning: "All module1 changes".to_string(),
        }],
    };

    // Add decision 2 (affects module2.rs)
    let decision2 = Decision {
        number: 2,
        title: "Module 2 changes".to_string(),
        rationale: Some("Changes to module 2".to_string()),
        code_impacts: vec![CodeImpact {
            file: "module2.rs".to_string(),
            line_ranges: vec![DecisionLineRange { start: 1, end: 100 }],
            reasoning: "All module2 changes".to_string(),
        }],
    };

    engine.state.decisions.add_decision(decision1);
    engine.state.decisions.add_decision(decision2);
    {
        let state_copy = engine.state.clone();
        engine
            .state
            .decisions
            .build_index_from_review_state(&state_copy);
    }

    // Approve only decision 1
    let result = engine.approve_decision(1, "reviewer".to_string()).unwrap();

    // Verify we got the cascade result
    match result {
        CascadeResult::DecisionApproved {
            decision_number,
            chunks_affected,
        } => {
            assert_eq!(decision_number, 1);
            assert_eq!(chunks_affected, 2);
        }
        _ => panic!("Expected DecisionApproved result"),
    }

    // Decision 1 should be approved
    assert!(engine.is_decision_approved(1));

    // Decision 2 should not be approved
    assert!(!engine.is_decision_approved(2));

    // Only module1 chunks should be approved
    for reviewable_diff in engine.state.reviewable_diffs.values() {
        if reviewable_diff.file_path == "module1.rs" {
            assert!(engine.state.is_approved(&reviewable_diff.id));
        } else {
            assert!(!engine.state.is_approved(&reviewable_diff.id));
        }
    }
}

#[test]
fn test_decision_progress_zero_chunks() {
    let engine = ReviewEngine::new(vec![], "test_author".to_string());

    // For non-existent decision, progress should be (0, 0)
    let (approved, total) = engine.decision_approval_progress(999);
    assert_eq!(approved, 0);
    assert_eq!(total, 0);
}

#[test]
fn test_get_decision_reviewable_diffs_returns_one_per_indexed_pair() {
    use crate::entities::decision::{CodeImpact, Decision, DecisionLineRange};

    // Two diffs on the same file, non-overlapping line ranges
    let diff1 = create_test_reviewable_diff("src/main.rs", 1); // lines 1-11
    let diff2 = create_test_reviewable_diff("src/main.rs", 20); // lines 20-30

    let mut engine = ReviewEngine::new(vec![diff1, diff2], "test_author".to_string());

    // One decision with two code impacts — each overlaps one of the two diffs
    let decision = Decision {
        number: 1,
        title: "Test decision".to_string(),
        rationale: None,
        code_impacts: vec![
            CodeImpact {
                file: "src/main.rs".to_string(),
                line_ranges: vec![DecisionLineRange { start: 5, end: 15 }],
                reasoning: "overlaps diff1".to_string(),
            },
            CodeImpact {
                file: "src/main.rs".to_string(),
                line_ranges: vec![DecisionLineRange { start: 25, end: 35 }],
                reasoning: "overlaps diff2".to_string(),
            },
        ],
    };

    let mut decisions = crate::entities::ReviewDecisions::new();
    decisions.add_decision(decision);
    engine.set_decisions_with_index(decisions);

    let diffs = engine.get_decision_reviewable_diffs();
    assert_eq!(diffs.len(), 2);
    assert!(diffs.iter().all(|d| d.decision_number == 1));
}

// === Decision Instruction Tests ===

#[test]
fn test_add_decision_instruction_success() {
    let mut engine = create_engine_with_decision();

    let result = engine.add_decision_instruction(
        1,
        "Review this carefully".to_string(),
        "reviewer".to_string(),
    );

    assert!(result.is_ok());
    assert!(engine.get_decision_instructions(1).is_some());
    assert_eq!(engine.get_decision_instructions(1).unwrap().len(), 1);
}

#[test]
fn test_add_decision_instruction_invalid_decision() {
    let mut engine = create_engine_with_decision();

    let result =
        engine.add_decision_instruction(999, "Should fail".to_string(), "reviewer".to_string());

    assert!(result.is_err());
    match result.unwrap_err() {
        crate::errors::DiffVizError::Review(crate::errors::ReviewError::InvalidDecision {
            decision_number,
        }) => assert_eq!(decision_number, 999),
        e => panic!("Expected InvalidDecision, got: {e:?}"),
    }
}

#[test]
fn test_add_decision_instruction_multiple_to_same_decision() {
    let mut engine = create_engine_with_decision();

    engine
        .add_decision_instruction(1, "First instruction".to_string(), "author1".to_string())
        .unwrap();
    engine
        .add_decision_instruction(1, "Second instruction".to_string(), "author2".to_string())
        .unwrap();

    let instructions = engine.get_decision_instructions(1).unwrap();
    assert_eq!(instructions.len(), 2);
}

#[test]
fn test_remove_decision_instruction_success() {
    let mut engine = create_engine_with_decision();

    engine
        .add_decision_instruction(1, "To be removed".to_string(), "author".to_string())
        .unwrap();

    let instruction_id = engine
        .get_decision_instructions(1)
        .unwrap()
        .first()
        .unwrap()
        .id
        .clone();

    let result = engine.remove_decision_instruction(&instruction_id);
    assert!(result.is_ok());
    assert_eq!(engine.get_decision_instructions(1).unwrap().len(), 0);
}

#[test]
fn test_remove_decision_instruction_not_found() {
    let mut engine = create_engine_with_decision();

    let result = engine.remove_decision_instruction("nonexistent-id");

    assert!(result.is_err());
    match result.unwrap_err() {
        crate::errors::DiffVizError::Review(crate::errors::ReviewError::InstructionNotFound {
            instruction_id,
        }) => assert_eq!(instruction_id, "nonexistent-id"),
        e => panic!("Expected InstructionNotFound, got: {e:?}"),
    }
}

#[test]
fn test_get_decision_instructions_returns_all() {
    let mut engine = create_engine_with_decision();

    engine
        .add_decision_instruction(1, "First".to_string(), "author".to_string())
        .unwrap();
    engine
        .add_decision_instruction(1, "Second".to_string(), "author".to_string())
        .unwrap();
    engine
        .add_decision_instruction(1, "Third".to_string(), "author".to_string())
        .unwrap();

    let instructions = engine.get_decision_instructions(1).unwrap();
    assert_eq!(instructions.len(), 3);
}

#[test]
fn test_get_decision_instructions_for_missing_decision() {
    let engine = create_engine_with_decision();

    let result = engine.get_decision_instructions(999);
    assert!(result.is_none());
}

#[test]
fn test_add_and_remove_decision_instructions() {
    let mut engine = create_engine_with_decision();

    engine
        .add_decision_instruction(1, "Keep this".to_string(), "author".to_string())
        .unwrap();
    engine
        .add_decision_instruction(1, "Remove this".to_string(), "author".to_string())
        .unwrap();

    let remove_id = engine
        .get_decision_instructions(1)
        .unwrap()
        .iter()
        .find(|i| i.content == "Remove this")
        .unwrap()
        .id
        .clone();

    engine.remove_decision_instruction(&remove_id).unwrap();

    let remaining = engine.get_decision_instructions(1).unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].content, "Keep this");
}

#[test]
fn test_decision_instructions_persist_through_state() {
    let mut engine = create_engine_with_decision();

    engine
        .add_decision_instruction(
            1,
            "Persistent instruction".to_string(),
            "author".to_string(),
        )
        .unwrap();

    // Access via state directly to verify persistence
    let count = engine.state().decision_instructions.total_instructions();
    assert_eq!(count, 1);
}

#[test]
fn test_decision_instructions_independent_from_reviewable_instructions() {
    let diff = create_test_reviewable_diff("test.rs", 1);
    let diff_id = diff.id.clone();
    let mut engine = create_engine_with_decision();
    engine
        .add_instruction(
            diff_id,
            "Code instruction".to_string(),
            "author".to_string(),
        )
        .unwrap();
    engine
        .add_decision_instruction(1, "Decision instruction".to_string(), "author".to_string())
        .unwrap();

    assert_eq!(engine.state().instructions.total_instructions(), 1);
    assert_eq!(engine.state().decision_instructions.total_instructions(), 1);
}

// Suppress unused warning for create_engine_with_two_decisions
// (kept for future tests)
#[allow(dead_code)]
fn _use_two_decisions() {
    let _ = create_engine_with_two_decisions();
}
