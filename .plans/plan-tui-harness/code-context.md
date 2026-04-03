# Code Context for tui-harness

## Source: diffviz-review-tui test harness (extraction source)

- **`parse_input_sequence()`** (`diffviz-review-tui/src/test_harness/input_parser.rs:27`) — Parses vim-style compact notation (`"jjk<Enter><C-c>"`) into `Vec<KeyEvent>`. Fully portable, no diffviz coupling. Moves verbatim into tui-harness.
- **`InputTestHarness`** (`diffviz-review-tui/src/test_harness/input_test.rs:13`) — Runs sequences on `HeadlessApp`, returns `StateSnapshot` per step. Replace with generic `InputTestHarness<M: ELMApp>`.
- **`RenderTestHarness`** (`diffviz-review-tui/src/test_harness/render_test.rs:11`) — Uses `ratatui::backend::TestBackend` for headless rendering. Already app-agnostic except for `ui::draw()` call — generalize via `ELMApp::draw()`.
- **`CombinedTestHarness`** (`diffviz-review-tui/src/test_harness/combined.rs:27`) — Chains input + render, emits `CombinedTestResult { state, visual }` per step. Generalize via `ELMApp`.
- **`StateSnapshot`** (`diffviz-review-tui/src/test_harness/snapshot.rs:11`) — Diffviz-specific serializable state. Stays in diffviz; tui-harness just requires `Snapshot: serde::Serialize`.
- **`HeadlessApp`** (`diffviz-review-tui/src/app.rs:182`) — Terminal-free app for testing. Implements `process_key_event(KeyEvent) -> Result<Command>`. Will implement `ELMApp` in Phase 2.
- **`process_key_event_impl()`** (`diffviz-review-tui/src/app.rs:213`) — Shared impl between `ReviewTuiApp` and `HeadlessApp` — key event → UI event → business event → Command.

## Source: sam-tui (integration target, Phase 3)

- **`Value` trait** (`sam-tui/src/modal_view/state/mod.rs:14`) — Generic item type: `text() + preview()`, bounds `Eq + Hash + Clone + Debug`.
- **`Event` enum** (`sam-tui/src/modal_view/state/mod.rs:20`) — 8 variants: `AppClosed, ToggleViewMode, InputChar(char), Backspace, Entr, Up, Down, Mark, MarkAll`.
- **`ViewState<V: Value>`** (`sam-tui/src/modal_view/state/view_state.rs`) — ELM Model: `current_mod`, `list: ListState<V>`, `options: OptionsState`. `update(&Event) -> ExecutionState`.
- **`ModalView<V>.run()`** (`sam-tui/src/modal_view/view.rs:43`) — Recursive event loop. `key_transformer(termion::Key) -> Option<Event>` maps raw keys to domain events.
- **`UIModal<V>.draw()`** (`sam-tui/src/modal_view/ui.rs`) — Rendering with `tui 0.19` (TermionBackend). Must migrate to `ratatui 0.28` in Phase 3.

## Key Version Constraints

- `diffviz-review-tui/Cargo.toml`: `ratatui = "0.28"`, `crossterm = "0.28"` — reference stack
- `sam-tui/Cargo.toml`: `tui = "0.19.0"` + `termion = "4.0.5"` — must migrate in Phase 3
- `ratatui` is a maintained fork of `tui`; API is largely the same (widget/layout/style APIs compatible)
- `crossterm` replaces `termion` (Unix-only → cross-platform)

## Key Files to Modify per Phase

**Phase 1 (create tui-harness):**
- `/Users/ryad/workspace/patina/Cargo.toml` — add `tui-harness` to workspace members
- `tui-harness/` — new crate (all files new)

**Phase 2 (port diffviz-review-tui):**
- `diffviz-review-tui/Cargo.toml` — add `tui-harness` dep, retain `test-harness` feature gate
- `diffviz-review-tui/src/app.rs` — add `ELMApp` impl for `HeadlessApp`
- `diffviz-review-tui/src/test_harness/` — replace bodies with thin delegations to `tui-harness`; delete `input_parser.rs`

**Phase 3 (rewrite sam-tui):**
- `sam-tui/Cargo.toml` — replace `tui + termion` with `ratatui + crossterm`; add `tui-harness` path dep
- `sam-tui/src/modal_view/view.rs` — rewrite: termion → crossterm, recursive loop → iterative
- `sam-tui/src/modal_view/ui.rs` + `ui_insert_mode.rs` + `ui_options_mode.rs` — update imports to ratatui
- `sam-tui/src/modal_view/state/` — add Serialize derives or snapshot struct for `ELMApp::Snapshot`
- `sam-tui/src/modal_view/headless.rs` — new: `HeadlessModalView<V>` implementing `ELMApp`
