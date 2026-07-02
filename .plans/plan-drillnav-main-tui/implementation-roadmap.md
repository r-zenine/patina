# Implementation Roadmap for DrillNav Main TUI

## Execution Strategy

**Strategy**: TDD
**Approach**: Phase 0 encodes the DrillNav contract as failing harness tests (with minimal compiling stubs). Phases 1–3 make test groups green bottom-up: state machine → views → notes/paging/annotations. Phase 4 deletes the old UI. The test binary and harness stay functional after every phase; fmt/clippy/check clean after every phase (ZERO WARNING rule). The `diffviz-tui-contribution` skill is mandatory for every phase.

---

## Phase 0: Contract & Test Design

**Description**: Define DrillNav's behavior as failing integration tests, plus the minimal type scaffolding (DrillNavState stub in UiState, StateSnapshot v2) so the suite compiles. No behavior changes; all existing tests still pass.

**Objectives**:
- **Implementation**: Add `DrillNavState`/`FileView` types to `src/state.rs` as a new `UiState.drill_nav` field with stub methods (ported signatures from the prototype, bodies `todo!()`-free but inert)
- **Implementation**: StateSnapshot v2 in `src/state_snapshot.rs` per decision D6 (new fields serialized from the stub; old fields kept temporarily so existing tests compile)
- **Implementation**: Write failing tests: `tests/drillnav_navigation_tests.rs` (browse j/k bounds, Enter drill-in, Esc back restores cursor, h/l wrap + per-file retention, Tab context, i note-expansion), `tests/drillnav_approval_tests.rs` (a on decision cascades + progress counts, a on chunk, error → status_message), `tests/drillnav_note_tests.rs` (n opens input modal, submit appends — single note asserted via engine state, cancel restores navigation)
- **Design**: Test-data audit — confirm the mock fixtures + hardcoded decisions in `src/main.rs` exercise multi-file decisions and multi-chunk files (needed for h/l and dot-pagination tests); extend the hardcoded decisions if not

**Testing Criteria**:
- `cargo test --package diffviz-review-tui --features test-harness` compiles; new drillnav tests fail for behavioral (not compile) reasons; all pre-existing tests still green
- Failing tests cover every keybinding in the target key table (j/k/h/l, Enter, Esc, Tab, i, n, a, Ctrl-d/u, g/G, Space leader, ?, q)

**Dependencies**: None

**Relevant Local Skills**: diffviz-tui-contribution

**Files to Modify**:
- `src/state.rs` — DrillNavState/FileView stub types + field
- `src/state_snapshot.rs` — schema v2 (additive for now)
- `tests/drillnav_navigation_tests.rs`, `tests/drillnav_approval_tests.rs`, `tests/drillnav_note_tests.rs` — new
- `src/main.rs` — richer hardcoded decisions if the audit requires

---

## Phase 1: DrillNav State Machine & Event Flow

**Description**: Implement the update layer — DrillNav navigation methods on UiState, event remapping, business-event context resolution, status-message error surfacing. State-assertion tests (`--test-input`) go green; visual tests stay red. The old draw path still renders (views untouched), so the binary stays usable.

**Objectives**:
- **Implementation**: Port the prototype's state machine into `UiState` methods (navigate_up/down/left/right, drill_in, back, toggle_context, toggle_note_expansion, page_up/down, to_top/bottom) with the precomputed decision→files→chunks index (D6) built in `ReviewTuiApp::initialize_ui_state`
- **Implementation**: Rewrite `handle_navigation_keys` key table (`n` binding, remove `v`, keep leader/help/quit) and `handle_ui_event_impl` to drive DrillNav instead of FocusPanel matches
- **Implementation**: `ui_event_to_business_event` resolves approve/note targets from DrillNav position (Browse → decision, Drill → chunk); `n` enters the existing InputMode with the right target
- **Implementation**: `UiState.status_message` one-shot error surfacing (D7): business-event errors caught in `process_key_event_impl`, cleared on next key
- **Implementation**: StateSnapshot v2 populated from real state; drop old fields from the snapshot **only** where no surviving test reads them yet

**Testing Criteria**:
- All `--test-input` assertions in the three drillnav test files pass
- `--test-input "<Enter>hjka"` style sequences produce correct drill position + approval state JSON
- Failed approval (unknown id forced in test) yields `status_message` non-null, next key clears it
- input_mode_tests and leader_key_tests adapted minimally and green

**Dependencies**: Phase 0

**Relevant Local Skills**: diffviz-tui-contribution

**Files to Modify**:
- `src/state.rs` — real state machine + methods + index
- `src/app.rs` — handle_ui_event_impl rewrite, error catch, init index
- `src/events/input.rs` — key table rewrite
- `src/events/business.rs` — DrillNav-aware context resolution
- `src/state_snapshot.rs` — populate v2
- `tests/input_mode_tests.rs`, `tests/leader_key_tests.rs` — adapt targets

---

## Phase 2: Views

**Description**: Port render_browse/render_drill from the prototype into pure view functions; DrillNav becomes the drawn UI. Visual (`--test-full`) tests go green. Old panel components are no longer referenced by `ui::draw` but not yet deleted.

