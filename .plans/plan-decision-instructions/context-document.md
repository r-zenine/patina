# Context Document: Decision Instructions Feature

## Behavioral Spec

### What are Decision Instructions?

Decision instructions are annotations on architectural decisions that provide context, guidance, or constraints for reviewers to understand and evaluate each decision. They function identically to ReviewableDiff instructions (which are already implemented), but are scoped to the decision level rather than code level.

### How They Work

1. **Associating Instructions with Decisions**: Reviewers can annotate each decision with instructions that provide:
   - Decision rationale or supporting context
   - Constraints or requirements driving the decision
   - Expected outcomes or success criteria
   - Reviewers' questions or concerns about the decision
   - References to external documentation, RFCs, or requirements

2. **Dual Instruction System**:
   - **ReviewInstructions**: Keyed by `ReviewableDiffId` (specific code ranges) - EXISTING
   - **DecisionInstructions**: Keyed by decision number `u32` (decisions) - NEW
   - Both use the same `Instruction` struct with id, author, timestamp, content, status

3. **Review Workflow with Decision Instructions**:
   - A reviewer opens a decision in the review UI
   - They can add instructions like: "Need to verify this doesn't break module X compatibility"
   - Instructions are tagged as Active (pending) or Addressed (completed)
   - When exporting review state, both ReviewInstructions and DecisionInstructions are included
   - When importing a saved review session, both instruction types are restored

4. **Serialization and Persistence**:
   - Decision instructions are serialized as part of ReviewState (alongside approvals, instructions, decisions, decision_approvals)
   - Can be exported to JSON for sharing review guidance with other reviewers or tools
   - Can be imported from JSON to restore previous review sessions

---

## Architecture Summary

### Current Instruction System (ReviewInstructions)

The existing ReviewInstructions system is a well-structured pattern we will mirror:

```rust
// Entity (instruction.rs, lines 38-115)
pub struct ReviewInstructions {
    pub instructions: HashMap<ReviewableDiffId, Vec<Instruction>>,
}

impl ReviewInstructions {
    pub fn add_instruction(&mut self, reviewable_id: ReviewableDiffId, instruction: Instruction)
    pub fn get_instructions(&self, reviewable_id: &ReviewableDiffId) -> Option<&Vec<Instruction>>
    pub fn has_instructions(&self, reviewable_id: &ReviewableDiffId) -> bool
    pub fn remove_instructions(&mut self, reviewable_id: &ReviewableDiffId) -> Option<Vec<Instruction>>
    pub fn remove_instruction_by_id(&mut self, instruction_id: &str) -> Option<Instruction>
    pub fn total_instructions(&self) -> usize
    pub fn get_instructions_by_status(&self, status: &InstructionStatus) -> Vec<&Instruction>
    // ... more methods
}
```

### The Instruction Type (Reused)

Both ReviewInstructions and DecisionInstructions use the same `Instruction` struct:

```rust
// instruction.rs, lines 27-35
pub struct Instruction {
    pub id: String,                    // UUID-based identifier
    pub author: String,                // Username of creator
    pub timestamp: String,             // UTC timestamp
    pub content: String,               // The instruction text
    pub status: InstructionStatus,     // Active or Addressed
}

pub enum InstructionStatus {
    Active,      // Instruction points to current/valid state
    Addressed,   // User marked instruction as handled/completed
}
```

### Decision Entity Context

Decisions are the top-level semantic unit that instructions will annotate:

```rust
// decision.rs, lines 60-83
pub struct Decision {
    pub number: u32,                   // Sequential: 1, 2, 3, ...
    pub title: String,                 // "Add authentication middleware"
    pub rationale: Option<String>,     // Why this decision was chosen
    pub code_impacts: Vec<CodeImpact>,  // What files/lines it affects
}

pub struct ReviewDecisions {
    pub decisions: HashMap<u32, Decision>,  // Keyed by decision number
    pub decision_index: HashMap<ReviewableDiffId, Vec<u32>>,  // Reverse index
}
```

### Where DecisionInstructions Fits

