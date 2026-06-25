# Context Handoff - Phase 0 Test Design

## 🎯 Core Result (What agents get from this work)
**Built**: 8 failing tests in `diffviz-review-tui/tests/reasoning_annotation_tests.rs` covering the full inline reasoning annotation feature contract.
**Key insight**: Group 1 tests cause compile failure — they access `snapshot.show_reasoning` which doesn't exist yet. This means `cargo test --package diffviz-review-tui --features test-harness --no-run` reports 3 compile errors until Phase 1 is done. Groups 2 and 3 tests compile only after Phase 1 (once the field exists) but fail at runtime until Phases 2-3.

## 🚦 Current State (Agent decision points)
**✅ Solid foundation**: Test structure mirrors the existing toggle tests (e.g., `show_instructions` in `leader_key_tests.rs`). The engine setup pattern and `CombinedTestHarness` usage follow established conventions exactly.
**⚠️ Needs attention**: Phase 1 must add `show_reasoning: bool` to both `UiState` (state.rs) and `StateSnapshot` (state_snapshot.rs). Until then the whole test file won't compile. Check after Phase 1 that all 8 tests compile and groups 2-3 fail at runtime (not just panic).
**⏸️ Deferred**: The `title_badge_hidden_when_reasoning_on` test (Group 2) currently passes trivially before Phase 3 because `◆` doesn't exist anywhere in the rendered output. It becomes a meaningful regression guard once Phase 3 adds the badge.

## 👥 Next Agent Guidance (Specific handoff)
**Phase 1 implementer**: Add `show_reasoning: bool` (default `false`) to `UiState` (after `show_instructions` per the roadmap). Add matching field to `StateSnapshot` and its `from_ui_state()` mapping. Add `toggle_reasoning()` method. Add `ToggleReasoning` to `UiEvent`, wire `(Some('t'), KeyCode::Char('r'))` in `handle_leader_keys()`, and handle in `handle_ui_event_impl()`. After this, `cargo test --package diffviz-review-tui --features test-harness` should have Group 1 pass and Groups 2-3 still fail.
**Phase 2 implementer**: Implement `ReasoningAnnotation` struct and `with_reasoning_annotations()` builder on `RenderableDiffWidget` per the roadmap. Group 3 tests will pass once Phase 3 also wires the data flow.
**Phase 3 implementer**: Wire everything in `diff_view.rs::render_diff_content()` — compute decisions, build annotations, pass to widget, add badge to title. All 8 tests should pass after this phase.

---
## 🔗 Integration Points (Technical context)
**Expects**: `MockDiffProvider::from_review_fixtures()` loads `src/models/calculator.rs` (72 lines). `<Tab>j` navigation sequence expands Decision 1 and moves to its first chunk at depth 1. `DECISION_REASONING` const is a unique string that will not appear in fixture source code.
**Provides**: The test file acts as the implementation spec. Each test name maps directly to a specific behavior in the roadmap. The `DECISION_REASONING` constant is the exact string that Group 3 visual tests check for.

## 📋 Reference Links
- [decision-log.yaml](decision-log.yaml) - Technical choices made
