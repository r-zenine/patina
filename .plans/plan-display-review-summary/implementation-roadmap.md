# Implementation Roadmap: display review summary command

## Strategy: Steel Thread

One phase delivers the complete vertical slice: domain logic → CLI wiring → working command.
No TDD overhead needed — the logic is a straightforward cross-reference between two files.

---

## Phase 1 — Working `diffviz display review` command

**Objective**: Implement the full feature as a single deliverable.

**Deliverables**:
- `diffviz-review/src/summary.rs` — `ReviewSummary` domain struct + `to_yaml()` 
- `diffviz-review/src/persistence.rs` — `summarize_review_state(folder)` function
- `diffviz-review/src/lib.rs` — re-exports
- `diffviz-cli/src/commands/display.rs` — `DisplayReviewCommand`
- `diffviz-cli/src/commands/mod.rs` — `pub mod display`
- `diffviz-cli/src/main.rs` — `Commands::Display` + `DisplayCommands::Review`

**Acceptance criteria**:
1. `diffviz display review .plans/plan-tui-error-consolidation/contributions/002-phase-2-implementation-code-general-purpose/` emits valid YAML to stdout
2. YAML contains `decisions.approved` matching all 4 `decision_approvals` entries in the real review-state.json
3. YAML contains `decisions.unapproved: []` (since all are approved in that fixture)
4. Running against a folder with no `review-state.json` shows all decisions as unapproved
5. `cargo clippy --workspace` → zero warnings

**Implementation order** (bottom-up, each step compiles):
1. `diffviz-review/src/summary.rs` — data types + `to_yaml()`
2. `diffviz-review/src/persistence.rs` — `summarize_review_state()`  
3. `diffviz-review/src/lib.rs` — add exports, `cargo check -p diffviz-review`
4. `diffviz-cli/src/commands/display.rs` — command struct
5. `diffviz-cli/src/commands/mod.rs` + `main.rs` — wiring, `cargo check --workspace`
6. `cargo clippy --workspace && cargo fmt --all`
7. Manual smoke test with real `.plans/` fixtures

**Key data model for `ReviewSummary`**:
```rust
pub struct ReviewSummary {
    pub commit: String,
    pub contribution_folder: String,
    pub approved_decisions: Vec<ApprovedDecisionEntry>,
    pub unapproved_decisions: Vec<UnapprovedDecisionEntry>,
    pub active_instructions: Vec<InstructionEntry>,
    pub addressed_instructions: Vec<InstructionEntry>,
}

pub struct ApprovedDecisionEntry { number, title, approved_by, approval_timestamp }
pub struct UnapprovedDecisionEntry { number, title, rationale, code_impacts }
pub struct InstructionEntry { file, lines, content, author, timestamp }
```

All fields serialize to the YAML shape specified in context-document.md.

**`summarize_review_state` logic**:
```
1. Read + parse decision-log.yaml → DecisionLog
2. Try read review-state.json; if missing → empty approvals
3. Build HashSet of approved decision numbers from decision_approvals
4. For each Decision in DecisionLog:
   - if in approved set → ApprovedDecisionEntry
   - else → UnapprovedDecisionEntry (full rationale + code_impacts)
5. Parse instructions from review-state.json.instructions.instructions
6. Split into active / addressed by status field
7. Return ReviewSummary
```