```
ReviewState (state/mod.rs, lines 30-48)
├── reviewable_diffs: BTreeMap<ReviewableDiffId, ReviewableDiff>
├── approvals: ReviewApprovals          // Approvals for code chunks
├── instructions: ReviewInstructions    // Instructions for code chunks (EXISTING)
├── decisions: ReviewDecisions          // Architectural decisions (EXISTING)
├── decision_approvals: DecisionApprovals  // Approvals for decisions (EXISTING)
├── decision_instructions: DecisionInstructions  // Instructions for decisions (NEW)
└── ...
```

DecisionInstructions will be added as a peer to instructions, both managing the same instruction data type but at different scopes.

### Review Engine Operations

ReviewEngine orchestrates all review operations through methods like:

```rust
// review_engine.rs, lines 142-288
pub struct ReviewEngine {
    state: ReviewState,
    renderable_cache: HashMap<ReviewableDiffId, String>,
}

impl ReviewEngine {
    pub fn add_instruction(&mut self, reviewable_id, content, author, callback) -> Result<()>
    pub fn approve(&mut self, reviewable_id, reviewer, callback) -> Result<()>
    pub fn reject(&mut self, reviewable_id, callback) -> Result<()>
    // ... many more operations
}
```

New Decision instruction operations will follow the same pattern:
- `add_decision_instruction(decision_number, content, author)`
- `remove_decision_instruction(decision_number, instruction_id)`
- `get_decision_instructions(decision_number)`

### Export System

The export system converts ReviewState to JSON for external tools:

```rust
// review_engine.rs, lines 510-602
pub fn export_instructions_json(&self, scope: ExportScope) -> Result<String>

// Exports ReviewInstructions in this format:
ExportedInstructions {
    _meta: ExportMetadata { ... },  // Format version, field descriptions, examples
    instructions: Vec<ExportedInstruction> {
        file, query, line_range, content, author, timestamp, status, ...
    }
}
```

New Decision instruction export will follow the same pattern but keyed by decision number instead of ReviewableDiffId.

---

## Key Constraints

### 1. **Storage Key Difference**
- **ReviewInstructions**: HashMap<ReviewableDiffId, Vec<Instruction>>
  - Keyed by code location (file + line range + diff query)
  - Each code range can have multiple instructions

- **DecisionInstructions**: HashMap<u32, Vec<Instruction>>
  - Keyed by decision number (sequential integer)
  - Each decision can have multiple instructions
  - Simpler key type (no ReviewableDiffId complexity)

### 2. **Serialization Requirement**
- Must serialize with ReviewState to enable round-trip session persistence
- All components of ReviewState must be serializable (ReviewApprovals, ReviewInstructions, etc.)
- DecisionInstructions must implement the same patterns as ReviewInstructions

### 3. **Export Must Include Decisions**
- Current `export_instructions_json()` only exports ReviewInstructions
- Export process must be enhanced to optionally include DecisionInstructions
- May need separate export method or enhanced scope options

### 4. **No String-Based Operations in Core Layer**
- Per CLAUDE.md: diffviz-core cannot use string-based analysis
- DecisionInstructions is in diffviz-review (not core), so this constraint doesn't apply
- DecisionInstructions can be simpler than ReviewInstructions (just string content)

