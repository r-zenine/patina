# Context Handoff - Phase 3 Implementation (Keybinding Registry)

## 🎯 Core Result
**Built**: the keybinding registry (`events/bindings.rs`) — one static table feeding dispatch, which-key, help overlay, `--describe` bindings, and per-state affordances. 250 lines of match arms deleted; `--test-full` now prints an `Affordances:` section per step. Commit `fd91a4c`.

**Key insight**: the Phase 0 characterization suite passed UNCHANGED on the first run after the dispatch swap — zero behavior drift, proven not asserted. Affordances are now live end-to-end: `--test-full "<Space>a"` shows exactly the actions-submenu moves.

## 🚦 Current State
**✅ Solid foundation**: 163 diffviz + 33 tui-harness + 18 sam tests green, clippy clean. Registry invariants tested (no duplicate keys, docs aligned, submenus reachable).

**⚠️ Needs attention**: which-key wording is now static per row (lost the browse-cursor-sensitive "Approve decision/chunk" variants — decision 2). The `--test-full` format grew an Affordances section; anything parsing that output by section must tolerate it.

**⏸️ Deferred**: leader modifier-blindness quirk kept (Phase 0 pin); `ExportAll`/`LeaderTimeout` UiEvent variants remain unbound in the registry (they were unbound before too — synthetic/unused events, not regressions).

## 👥 Next Agent Guidance
**Phase 4 (REPL, next)**: Gate 1 design objective first — the NDJSON protocol (commands `keys`/`describe`/`render`/`quit`, error semantics, envelope version). Build generically in `tui-harness/src/repl.rs` over `parse_input_sequence` + `InputStep::apply` + `RenderTestHarness`; every response should carry affordances (they're on `CombinedTestResult` now). Wire the flag into `agent_cli.rs::run_agent_cli` — review-tui inherits it with zero app code.

**Phase 5 (sam-tui)**: `Affordance` conversion pattern to copy is `app.rs::affordances()` (registry-backed) — sam's stays hand-rolled per D006.

## 🔗 Integration Points
**Expects**: `UiEvent: Clone + PartialEq`; registry rows const-constructible.
**Provides**: `bindings::{BINDINGS, lookup, scope_of, bindings_for, catch_all_for, scope_label, SUBMENUS}`; affordances on every combined-harness step.

## 📋 Reference Links
- [decision-log.yaml](decision-log.yaml) - Technical choices made
- [../004-phase-3-design-registry-fable/design-doc.md](../004-phase-3-design-registry-fable/design-doc.md) - The design this implements
