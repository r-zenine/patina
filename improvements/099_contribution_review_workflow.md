# Contribution Review Workflow

## Overview

Decision-based review workflow that integrates dev-contribute, dev-strategy, and diffviz-cli review into a focused, opinionated process.

**Core Principle:** Reviews are structured around decisions (the "why") not just code changes (the "what").

## Requirements

### R1: Decision-Based Reviews
- Reviews must be organized around discrete decisions, not raw diffs
- Each decision must have: question, decision, rationale, impact
- Code changes must be associated with specific decisions

### R2: Review Blocking
- dev-contribute must check for unreviewed contributions before starting new work
- If unreviewed contributions exist, dev-contribute must block and prompt user to review
- If changes were requested in review, dev-contribute must address feedback before new work

### R3: Review State Persistence
- Review progress must persist across sessions
- If TUI closes mid-review, user can resume from last reviewed decision
- Review state must track: which decisions reviewed, approval/changes status, comments

### R4: Default to Latest Contribution
- diffviz review should default to reviewing the most recent contribution
- Should compare HEAD^1..HEAD by default (last commit)
- User shouldn't need to manually specify commit ranges

### R5: Semantic Code Filtering
- For each decision, only show code changes relevant to that decision
- Filter by files and symbols mentioned in decision scope
- Must gracefully handle cases where symbols don't match

### R6: Structured Metadata
- Use YAML for all metadata (decision logs, review status)
- Must be both human-readable and machine-parseable
- Schema must be strict and validated

## Contribution Structure

```
contributions/NNN-phase-X-type-specialty-agent/
├── decision-log.yaml        # Agent-generated, immutable
├── review_status.yaml       # TUI-generated, mutable
├── changelog.md             # Human-readable summary
└── context-handoff.md       # Context for next contribution
```

## YAML Schemas

### decision-log.yaml

```yaml
metadata:
  contribution: "001-phase-1-implementation-test-engine"
  phase: 1
  agent: "dev-contribute"
  created: "2026-01-24T10:30:00Z"
  commit_range: "HEAD^1..HEAD"

decisions:
  - id: 1
    question: "How should we structure the test engine?"
    decision: "Use separate test runner with fixture isolation"
    rationale: "Prevents test pollution and enables parallel execution"
    impact: "Requires test_utils module with TestFixture trait"
    scope:
      files:
        - "src/test_engine.rs"
        - "src/test_utils.rs"
      symbols:
        - "TestEngine"
        - "TestFixture"
```

### review_status.yaml

```yaml
contribution: "001-phase-1-implementation-test-engine"
started_at: "2026-01-24T10:30:00Z"
last_updated: "2026-01-24T10:45:00Z"
reviewer: "ryad"
completed: false

decisions:
  - decision_id: 1
    status: "approved"  # approved | changes_requested | pending
    reviewed_at: "2026-01-24T10:35:00Z"
    comments: null

  - decision_id: 2
    status: "changes_requested"
    reviewed_at: "2026-01-24T10:42:00Z"
    comments: "Consider using Arc instead of Rc for thread safety"

  - decision_id: 3
    status: "pending"
    reviewed_at: null
    comments: null
```

## Expected Workflow

### Agent Contribution Flow

1. dev-contribute starts
2. **Pre-work validation check:**
   - Scan `contributions/` for folders
   - Check each for `review_status.yaml`
   - If any contribution has `completed: false`:
     - Block and display: "Contribution {N} awaiting review. Run 'diffviz review' first."
     - Exit
   - If any contribution has status `changes_requested`:
     - Agent must address feedback before new work
   - If all reviews complete or no contributions exist:
     - Proceed
3. Agent performs work, making decisions
4. Agent generates `decision-log.yaml` with:
   - Each decision documented
   - Scope annotations (files, symbols) for each decision
5. Agent commits work (single commit or range)
6. Contribution folder created with decision-log.yaml

### Human Review Flow

1. User runs `diffviz review`
2. **Startup:**
   - Search for dev-strategy projects (via `implementation-roadmap.md`)
   - Display list of "on going" projects
   - User selects project
3. **Contribution selection:**
   - Scan `contributions/` folder
   - Find first contribution where `review_status.yaml` is missing or `completed: false`
   - Load `decision-log.yaml` from that contribution
   - Load `review_status.yaml` if exists (resume case)
