# Code Context: display review summary command

## Entry Point

### `diffviz-cli/src/main.rs` — CLI definition and dispatch
- Lines 19-96: `Cli` struct and `Commands` enum — add `Display` variant here
- Lines 157-219: `match (cli.folder, cli.command)` dispatch — wire new command here
- Pattern: `Templates` and `Validate` commands show how to run without `Environment` (no git needed)

## Command Pattern (no-Environment)

### `diffviz-cli/src/commands/templates.rs` — reference command (no git)
- Shows how a command reads a file path, does pure processing, writes to stdout
- Same pattern to follow for `DisplayCommand`

### `diffviz-cli/src/commands/mod.rs` — command registry
- Lines 6-9: module declarations — add `pub mod display;` here
- `CommandExecutor` trait defined here (not needed for no-Environment commands)

## Domain Types (diffviz-review)

### `diffviz-review/src/entities/decision.rs` — decision domain
- `DecisionLog` (line 96): `commit: String`, `decisions: Vec<Decision>` — parse with `DecisionLog::parse(content)`
- `Decision` (line 61): `number: u32`, `title: String`, `rationale: Option<String>`, `code_impacts: Vec<CodeImpact>`
- `CodeImpact` (line 35): `file: String`, `reasoning: String`, `line_ranges: Vec<DecisionLineRange>`
- `DecisionLineRange` (line 22): `start: usize`, `end: usize`

### `diffviz-review/src/persistence.rs` — review-state deserialization
- `ReviewStateFile` (line 33): private struct with `approvals`, `instructions`, `decision_approvals`
- Currently private — the summarizer needs direct `serde_json` deserialization of the file
- Alternative: add a `pub fn read_review_state_raw(folder: &Path) -> Result<ReviewStateSummary>` to persistence.rs

### `diffviz-review/src/entities/instruction.rs` — instruction types
- `Instruction` (line 29): `id`, `author`, `timestamp`, `content`, `status: InstructionStatus`
- `InstructionStatus` (line 14): `Active | Addressed`
- `ExportedInstruction` in `engines/review_engine/export_import.rs` (line 12): flat format used in JSON

### `diffviz-review/src/lib.rs` — public re-exports
- `DecisionLog`, `Decision`, `CodeImpact`, `DecisionLineRange` all exported
- `ApprovalRecord`, `DecisionApprovals`, `ReviewApprovals` all exported

## Review-State JSON Schema (from persistence.rs)
```
{
  "approvals": [{ "reviewable_id": {...}, "approved": bool, "approved_by": str, "approval_timestamp": str }],
  "instructions": { "_meta": {...}, "instructions": [...] },
  "decision_approvals": [{ "decision_number": u32, "approved": bool, "approved_by": str, "approval_timestamp": str }]
}
```

## Decision-Log YAML Schema (from entities/decision.rs)
```yaml
commit: "<hash>"
decisions:
  - number: 1
    title: "..."
    rationale: "..."
    code_impacts:
      - file: "path/to/file.rs"
        reasoning: "..."
        line_ranges:
          - start: 10
            end: 20
```
