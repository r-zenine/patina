# Context Handoff — Phase 2: Port diffviz-review-tui to tui-harness

## What Was Done

Ported `diffviz-review-tui` to consume `tui-harness` as planned. All bespoke harness
infrastructure now delegates to generic implementations; `ReviewTuiApp` no longer owns
a terminal or manages its own event loop.

### tui-harness changes (prerequisite for Phase 2 correctness)

- `src/traits.rs` — added `fn on_tick(&mut self) {}` default method to `ELMApp`.
  Required because `ReviewTuiApp` had per-frame leader-timeout logic that must run even
  when no key event arrives. Without this hook the leader key would never expire while
  the app is idle.
- `src/runtime.rs` — calls `app.on_tick()` at the top of each frame loop iteration.

### diffviz-review-tui changes

**`Cargo.toml`** — added `tui-harness = { path = "../tui-harness" }` under `[dependencies]`
(not dev-only; `run_app` is called from production `ReviewTuiApp::run()`).

**`src/app.rs`** — major restructuring:
- Removed `terminal: Terminal<CrosstermBackend<Stdout>>` field from `ReviewTuiApp`.
- Changed `review_engine: Option<ReviewEngine>` → plain `ReviewEngine` (the `Option` was
  only needed to support `into_review_engine()` alongside a `Drop` impl; both are gone).
- Removed terminal setup from `new()`, removed `render()`, `drain_events_for_frame()`, and
  the `Drop` impl — all terminal lifecycle now lives in `tui_harness::run_app`.
- `run()` body replaced with `tui_harness::run_app(self)?`.
- `into_review_engine(self)` is now trivial (just `self.review_engine`).
- Added `pub struct DispatchError(String)` with `impl std::error::Error` — required because
  `anyhow::Error` deliberately does NOT implement `std::error::Error` (see decision log).
  `dispatch_key` converts internal anyhow errors to `DispatchError` via `format!("{e:#}")`.
- `impl ELMApp for ReviewTuiApp`: `type Snapshot = ()`, `type Error = DispatchError`,
  `on_tick` handles leader timeout.
- `impl ELMApp for HeadlessApp` (feature-gated): `type Snapshot = StateSnapshot`,
  `type Error = DispatchError`.

**`src/test_harness/input_parser.rs`** — DELETED. Replaced by `tui_harness::parse_input_sequence`.

**`src/test_harness/input_test.rs`** — thin wrapper: `InputTestHarness` owns
`tui_harness::InputTestHarness<HeadlessApp>`; constructor takes `ReviewEngine` and
creates `HeadlessApp` internally. Public API unchanged.

**`src/test_harness/render_test.rs`** — thin wrapper: `RenderTestHarness` owns
`tui_harness::RenderTestHarness`; public `render(&mut UiState, &ReviewEngine)` API
preserved via a local `RenderAdapter<'a>` that implements `ELMApp` using those references.
This avoids requiring callers to construct a `HeadlessApp`.

**`src/test_harness/combined.rs`** — thin wrapper: `CombinedTestHarness` owns
`tui_harness::CombinedTestHarness<HeadlessApp>`; `CombinedTestResult` is re-exported
as `tui_harness::CombinedTestResult` (the generic struct, instantiated with `StateSnapshot`
by return type inference).

**`src/test_harness/mod.rs`** — removed `input_parser` module; re-exports
`tui_harness::parse_input_sequence` directly.

## Testing Results

- `cargo test --package diffviz-review-tui --features test-harness` — 163 passed, 21 ignored ✓
- `cargo clippy --package diffviz-review-tui --features test-harness` — zero warnings ✓
- `cargo clippy --package diffviz-review-tui` (no feature) — zero warnings ✓
- `cargo check --package diffviz-review-tui` (no feature) — compiles ✓ (no harness leakage)
- All three binary modes (`--test-input`, `--test-render`, `--test-full`) verified working ✓
- `cargo test --workspace` — 202 passed, 1 ignored ✓

## What Phase 3 Needs to Know

**`ELMApp` trait now has `on_tick`**: Phase 3 (sam-tui) should implement it if it has
any per-frame time-based logic. Default is a no-op, so implementing `ELMApp` without
`on_tick` is fine.

**`DispatchError` pattern**: sam-tui will also hit the anyhow/std::error::Error mismatch
(if it uses anyhow). The same `DispatchError(String)` pattern works. Alternatively, since
sam-tui is being migrated fresh, it can use `thiserror`-based errors from the start
(`SamTuiError`) and avoid anyhow entirely in the ELMApp impl.

**`RenderAdapter` pattern**: If any sam-tui test needs to render a bare `(ViewState, ...)`
pair without constructing a full `HeadlessModalView`, the same adapter trick applies.

**`tui_harness::CombinedTestResult<S>` is generic**: When the thin wrapper returns
`Vec<CombinedTestResult<StateSnapshot>>`, callers access `.state` and `.visual` identically
to before. No caller changes were needed.

**The `input_parser` was moved**: `tui_harness::parse_input_sequence` is now the canonical
source. The diffviz `mod.rs` re-exports it as `parse_input_sequence` for any existing
callers. Phase 3 uses `tui_harness::parse_input_sequence` directly.
