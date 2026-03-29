# Implementation Roadmap: Decision Instructions Support

## Overview

This roadmap defines the Steel Thread phases for adding Decision instruction support to DiffViz. Each phase delivers complete, testable, working functionality that can stand alone and forms the foundation for the next phase.

---

## Phase 1: Foundation - DecisionInstructions Entity

**Objective**: Create the DecisionInstructions entity with all CRUD operations and full test coverage.

**Duration Estimate**: Small (1-2 hours)

**Deliverables**:
- New file: `diffviz-review/src/entities/decision_instructions.rs`
- Updated file: `diffviz-review/src/entities/mod.rs` (export declarations)
- 20+ unit tests for entity operations

### Tasks

#### 1.1: Create DecisionInstructions struct and basic operations
**File**: `diffviz-review/src/entities/decision_instructions.rs`

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::entities::instruction::Instruction;

/// Collection of instructions organized by decision number
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DecisionInstructions {
    pub instructions: HashMap<u32, Vec<Instruction>>,
}

impl DecisionInstructions {
    /// Create a new empty DecisionInstructions collection
    pub fn new() -> Self {
        Self {
            instructions: HashMap::new(),
        }
    }

    /// Add an instruction to a specific decision
    pub fn add_instruction(&mut self, decision_number: u32, instruction: Instruction) {
        self.instructions
            .entry(decision_number)
            .or_default()
            .push(instruction);
    }

    /// Get instructions for a specific decision
    pub fn get_instructions(&self, decision_number: u32) -> Option<&Vec<Instruction>> {
        self.instructions.get(&decision_number)
    }

    /// Check if a decision has any instructions
    pub fn has_instructions(&self, decision_number: u32) -> bool {
        self.instructions
            .get(&decision_number)
            .is_some_and(|instructions| !instructions.is_empty())
    }

    /// Get total count of all instructions across all decisions
    pub fn total_instructions(&self) -> usize {
        self.instructions
            .values()
            .map(|instructions| instructions.len())
            .sum()
    }

    /// Get all instructions across all decisions as flat list
    pub fn get_all_instructions(&self) -> Vec<&Instruction> {
        self.instructions
            .values()
            .flat_map(|instructions| instructions.iter())
            .collect()
    }

    /// Get first instruction for a specific decision (convenience)
    pub fn get_instructions_for_decision(
        &self,
        decision_number: u32,
    ) -> Option<&Instruction> {
        self.instructions.get(&decision_number)?.first()
    }

    /// Remove all instructions for a specific decision
    pub fn remove_instructions(&mut self, decision_number: u32) -> Option<Vec<Instruction>> {
        self.instructions.remove(&decision_number)
    }

    /// Get all instructions filtered by status
    pub fn get_instructions_by_status(
        &self,
        status: &crate::entities::instruction::InstructionStatus,
    ) -> Vec<&Instruction> {
        self.instructions
            .values()
            .flat_map(|instructions| instructions.iter())
            .filter(|instruction| &instruction.status == status)
            .collect()
    }

