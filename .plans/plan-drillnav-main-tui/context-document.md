# Context Document for DrillNav Main TUI

## Behavioral Specification

Replace the main diffviz review TUI's two-panel layout (file list + diff view with a decision tree) with the DrillNav experience prototyped in `examples/review_navigator.rs`. Reviewers land on a Browse screen of decision cards (title, rationale, affected files, chunk-approval progress), press Enter to drill into a decision — where the decision/file summary stays pinned while chunk cards scroll below — cycle sibling files with h/l (per-file position retained), approve decisions or chunks with `a` (decision approval cascades), expand code context with Tab, expand/collapse a chunk's note with `i`, and append to the note with `n` (single-note model: a chunk has exactly one note; adding appends). Inline reasoning annotations remain available; range selection and the semantic-highlight toggle are removed.

## Codebase Patterns to Follow

- **Architecture**: Elm — pure views (`&UiState`, never `&mut`), update in `app.rs` handlers returning `Command`, side effects only via `Command`, time via `on_tick`
- **Event flow**: `KeyEvent → UiEvent → BusinessEvent → Command` (`events/input.rs` → `events/business.rs` → `app.rs`)
- **State encapsulation**: nested state accessed through `UiState` methods only (V4 rule)
- **Design system**: tui-design cards/elevation — crust terminal floor, mantle widget floor, CardTier Content/Body/Header, surface2 reserved for selection, accent-bar focus
- **Testing**: harness-driven integration tests (`--features test-harness`), state assertions via `StateSnapshot` JSON, visual assertions via `CombinedTestHarness`
- **Errors**: engine failures surface as a one-shot status-bar message (prototype pattern), never silently discarded

## Technical Constraints

- `ReviewTuiApp::{new, run, into_review_engine}` signatures frozen — `diffviz-cli::run_contribution_review` depends on them
- Single-note-per-chunk invariant is engine-enforced (`InstructionMap::add_instruction` appends); UI must not assume multiple notes
- Chunk-granular cursor: j/k moves between chunks; `Ctrl-d/u` page the drill viewport, `g/G` jump; no line-level cursor inside chunks
- Leader-key system, which-key popup, help overlay, instruction input modal, and export (`Space-e` path) are retained
- ZERO WARNING rule: fmt + clippy + check clean after every phase
- Test harness must remain functional every phase — it is the only way agents (no TTY) can drive the TUI
- Old UI components deleted only in the final phase; no feature flag, no dual maintenance
- Keys freed by removals: `v` (range selection), leader `t-s` (semantic); `n` is free for note entry

## Local Repository Skills

- **diffviz-tui-contribution** (`.claude/skills/diffviz-tui-contribution/`) — MANDATORY for every phase: architecture compliance rules (V1–V5), testing workflow, rejection criteria. Note: its verification commands reference `decision_tree` fields — the skill file itself must be updated in the final phase.
- **filing-bugs** — if a phase surfaces an unrelated bug, file it as a skipped test
- **verify** — run before committing each phase

## Research Findings

No new technologies involved; no research phase executed. All building blocks (tui-design, engine APIs, harness) exist in-repo and were validated during prototyping (2026-06/07).
