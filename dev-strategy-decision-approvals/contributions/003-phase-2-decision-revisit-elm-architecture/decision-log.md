# Decision Revisit - Callback Handling in Cascade Methods

## Context

Contribution 002 made a decision to use callbacks for UI feedback in cascade methods:
- `approve_decision(decision_number, author, callback: Option<Box<dyn FnOnce(...) + Send>>)`
- `reject_decision(decision_number, author, callback: Option<Box<dyn FnOnce(...) + Send>>)`

However, diffviz-review-tui is built around ELM architecture principles, where:
- Components are pure functions that render state (not emit callbacks)
- Events flow as messages through a central dispatcher
- UI feedback comes from state changes that trigger re-renders
- Side effects are deferred and executed separately

This decision needs revisiting to align with the TUI's architectural patterns.

---

## 2026-01-23T00:00:00Z - Callback Handling Approach Options for ELM Architecture

**Category**: API Design - TUI Integration

**Problem**: The current callback approach doesn't fit ELM architecture. Need alternatives that:
1. Provide UI feedback about cascade operations
2. Respect the event-driven message pattern
3. Integrate with existing approval/business event flow
4. Enable TUI to show operation results without breaking architecture

**Considered Alternatives**:

### Option A: No Callbacks, State-Based Feedback
- Remove callback parameter entirely
- Return success/failure from methods
- TUI reads updated state on next render cycle
- Feedback shown via progress indicator updates in components

**Pros**:
- Pure ELM: operations are queries returning Result<()>
- No callbacks at all - aligns with functional style
- Components automatically show updates without polling
- Simplest API surface

**Cons**:
- No immediate feedback about what changed
- TUI must infer operation scope from state comparison
- User sees changes appear on next render (minor latency)

**Example**:
```rust
pub fn approve_decision(&mut self, decision_number: u32, author: String) -> Result<()> {
    // ... approve logic ...
    // Return just success/failure
}

// In TUI: automatic feedback on next render cycle
// Components show: "Decision #1: 3/5 chunks approved"
```

---

### Option B: Return Change Summary
- Methods return `OperationSummary` struct with what changed
- TUI stores summary in UI state
- Components can display summary (e.g., in status bar)
- Summary cleared after N renders or on next user action

**Pros**:
- TUI has explicit information about what happened
- Can show "Approved Decision #1 and 3 chunks" in status bar
- Still no callbacks - summary is just a return value
- Fits ELM model: return data that triggers re-render

**Cons**:
- New struct needed (OperationSummary)
- TUI must manage summary state (when to clear it)
- Slightly more complex than pure Result<()>

**Example**:
```rust
pub struct OperationSummary {
    pub decision_number: u32,
    pub chunks_affected: usize,
    pub success: bool,
    pub message: String,
}

pub fn approve_decision(&mut self, decision_number: u32, author: String) -> Result<OperationSummary> {
    // ... logic ...
    Ok(OperationSummary {
        decision_number,
        chunks_affected: 3,
        success: true,
        message: "Decision #1 and 3 chunks approved".to_string(),
    })
}

// In TUI UI state:
struct UiState {
    last_operation: Option<OperationSummary>,
    operation_shown_until_frame: u64,
}

// In render: status_bar.rs shows last_operation if recent
```

---

### Option C: Business Event Emission
- Methods emit new `BusinessEvent` variants describing what happened
- Event queue in ReviewEngine that TUI can poll
- TUI handles events like `DecisionApproved { decision_number, chunks: usize }`
- Feedback shown through existing event handler pattern

**Pros**:
- Natural fit for ELM: events drive state changes
- Reuses existing `BusinessEvent` enum
- TUI already knows how to handle business events
- Events can trigger other side effects (commands, exports)
- Supports transaction-like semantics (event for each chunk approved)

**Cons**:
- Requires event queue infrastructure in ReviewEngine
- More complex: multiple events per operation
- Could introduce ordering dependencies

**Example**:
```rust
pub enum BusinessEvent {
    DecisionApproved { decision_number: u32, chunks_affected: usize },
    DecisionUnapproved { decision_number: u32, chunks_affected: usize },
    // ...
}

// ReviewEngine result includes events
pub fn approve_decision(&mut self, decision_number: u32, author: String) -> Result<Vec<BusinessEvent>> {
    // ... logic ...
    let events = vec![
        BusinessEvent::DecisionApproved { decision_number: 1, chunks_affected: 3 }
    ];
    Ok(events)
}

// In TUI: process returned events
for event in engine.approve_decision(...)? {
    self.handle_business_event_result(event)?;
}
```

---

### Option D: Cascading Returned as Detailed Operation Result
- Return structured `CascadeResult` with all details
- Enum variants for different operation types
- Includes what-changed info and any warnings/errors per chunk

**Pros**:
- Explicit: shows exactly what happened
- Transaction semantics: can validate entire cascade before committing
- Handles partial failures gracefully
- TUI gets comprehensive operation details

**Cons**:
- Most complex to implement
- Overkill if cascade either succeeds or fails completely
- More data structures needed

**Example**:
```rust
pub enum CascadeResult {
    DecisionApproved {
        decision_number: u32,
        chunks_approved: Vec<ReviewableDiffId>,
    },
    DecisionUnapproved {
        decision_number: u32,
        chunks_unapproved: Vec<ReviewableDiffId>,
    },
    PartialSuccess {
        decision_number: u32,
        chunks_approved: Vec<ReviewableDiffId>,
        failures: Vec<(ReviewableDiffId, String)>,
    },
}

pub fn approve_decision(&mut self, decision_number: u32, author: String) -> Result<CascadeResult> {
    // ... logic ...
}
```

---

## Summary Table

| Option | Callback | Complexity | ELM Fit | TUI Feedback |
|--------|----------|-----------|---------|-------------|
| A: State-Based | ❌ | Low | ✅ Excellent | Implicit via re-render |
| B: Summary Return | ❌ | Medium | ✅ Good | Explicit via return data |
| C: Event Emission | ❌ | Medium | ✅✅ Excellent | Native event handling |
| D: Detailed Result | ❌ | High | ✅ Good | Rich detail available |

---

## Next Steps

1. **Gather user input** on which option aligns best with:
   - Desired TUI feedback richness
   - Architectural consistency goals
   - Implementation complexity tolerance

2. **Once chosen**: Update all cascade methods to use selected approach
   - Remove callback parameters
   - Implement return type from chosen option
   - Update ReviewEngine integration tests
   - Update TUI integration points

3. **TUI Integration**: Update handlers to work with chosen feedback mechanism