    /// Remove a specific instruction by ID
    pub fn remove_instruction_by_id(&mut self, instruction_id: &str) -> Option<Instruction> {
        for instructions in self.instructions.values_mut() {
            if let Some(pos) = instructions.iter().position(|i| i.id == instruction_id) {
                return Some(instructions.remove(pos));
            }
        }
        None
    }
}
```

**Success Criteria**:
- Code compiles without warnings
- All methods implemented
- InstructionStatus import works correctly

#### 1.2: Write comprehensive unit tests
**File**: `diffviz-review/src/entities/decision_instructions.rs` (in tests module)

Tests to cover:
- `test_new_creates_empty_collection()`
- `test_add_instruction_to_decision()`
- `test_has_instructions_returns_true_when_present()`
- `test_has_instructions_returns_false_when_absent()`
- `test_get_instructions_for_missing_decision_returns_none()`
- `test_total_instructions_counts_all_decisions()`
- `test_get_all_instructions_returns_flat_list()`
- `test_remove_instructions_removes_all_for_decision()`
- `test_remove_instruction_by_id_success()`
- `test_remove_instruction_by_id_not_found()`
- `test_add_multiple_instructions_to_same_decision()`
- `test_add_instructions_to_different_decisions()`
- `test_get_instructions_by_status_active()`
- `test_get_instructions_by_status_addressed()`
- `test_serialization_round_trip()`
- `test_deserialization_preserves_structure()`

**Success Criteria**:
- All 16+ tests pass
- 100% code coverage of DecisionInstructions
- Tests document expected behavior
- Clippy and fmt have no warnings

#### 1.3: Update entities module exports
**File**: `diffviz-review/src/entities/mod.rs`

Add after line 6:
```rust
pub mod decision_instructions;
```

Add after line 20:
```rust
pub use decision_instructions::DecisionInstructions;
```

**Success Criteria**:
- `use diffviz_review::entities::DecisionInstructions;` works
- Compiles without warnings
- Existing tests still pass

### Phase 1 Definition of Done

**Acceptance Criteria**:
- DecisionInstructions.rs exists with all 9+ methods
- All 16+ unit tests pass
- `cargo test --package diffviz-review` passes
- `cargo clippy --package diffviz-review` has no warnings
- `cargo fmt --check --package diffviz-review` passes
- No blocking dependencies on other phases

**Tests That Must Pass**:
```
test_decision_instructions_operations
test_add_instruction_to_decision
test_has_instructions
test_remove_instruction_by_id
test_serialization_round_trip
... (and 11+ more)
```

**What's NOT Done Yet** (that's OK):
- Integration with ReviewState
- ReviewEngine operations
- Export/import functionality
- CLI integration

**Blocks**: None - this is foundation phase

**Unblocks**: Phase 2 (State serialization)

---

## Phase 2: Integration - Serialization into ReviewState

**Objective**: Add DecisionInstructions field to ReviewState and verify round-trip serialization.

**Duration Estimate**: Small (1-2 hours)

**Dependencies**: Phase 1 (DecisionInstructions entity must exist)

**Deliverables**:
- Updated file: `diffviz-review/src/state/mod.rs` (add field, update constructors)
- 5+ serialization tests for ReviewState
- All existing ReviewState tests still pass

### Tasks

#### 2.1: Add DecisionInstructions field to ReviewState
**File**: `diffviz-review/src/state/mod.rs` (after line 41)

In ReviewState struct, add:
```rust
pub decision_instructions: DecisionInstructions,
```

**Location in struct** (showing context):
```rust
pub struct ReviewState {
    pub reviewable_diffs: BTreeMap<ReviewableDiffId, ReviewableDiff>,
    pub approvals: ReviewApprovals,
    pub instructions: ReviewInstructions,
    pub decisions: ReviewDecisions,
    pub decision_approvals: DecisionApprovals,
    pub decision_instructions: DecisionInstructions,  // NEW LINE
    pub journey: ReviewJourney,
    pub author: String,
    pub session_metadata: Option<SessionMetadata>,
}
```

**Success Criteria**:
- Field compiles
- ReviewState still derives Debug, Clone
- All existing tests still compile (will need updating)

#### 2.2: Update ReviewState::new() constructor
**File**: `diffviz-review/src/state/mod.rs` (lines 93-109)

Update constructor to initialize decision_instructions:
```rust
pub fn new(reviewable_diffs: Vec<ReviewableDiff>, author: String) -> Self {
    let mut diffs_map = BTreeMap::new();
    for diff in reviewable_diffs {
        diffs_map.insert(diff.id.clone(), diff);
    }

    Self {
        reviewable_diffs: diffs_map,
        approvals: ReviewApprovals::new(),
        instructions: ReviewInstructions::new(),
        decisions: ReviewDecisions::new(),
        decision_approvals: DecisionApprovals::new(),
        decision_instructions: DecisionInstructions::new(),  // NEW LINE
        journey: ReviewJourney::new(),
        author,
        session_metadata: None,
    }
}
```

**Success Criteria**:
- Constructor compiles
- Initializes with empty DecisionInstructions
- ReviewState::new() still creates valid state

#### 2.3: Update ReviewState::with_review_data() constructor
**File**: `diffviz-review/src/state/mod.rs` (lines 112-136)

Add parameter and initialization:
```rust
pub fn with_review_data(
    reviewable_diffs: Vec<ReviewableDiff>,
    author: String,
    journey: ReviewJourney,
    approvals: ReviewApprovals,
    instructions: ReviewInstructions,
    decisions: ReviewDecisions,
    decision_approvals: DecisionApprovals,
    decision_instructions: DecisionInstructions,  // NEW PARAMETER
) -> Self {
    let mut diffs_map = BTreeMap::new();
    for diff in reviewable_diffs {
        diffs_map.insert(diff.id.clone(), diff);
    }

    Self {
        reviewable_diffs: diffs_map,
        approvals,
        instructions,
        decisions,
        decision_approvals,
        decision_instructions,  // NEW ASSIGNMENT
        journey,
        author,
        session_metadata: None,
    }
}
```

**Success Criteria**:
- Constructor signature updated
- All callers still compile (may need updating)
- State correctly preserves decision_instructions parameter

#### 2.4: Write serialization round-trip tests
**File**: `diffviz-review/src/state/mod.rs` (in tests module, after existing tests)

Tests to add:
- `test_decision_instructions_field_initializes_empty()`
- `test_decision_instructions_survives_state_clone()`
- `test_review_state_with_review_data_includes_decisions()`
- `test_decision_instructions_accessible_through_state()`

**Test Example**:
```rust
#[test]
fn test_decision_instructions_field_initializes_empty() {
    let diff = create_test_reviewable_diff();
    let state = ReviewState::new(vec![diff], "test_author".to_string());

    assert_eq!(state.decision_instructions.total_instructions(), 0);
}

