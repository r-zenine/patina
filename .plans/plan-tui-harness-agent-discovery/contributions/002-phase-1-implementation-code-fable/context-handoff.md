# Context Handoff - Phase 1 Implementation (Discovery Contract + Generic Agent CLI)

## 🎯 Core Result
**Built**: `run_agent_cli` in tui-harness — every ELMApp now gets `--describe` / `--test-input` / `--test-full` from the harness. `ELMApp` gained defaulted `describe()`/`affordances()` and the `Snapshot: JsonSchema` bound; `kitchen_sink --describe` emits a full manifest (modes, bindings, notation grammar, snapshot schema). Commit `178ee27`; 13 new contract tests.

**Key insight**: the manifest is split — apps author `AppDescription`, the harness generates the notation grammar (from the parser's own `SPECIAL_KEYS` table) and the snapshot schema (schemars). Apps physically cannot produce drifted notation or schema docs.

## 🚦 Current State
**✅ Solid foundation**: 33 tui-harness + 149 diffviz + 18 sam tests green, workspace clippy clean. `--test-input`/`--test-full` output formats are byte-compatible with diffviz's hand-rolled modes (verified by contract test against `to_string_pretty`).

**⚠️ Needs attention**: `parse_agent_args` rejects unknown flags loudly (deliberate — a typo must not hang a headless agent in the interactive TUI). diffviz `main.rs` still has its own arg matching with a fall-through-to-TUI default and a dead `--test-render` stub; Phase 2 must reconcile the two (route recognized flags to `run_agent_cli`, decide fate of fall-through).

**⏸️ Deferred**: `bindings` in review-tui's manifest stay empty until the Phase 3 registry (never hand-write them); affordances in `--test-full` output (Phase 3); REPL (Phase 4).

## 👥 Next Agent Guidance
**Phase 2 (diffviz migration)**: Capture a golden of `review-tui --test-input "j"` BEFORE deleting `run_input_test`/`run_render_test`/`run_combined_test` from `main.rs:158-227`; the byte-compat gate depends on it. `create_test_review_engine()` (main.rs:67) is the app factory to keep. Note the StateSnapshot/SamSnapshot JsonSchema derives already landed here (compiler-forced by the bound — see decision 3), so Phase 2 is only: main.rs delegation + `describe()` override on `ReviewTuiApp` + golden tests.

**Phase 3 (registry)**: `AppDescription.bindings`/`Affordance` types are final — build the registry to emit them directly.

## 🔗 Integration Points
**Expects**: schemars 1.0 workspace dep; `M::Snapshot: Serialize + JsonSchema` on all ELMApp impls.
**Provides**: `run_agent_cli`, `build_manifest`, `describe_output`/`input_test_output`/`full_test_output` (in-process testable), `TuiError::Usage`, manifest types (`AppDescription`, `KeyBindingDoc`, `ModeDoc`, `Affordance`, `NotationDoc`).

## 📋 Reference Links
- [decision-log.yaml](decision-log.yaml) - Technical choices made
