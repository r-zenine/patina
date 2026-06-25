# Context Handoff - Phase 1 Toggle Infrastructure

## 🎯 Core Result (What agents get from this work)
**Built**: Toggle infrastructure for `show_reasoning` — all 4 files changed per the roadmap. Group 1 tests (3 toggle state tests) now pass. Groups 2 and 3 still fail at runtime, which is the expected TDD state after Phase 1.

**Commit**: `5eb7448`

## 🚦 Current State (Agent decision points)
**✅ Solid**: The `show_reasoning` field follows the exact same pattern as `show_instructions` — field on `UiState`, field on `StateSnapshot`, `ToggleReasoning` in `UiEvent`, key `<Space>tr` in the `t` submenu of `handle_leader_keys()`, and a match arm in `handle_ui_event_impl()` that calls `toggle_reasoning()` then `deactivate_leader()`.

**✅ Tests**: `cargo test --package diffviz-review-tui --features test-harness` → 5 pass, 3 fail. The 3 failures are all Group 2 (title badge) and Group 3 (annotation injection) tests, which require Phases 2 and 3.

**✅ Zero warnings**: `cargo clippy --workspace` is clean.

## 👥 Next Agent Guidance (Specific handoff)
**Phase 2 implementer**: Add `ReasoningAnnotation` struct and `with_reasoning_annotations()` builder to `RenderableDiffWidget` in `diffviz-review-tui/src/ui/components/renderable_diff_widget.rs`. Inject annotation lines before matching `RenderableLine` entries in both render loops. Group 3 tests will pass once Phase 3 also wires the data flow from `diff_view.rs`.

Key file: `diffviz-review-tui/src/ui/components/renderable_diff_widget.rs`
- `RenderableDiffWidget` struct at ~L29-39: add `reasoning_annotations: &'a [ReasoningAnnotation]` field
- `with_instruction_indicators()` at ~L99-103: pattern for new builder method
- `Widget::render()` at ~L106-162: injection site (check `line.line_number` against annotation map before `append_line()`)
- `hidden_indicator()` at ~L164-175: precedent for synthetic line injection

**Phase 3 implementer**: Wire data flow in `diff_view.rs::render_diff_content()`. Call `review_engine.get_decisions_for_diff()`, build `Vec<ReasoningAnnotation>`, pass to widget, and add `◆ D{n}` badge to title when `!show_reasoning && !decisions.is_empty()`.

---
## 🔗 Integration Points (Technical context)
**`StateSnapshot.show_reasoning`**: accessible in Group 1 tests via `harness.run_sequence_final_state()`. Test assertions check `state.show_reasoning == true/false`.

**Key binding path**: `<Space>` → `ActivateLeader` → `<t>` → `EnterLeaderSubmenu('t')` → `<r>` → `ToggleReasoning` → `toggle_reasoning()`.

## 📋 Reference Links
- [decision-log.yaml](decision-log.yaml) - Technical choices made
