# Code Context: Decision Instructions Support

## Files to be Modified or Created

### 1. **New File: DecisionInstructions Entity**
**Location:** `/Users/ryad/workspace/patina/diffviz-review/src/entities/decision_instructions.rs`

A new entity file that will contain:
- `DecisionInstructions` struct (HashMap<u32, Vec<Instruction>>) - mirrors ReviewInstructions structure
- CRUD operations: `add_instruction()`, `get_instructions()`, `has_instructions()`, `remove_instructions()`, `remove_instruction_by_id()`, `total_instructions()`, `get_all_instructions()`, `get_instructions_by_status()`
- Full test suite for entity operations

### 2. **Instruction Entity (Existing)**
**Location:** `/Users/ryad/workspace/patina/diffviz-review/src/entities/instruction.rs`
**Status:** No changes needed - will be reused as-is

This file already contains:
- `InstructionStatus` enum (Active, Addressed)
- `Instruction` struct with id, author, timestamp, content, status fields
- `ReviewInstructions` struct (HashMap<ReviewableDiffId, Vec<Instruction>>)
- Full test suite

### 3. **Decision Entity (Existing)**
**Location:** `/Users/ryad/workspace/patina/diffviz-review/src/entities/decision.rs`
**Status:** Changes needed

Current structure (lines 54-104):
```rust
pub struct Decision {
    pub number: u32,           // Sequential identifier (starting from 1)
    pub title: String,         // One-sentence summary
    pub rationale: Option<String>,  // Why decision was chosen
    pub code_impacts: Vec<CodeImpact>,  // Files and line ranges affected
}

pub struct DecisionLog {
    pub commit: String,
    pub decisions: Vec<Decision>,
}

pub struct ReviewDecisions {
    pub decisions: HashMap<u32, Decision>,
    pub decision_index: HashMap<ReviewableDiffId, Vec<u32>>,
}
```

No changes needed for Decision itself, but will interact with new DecisionInstructions.

### 4. **ReviewState (Existing)**
**Location:** `/Users/ryad/workspace/patina/diffviz-review/src/state/mod.rs`
**Status:** Changes needed

Current structure (lines 30-48):
```rust
pub struct ReviewState {
    pub reviewable_diffs: BTreeMap<ReviewableDiffId, ReviewableDiff>,
    pub approvals: ReviewApprovals,
    pub instructions: ReviewInstructions,
    pub decisions: ReviewDecisions,
    pub decision_approvals: DecisionApprovals,
    pub journey: ReviewJourney,
    pub author: String,
    pub session_metadata: Option<SessionMetadata>,
}
```

**Changes needed:**
- Add field: `pub decision_instructions: DecisionInstructions` (after line 41)
- Update `ReviewState::new()` constructor (lines 93-109) to initialize decision_instructions
- Update `ReviewState::with_review_data()` constructor (lines 112-136) to accept and pass decision_instructions parameter
- Update `ReviewState::get_instructions()` method if needed to provide Decision-specific access

### 5. **Entities Module Export (Existing)**
**Location:** `/Users/ryad/workspace/patina/diffviz-review/src/entities/mod.rs`
**Status:** Changes needed

Current (lines 14-20):
```rust
pub use approval::{Approval, ReviewApprovals};
pub use cascade_result::CascadeResult;
pub use decision::{...};
pub use instruction::{Instruction, ReviewInstructions};
```

**Changes needed:**
- Add new line to export DecisionInstructions: `pub use decision_instructions::DecisionInstructions;`
- Add new module declaration: `pub mod decision_instructions;`

### 6. **ReviewEngine (Existing)**
**Location:** `/Users/ryad/workspace/patina/diffviz-review/src/engines/review_engine.rs`
**Status:** Changes needed

Current structure around line 133-147:
```rust
pub struct ReviewEngine {
    state: ReviewState,
    renderable_cache: HashMap<ReviewableDiffId, String>,
}

impl ReviewEngine {
    pub fn new(reviewable_diffs: Vec<ReviewableDiff>, author: String) -> Self { ... }
    pub fn approve(...) -> Result<()> { ... }
    pub fn reject(...) -> Result<()> { ... }
    pub fn add_instruction(...) -> Result<()> { ... }
    pub fn approve_all_in_file(...) -> Result<()> { ... }
}
```

**Changes needed:**
- Add method `add_decision_instruction()` - mirrors `add_instruction()` but for decisions
- Add method `remove_decision_instruction()` - mirrors removal for decisions
- Add method `get_decision_instructions()` - query instructions for a specific decision
- Add method `export_decision_instructions_json()` - export Decision instructions similar to export_instructions_json()

### 7. **Export/Reporting Code (Existing)**
**Location:** `/Users/ryad/workspace/patina/diffviz-review/src/engines/review_engine.rs`
**Status:** Changes needed - export functionality

Key export methods (lines 509-602):
```rust
pub fn export_instructions_json(&self, scope: ExportScope) -> Result<String> {
    // Current implementation at lines 510-602
    // Exports ReviewInstructions by ReviewableDiffId
}
```

**Export structures (lines 22-121):**
- `ExportScope` enum (lines 22-31) - defines filtering
- `ExportedInstruction` struct (lines 34-48) - JSON representation
- `ExportMetadata` struct (lines 66-77) - format documentation
- `ExportedInstructions` struct (lines 116-121) - container

**Changes needed:**
- Create `ExportedDecisionInstruction` struct (similar to ExportedInstruction but keyed by decision number)
- Extend `ExportScope` enum to support Decision filtering (e.g., `SingleDecision(u32)`, `AllDecisions`)
- Add method `export_decision_instructions_json()` - exports DecisionInstructions in similar format to ReviewInstructions
- Update existing export documentation to clarify that Decision instructions exist separately

### 8. **CLI Integration (Existing)**
**Location:** `/Users/ryad/workspace/patina/diffviz-cli/src/main.rs` and `/Users/ryad/workspace/patina/diffviz-cli/src/commands/`
**Status:** Changes needed for export/reporting

Current export code (main.rs):
```rust
.export_instructions_json(ExportScope::All)  // Line referenced in grep results
```

**Changes needed:**
- Potentially add CLI flag for exporting Decision instructions
- Call new `export_decision_instructions_json()` method when appropriate

## Summary Table

| File | Type | Phase | Changes |
|------|------|-------|---------|
| `/diffviz-review/src/entities/decision_instructions.rs` | New | 1 | Create full entity with CRUD ops + tests |
| `/diffviz-review/src/entities/instruction.rs` | Existing | - | No changes |
| `/diffviz-review/src/entities/decision.rs` | Existing | - | No changes |
| `/diffviz-review/src/entities/mod.rs` | Existing | 1 | Add export declarations |
| `/diffviz-review/src/state/mod.rs` | Existing | 2 | Add field, update constructors |
| `/diffviz-review/src/engines/review_engine.rs` | Existing | 3-4 | Add operations & export methods |
| `/diffviz-cli/src/main.rs` | Existing | 4 | Update export integration |
