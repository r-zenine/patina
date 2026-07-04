# Design Document - Agent REPL Protocol

> **Target**: < 100 lines total
> **Note**: Constraints pre-pinned by plan decision D005 (NDJSON over stdio,
> no MCP) and Phase 3 (affordances available on every step).

## Decision: Four Commands, One Uniform Observation Envelope, Sessions Survive Errors

`--agent-repl` reads one JSON object per stdin line and writes exactly one
single-line JSON response per command. Observation is uniform: every
successful `keys`/`render` response carries `state` + `visual` +
`affordances`, so the agent's loop is always act → observe legal moves.
Errors never kill the session — a typo must not cost an agent its
exploration state.

## Why This Design

**Constraints That Led Here:**
- Responses must be single-line (line-delimited framing) — compact JSON, not
  pretty-printed
- The session exists to eliminate O(n²) prefix replay, so surviving bad
  input is the core property: state is the asset being protected
- Affordances per step already exist (`CombinedTestResult`, Phase 3)

**Simplicity Rationale:**
One envelope shape for all observations means an agent parses one schema.
No subscriptions, no ids, no pipelining — strictly request/response, which
stdio ordering gives for free.

## How It Works

**Requests** (`{"cmd": ...}` tagged):
- `{"cmd":"keys","input":"jj<Wait:100>k"}` — apply steps one at a time
  (reuses `parse_input_sequence` + `InputStep::apply`; Wait sleeps + ticks)
- `{"cmd":"render"}` — re-observe without acting
- `{"cmd":"describe"}` — full manifest (same JSON as `--describe`)
- `{"cmd":"quit"}` — acknowledge, then exit 0

**Responses** (all carry `"v":1`):
- keys/render ok: `{"v":1,"ok":true,"state":{...},"visual":"...","affordances":[...]}`
- describe ok: `{"v":1,"ok":true,"manifest":{...}}`
- quit ok: `{"v":1,"ok":true}`
- error: `{"v":1,"ok":false,"error":"..."}`; `keys` errors add
  `"applied":N` (steps applied before failure) since partial application
  mutates state the agent must account for

**Error semantics (session always survives):**
- malformed JSON line → error response, next line processed normally
- unknown `cmd` → error response
- unparseable key sequence → error, `applied: 0`
- app error mid-sequence → error with `applied` count; agent can `render`
  to re-observe the partial state
- EOF on stdin → clean exit 0 (agent hangup is not a failure)
- `should_quit() == true` does NOT end the session — the snapshot shows it
  (`should_quit: true`) and the agent decides; a headless session may want
  to inspect the final state

**Implementation home:**
- `tui-harness/src/repl.rs`: `run_repl_io<M: ELMApp>(app, impl BufRead,
  impl Write)` — generic over streams so tests run in-process with byte
  buffers; `run_repl(app)` binds stdio
- `agent_cli.rs`: `AgentMode::Repl` for `--agent-repl`; apps inherit with
  zero app-side code

## What We're NOT Doing

**Rejected Alternatives:**
- **Request ids / pipelining**: stdio ordering is the correlation; ids are
  ceremony until a multiplexed transport (MCP) exists
- **Pretty-printed responses**: breaks line framing; agents parse JSON, not
  humans
- **Auto-exit on should_quit**: throws away observable final state and makes
  `q` in a key sequence a session-killer
- **Visual on request only**: saving bytes by omitting `visual` makes the
  common loop two commands instead of one

**Out of Scope:**
- MCP server wrapping this protocol (D005 — when a consumer exists)
- Timeout/idle handling (the parent process owns the child's lifetime)

## Implementation Guidance

**For Next Contributor:**
- Deserialize requests with a `#[serde(tag = "cmd")]` enum; serialize
  responses with `serde_json::json!` — no response structs needed
- Reuse `build_manifest`, `RenderTestHarness`, `InputStep::apply`

**Testing Strategy:**
- In-process: byte-buffer streams through `run_repl_io` (happy paths, every
  error class, session survival, Wait ticking, quit, EOF)
- Subprocess: one scripted multi-command session against the review-tui
  binary proving state persists across commands (the phase gate)

**Success Criteria:**
- A 3-command scripted session (drill in → navigate → observe) against
  review-tui shows cursor state persisting between commands