### 5. **Fail-Fast on Invalid State**
- When adding decision instructions, verify decision number exists
- When accessing instructions for non-existent decisions, fail early (don't return None)
- Maintain consistency between ReviewDecisions and DecisionInstructions

### 6. **Decision Number is Primary Key**
- Decision numbers are u32 (sequential: 1, 2, 3...)
- Must never use DecisionId or other identifier types
- Decisions are identified solely by number within a review session

---

## Design Decisions Already Made

### 1. **Steel Thread Execution Strategy Chosen**
The implementation will follow a Steel Thread approach, delivering end-to-end working functionality in incremental phases:
- **Phase 1 (Foundation)**: Create DecisionInstructions entity with full CRUD operations and tests
- **Phase 2 (Serialization)**: Integrate into ReviewState with round-trip serialization
- **Phase 3 (Operations)**: Add ReviewEngine methods for managing decision instructions
- **Phase 4 (Export)**: Integrate into JSON export/import system

Each phase delivers testable, working functionality that depends on previous phases.

### 2. **Instruction Reuse (Not Duplication)**
Instead of creating DecisionInstruction struct, DecisionInstructions will use the same `Instruction` struct as ReviewInstructions. This ensures:
- Consistent data model across the system
- No duplication of instruction logic
- Both instruction types have id, author, timestamp, content, status
- Easier maintenance and testing

### 3. **HashMap Storage Pattern**
DecisionInstructions will use HashMap<u32, Vec<Instruction>> (same pattern as ReviewInstructions):
- `u32` key is the decision number
- Each decision can have multiple instructions (parallel instructions allowed)
- Enables future querying like "get all instructions for decision #3"
- Consistent with existing ReviewInstructions pattern

### 4. **Integration Point: ReviewState**
DecisionInstructions will be added to ReviewState alongside existing instruction and approval types:
- Centralized state container for all review-related data
- Enables atomic serialization/deserialization
- Enables ReviewEngine to manage both instruction types uniformly
- Makes it natural for UI to display both instruction types

### 5. **Separate Export Method or Extended Scope**
Export of decision instructions will require:
- Either a separate method: `export_decision_instructions_json(scope)`
- Or enhanced ExportScope to include decision-level filtering
- TBD in Phase 4 based on usage patterns

### 6. **No Advanced Features in Phase 1-4**
Steel Thread focuses on core functionality only:
- NO LLM consumption of decision instructions
- NO UI components (that's diffviz-review-tui responsibility)
- NO constraint validation or semantic analysis
- NO caching optimizations
- Only: basic CRUD, serialization, export

---

## Behavioral Guarantees

### When Adding an Instruction to a Decision
```
add_decision_instruction(decision_num: u32, content: String, author: String) -> Result<()>
```

1. **Idempotency**: Multiple adds to same decision create separate instruction objects (each gets unique ID)
2. **Validation**: Fail immediately if decision_number doesn't exist in ReviewDecisions
3. **Timestamp**: Automatically assigned as current UTC time (not caller-provided)
4. **Author**: Caller-provided, allows tracking who added the instruction
5. **Status**: Always defaults to InstructionStatus::Active for new instructions
6. **Returns**: Ok(()) on success, Err() with context on failure

### When Exporting Decision Instructions
```
export_decision_instructions_json(scope: ExportScope) -> Result<String>
```

1. **Format Compatibility**: Uses same JSON structure as ReviewInstructions export
2. **Metadata**: Includes format documentation (version, field descriptions, examples)
3. **No Filtering**: Exports ALL decision instructions in scope (no deduplication or merging)
4. **Validity Checks**: Includes decision existence verification in exported metadata

### When Importing Decision Instructions
Decision instructions import follows the same pattern as ReviewInstructions:
1. Skip instructions for non-existent decisions
2. Detect duplicates and skip them
3. Return ImportSummary with counts and error list
4. Never fail entirely due to bad data - collect errors and continue

---

## Success Criteria for Implementation

By end of Phase 4, the feature will be complete when:

1. **Phase 1 Done**: DecisionInstructions entity exists with:
   - HashMap<u32, Vec<Instruction>> storage
   - All CRUD operations (add, get, remove, etc.)
   - 100% test coverage of entity operations
   - Compiles without warnings

2. **Phase 2 Done**: ReviewState integration:
   - `decision_instructions` field added
   - Constructors updated to initialize and accept it
   - Serialization round-trip verified by tests
   - No warnings from clippy or compiler

3. **Phase 3 Done**: ReviewEngine operations:
   - add_decision_instruction() working
   - remove_decision_instruction() working
   - get_decision_instructions() working
   - All operations tested with ReviewEngine
   - No cascade effects or hidden dependencies

4. **Phase 4 Done**: Export/import:
   - export_decision_instructions_json() returns valid JSON
   - import_decision_instructions_json() restores state
   - Round-trip test verifies: export -> import -> same state
   - JSON includes metadata for tool understanding
   - Integrated with CLI export command
