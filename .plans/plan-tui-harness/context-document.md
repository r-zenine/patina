# Context Document for tui-harness

## Behavioral Specification

Extract a standalone `tui-harness` Rust crate that provides a generic `ELMApp` trait and three testing harnesses (input-only, render-only, combined), enabling agentic headless testing of any ELM-architecture TUI application. Once the crate exists, port `diffviz-review-tui` to consume it (replacing its bespoke test harness module), then rewrite `sam-tui` to use it (migrating from `tui 0.19 + termion` to `ratatui 0.28 + crossterm` as part of that port). A kitchen-sink example binary in the crate demonstrates all harness features against a self-contained toy ELM app.

## Codebase Patterns to Follow

- **Workspace layout**: Rust workspace with path dependencies between crates (`diffviz-review-tui` → `diffviz-review`, etc.)
- **Feature gates**: `diffviz-review-tui` uses `[features] test-harness = []` to exclude test infrastructure from production builds. `tui-harness` itself is always test-time infrastructure and does not need a feature gate — consumers opt in via `[dev-dependencies]` or the `test-harness` feature.
- **Error handling**: `anyhow::Result` at application layer, `thiserror` for library errors. `diffviz` ZERO WARNING RULE: all `cargo check + clippy` warnings must be fixed after every change.
- **Testing style**: Unit tests live in the same file (`#[cfg(test)] mod tests`). Integration tests go in `tests/`. Headless tests use `HeadlessApp` + harness, never a real terminal.
- **Rust edition**: 2024 for diffviz crates; 2021 for sam crates. `tui-harness` should use edition 2024 to match diffviz.
- **No fallbacks in diffviz-core**: fail-fast, no defensive programming. Not relevant to tui-harness itself.

## ELMApp Trait Design

`tui-harness` serves two roles built on a single trait: a **development runtime** that eliminates event-loop boilerplate, and a **testing framework** for agentic headless testing. Implementing `ELMApp` once gives both.

```rust
pub trait ELMApp {
    /// Serializable state snapshot for test assertions and debug modes
    type Snapshot: serde::Serialize;

    /// App-owned error type — no framework type imposed
    type Error: std::error::Error + Send + Sync + 'static;

    /// Process a raw key event, mutating internal state
    fn dispatch_key(&mut self, key: crossterm::event::KeyEvent) -> std::result::Result<(), Self::Error>;

    /// Draw current state into a ratatui Frame
    fn draw(&self, frame: &mut ratatui::Frame);

    /// Whether the app should stop running (drives run_app loop)
    fn should_quit(&self) -> bool;

    /// Capture current state as a serializable snapshot
    fn snapshot(&self) -> Self::Snapshot;
}
```

The framework provides `run_app<M: ELMApp>(app: &mut M) -> Result<()>` which owns terminal setup, the 60fps frame loop, and teardown. Apps call it instead of implementing their own event loops.

The trait dispatches `KeyEvent` (not domain events): each app owns its key→domain-event mapping internally. This mirrors Elm's runtime — it delivers `Msg` values; the app's `update` handles them. No shared event abstraction is needed or possible across unrelated apps.

## Technical Constraints

- **No anyhow in tui-harness**: `anyhow` is for application binaries, not library crates. `tui-harness` uses `thiserror` for its own `TuiError`; consumers expose their errors via `type Error` without the framework imposing a type. diffviz can keep `type Error = anyhow::Error` (it implements `std::error::Error`); sam uses a new `SamTuiError` via `thiserror`.
- **No TTY available to agents**: all test execution must be headless (no real terminal). This is the primary motivation for the harness.
- **Monorepo**: All sam crates have been moved into the patina workspace. `tui-harness` and `sam-tui` are now siblings in the same workspace — the dependency is simply `path = "../tui-harness"`. No cross-workspace path dep is needed.
- **sam-tui migration prerequisite**: Phase 3 requires migrating sam-tui from `tui 0.19 + termion` to `ratatui 0.28 + crossterm` before ELMApp can be implemented. The migration is low-risk (ratatui is a maintained fork of tui; API is largely the same).
- **Recursive loop in sam-tui**: `ModalView.run()` is recursive (stack-based). Phase 3 converts this to an iterative loop consistent with diffviz's frame-based approach.
- **`ViewState<V: Value>` snapshot**: `V` may not be `Serialize`. The `ELMApp::Snapshot` for sam's headless app should be a concrete struct capturing only the primitives (cursor, filter, mode, marked count), not the generic items.

## Local Repository Skills

- **diffviz-tui-contribution** — Mandatory whenever interacting with `diffviz-review-tui` (adding code, testing, planning changes). Use for Phases 1 and 2.
- **dev-contribute** — Use when implementing each phase (tracks contribution artifacts, ensures proper documentation and knowledge transfer).
- **design-contribute** — Use when a Design objective in the roadmap needs collaborative refinement before implementation.
- **contribution-system** — Canonical reference for artifact schemas, folder naming, and the two-gate system.