4. **Review session:**
   - Get git diff for `commit_range` from decision-log metadata
   - Parse diff using semantic analysis (ReviewableDiff)
   - For each decision (starting from first with status `pending`):
     - **Display decision context:**
       - Question
       - Decision made
       - Rationale
       - Impact
     - **Display filtered code changes:**
       - Filter ReviewableDiff to `scope.files`
       - Match semantic units containing `scope.symbols`
       - Show only relevant changes
     - **Collect user input:**
       - Approve OR Request Changes
       - If Request Changes: capture comment text
     - **Persist immediately:**
       - Update `review_status.yaml` with decision status
       - Write to disk
       - Continue to next decision
5. **Session completion:**
   - If all decisions reviewed: set `completed: true` in review_status.yaml
   - If TUI closed mid-review: partial state preserved in review_status.yaml
   - On next launch: resume from first decision with status `pending`

### Feedback Loop

1. dev-contribute starts next session
2. Checks `review_status.yaml` from previous contribution
3. **If changes requested:**
   - Read comments for each decision marked `changes_requested`
   - Agent addresses feedback
   - Modifies code
   - Creates new commit
   - User reviews again (cycle repeats)
4. **If all approved:**
   - Proceed with next contribution
   - Clean slate

## Expected Behaviors

### B1: Review State Persistence
- After reviewing each decision, state written to `review_status.yaml`
- If TUI crashes or user closes it, progress is not lost
- Next launch loads `review_status.yaml` and skips to first `pending` decision

### B2: Semantic Filtering
- For decision with `scope.files: ["src/foo.rs"]` and `scope.symbols: ["Foo"]`:
  - Load full diff
  - Filter to changes in src/foo.rs
  - Within that file, show only changes to:
    - `struct Foo`
    - `impl Foo`
    - `fn foo_helper()` (fuzzy match on "Foo")
- If no symbols match: show all changes in scope.files
- If no files match: show full diff (graceful degradation)

### B3: Blocking Behavior
- dev-contribute refuses to start if `review_status.yaml` exists with `completed: false`
- Error message must be clear and actionable
- No override mechanism (enforce review discipline)

### B4: Default Review Target
- `diffviz review` without arguments defaults to HEAD^1..HEAD
- Reviews the most recent commit
- User can override with explicit commit range if needed

### B5: Atomic Review Updates
- Each decision review is committed to `review_status.yaml` immediately
- No batch updates at end of session
- Prevents data loss if session interrupted

### B6: Structured Decision Validation
- decision-log.yaml must have valid schema
- Required fields: id, question, decision, rationale, impact, scope
- Missing or invalid fields cause hard error (no fallbacks)
- Error messages show which field is invalid and why

### B7: Resume Mid-Review
- When review_status.yaml exists with `completed: false`:
  - Load both decision-log.yaml and review_status.yaml
  - Display summary: "Resuming review: 2/5 decisions reviewed"
  - Start from first decision with `status: pending`
  - Preserve all previous review decisions

### B8: Changes Requested Workflow
- When user marks decision as "changes_requested":
  - Must capture comment (required, cannot be empty)
  - Comment written to review_status.yaml
  - dev-contribute sees this on next startup
  - Agent must address before proceeding

### B9: All or Nothing Completion
- Review is only `completed: true` when ALL decisions have status != `pending`
- Partial reviews never mark contribution as complete
- dev-contribute only proceeds when `completed: true` and all `approved`

## Edge Cases

### E1: TUI Closed Mid-Decision
- Current decision not yet reviewed (user didn't approve/request changes)
- That decision remains `status: pending`
- Next launch asks user to review that decision again

### E2: Malformed YAML
- If decision-log.yaml is invalid: hard error, clear validation message
- If review_status.yaml is corrupt: hard error, cannot resume
- No silent fallbacks, no default values

### E3: Git Range Invalid
- If commit_range in decision-log.yaml doesn't exist: hard error
- Message must suggest checking git log

### E4: No Symbols Found
- decision has `scope.symbols: ["Foo"]` but no Foo in diff
- Fallback: show all changes in scope.files
- Display warning: "Symbol 'Foo' not found, showing all changes in scoped files"

### E5: Multiple Contributions Unreviewed
- Only the oldest unreviewed contribution blocks dev-contribute
- User must review contributions in order
- Cannot skip to newer contributions

### E6: User Deletes review_status.yaml
- Treated as if review never happened
- dev-contribute blocks until review completed
- User must review from scratch
