# Implementation Roadmap for tui-harness

## Execution Strategy

**Strategy**: Core-then-Integrate  
**Approach**: Build the `tui-harness` crate and validate it with a kitchen-sink example before touching either consumer. Once the trait is proven, port diffviz-review-tui (simple: most code already exists), then rewrite sam-tui (harder: requires ratatui migration + headless variant).

---

## Phase 1: Create tui-harness crate with kitchen-sink example

**Description**: Stand up the `tui-harness` crate in the patina workspace. Define the `ELMApp` trait, port the input parser from diffviz, implement the three generic harnesses, and validate everything with a kitchen-sink example binary.

**Objectives**:
- **Implementation**: Add `tui-harness` to `Cargo.toml` workspace members in `/Users/ryad/workspace/patina/Cargo.toml`
- **Implementation**: Create `tui-harness/Cargo.toml` with deps: `ratatui = "0.28"`, `crossterm = "0.28"`, `serde = { features = ["derive"] }`, `serde_json`, `anyhow`
- **Implementation**: Define `ELMApp` trait in `tui-harness/src/traits.rs` — associated types `Snapshot: Serialize`, methods `dispatch_key(KeyEvent) -> Result<()>`, `snapshot() -> Snapshot`, `draw(&self, frame: &mut Frame)`
- **Implementation**: Port `parse_input_sequence()` from `diffviz-review-tui/src/test_harness/input_parser.rs` verbatim into `tui-harness/src/input_parser.rs` (zero coupling, moves cleanly)
- **Implementation**: Implement `InputTestHarness<M: ELMApp>` in `tui-harness/src/input_test.rs` — `run_sequence(input: &str) -> Result<Vec<M::Snapshot>>` and `run_sequence_final_state(input: &str) -> Result<M::Snapshot>`
- **Implementation**: Implement `RenderTestHarness` in `tui-harness/src/render_test.rs` — `new()`, `with_size(w, h)`, `render<M: ELMApp>(app: &M) -> Result<String>` using `Terminal<TestBackend>` + `buffer_to_string()`
- **Implementation**: Implement `CombinedTestHarness<M: ELMApp>` in `tui-harness/src/combined.rs` — `run_sequence_with_renders(input: &str) -> Result<Vec<CombinedTestResult<M::Snapshot>>>`
- **Implementation**: Create kitchen-sink example in `tui-harness/examples/kitchen_sink.rs`:
  - Toy counter ELM app: `CounterApp` with `CounterSnapshot { count: i32, mode: String }`
  - Keys: `j`/`k` increment/decrement, `r` reset, `i` enter label-edit mode, `<Esc>` exit mode, `q` quit
  - `CounterApp` implements `ELMApp`
  - `--test-input <seq>` flag: run `InputTestHarness`, print JSON snapshots, exit
  - `--test-render` flag: run `RenderTestHarness`, print visual output, exit
  - `--test-full <seq>` flag: run `CombinedTestHarness`, print each step, exit
  - Default (no flags): run the TUI interactively

**Testing Criteria**:
- `cargo build --package tui-harness` compiles with zero warnings
- `cargo test --package tui-harness` passes (unit tests in each module)
- `cargo run --example kitchen_sink -- --test-input "jjjkr"` prints 5 JSON snapshots showing count going 0→1→2→3→2→0
- `cargo run --example kitchen_sink -- --test-render` prints non-empty ASCII art of the counter UI
- `cargo run --example kitchen_sink -- --test-full "jq"` prints both state and visual for each step
- `cargo clippy --package tui-harness` zero warnings

**Dependencies**: None

**Relevant Local Skills**: `dev-contribute`, `diffviz-tui-contribution` (for any diffviz context)

**Files to Modify**:
- `/Users/ryad/workspace/patina/Cargo.toml` — add `"tui-harness"` to workspace members
- `tui-harness/Cargo.toml` — new file
- `tui-harness/src/lib.rs` — new file, pub mod declarations + re-exports
- `tui-harness/src/traits.rs` — new: `ELMApp` trait
- `tui-harness/src/input_parser.rs` — new: ported from diffviz (verbatim + tests)
- `tui-harness/src/input_test.rs` — new: generic `InputTestHarness<M: ELMApp>`
- `tui-harness/src/render_test.rs` — new: generic `RenderTestHarness`
- `tui-harness/src/combined.rs` — new: generic `CombinedTestHarness<M: ELMApp>` + `CombinedTestResult<S>`
- `tui-harness/examples/kitchen_sink.rs` — new: kitchen-sink counter app + harness demo

