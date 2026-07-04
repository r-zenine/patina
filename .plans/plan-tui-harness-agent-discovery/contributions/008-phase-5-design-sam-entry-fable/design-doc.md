# Design Document - sam-tui Agent CLI Entry Point

> **Target**: < 100 lines total
> **Note**: Scope pre-pinned by D006 (minimal manifest, hand-rolled
> affordances, no registry for sam).

## Decision: Fixture-Driven Example Binary in sam-tui

The agent CLI entry for sam's modal view is `sam-tui/examples/modal_agent.rs`
— a fixture-driven example (mock items + one option toggle, multi-select on)
that calls `run_agent_cli(HeadlessModalView::new(...), args)`. Runnable as
`cargo run -p sam-tui --example modal_agent -- --describe` (or
`--agent-repl`, `--test-input`, `--test-full`).

## Why This Design

**Constraints That Led Here:**
- Must not affect sam's production CLI UX (`sam` is sam-cli's binary; its
  arg surface is user-facing and must not grow agent flags)
- sam-tui is a library — it has no binary today, and its modal view is
  generic over `Value`, so any runnable entry needs concrete fixture data
- The existing test suite already builds `HeadlessModalView<MockValue>`
  fixtures; the example reuses that exact construction

**Simplicity Rationale:**
Examples don't ship, don't add features to production binaries, and mirror
the kitchen_sink precedent exactly. A hidden `sam __tui-agent` subcommand
would still be reachable production surface; a feature-gated `[[bin]]` is
an example with extra configuration.

## How It Works

**Key Interfaces:**
- `HeadlessModalView::describe()`: minimal `AppDescription` — app
  name/version, two `ModeDoc`s (InsertMode, OptionsMode), bindings from a
  single hand-written const list
- `HeadlessModalView::affordances()`: the same const list filtered by
  construction config (`has_options` gates Esc/ToggleViewMode;
  `allow_multi_select` gates Ctrl-s/Ctrl-a)

**Core Pattern:**
One `&[SamBindingDoc]`-style const in `headless.rs` feeds both `describe()`
and `affordances()` — hand-rolled (per D006) but single-sourced within sam,
so sam's two discovery surfaces cannot disagree with each other. Dispatch
(`key_transformer`) remains untouched: no registry for ~12 keys.

**Integration Points:**
- `sam-tui/examples/modal_agent.rs` → `tui_harness::run_agent_cli`
- Skill docs (`.claude/skills/diffviz-tui-contribution/`) updated in the
  same phase: agents start from `--describe`, `--agent-repl` documented,
  hardcoded keybinding tables removed (drift copy #4 retired)

## What We're NOT Doing

**Rejected Alternatives:**
- **`sam __tui-agent` hidden subcommand**: production arg surface, violates
  the no-UX-impact constraint
- **Feature-gated `[[bin]]` in sam-tui**: an example with more ceremony
- **Binding registry for sam**: D006 — ~12 keys in one match, speculative

**Out of Scope:**
- Wiring discovery into sam-cli's real modal invocations (needs a concrete
  consumer; the example proves the abstraction generalizes, which is the
  phase goal)

## Implementation Guidance

**For Next Contributor:**
- Copy the affordance-conversion shape from `diffviz-review-tui/src/app.rs`
  but source it from the const list, not a registry
- Filter by `self.has_options` / `self.allow_multi_select` — config known at
  construction

**Testing Strategy:**
- `sam-tui/tests/agent_discovery_tests.rs`: manifest carries the
  SamSnapshot schema; affordances differ between multi-select on/off and
  options on/off; existing 18 harness tests untouched

**Success Criteria (plan end-to-end gate):**
- Given only a binary path, an agent can `--describe`, pick a key from
  `bindings`, drive it via `--agent-repl`, and observe the affordance
  change — verified as a scripted check for both review-tui and modal_agent
