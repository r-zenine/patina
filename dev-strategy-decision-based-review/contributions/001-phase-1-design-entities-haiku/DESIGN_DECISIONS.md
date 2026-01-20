# Phase 1 Design Decisions - Decision-Based Review Entity Implementation

## Decision Entity Architecture

### Decision Type Structure
Based on the JSON schema and EntityCentric design pattern in diffviz-review, we model:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub number: u32,                          // Sequential decision number
    pub title: String,                        // User-facing title
    pub summary: String,                      // Brief description
    pub decision_log_line: Option<usize>,    // Reference to line in decision-log.md
    pub code_impacts: Vec<CodeImpact>,       // Function-level code changes
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeImpact {
    pub file: String,                         // File path
    pub line_ranges: Vec<LineRange>,         // Function boundaries (not exact diff lines)
    pub change_type: ChangeType,             // "addition", "modification", "deletion"
    pub confidence: Confidence,              // high/medium/low certainty
    pub reasoning: String,                   // Why this decision affects this code
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    Addition,
    Modification,
    Deletion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Confidence {
    High,
    Medium,
    Low,
}

pub struct LineRange {
    pub start: usize,
    pub end: usize,
}
```

### Decision Collection Type
Following the pattern of ReviewApprovals and ReviewInstructions:

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReviewDecisions {
    // Map decisions by number for lookup
    pub decisions: HashMap<u32, Decision>,

    // Index mapping ReviewableDiffId -> Vec<DecisionNumbers>
    // This allows querying "which decisions affect this code block?"
    pub decision_index: HashMap<ReviewableDiffId, Vec<u32>>,
}

impl ReviewDecisions {
    pub fn add_decision(&mut self, decision: Decision);
    pub fn get_decision(&self, number: u32) -> Option<&Decision>;
    pub fn get_decisions_for_diff(&self, reviewable_id: &ReviewableDiffId) -> Vec<&Decision>;
    pub fn all_decisions(&self) -> Vec<&Decision>;
}
```

## Integration Points

### 1. ReviewState Integration
Add to ReviewState in diffviz-review/src/state/mod.rs:
```rust
pub struct ReviewState {
    // ... existing fields ...
    pub decisions: ReviewDecisions,
}
```

### 2. ReviewEngine Decision API
Add methods to ReviewEngine for querying:
```rust
pub fn get_decisions(&self, reviewable_id: &ReviewableDiffId) -> Vec<&Decision>;
pub fn set_decision_index(&mut self, decisions: ReviewDecisions);
```

### 3. TUI Integration
In diffviz-review-tui, decisions are display-only in Phase 1:
- File list shows decision number badges
- No navigation changes yet (that's Phase 2)
- Hardcoded data in main.rs test binary

## Design Rationale

1. **Entity-Centric Design**: Following existing patterns with individual Decision type and ReviewDecisions collection
2. **Function-Level Granularity**: Map decisions to line ranges (function boundaries), not exact diff lines - simpler to generate and understand
3. **HashMap for Decision Index**: Quick lookup of decisions affecting a ReviewableDiffId
4. **Serde Support**: Enable future JSON loading and serialization
5. **No Fallbacks**: Fail-fast approach per CLAUDE.md - decisions either load completely or fail
6. **Display-Only in Phase 1**: No decision navigation, just showing decision context on files

## Future Phases

**Phase 2**: Decision navigation view, JSON loading, decision list/detail UI
**Phase 3**: dev-contribute integration for automatic mapping generation
