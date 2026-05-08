# Context Document: display review summary command

## Behavioral Spec

### Command
```
diffviz display review <folder>
```
where `<folder>` is a contribution folder path (e.g. `.plans/plan-foo/contributions/001-phase-1-...`).

### What it does
1. Reads `<folder>/decision-log.yaml` — authoritative list of decisions that should be reviewed
2. Reads `<folder>/review-state.json` — what has been approved so far (may not exist yet)
3. Cross-references: any decision in `decision-log.yaml` not present in `review-state.json`'s `decision_approvals` is **unapproved** and surfaces as an agent action item
4. Emits YAML to stdout for agent consumption

### Output YAML shape
```yaml
commit: "3a4670b"
contribution_folder: ".plans/plan-foo/contributions/001-phase-1-..."

decisions:
  approved:
    - number: 1
      title: "Add thiserror..."
      approved_by: "ryad"
      approval_timestamp: "2026-05-08 09:01:21 UTC"
  unapproved:
    - number: 5
      title: "Refactor error handling"
      rationale: "..."
      code_impacts:
        - file: "src/foo.rs"
          reasoning: "..."
          line_ranges:
            - start: 10
              end: 20

instructions:
  active:
    - file: "src/foo.rs"
      lines: "10-20"
      content: "Fix the error handling here"
      author: "ryad"
      timestamp: "2026-05-08T..."
  addressed: []

summary:
  total_decisions: 4
  approved_decisions: 4
  unapproved_decisions: 0
  total_instructions: 0
  active_instructions: 0
```

### Error cases
- `decision-log.yaml` missing → hard error (fail fast, no fallback)
- `review-state.json` missing → treat as empty (zero approvals), all decisions unapproved
- Invalid YAML/JSON → hard error with message

## Architecture

### Where the logic lives
- **`diffviz-review/src/persistence.rs`**: add `pub fn summarize_review_state(folder: &Path) -> Result<ReviewSummary, PersistenceError>` — reads both files, computes the cross-reference, returns a domain struct
- **`diffviz-review/src/summary.rs`** (new): `ReviewSummary` struct and its `to_yaml()` method (using `serde_yaml`)
- **`diffviz-cli/src/commands/display.rs`** (new): `DisplayCommand` that calls `summarize_review_state`, prints YAML
- **`diffviz-cli/src/main.rs`**: wire `Commands::Display { Review { folder } }` subcommand

### Why summarization logic in diffviz-review (not diffviz-cli)
The cross-reference between `decision-log.yaml` and `review-state.json` is domain logic — it reasons about review completeness. The CLI should only handle I/O and presentation, not business logic. This also makes the summarizer unit-testable in isolation.

### Dependency additions
- `serde_yaml` is already a workspace dependency (used by `entities/decision.rs` via `DecisionLog::parse`)
- No new dependencies needed

## Constraints
- Output is YAML only (no JSON flag needed for now — YAGNI)
- No git repository needed — pure file processing
- Zero warnings: run `cargo fmt --all && cargo clippy --workspace` after implementation