---

## Phase 2: Port diffviz-review-tui to use tui-harness

**Description**: Replace diffviz-review-tui's bespoke test harness module with thin delegations to tui-harness. Implement `ELMApp` for `HeadlessApp`. The public API of `diffviz_review_tui::test_harness` stays the same so callers don't change.

**Objectives**:
- **Implementation**: Add `tui-harness` to `diffviz-review-tui/Cargo.toml` under `[dev-dependencies]` (or behind `test-harness` feature); also add under `[dependencies]` gated on `test-harness` feature
- **Implementation**: Implement `ELMApp` for `HeadlessApp` in `diffviz-review-tui/src/app.rs` (behind `#[cfg(feature = "test-harness")]`):
  - `type Snapshot = StateSnapshot`
  - `dispatch_key(key) -> Result<()>` — calls `process_key_event(key)`, discards `Command`
  - `snapshot() -> StateSnapshot` — calls `StateSnapshot::from_ui_state(&self.ui_state)`
  - `draw(&self, frame) -> ()` — calls `ui::draw(frame, &self.ui_state, &self.review_engine)`
- **Implementation**: Delete `diffviz-review-tui/src/test_harness/input_parser.rs` — replaced by `tui_harness::parse_input_sequence`
- **Implementation**: Rewrite `diffviz-review-tui/src/test_harness/input_test.rs` as a thin type alias + wrapper around `tui_harness::InputTestHarness<HeadlessApp>`
- **Implementation**: Rewrite `diffviz-review-tui/src/test_harness/render_test.rs` as a thin wrapper around `tui_harness::RenderTestHarness` (same `new()`, `with_size()`, `render()` API but delegates to generic version)
- **Implementation**: Rewrite `diffviz-review-tui/src/test_harness/combined.rs` as a thin wrapper around `tui_harness::CombinedTestHarness<HeadlessApp>`
- **Implementation**: Keep `diffviz-review-tui/src/test_harness/snapshot.rs` unchanged — `StateSnapshot` remains diffviz-specific
- **Implementation**: Update `diffviz-review-tui/src/test_harness/mod.rs` to re-export from the new structure

**Testing Criteria**:
- `cargo test --package diffviz-review-tui --features test-harness` passes with zero regressions
- `cargo clippy --package diffviz-review-tui --features test-harness` zero warnings
- `cargo build --package diffviz-review-tui` (without feature) still compiles — no test harness code leaks into production build
- All existing tests in `input_test.rs`, `render_test.rs`, `combined.rs` still pass unchanged
- The `--test-input`, `--test-render`, `--test-full` binary modes still work (from `main.rs`)

**Dependencies**: Phase 1 must be complete

**Relevant Local Skills**: `dev-contribute`, `diffviz-tui-contribution` (mandatory for any diffviz-review-tui changes)

**Files to Modify**:
- `diffviz-review-tui/Cargo.toml` — add `tui-harness` dependency
- `diffviz-review-tui/src/app.rs` — add `ELMApp` impl for `HeadlessApp` (behind feature gate)
- `diffviz-review-tui/src/test_harness/mod.rs` — update re-exports
- `diffviz-review-tui/src/test_harness/input_parser.rs` — DELETE
- `diffviz-review-tui/src/test_harness/input_test.rs` — rewrite as thin wrapper
- `diffviz-review-tui/src/test_harness/render_test.rs` — rewrite as thin wrapper
- `diffviz-review-tui/src/test_harness/combined.rs` — rewrite as thin wrapper
- `diffviz-review-tui/src/test_harness/snapshot.rs` — no change

---

## Phase 3: Rewrite sam-tui to use tui-harness

**Description**: Migrate sam-tui from `tui 0.19 + termion` to `ratatui 0.28 + crossterm`, add a `HeadlessModalView<V>` implementing `ELMApp`, and introduce harness-based tests. The public API of sam-tui (`ModalView::new()` + `run()`) is preserved.

