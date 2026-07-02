# Code Context for DrillNav Main TUI

## The Prototype (source of truth for UX behavior)

- **DrillNavState / FileView** (`diffviz-review-tui/examples/review_navigator.rs:197-240`) — Browse/Drill state machine with per-file view retention (cursor, expanded context, expanded notes)
- **handle_key_event** (`examples/review_navigator.rs:179-193`) — pure key→UiEvent mapping incl. `i` = ToggleInstructions (note expansion)
- **App::toggle_approve** (`examples/review_navigator.rs:~280`) — decision vs chunk approval by mode, error → status message
- **render_browse** (`examples/review_navigator.rs:~646`) — decision cards: label + chunk progress, rationale, file preview, browse scrolling
- **render_drill** (`examples/review_navigator.rs:~741`) — pinned header via `render_drill_header`, dot pagination, chunk cards (note row + code lines), `scroll_into_view`
- **note_for / note_rows / visible_line_count** (`examples/review_navigator.rs:159-170, ~937-990`) — single-note access, wrapped note rows, exact height computation for scrolling

## Main TUI — Elm architecture (keep the skeleton, replace the organs)

- **ReviewTuiApp + ELMApp impl** (`diffviz-review-tui/src/app.rs:23-103`) — `dispatch_key` → `process_key_event` → `execute_command`; `draw`; `snapshot()`; `on_tick` (leader timeout). **Public API (`new`, `run`, `into_review_engine`) must not change** — diffviz-cli depends on it.
- **process_key_event_impl** (`src/app.rs:109-134`) — UiEvent → handle_ui_event → BusinessEvent → Command pipeline
- **handle_ui_event_impl** (`src/app.rs:136-368`) — the update function; currently matches on `FocusPanel`, will match on DrillNav mode
- **handle_business_event_impl** (`src/app.rs:370-433`) — ToggleApprove, ToggleApproveDecision, AddInstruction, ExportInstructions already exist
- **UiState** (`src/state.rs:42-117`) — fields to REMOVE: `focused_panel`, `cursor_index`, `selection_anchor`, `selection_range`, `highlight_semantics`, `decision_tree`, `show_instructions`, `scroll_offset` (replaced by drill paging). Fields to KEEP: `input_mode`, `input_buffer`, `input_cursor`, leader fields, `show_help`, `show_reasoning`, `should_quit`.
- **InputMode** (`src/state.rs:28-32`) — Instruction/DecisionInstruction input modes: reuse for note entry
- **UiEvent** (`src/events/input.rs:11-84`) — variants to keep/remap; to DELETE: `ToggleFocus`, `ToggleRangeSelection`, `ToggleSemanticHighlight`, `ScrollInactivePanelUp/Down`, `NavigateToFile`
- **handle_navigation_keys / handle_leader_keys** (`src/events/input.rs:108-309`) — key tables to rewrite; `n` is unbound today
- **BusinessEvent + ui_event_to_business_event** (`src/events/business.rs:12-86`) — context resolution reads `ui_state`; will read DrillNav position
- **Command / execute_command** (`src/command.rs:15-52`) — WriteFile/ShowMessage/Batch/None
- **DecisionNavigationTree** (`src/decision_navigation.rs`, 332 lines) — DELETED in final phase; DrillNav replaces it
- **StateSnapshot** (`src/state_snapshot.rs:11-107`) — schema v2 needed (nav mode, browse cursor, drill position, per-file view state)

## UI components

- **ui::draw** (`src/ui/mod.rs`) + **layout** (`src/ui/layout.rs`) — two-panel split to be replaced by full-width DrillNav
- **diff_view.rs:129-150** — reasoning-annotation computation (`ui_state.show_reasoning`, `impact.reasoning`) — logic PORTED into drill chunk cards, then file deleted
- **renderable_diff_widget.rs** — line rendering + selection; deleted (drill renders lines directly, prototype style)
- **decision_tree.rs, decision_details_panel.rs** — deleted
- **status_bar.rs** — rewritten for DrillNav hints (prototype status format)
- **input_modal.rs** — kept as-is for note entry
- **help_overlay.rs, which_key.rs** — content updated for new key table

## Design system (tui-design) — ready, no changes expected

- **CardTier / HierarchicalCard / render_drill_header / separator_line** (`tui-design/src/card.rs`)
- **scroll_into_view** (`tui-design/src/scroll.rs`) — minimal scrolling, tested
- **Theme / SurfaceRamp** (`tui-design/src/tokens.rs`) — surface2 reserved for selection
- **stylesheet** (`tui-design/src/stylesheet.rs`) — `terminal_floor`, `widget_floor`, `status_bar`, `error`

## Engine APIs (diffviz-review) — ready, no changes expected

- **approve / reject** (`src/engines/review_engine/mod.rs:40,56`)
- **approve_decision / reject_decision** (`decision.rs:61,72`) — cascades to chunks
- **decision_approval_progress** (`decision.rs:82`) — (approved, total) per decision
- **get_decision_reviewable_diffs** (`decision.rs:126`) — chunk↔decision pairs
- **add_instruction** (`mod.rs:91` → `entities/instruction.rs:54`) — single-note model: appends to existing note
- **get_renderable_diff_object**, **get_instructions**, **get_all_decisions**

## Entry points

- **Production**: `diffviz-cli/src/main.rs:125-148` (`run_contribution_review`) — real GitRepository + decision-log.yaml + state persistence
- **Test binary**: `diffviz-review-tui/src/main.rs:67-156` — mock fixtures + hardcoded decisions; harness modes `--test-input`, `--test-full`

## Testing Patterns

- **Harness**: `src/test_harness/{input_test,combined,render_test}.rs` — InputTestHarness (state JSON), CombinedTestHarness (state + visual)
- **Test files** (`tests/`): keybinding (375), core_navigation (423), decision_approval (1014), decision_tree_expansion (612), input_mode (712), leader_key (561), panel_management (454), reasoning_annotation (258)
- **Survives adapted**: input_mode, leader_key, reasoning_annotation. **Rewritten**: keybinding, core_navigation, decision_approval. **Deleted**: panel_management, decision_tree_expansion.
- Input notation: `jjk<Space>`, `<Enter>`, `<Esc>`, `<C-d>`, `<Wait:100>`
