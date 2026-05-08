# Implementation Roadmap — TUI Error Consolidation

## Execution Strategy
**Strategy**: TDD
**Approach**: Write compile-time type assertions and test scaffolding first, then make them pass with the minimal mechanical changes. Each phase is independently shippable and leaves the workspace in a clean, warning-free state.

---

## Phase 1: Delete SamTuiError, wire tui_harness::TuiError

**Description**: Remove `sam-tui/src/error.rs` entirely. The `ModalView` and its headless counterpart both set `ELMApp::Error = SamTuiError`; replace with `tui_harness::TuiError`. Since `SamTuiError::Terminal` maps exactly to `TuiError::Terminal`, no conversion logic changes.

**Objectives**:
- **Test**: Add a type assertion in `sam-tui/tests/harness_tests.rs` (or a dedicated compile test) confirming `ModalView::Error = tui_harness::TuiError`
- **Implementation**: Delete `sam-tui/src/error.rs`, remove `pub mod error` from `lib.rs`, update `view.rs:5,85` and `headless.rs:6,53` to import and use `tui_harness::TuiError`

**Testing Criteria**:
- Compile test / type assertion passes
- `cargo test --package sam-tui` stays green
- `cargo clippy --package sam-tui -- -D warnings` passes with zero warnings

**Dependencies**: None

**Relevant Local Skills**: None

**Files to Modify**:
- `sam-tui/src/error.rs` — delete
- `sam-tui/src/lib.rs` — remove `pub mod error;`
- `sam-tui/src/modal_view/view.rs` — replace `use crate::error::SamTuiError` → `use tui_harness::TuiError`; `type Error = SamTuiError` → `type Error = TuiError`
- `sam-tui/src/modal_view/headless.rs` — same substitution as view.rs

---

## Phase 2: Introduce ReviewTuiError, replace DispatchError

**Description**: Create `diffviz-review-tui/src/error.rs` with a `ReviewTuiError` enum using `thiserror`. It has two variants — one for key dispatch failures, one for command execution failures — both wrapping `Box<dyn std::error::Error + Send + Sync + 'static>` to preserve the error chain. Remove `DispatchError` from `app.rs` and the public `anyhow::Result` alias from `lib.rs`.

**Objectives**:
- **Test**: Add a compile-time assertion that `ReviewTuiError: std::error::Error + Send + Sync + 'static` (use a `fn _assert<E: std::error::Error + Send + Sync + 'static>() {}` style static assert)
- **Implementation**:
  1. Create `diffviz-review-tui/src/error.rs`:
     ```rust
     use thiserror::Error;

     #[derive(Debug, Error)]
     pub enum ReviewTuiError {
         #[error("key event dispatch failed: {0}")]
         KeyDispatch(Box<dyn std::error::Error + Send + Sync + 'static>),
         #[error("command execution failed: {0}")]
         CommandExecution(Box<dyn std::error::Error + Send + Sync + 'static>),
     }
     ```
  2. Add `pub mod error; pub use error::ReviewTuiError;` to `lib.rs`; remove `pub type Result<T> = anyhow::Result<T>`
  3. In `app.rs`: delete `DispatchError` struct and impls; set `type Error = ReviewTuiError`; update `dispatch_key` to use `.map_err(|e| ReviewTuiError::KeyDispatch(e.into()))` and `.map_err(|e| ReviewTuiError::CommandExecution(e.into()))`
  4. Fix any remaining anyhow usages in `app.rs` that now have no `Result` alias — add `use anyhow::Result;` locally in the file or qualify explicitly

**Testing Criteria**:
- Static assert for `ReviewTuiError` trait bounds passes
- `cargo test --package diffviz-review-tui` stays green
- `cargo clippy --workspace -- -D warnings` passes with zero warnings
- `cargo check --workspace` passes clean

**Dependencies**: Phase 1 must be complete (validates the pattern before applying to the more complex crate)

**Relevant Local Skills**: `diffviz-tui-contribution` — must be invoked before starting work on this phase

**Files to Modify**:
- `diffviz-review-tui/src/error.rs` — create new file
- `diffviz-review-tui/src/lib.rs` — add `pub mod error; pub use error::ReviewTuiError;`; remove `pub type Result<T> = anyhow::Result<T>`
- `diffviz-review-tui/src/app.rs` — delete `DispatchError`; set `type Error = ReviewTuiError`; update two `map_err` calls; add local `use anyhow::Result` or inline anyhow import
