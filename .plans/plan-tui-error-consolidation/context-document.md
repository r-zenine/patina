# Context Document — TUI Error Consolidation

## Behavioral Specification

Consolidate three independent TUI error definitions into a single canonical source. `tui-harness::TuiError` already exists and is well-structured; the goal is to delete `sam-tui::SamTuiError` (strict subset, unused outside sam-tui) and replace the stringly-typed `diffviz-review-tui::DispatchError` with a proper `ReviewTuiError` that preserves error chains. All TUI consumers will express their error boundary through types in `tui-harness` or their own well-typed error, not through string formatting.

## Codebase Patterns to Follow

- **Error design**: `thiserror` for library/boundary errors; `anyhow` is acceptable for internal application-level propagation (private functions)
- **ELMApp boundary**: `ELMApp::Error` is the public contract — must be a concrete, typed error implementing `std::error::Error + Send + Sync + 'static`
- **Error chain preservation**: use `Box<dyn Error + Send + Sync + 'static>` to wrap opaque errors without losing the chain (same pattern as `TuiError::App`)
- **Zero warning rule**: `cargo fmt`, `cargo clippy --workspace -- -D warnings`, and `cargo check --workspace` must pass clean after every change

## Technical Constraints

- `anyhow::Error` does not implement `std::error::Error` — it cannot be used as `ELMApp::Error` directly
- `diffviz-review-tui` internal functions (`process_key_event_impl`, `handle_ui_event_impl`, `handle_business_event_impl`) return `anyhow::Result` — **do not migrate these**; only the ELMApp dispatch boundary needs a concrete type
- `sam-tui` uses Rust 2021 edition; `diffviz-review-tui` uses Rust 2024 edition — no cross-edition concerns for this refactor
- `tui-harness` has no internal workspace dependencies — it is safe to depend on from any TUI consumer

## Two-Layer Error Model (agreed design)

```
tui-harness::TuiError          — framework layer: Terminal I/O errors (keep as-is)
diffviz-review-tui::ReviewTuiError  — app dispatch layer: replaces DispatchError
```

`ReviewTuiError` sits at the `ELMApp::Error` boundary in `diffviz-review-tui`. It must wrap errors from `process_key_event` and `execute_command` (both return `anyhow::Result`) by boxing them via `anyhow::Error`'s `Into<Box<dyn Error + Send + Sync + 'static>>` impl — preserving the full chain without formatting to string.

## Local Skills

- **`diffviz-tui-contribution`** — mandatory when touching `diffviz-review-tui`; must be invoked before contributing to that crate
