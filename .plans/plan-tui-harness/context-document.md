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

```rust
/// A headless-testable ELM-architecture TUI application.
pub trait ELMApp {
    /// Serializable state snapshot for assertions
    type Snapshot: serde::Serialize;

    /// Process a raw key event, mutating internal state
    fn dispatch_key(&mut self, key: crossterm::event::KeyEvent) -> anyhow::Result<()>;

    /// Capture current state as a serializable snapshot
    fn snapshot(&self) -> Self::Snapshot;

    /// Draw current state into a ratatui Frame (used by RenderTestHarness)
    fn draw(&self, frame: &mut ratatui::Frame);
}
```

The trait dispatches at `KeyEvent` (not domain events) because: (1) the harness is testing infrastructure, not a business framework, (2) each app manages its own key→domain-event mapping internally, (3) this avoids needing a shared event abstraction across unrelated apps.

## Technical Constraints

- **No TTY available to agents**: all test execution must be headless (no real terminal). This is the primary motivation for the harness.
- **Workspace boundary**: `tui-harness` lives in the patina workspace. `sam-tui` references it via `path = "../../patina/tui-harness"` (relative from sam workspace). This is an acceptable path dep for development; can be published later.
- **sam-tui migration prerequisite**: Phase 3 requires migrating sam-tui from `tui 0.19 + termion` to `ratatui 0.28 + crossterm` before ELMApp can be implemented. The migration is low-risk (ratatui is a maintained fork of tui; API is largely the same).
- **Recursive loop in sam-tui**: `ModalView.run()` is recursive (stack-based). Phase 3 converts this to an iterative loop consistent with diffviz's frame-based approach.
- **`ViewState<V: Value>` snapshot**: `V` may not be `Serialize`. The `ELMApp::Snapshot` for sam's headless app should be a concrete struct capturing only the primitives (cursor, filter, mode, marked count), not the generic items.

## Local Repository Skills

- **diffviz-tui-contribution** — Mandatory whenever interacting with `diffviz-review-tui` (adding code, testing, planning changes). Use for Phases 1 and 2.
- **dev-contribute** — Use when implementing each phase (tracks contribution artifacts, ensures proper documentation and knowledge transfer).
- **design-contribute** — Use when a Design objective in the roadmap needs collaborative refinement before implementation.
- **contribution-system** — Canonical reference for artifact schemas, folder naming, and the two-gate system.