#[test]
fn test_decision_instructions_survives_state_clone() {
    let diff = create_test_reviewable_diff();
    let state = ReviewState::new(vec![diff], "test_author".to_string());

    let cloned = state.clone();
    assert_eq!(cloned.decision_instructions.total_instructions(), 0);
}
```

**Success Criteria**:
- All 4+ serialization tests pass
- Tests verify state initialization and cloning
- No panics or unwraps in tests

### Phase 2 Definition of Done

**Acceptance Criteria**:
- `decision_instructions` field added to ReviewState
- Both constructors (new, with_review_data) updated
- `cargo test --package diffviz-review` passes
- `cargo clippy --package diffviz-review` has no warnings
- Existing ReviewState tests still pass
- New serialization tests verify field works

**Tests That Must Pass**:
```
test_decision_instructions_field_initializes_empty
test_decision_instructions_survives_state_clone
test_review_state_with_review_data_includes_decisions
test_review_state_can_be_created_with_only_instructions_and_approvals (existing)
... (and more)
```

**What's NOT Done Yet** (that's OK):
- ReviewEngine operations
- Export/import functionality
- CLI integration

**Blocks**: None - this phase improves Phase 1 foundation

**Unblocks**: Phase 3 (ReviewEngine operations)

---

## Phase 3: Integration - ReviewEngine Operations

**Objective**: Add ReviewEngine methods to manage decision instructions, enabling reviewers to add/remove/query instructions on decisions.

**Duration Estimate**: Medium (2-3 hours)

**Dependencies**: Phase 1 (entity), Phase 2 (state integration)

**Deliverables**:
- Updated file: `diffviz-review/src/engines/review_engine.rs` (add methods)
- 10+ tests for ReviewEngine decision instruction operations
- All existing ReviewEngine tests still pass

### Tasks

#### 3.1: Add add_decision_instruction() method to ReviewEngine
**File**: `diffviz-review/src/engines/review_engine.rs` (after add_instruction method, around line 248)

```rust
/// Add an instruction to a specific decision
pub fn add_decision_instruction(
    &mut self,
    decision_number: u32,
    content: String,
    author: String,
) -> Result<()> {
    // Validate decision exists
    if !self.state.decisions.decisions.contains_key(&decision_number) {
        return Err(crate::errors::DiffVizError::Review(
            crate::errors::ReviewError::InvalidDecision {
                decision_number,
            },
        ));
    }

    let instruction = Instruction {
        id: uuid::Uuid::new_v4().to_string(),
        author,
        timestamp: chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string(),
        content,
        status: crate::entities::instruction::InstructionStatus::Active,
    };

    self.state
        .decision_instructions
        .add_instruction(decision_number, instruction);

    Ok(())
}
```

**Success Criteria**:
- Method compiles
- Validates decision exists before adding
- Creates proper Instruction with timestamp and UUID
- Returns Result type for error handling

#### 3.2: Add remove_decision_instruction() method
**File**: `diffviz-review/src/engines/review_engine.rs`

```rust
/// Remove a specific instruction from a decision by instruction ID
pub fn remove_decision_instruction(&mut self, instruction_id: &str) -> Result<()> {
    let removed = self
        .state
        .decision_instructions
        .remove_instruction_by_id(instruction_id);

    match removed {
        Some(_) => Ok(()),
        None => Err(crate::errors::DiffVizError::Review(
            crate::errors::ReviewError::InstructionNotFound {
                instruction_id: instruction_id.to_string(),
            },
        )),
    }
}
```

**Success Criteria**:
- Method compiles
- Returns Err if instruction not found
- Uses existing remove_instruction_by_id() from entity

#### 3.3: Add get_decision_instructions() method
**File**: `diffviz-review/src/engines/review_engine.rs`

```rust
/// Get all instructions for a specific decision
pub fn get_decision_instructions(&self, decision_number: u32) -> Option<Vec<&Instruction>> {
    self.state
        .decision_instructions
        .get_instructions(decision_number)
        .map(|instructions| instructions.iter().collect())
}
```

**Success Criteria**:
- Method compiles
- Returns Option wrapping Vec of instruction references
- Used for reading decision instructions

#### 3.4: Write ReviewEngine operation tests
**File**: `diffviz-review/src/engines/review_engine.rs` (in tests module)

Tests to add:
- `test_add_decision_instruction_success()`
- `test_add_decision_instruction_invalid_decision()`
- `test_add_decision_instruction_multiple_to_same_decision()`
- `test_remove_decision_instruction_success()`
- `test_remove_decision_instruction_not_found()`
- `test_get_decision_instructions_returns_all()`
- `test_get_decision_instructions_for_missing_decision()`
- `test_add_and_remove_decision_instructions()`
- `test_decision_instructions_persist_through_state()`
- `test_decision_instructions_independent_from_reviewable_instructions()`

**Test Example**:
```rust
#[test]
fn test_add_decision_instruction_success() {
    let decision = Decision {
        number: 1,
        title: "Test decision".to_string(),
        rationale: None,
        code_impacts: vec![],
    };

    let mut decisions = ReviewDecisions::new();
    decisions.add_decision(decision);

    // Create engine with decision
    // ... (setup code)

    let result = engine.add_decision_instruction(
        1,
        "This is a test instruction".to_string(),
        "test_author".to_string(),
    );

    assert!(result.is_ok());
    assert!(engine.get_decision_instructions(1).is_some());
}
```

**Success Criteria**:
- All 10+ tests pass
- Tests cover success and error paths
- Tests verify state updates correctly

### Phase 3 Definition of Done

**Acceptance Criteria**:
- add_decision_instruction() method implemented and tested
- remove_decision_instruction() method implemented and tested
- get_decision_instructions() method implemented and tested
- All 10+ operation tests pass
- `cargo test --package diffviz-review` passes
- `cargo clippy --package diffviz-review` has no warnings
- Existing ReviewEngine tests still pass

**Tests That Must Pass**:
```
test_add_decision_instruction_success
test_add_decision_instruction_invalid_decision
test_remove_decision_instruction_success
test_get_decision_instructions_returns_all
... (and 6+ more)
```

**What's NOT Done Yet** (that's OK):
- Export/import functionality
- CLI integration
- LLM or advanced features

**Blocks**: None - ReviewEngine operations work end-to-end

**Unblocks**: Phase 4 (Export/import)

---

## Phase 4: Integration - Export and Reporting

**Objective**: Add JSON export/import capability for decision instructions, enabling review sessions to be shared and restored.

**Duration Estimate**: Medium (2-3 hours)

**Dependencies**: Phase 1, 2, 3 (all prior phases)

**Deliverables**:
- Updated file: `diffviz-review/src/engines/review_engine.rs` (add export methods and structures)
- New export structures (ExportedDecisionInstruction, etc.)
- 8+ tests for export/import round-trip
- Updated CLI integration (if applicable)

### Tasks

#### 4.1: Create export structures
**File**: `diffviz-review/src/engines/review_engine.rs` (after ExportedInstructions struct, around line 121)

```rust
/// JSON representation of a decision instruction for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedDecisionInstruction {
    pub decision_number: u32,
    pub content: String,
    pub author: String,
    pub timestamp: String,
    #[serde(default = "default_status")]
    pub status: String,
}

