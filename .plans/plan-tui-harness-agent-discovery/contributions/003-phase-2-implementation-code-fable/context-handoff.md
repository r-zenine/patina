# Context Handoff - Phase 2 Implementation (diffviz-review-tui Adopts the Agent CLI)

## 🎯 Core Result
**Built**: `review-tui --describe` works — versioned manifest with app identity, 3 mode docs, notation grammar, and the full 17-field StateSnapshot schema (doc comments surface as schema descriptions). `main.rs` shrank to a single `run_agent_cli` delegation; the three hand-rolled test modes (incl. the dead `--test-render` stub) are deleted. Commit `854ef78`; 7 binary-level tests via `CARGO_BIN_EXE_review-tui`.

**Key insight**: agent modes work WITHOUT `--features test-harness` now — `run_agent_cli` is unconditional. The feature still gates only the library's `test_harness` wrapper module.

## 🚦 Current State
**✅ Solid foundation**: `--test-input`/`--test-full` outputs diffed byte-identical against pre-migration goldens. 156 diffviz tests green, workspace clippy clean.

**⚠️ Needs attention**: skill docs (`.claude/skills/diffviz-tui-contribution/`) still say test modes need `--features test-harness` and don't mention `--describe` — Phase 5 updates them (deliberately deferred, roadmap 5c). Unknown flags now exit non-zero (was: silent fall-through to TUI).

**⏸️ Deferred**: `bindings: []` in the manifest — pinned by test `describe_bindings_stay_empty_until_the_registry_exists`, which Phase 3 must intentionally flip to assert registry-generated content.

## 👥 Next Agent Guidance
**Phase 3 (registry, next)**: Gate 1 first — the registry-shape design objective (roadmap 3a) via design-contribute: how (mode, submenu) scoping, aliases, parametric catch-alls (`InputChar(c)`, `EnterLeaderSubmenu(c)`) are represented so dispatch, which_key, help_overlay, manifest, and affordances read the same rows. Then: Phase 0's `dispatch_characterization_tests.rs` must pass UNCHANGED; flip the empty-bindings test; wire `describe().bindings` + `affordances()` from the registry filtered by `ui_state`.

**Phase 4 (REPL)**: `run_agent_cli`'s match in `agent_cli.rs` is where `--agent-repl` slots in; review-tui inherits it with zero app-side code.

## 🔗 Integration Points
**Expects**: `create_test_review_engine()` (main.rs) remains the binary's fixture factory; `AppDescription`/`ModeDoc` from tui-harness.
**Provides**: A discoverable review-tui binary — manifest + headless modes — that Phase 3 enriches (bindings/affordances) and Phase 4 makes session-persistent.

## 📋 Reference Links
- [decision-log.yaml](decision-log.yaml) - Technical choices made
