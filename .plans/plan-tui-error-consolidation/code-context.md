# Code Context — TUI Error Consolidation

## Canonical Error Type (keep, do not change)

- **`TuiError`** (`tui-harness/src/error.rs:1-14`) — framework-level error with `Terminal(#[from] io::Error)` and `App(Box<dyn Error + Send + Sync>)`; already re-exported from `tui-harness/src/lib.rs:10`
- **`tui_harness::Result<T>`** (`tui-harness/src/error.rs:14`) — canonical result alias exported from the harness

## Phase 1 — Files to Change (sam-tui cleanup)

- **`sam-tui/src/error.rs:1-7`** — `SamTuiError { Terminal(#[from] io::Error) }` — **DELETE this file**
- **`sam-tui/src/lib.rs:1`** — `pub mod error;` — **remove this line**
- **`sam-tui/src/modal_view/view.rs:5,85`** — imports `SamTuiError`, sets `type Error = SamTuiError` — swap to `tui_harness::TuiError`
- **`sam-tui/src/modal_view/headless.rs:6,53`** — same pattern as view.rs — swap to `tui_harness::TuiError`

## Phase 2 — Files to Change (diffviz-review-tui error layer)

- **`diffviz-review-tui/src/lib.rs:31`** — `pub type Result<T> = anyhow::Result<T>` — this is the public crate alias; **remove it** (internal modules can use anyhow privately)
- **`diffviz-review-tui/src/app.rs:22-34`** — `DispatchError(String)` wrapper that formats anyhow errors to strings, losing the chain — **replace with `ReviewTuiError`**
- **`diffviz-review-tui/src/app.rs:88-98`** — `ELMApp::Error = DispatchError` impl — **update to `ReviewTuiError`**
- **`diffviz-review-tui/src/app.rs:92-96`** — two `map_err(|e| DispatchError(format!("{e:#}")))` calls — **update to use `ReviewTuiError` variants**

## Key Architectural Constraint

`ELMApp::Error` must satisfy `std::error::Error + Send + Sync + 'static` (see `tui-harness/src/traits.rs:10`). `anyhow::Error` intentionally does not implement `std::error::Error` — this is why `DispatchError` exists today. `ReviewTuiError` must wrap errors using `Box<dyn Error + Send + Sync + 'static>`, not strings.

## Internal anyhow Usage (do not touch)

These functions return `anyhow::Result` internally and should remain as-is:
- `app.rs::process_key_event_impl` (line 123) — private
- `app.rs::handle_ui_event_impl` (line 150) — private
- `app.rs::handle_business_event_impl` (line 376) — private

## Testing Patterns

- Validation: `cargo check --workspace` and `cargo clippy --workspace -- -D warnings` must pass clean after each phase
- `sam-tui/tests/harness_tests.rs` — existing integration tests must remain green
