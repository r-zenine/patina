# Context Handoff - Phase 4 Implementation (Agent REPL)

## 🎯 Core Result
**Built**: `--agent-repl` on every ELMApp (`tui-harness/src/repl.rs` + one `AgentMode::Repl` arm). NDJSON in, one compact observation line out (`state` + `visual` + `affordances`). Sessions survive all error classes; `keys` errors report `applied:N`. Commit `f3ca860`; 12 in-process protocol tests + the phase-gate subprocess test against review-tui (drill → navigate → observe, cursor persists across commands).

**Key insight**: review-tui inherited the flag with literally zero app-side code — the D003 bet (harness owns the CLI) paid out exactly as designed.

## 🚦 Current State
**✅ Solid foundation**: 164 diffviz + 45 tui-harness + 18 sam tests green, clippy "No issues found". The whole plan surface is now live on review-tui: `--describe`, registry-backed bindings/affordances, persistent sessions.

**⚠️ Needs attention**: REPL renders at the default 80×24 (`RenderTestHarness::new()`); a session cannot change terminal size. If an agent needs wide renders, that's a protocol extension (e.g. optional `cols`/`rows` on `render`) — don't add until asked (YAGNI).

**⏸️ Deferred**: MCP shim (D005 — when a consumer exists); request ids/pipelining (rejected in 006 design).

## 👥 Next Agent Guidance
**Phase 5 (sam-tui, final)**: Gate 1 design objective — where the agent CLI entry lives (sam-tui is a lib; the binary is `sam-cli`'s `sam`). Then: minimal `describe()` + hand-rolled `affordances()` on modal_view (copy the pattern from `diffviz-review-tui/src/app.rs`, but WITHOUT a registry — D006), and update the `diffviz-tui-contribution` skill docs (point agents at `--describe` first, document `--agent-repl`, delete hardcoded keybinding tables = drift copy #4 retired).

## 🔗 Integration Points
**Expects**: `parse_input_sequence`/`InputStep::apply` semantics (Wait sleeps + ticks); `build_manifest`; `RenderTestHarness`.
**Provides**: `run_repl`, `run_repl_io`, `REPL_PROTOCOL_VERSION`; `--agent-repl` on review-tui and kitchen_sink today, sam's entry after Phase 5.

## 📋 Reference Links
- [decision-log.yaml](decision-log.yaml) - Technical choices made
- [../006-phase-4-design-repl-protocol-fable/design-doc.md](../006-phase-4-design-repl-protocol-fable/design-doc.md) - The protocol this implements
