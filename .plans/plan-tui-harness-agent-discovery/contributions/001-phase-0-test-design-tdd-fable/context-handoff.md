# Context Handoff - Phase 0 Test Design (Dispatch Characterization Gate)

## 🎯 Core Result
**Built**: 15 characterization tests (`diffviz-review-tui/tests/dispatch_characterization_tests.rs`) freezing the complete `(mode, leader state, key) → UiEvent` dispatch contract — all three handlers, every alias pair, modifier sensitivity, and negative cases. All pass against the current match-arm dispatch; commit `c9af274`.

**Key insight**: two quirks were discovered and deliberately pinned, not fixed — leader dispatch ignores modifiers (`Ctrl-a` == `a`), and any unknown key in leader mode silently deactivates it. The Phase 3 registry design must reproduce both, or explicitly revise decision #1 first.

## 🚦 Current State
**✅ Solid foundation**: Tests call `handle_key_event` directly (pure function, no `test-harness` feature, no engine fixtures) — they run in any test invocation and survive the registry refactor by construction. Baseline: 149 tests green, clippy clean.

**⚠️ Needs attention**: The characterization is enumerated, not exhaustive over the key space. If Phase 3's registry design adds *new* bindings, add new tests — never edit existing assertions.

**⏸️ Deferred**: Fixing the leader modifier-blindness quirk (post-Phase-3 decision if wanted); characterizing `process_key_event`-level behavior (already covered by the 134 flow tests).

## 👥 Next Agent Guidance
**Phase 1 (tui-harness agent CLI)**: Independent of this file — start from the roadmap's Phase 1 test list (`agent_cli_tests.rs` against a fixture app). Note the uncommitted working-tree changes in `tui-harness/` (InputStep/Wait semantics, panic guard) are prerequisites for Phase 1 and are NOT yet committed — commit them with Phase 1's work or before it.

**Phase 3 (registry implementation)**: This suite is your primary gate. Run `cargo test -p diffviz-review-tui --test dispatch_characterization_tests` after every dispatch change. It must pass byte-for-byte unchanged.

## 🔗 Integration Points
**Expects**: `handle_key_event`, `UiEvent`, `InputMode` stay `pub` (they are re-exported via `lib.rs` today).
**Provides**: The behavioral contract Phase 3's registry lookup must satisfy; a template for adding characterization when new bindings land.

## 📋 Reference Links
- [decision-log.yaml](decision-log.yaml) - Technical choices made
