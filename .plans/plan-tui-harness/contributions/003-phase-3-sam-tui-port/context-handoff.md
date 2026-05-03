# Context Handoff — Phase 3: Rewrite sam-tui to use tui-harness

## What Was Done

Migrated sam-tui from `tui 0.19 + termion` to `ratatui 0.28 + crossterm`, implemented `ELMApp`
for both the production `ModalView<V>` and the new headless `HeadlessModalView<V>`, and added
11 integration tests using `tui-harness` harnesses.

### sam-tui/Cargo.toml

Removed `tui = "0.19"` and `termion = "4.0.5"`. Added `ratatui = "0.28"`, `crossterm = "0.28"`,
`tui-harness = { path = "../tui-harness" }`, `serde`, `serde_json`.

### New files

**`src/error.rs`** — `SamTuiError` with a single `Terminal(#[from] std::io::Error)` variant.
crossterm 0.28 surfaces all errors as `std::io::Error`, so one variant covers everything.

**`src/modal_view/headless.rs`** — `HeadlessModalView<V: Value>` and `SamSnapshot`.
`SamSnapshot` is a concrete serializable struct: `{ current_mode, cursor, filter_query, item_count, marked_count }`.
Does not require `V: Serialize`. `ELMApp` impl shares `key_transformer` from `view.rs`.

**`tests/harness_tests.rs`** — 11 integration tests covering navigation, filtering, mode toggle,
multi-select, confirm, and render output. All headless via `InputTestHarness` and `RenderTestHarness`.

### Modified files

**`src/modal_view/theme.rs`** — `tui::style` → `ratatui::style`, otherwise identical.

**`src/modal_view/ui_insert_mode.rs`** — `tui::` → `ratatui::`. Removed `Frame<B> where B: Backend`
(now just `Frame`). Constraints no longer need `.as_ref()`. Block returns `Block<'static>`.

**`src/modal_view/ui_options_mode.rs`** — Same migration. `f.size()` → `f.area()`.

**`src/modal_view/ui.rs`** — Major simplification. Removed `terminal: RefCell<Terminal<...>>`,
`last_update: Cell<Option<SystemTime>>`, raw mode setup, Drop impl, and rate-limiting logic.
`UIModal<V>` now holds only a `UITheme` and exposes `draw(&self, frame: &mut Frame, state: &ViewState<V>)`.
The 75ms refresh throttle is gone — `run_app`'s 60fps poll covers this.

**`src/modal_view/view.rs`** — Full rewrite. `ModalView<V>` no longer holds terminal, `Keys<Stdin>`,
or `init` flag. Adds `should_quit: bool` and `cancelled: bool`. `run()` calls `tui_harness::run_app(&mut self)`.
`key_transformer` maps `crossterm::event::KeyEvent` to `AppEvent`. `j`/`k` added as vim-style
Down/Up navigation (with `KeyModifiers::NONE`). Public API unchanged: `ModalView::new()` + `run()`.

**`src/modal_view/mod.rs`** — Added `pub mod headless` and re-exports for `HeadlessModalView`, `SamSnapshot`.

## Testing Results

- `cargo test --package sam-tui` — 18 passed (8 existing unit tests + 10 new integration tests) ✓
- `cargo clippy --package sam-tui` — zero warnings ✓
- `cargo check --package sam-tui` — compiles ✓
- `cargo test --workspace` — 269 passed, 1 ignored ✓

## Decisions Worth Noting

**j/k navigation**: The roadmap testing criteria required `run_sequence("jj")` to produce `cursor: 2`.
This means `j` = Down, `k` = Up. This is a behavior change from the original termion version where
`j`/`k` were filter characters. The trade-off: users can no longer filter by typing 'j' or 'k'.
In OptionsMode, 'j'/'k' option keys would not toggle (the Down/Up events are ignored in that mode).

**No `on_tick` needed**: `ModalView` has no leader-key timeout or per-frame time-based logic,
so the `ELMApp::on_tick` default no-op is sufficient. No implementation required.

**`ModalView::Snapshot = ()`**: The production app doesn't need snapshots, only the headless
variant does. Using `()` for Snapshot on `ModalView` is valid — the trait allows it.

## Phase Completion

Phase 3 is the final phase of this roadmap. All three objectives are complete:
1. Phase 1: `tui-harness` crate with kitchen-sink example ✓
2. Phase 2: `diffviz-review-tui` ported to tui-harness ✓
3. Phase 3: `sam-tui` migrated to ratatui + crossterm + tui-harness ✓

The `ELMApp` trait now has three implementations across the workspace:
- `ReviewTuiApp` (diffviz) — production
- `HeadlessApp` (diffviz) — test harness
- `ModalView<V>` (sam) — production
- `HeadlessModalView<V>` (sam) — test harness
- `CounterApp` (tui-harness example) — kitchen-sink demo