/// Container for exported decision instructions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedDecisionInstructions {
    #[serde(rename = "_meta")]
    pub meta: ExportMetadata,
    pub decision_instructions: Vec<ExportedDecisionInstruction>,
}

/// Extended scope for decision exports
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecisionExportScope {
    /// Export instructions for a specific decision
    SingleDecision(u32),
    /// Export all decision instructions
    All,
}
```

**Success Criteria**:
- Structures compile
- Proper serde derive attributes
- Parallel to ExportedInstruction structures

#### 4.2: Implement export_decision_instructions_json() method
**File**: `diffviz-review/src/engines/review_engine.rs` (after export_instructions_json, around line 602)

```rust
/// Export decision instructions to JSON format
pub fn export_decision_instructions_json(&self, scope: DecisionExportScope) -> Result<String> {
    // Collect decision instructions matching scope
    let decision_instructions: Vec<ExportedDecisionInstruction> = self
        .state
        .decision_instructions
        .instructions
        .iter()
        .filter(|(decision_num, _)| match &scope {
            DecisionExportScope::SingleDecision(num) => *decision_num == num,
            DecisionExportScope::All => true,
        })
        .flat_map(|(decision_num, instructions)| {
            instructions.iter().map(move |inst| ExportedDecisionInstruction {
                decision_number: *decision_num,
                content: inst.content.clone(),
                author: inst.author.clone(),
                timestamp: inst.timestamp.clone(),
                status: match inst.status {
                    InstructionStatus::Active => "active".to_string(),
                    InstructionStatus::Addressed => "addressed".to_string(),
                },
            })
        })
        .collect();

    // Create metadata
    let meta = ExportMetadata {
        format_version: "1.1".to_string(),
        description: "DiffViz decision instructions export for code reviews".to_string(),
        field_descriptions: Some(ExportFieldDescriptions {
            file: "N/A - decision level annotations".to_string(),
            query: "N/A - decision level annotations".to_string(),
            line_range: "N/A - decision level annotations".to_string(),
            content: "The instruction text for reviewers to consider".to_string(),
            author: "Username/identifier of instruction author".to_string(),
            timestamp: "When instruction was created (UTC format)".to_string(),
            status: "Instruction status: 'active' (pending), 'addressed' (completed)".to_string(),
            file_content_hash: "N/A - decision level annotations".to_string(),
            content_snapshot: "N/A - decision level annotations".to_string(),
        }),
        query_formats: None,
        git_usage_examples: None,
    };

    let export = ExportedDecisionInstructions {
        meta,
        decision_instructions,
    };

    serde_json::to_string_pretty(&export).map_err(|e| {
        crate::errors::DiffVizError::Review(crate::errors::ReviewError::ExportFailed {
            reason: format!("JSON serialization failed: {e}"),
        })
    })
}
```

**Success Criteria**:
- Method compiles
- Filters correctly based on scope
- Returns valid JSON
- Includes metadata for tool understanding

#### 4.3: Implement import_decision_instructions_json() method
**File**: `diffviz-review/src/engines/review_engine.rs`

```rust
/// Import decision instructions from JSON
pub fn import_decision_instructions_json(&mut self, json: &str) -> Result<ImportSummary> {
    let exported: ExportedDecisionInstructions = serde_json::from_str(json)
        .map_err(|e| {
            crate::errors::DiffVizError::Review(crate::errors::ReviewError::ImportFailed {
                reason: format!("Failed to parse JSON: {e}"),
            })
        })?;

    let mut summary = ImportSummary::default();

    for exported_inst in exported.decision_instructions {
        // Validate decision exists
        if !self.state.decisions.decisions.contains_key(&exported_inst.decision_number) {
            summary.errors.push(format!(
                "Skipping instruction for non-existent decision #{}",
                exported_inst.decision_number
            ));
            continue;
        }

        // Create instruction
        let instruction = Instruction {
            id: format!(
                "decision_{}:{}",
                exported_inst.decision_number,
                chrono::Utc::now().timestamp_millis()
            ),
            author: exported_inst.author,
            timestamp: exported_inst.timestamp,
            content: exported_inst.content,
            status: InstructionStatus::Active,
        };

        // Add to state
        self.state
            .decision_instructions
            .add_instruction(exported_inst.decision_number, instruction);

        summary.total_imported += 1;
        summary.active_count += 1;
    }

    Ok(summary)
}
```

**Success Criteria**:
- Method compiles
- Validates decisions exist before importing
- Collects errors without failing
- Returns ImportSummary with counts

#### 4.4: Write export/import tests
**File**: `diffviz-review/src/engines/review_engine.rs` (in tests module)

Tests to add:
- `test_export_all_decision_instructions()`
- `test_export_single_decision()`
- `test_export_empty_decision_instructions()`
- `test_export_decision_json_structure()`
- `test_import_decision_instructions_success()`
- `test_import_decision_instructions_invalid_decision()`
- `test_export_import_round_trip()`
- `test_import_skips_duplicate_decisions()`

**Test Example (Round-trip)**:
```rust
#[test]
fn test_export_import_round_trip() {
    // Create engine with decision and instructions
    let mut engine = create_test_engine_with_decision();

    engine.add_decision_instruction(
        1,
        "Test instruction".to_string(),
        "author".to_string(),
    ).unwrap();

    // Export
    let json = engine.export_decision_instructions_json(
        DecisionExportScope::All
    ).unwrap();

    // Create new engine and import
    let mut new_engine = create_fresh_engine();
    let summary = new_engine.import_decision_instructions_json(&json).unwrap();

    // Verify
    assert_eq!(summary.total_imported, 1);
    assert_eq!(
        new_engine.get_decision_instructions(1).unwrap().len(),
        1
    );
}
```

**Success Criteria**:
- All 8+ tests pass
- Round-trip test verifies export -> import restores state
- Tests verify JSON format and metadata

#### 4.5: CLI Integration (Optional - Phase 4 bonus)
**File**: `/Users/ryad/workspace/patina/diffviz-cli/src/main.rs` or commands

If applicable, add:
- Option to export decision instructions
- Option to import decision instructions
- Display in debug output

This is optional for MVP but recommended for completeness.

### Phase 4 Definition of Done

**Acceptance Criteria**:
- ExportedDecisionInstruction and ExportedDecisionInstructions structures created
- export_decision_instructions_json() implemented and tested
- import_decision_instructions_json() implemented and tested
- All 8+ export/import tests pass
- Round-trip test verifies complete serialization cycle
- `cargo test --package diffviz-review` passes
- `cargo clippy --package diffviz-review` has no warnings
- Existing export tests still pass

**Tests That Must Pass**:
```
test_export_all_decision_instructions
test_export_single_decision
test_export_import_round_trip
test_import_decision_instructions_success
... (and 4+ more)
```

**What's NOT Done Yet** (that's OK):
- LLM features (future phase)
- Constraint validation (future phase)
- Advanced UI (TUI responsibility)

**Blocks**: None - feature complete

**Unblocks**: Future phases (LLM-v2, validation-v1, etc.)

---

## Overall Success Criteria (End of All Phases)

When all four phases are complete:

1. **Functionality**: Reviewers can add, view, edit, remove instructions on architectural decisions
2. **Persistence**: Decision instructions serialize with ReviewState and survive round-trip
3. **Export**: Decision instructions can be exported to JSON and imported into other review sessions
4. **Testing**: 40+ tests covering all entity, state, engine, and export operations
5. **Quality**: Zero compiler warnings, zero clippy warnings, all code formatted
6. **Documentation**: Clear test names and module comments explaining behavior

### Feature Scope Boundaries

**In Scope (Phases 1-4)**:
- CRUD operations for decision instructions
- Serialization/deserialization
- JSON export/import
- Integration with ReviewEngine
- Integration with ReviewState
- Error handling with clear messages

**Out of Scope (Future Phases)**:
- LLM understanding or generation of instructions
- Automatic constraint validation
- UI/TUI components (that's diffviz-review-tui)
- Version control integration for instructions
- Differential instruction tracking

### Test Coverage Summary

| Phase | Unit Tests | Integration Tests | Total |
|-------|-----------|------------------|-------|
| Phase 1 | 16+ | 0 | 16+ |
| Phase 2 | 0 | 4+ | 4+ |
| Phase 3 | 0 | 10+ | 10+ |
| Phase 4 | 0 | 8+ | 8+ |
| **Total** | **16+** | **22+** | **38+** |

Each test is focused, documented, and verifies specific behavior.

---

## Rollout Checklist

**Before Phase 1**:
- [ ] Review this plan with team
- [ ] Confirm DecisionInstructions struct design
- [ ] Create branch for Phase 1 work

**Phase 1 Completion**:
- [ ] All 16+ unit tests pass
- [ ] No clippy/fmt warnings
- [ ] Code review approval
- [ ] Merge to main

**Phase 2 Completion**:
- [ ] ReviewState field added
- [ ] All constructors updated
- [ ] Serialization tests pass
- [ ] Existing tests still pass
- [ ] Code review approval
- [ ] Merge to main

**Phase 3 Completion**:
- [ ] ReviewEngine methods implemented
- [ ] All 10+ operation tests pass
- [ ] Error cases handled
- [ ] Code review approval
- [ ] Merge to main

**Phase 4 Completion**:
- [ ] Export/import implemented
- [ ] Round-trip test passes
- [ ] JSON format documented
- [ ] CLI integration (if applicable)
- [ ] Code review approval
- [ ] Merge to main
- [ ] Update CLAUDE.md if needed

**Post-Launch**:
- [ ] Update project documentation
- [ ] Add feature to release notes
- [ ] Link to decision-log.yaml in commit messages
