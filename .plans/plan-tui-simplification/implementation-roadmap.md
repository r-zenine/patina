# Implementation Roadmap — TUI Simplification

## Execution Strategy

**Strategy**: TDD
**Approach**: Each phase has a mechanical test criterion — `cargo clippy --workspace -- -D warnings` passes clean. Phases are ordered safest-first (fewest files, no cross-file coordination) to broadest-surface (state + snapshot + multiple components). Each phase is a complete deliverable; the build is never broken between phases.

---

## Phase 1: Trivial Cleanups

**Description**: Remove a misleading comment, extract magic numbers to named constants, and replace the unsafe `into_review_engine()` with a safe `Option<ReviewEngine>` + `take()` pattern. All changes are self-contained within 2 files.

**Objectives**:
- **Implementation**: Delete misleading `// TODO: Fix lifetime issues properly` comment from `diff_formatter.rs:144` — the owned-string approach is correct
- **Implementation**: Add 3 named constants to `state.rs` — `LEADER_TIMEOUT`, `PAGE_SCROLL_STEP`, `CURSOR_VIEW_HEIGHT_FALLBACK` — and replace all magic number occurrences
- **Implementation**: Change `review_engine: ReviewEngine` field in `ReviewTuiApp` to `Option<ReviewEngine>`; update `new()`, all access sites, and rewrite `into_review_engine()` using `Option::take()`; remove `ManuallyDrop` + `ptr::read` unsafe block

**Testing Criteria**:
- `cargo clippy --workspace -- -D warnings` passes with zero warnings
- `cargo check --workspace` compiles clean
- No unsafe blocks remain in `app.rs`

**Dependencies**: None

**Relevant Local Skills**: `diffviz-tui-contribution`

**Files to Modify**:
- `diffviz-review-tui/src/formatting/diff_formatter.rs` — remove TODO comment at line 144
- `diffviz-review-tui/src/state.rs` — add 3 constants, replace 7 magic number occurrences
- `diffviz-review-tui/src/app.rs` — change field type, update access sites, rewrite `into_review_engine()`

---

## Phase 2: Remove Disabled Inline Diff System

**Description**: Delete the `diff/inline.rs` module and all its dead plumbing through `renderable_diff_widget.rs` and `diff_view.rs`. The `derive_inline_diff_map` function always returns an empty map, making the entire code path unreachable.

**Objectives**:
- **Implementation**: Delete `src/diff/inline.rs` and `src/diff/mod.rs`; remove `pub mod diff` from `lib.rs`
- **Implementation**: Remove all inline-diff-related fields, methods, and types from `renderable_diff_widget.rs` — `show_inline_old`, `inline_changes`, `with_inline_changes()`, `create_inline_old_line()`, `InlineDiffMap`/`InlineOldLine`/`InlineOldSegment` re-exports, the `Cow` derivation block, and the `append_line` inline block
- **Implementation**: Remove `.show_inline_old(true)` from the widget builder chain in `diff_view.rs:112`

**Testing Criteria**:
- `cargo clippy --workspace -- -D warnings` passes with zero warnings
- `cargo check --workspace --all-features` compiles clean
- `GutterBracketMap` (active gutter indicator type) is unaffected and still compiles

**Dependencies**: Phase 1

**Relevant Local Skills**: `diffviz-tui-contribution`

**Files to Modify**:
- `diffviz-review-tui/src/diff/inline.rs` — **delete**
- `diffviz-review-tui/src/diff/mod.rs` — **delete**
- `diffviz-review-tui/src/lib.rs` — remove `pub mod diff`
- `diffviz-review-tui/src/ui/components/renderable_diff_widget.rs` — remove imports, struct fields, builder methods, Cow block, inline rendering block, `create_inline_old_line` function
- `diffviz-review-tui/src/ui/components/diff_view.rs` — remove `.show_inline_old(true)` from builder chain

---

## Phase 3: Remove Incomplete `EditContent` Feature

**Description**: Delete `InputMode::Edit`, `BusinessEvent::EditContent`, their keybinding, and all match arms across 6 files. The handler is `Ok(Command::None)` — the feature was never built.

**Objectives**:
- **Implementation**: Remove `EnterEditMode` from `UiEvent` enum and its `Space → i → e` keybinding from `input.rs`
- **Implementation**: Remove `EditContent` variant from `BusinessEvent` and its conversion arm in `business.rs`
- **Implementation**: Remove `UiEvent::EnterEditMode` handler arm and `BusinessEvent::EditContent` no-op arm from `app.rs`
- **Implementation**: Remove `InputMode::Edit` match arms from `input_modal.rs` and `status_bar.rs`
- **Implementation**: Remove `Edit` variant from `InputMode` enum and `start_edit_mode()` method from `state.rs`

**Testing Criteria**:
- `cargo clippy --workspace -- -D warnings` passes with zero warnings — all match arms exhaustive
- `cargo check --workspace --all-features` compiles clean

**Dependencies**: Phase 2

**Relevant Local Skills**: `diffviz-tui-contribution`

**Files to Modify**:
- `diffviz-review-tui/src/events/input.rs` — remove `EnterEditMode` variant + keybinding
- `diffviz-review-tui/src/events/business.rs` — remove `EditContent` variant + conversion arm
- `diffviz-review-tui/src/app.rs` — remove `EnterEditMode` handler + `EditContent` no-op arm
- `diffviz-review-tui/src/ui/components/input_modal.rs` — remove `Edit` match arm
- `diffviz-review-tui/src/ui/components/status_bar.rs` — remove `Edit` match arm
- `diffviz-review-tui/src/state.rs` — remove `Edit` variant + `start_edit_mode()` method

---

## Phase 4: Remove Dead Navigation Infrastructure

**Description**: Delete the three legacy navigation files and clean up the orphaned state fields (`file_list_selection`, `expanded_files`) from `UiState` and `StateSnapshot`. The `decision_tree` system is the active navigation — these files have no live callers.

**Objectives**:
- **Implementation**: Delete `navigation.rs`, `decision_list.rs`, `file_list.rs`; remove their module declarations from `lib.rs` and `ui/components/mod.rs`
- **Implementation**: Remove `file_list_selection`, `expanded_files`, `toggle_file_expansion()`, `is_file_expanded()`, and `use std::collections::HashSet` from `state.rs`
- **Implementation**: Remove `file_list_selection` and `expanded_files` fields from `StateSnapshot` struct, `from_ui_state()`, and test constructors in `test_harness/snapshot.rs`

**Testing Criteria**:
- `cargo clippy --workspace -- -D warnings` passes with zero warnings
- `cargo check --workspace --all-features` compiles clean
- `cargo test --workspace --all-features` all tests pass

**Dependencies**: Phase 3

**Relevant Local Skills**: `diffviz-tui-contribution`

**Files to Modify**:
- `diffviz-review-tui/src/navigation.rs` — **delete**
- `diffviz-review-tui/src/ui/components/decision_list.rs` — **delete**
- `diffviz-review-tui/src/ui/components/file_list.rs` — **delete**
- `diffviz-review-tui/src/lib.rs` — remove `pub mod navigation`
- `diffviz-review-tui/src/ui/components/mod.rs` — remove `pub mod decision_list` and `pub mod file_list`
- `diffviz-review-tui/src/state.rs` — remove fields, methods, and `HashSet` import
- `diffviz-review-tui/src/test_harness/snapshot.rs` — remove `file_list_selection` and `expanded_files` from struct, `from_ui_state()`, and test