**Objectives**:
- **Implementation**: New `src/ui/components/drillnav_browse.rs` and `drillnav_drill.rs` — pure `fn render(f, area, ui_state: &UiState, engine: &ReviewEngine)` ports of the prototype views (cards, pinned header, dot pagination, note rows, scroll_into_view, chunk progress)
- **Implementation**: Rewire `ui::draw`/`ui/layout.rs`: full-width DrillNav + status bar; keep input-modal, help, which-key overlays on top
- **Implementation**: Status bar rewrite — contextual hints per mode (prototype format: conditional h/l, note hint, approve counts), red `status_message` preemption
- **Implementation**: Viewport paging from Phase 1 state honored in drill rendering (page offset combined with scroll_into_view)

**Testing Criteria**:
- `--test-full "<Enter>"` visual output shows pinned header, chunk cards, dot pagination for multi-file decisions
- Note truncation `…` and `i`-expansion visible in visual output; approved badge `✓` appears after `a`
- keybinding_tests, core_navigation_tests, decision_approval_tests rewritten against DrillNav and green
- Manual smoke test: `cargo run --bin review-tui` navigates correctly at 80×24 and 200×60

**Dependencies**: Phase 1

**Relevant Local Skills**: diffviz-tui-contribution

**Files to Modify**:
- `src/ui/components/drillnav_browse.rs`, `drillnav_drill.rs` — new
- `src/ui/mod.rs`, `src/ui/layout.rs` — draw path
- `src/ui/components/status_bar.rs` — rewrite
- `tests/keybinding_tests.rs`, `tests/core_navigation_tests.rs`, `tests/decision_approval_tests.rs` — rewrite

---

## Phase 3: Reasoning Annotations & Note Polish

**Description**: The two remaining feature carriers: inline reasoning annotations inside chunk cards, and the note-entry UX end-to-end.

**Objectives**:
- **Design**: Visual treatment of reasoning annotations inside chunk cards (tier, color, trigger-line placement, ◆ badge location in DrillNav) — refine via design-contribute before implementing
- **Implementation**: Port annotation computation from `diff_view.rs:129-150` into the drill view per the design; `Space-t-r` toggle preserved
- **Implementation**: Note-entry modal polish: modal title shows existing note authors ("append to alice's note"), submit → engine append → note row re-renders with merged authors
- **Implementation**: Help overlay + which-key tables updated to the final key map

**Testing Criteria**:
- reasoning_annotation_tests.rs adapted: `<Space>tr` toggles annotation lines in drill visual output at correct positions
- drillnav_note_tests fully green including append-to-existing (authors merge, single note asserted)
- Help overlay lists exactly the live keybindings (no dead keys)

**Dependencies**: Phase 2

**Relevant Local Skills**: diffviz-tui-contribution, design-contribute (for the Design objective)

**Files to Modify**:
- `src/ui/components/drillnav_drill.rs` — annotation rendering
- `src/ui/components/input_modal.rs` — title context
- `src/ui/components/help_overlay.rs`, `which_key.rs` — key tables
- `tests/reasoning_annotation_tests.rs` — adapt

---

## Phase 4: Deletion & Cleanup

**Description**: Remove the old UI wholesale. The codebase ends with one navigation model, no dead events, no dead state, zero warnings.

**Objectives**:
- **Implementation**: Delete `src/decision_navigation.rs`, `src/ui/components/{decision_tree,diff_view,renderable_diff_widget,decision_details_panel}.rs`; purge dead UiState fields (focused_panel, cursor_index, selection_*, highlight_semantics, show_instructions, scroll_offset, decision_tree), dead UiEvents (ToggleFocus, ToggleRangeSelection, ToggleSemanticHighlight, ScrollInactivePanel*, NavigateToFile) and their key mappings, dead StateSnapshot fields
- **Implementation**: Delete `tests/panel_management_tests.rs`, `tests/decision_tree_expansion_tests.rs`; sweep remaining tests for old-model references
- **Implementation**: Delete `examples/review_navigator.rs` (superseded) and its resources, or demote to a tui-design showcase — decide at execution time based on whether tui-design wants a living example
- **Implementation**: Update docs that encode the old model: `.claude/skills/diffviz-tui-contribution/SKILL.md` (verification commands reference `decision_tree` fields), `onboarding.md`, `TEST_HARNESS.md`
- **Implementation**: Full workspace sweep: `cargo fmt --all`, `cargo clippy --workspace --examples --features diffviz-review-tui/test-harness`, `cargo test --workspace`, manual run via diffviz-cli against a real contribution folder

**Testing Criteria**:
- Zero clippy/compiler warnings across the workspace
- `grep -rn "FocusPanel\|decision_tree\|cursor_index\|selection_range" src/` returns nothing
- `diffviz review <contribution-folder>` end-to-end: navigate, approve, append note, quit — state persisted to the folder and reloadable
- Full test suite green

**Dependencies**: Phase 3

**Relevant Local Skills**: diffviz-tui-contribution, verify

**Files to Modify**:
- Deletions as listed above
- `src/state.rs`, `src/events/input.rs`, `src/state_snapshot.rs`, `src/lib.rs` — purge
- `.claude/skills/diffviz-tui-contribution/SKILL.md`, `diffviz-review-tui/onboarding.md`, `diffviz-review-tui/TEST_HARNESS.md` — doc updates