**Objectives**:
- **Implementation**: Update `sam-tui/Cargo.toml`: replace `tui = "0.19" + termion = "4.0"` with `ratatui = "0.28" + crossterm = "0.28"`; add `tui-harness = { path = "../../patina/tui-harness" }` under `[dev-dependencies]`; add `serde = { features = ["derive"] }` + `serde_json`
- **Implementation**: Migrate `sam-tui/src/modal_view/ui.rs` + `ui_insert_mode.rs` + `ui_options_mode.rs`: update all `tui::` imports to `ratatui::`, update `TermionBackend` → `CrosstermBackend`, update `Terminal::new()` init to use crossterm raw mode setup
- **Implementation**: Rewrite `sam-tui/src/modal_view/view.rs`: replace `termion::input::Keys` + recursive `run()` with crossterm `event::read()` iterative loop; replace `key_transformer(termion::Key)` with `key_transformer(crossterm::event::KeyEvent) -> Option<Event>`; add Drop impl for terminal cleanup
- **Design**: Determine the right shape for `SamSnapshot` — what fields are needed for testing navigation, filtering, and marking? (Keep simple: mode, cursor, filter query, item count, marked count)
- **Implementation**: Create `sam-tui/src/modal_view/headless.rs`: `HeadlessModalView<V: Value>` wrapping `ViewState<V>` + `key_transformer`; `SamSnapshot` struct with `#[derive(Serialize)]`; `ELMApp` impl for `HeadlessModalView<V>`
- **Implementation**: Add `tests/` integration tests in sam-tui using `InputTestHarness<HeadlessModalView<MockValue>>`: test search filtering (`"co<Backspace>"`), mode toggle (`"<Esc>"`), navigation (`"jjk"`), multi-select (`"<C-s>"`), confirm (`"<Enter>"`)

**Testing Criteria**:
- `cargo build --package sam-tui` compiles with zero warnings on both macOS and Linux
- `cargo test --package sam-tui` passes — all existing unit tests pass, new integration tests pass
- `cargo clippy --package sam-tui` zero warnings
- Navigation test: `InputTestHarness::run_sequence("jj")` on a 5-item list → snapshot shows `cursor: 2`
- Filter test: `InputTestHarness::run_sequence("abc")` → snapshot shows `filter_query: "abc"`, `item_count` reduced
- Mode toggle: `InputTestHarness::run_sequence("<Esc>")` → snapshot shows `current_mode: "OptionsMode"`
- Render test: `RenderTestHarness::render(headless_app)` returns non-empty string on 80×24

**Dependencies**: Phase 1 must be complete; Phase 2 must be complete (validates ELMApp pattern before sam port)

**Relevant Local Skills**: `dev-contribute`

**Files to Modify**:
- `sam-tui/Cargo.toml` — replace deps, add tui-harness + serde
- `sam-tui/src/modal_view/view.rs` — full rewrite (crossterm + iterative loop)
- `sam-tui/src/modal_view/ui.rs` — import migration (tui → ratatui)
- `sam-tui/src/modal_view/ui_insert_mode.rs` — import migration
- `sam-tui/src/modal_view/ui_options_mode.rs` — import migration
- `sam-tui/src/modal_view/headless.rs` — new: `HeadlessModalView<V>` + `SamSnapshot` + `ELMApp` impl
- `sam-tui/src/modal_view/mod.rs` — expose `headless` module
- `sam-tui/tests/harness_tests.rs` — new: integration tests using InputTestHarness + RenderTestHarness

---

## Implementation Notes

**Two-gate system**: The "Design: Determine SamSnapshot shape" in Phase 3 is intentionally minimal — wait until Phase 3 starts before committing to exact field set, as the ratatui migration may reveal constraints. If the shape is obvious at Phase 3 start, skip the design gate and implement directly.

**Rollback safety**: Each phase is independently mergeable. Phase 2 is a pure refactor (zero behavior change). Phase 3 is additive to sam (new headless module + tests; existing `ModalView` API unchanged).

**ZERO WARNING RULE**: After every change in diffviz workspace, run `cargo fmt --all && cargo clippy --workspace && cargo check --workspace`. Fix all warnings before moving on.
