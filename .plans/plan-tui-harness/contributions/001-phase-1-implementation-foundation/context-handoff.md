# Context Handoff — Phase 1: tui-harness crate

## What Was Done

Created the `tui-harness` workspace crate from scratch with all planned components:

- `src/error.rs` — `TuiError` (two variants: `Terminal` + `App`) and `Result<T>` alias
- `src/traits.rs` — `ELMApp` trait with `Snapshot`, `Error`, `dispatch_key`, `draw`, `should_quit`, `snapshot`
- `src/runtime.rs` — `run_app<M: ELMApp>()` production 60fps event loop with proper crossterm setup/teardown
- `src/input_parser.rs` — ported from diffviz-review-tui verbatim (only error type changed: anyhow → TuiError::App)
- `src/input_test.rs` — generic `InputTestHarness<M: ELMApp>` with `run_sequence` + `run_sequence_final_state`
- `src/render_test.rs` — generic `RenderTestHarness` with `render<M: ELMApp>()` using `TestBackend`
- `src/combined.rs` — generic `CombinedTestHarness<M: ELMApp>` + `CombinedTestResult<S>`
- `examples/kitchen_sink.rs` — `CounterApp` + `CounterSnapshot` demonstrating all three harness modes

All testing criteria from the roadmap pass:
- `cargo build --package tui-harness` — zero warnings ✓
- `cargo test --package tui-harness` — 14 tests pass ✓
- `--test-input "jjjkr"` — 6 snapshots, count 0→1→2→3→2→0 ✓
- `--test-render` — non-empty ASCII box art ✓
- `--test-full "jq"` — state + visual per step ✓
- `cargo clippy --package tui-harness` — zero warnings ✓

## Key Deviation from Plan

**`crossterm::ErrorKind` does not exist in crossterm 0.28.** The plan's `TuiError::Crossterm(#[from] crossterm::ErrorKind)` was dropped. Crossterm operations produce `std::io::Error`, which `TuiError::Terminal(#[from] std::io::Error)` already handles. See decision-log.yaml #1.

## What Phase 2 Needs to Know

**The ELMApp trait is settled.** Phase 2 (`diffviz-review-tui` port) can implement it without surprises:
- `type Error = anyhow::Error` works fine — `anyhow::Error` implements `std::error::Error`
- `dispatch_key` receives a `crossterm::event::KeyEvent` (same as diffviz-review-tui already uses)
- `snapshot()` must return `Self::Snapshot: serde::Serialize` — `StateSnapshot` in diffviz already derives `Serialize`

**The existing `input_parser.rs` in diffviz-review-tui is now a duplicate** of `tui_harness::parse_input_sequence`. Phase 2 will delete `diffviz-review-tui/src/test_harness/input_parser.rs` and replace usages with the tui-harness version.

**Feature gating**: `tui-harness` should be a regular `[dependency]` in `diffviz-review-tui/Cargo.toml` (not dev-only), because `run_app` is called from production code (`ReviewTuiApp::run`). The `test-harness` feature gate in diffviz-review-tui guards the headless harness wrappers, not the runtime.

**render_test.rs API change**: The diffviz `RenderTestHarness::render(&self, ui_state, engine)` becomes `RenderTestHarness::render(&self, app: &M)` in the generic version. Phase 2's thin wrapper will need to adapt this — the `HeadlessApp` carries both `ui_state` and `review_engine` internally, so this is a simplification.
