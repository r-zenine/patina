# diffviz-review-tui Simplification Plan

## Context

An audit against project design principles (YAGNI, Kent Beck's 4 Rules, Sufficient Implementation) identified 6 categories of violations in `diffviz-review-tui`. This plan removes dead code, disabled features, and an unsafe workaround that has a safe alternative.

> **Note:** The test harness is intentionally excluded — it serves agentic testing (agents cannot launch a TTY to drive the TUI directly).

---

## Execution Order

Steps are ordered safest-first. After each step: `cargo fmt --all && cargo clippy --workspace -- -D warnings && cargo check --workspace`.

---

### Step 1 — Remove misleading TODO in `diff_formatter.rs`

**File:** `src/formatting/diff_formatter.rs:144`

Delete the comment `// TODO: Fix lifetime issues properly`. The current approach (owned strings → `Line<'static>`) is correct. The comment is misleading, not the code.

---

### Step 2 — Extract magic number constants in `state.rs`

**File:** `src/state.rs`

Add near the top (below `use` block):
```rust
const LEADER_TIMEOUT: Duration = Duration::from_secs(2);
const PAGE_SCROLL_STEP: usize = 10;
const CURSOR_VIEW_HEIGHT_FALLBACK: usize = 20;
```

Replace:
- Lines 396, 405: `Duration::from_secs(2)` → `LEADER_TIMEOUT`
- Lines 233, 243, 254: `10` (page scroll step) → `PAGE_SCROLL_STEP`
- Lines 272, 293: `20` (view height fallback) → `CURSOR_VIEW_HEIGHT_FALLBACK`

---

### Step 3 — Fix unsafe `into_review_engine()` in `app.rs`

**File:** `src/app.rs`

The current `into_review_engine()` uses `ManuallyDrop` + `unsafe { ptr::read() }`. Safe alternative: `Option<ReviewEngine>` + `take()`.

Changes:
1. Change field: `review_engine: ReviewEngine` → `review_engine: Option<ReviewEngine>`
2. In `new()`: pass raw `&review_engine` to `initialize_ui_state()`, then wrap with `Some(review_engine)`
3. All read access: `self.review_engine` → `self.review_engine.as_ref().expect("review_engine present")`
4. All mut access: → `self.review_engine.as_mut().expect("review_engine present")`
5. Replace `into_review_engine()` body:
   ```rust
   pub fn into_review_engine(mut self) -> ReviewEngine {
       // Drop impl handles terminal cleanup when self drops at end of function
       self.review_engine.take().expect("review_engine already taken")
   }
   ```
6. The existing `Drop` impl accesses only `self.terminal` — no change needed there.

---

### Step 4 — Remove disabled inline diff system

`derive_inline_diff_map()` always returns `InlineDiffMap::default()` (empty). `show_inline_old(true)` is hardcoded in `diff_view.rs` but the map is always empty — the entire code path is dead.

**`src/ui/components/renderable_diff_widget.rs`:**
- Remove `use crate::diff::inline::derive_inline_diff_map` import (line 1)
- Remove `pub use crate::diff::inline::{InlineDiffMap, InlineOldLine, InlineOldSegment}` re-export (line 14)
- Remove `show_inline_old: bool` and `inline_changes: Option<&'a InlineDiffMap>` from struct and `LineRenderContext`
- Remove builder methods `show_inline_old()` and `with_inline_changes()`
- Remove the `Cow` block for `inline_changes` in `render()` (~lines 142–151) and `inline_changes_ref` binding
- Remove `if ctx.show_inline_old { ... }` block in `append_line()` (~lines 347–353)
- Remove `create_inline_old_line()` function (~lines 382–442)
- Remove `use std::borrow::Cow` if now unused

**`src/ui/components/diff_view.rs`:**
- Line 112: remove `.show_inline_old(true)` from the widget builder chain

**`src/diff/inline.rs`** → delete file

**`src/diff/mod.rs`** → delete file

**`src/lib.rs`:**
- Remove `pub mod diff;`

---

### Step 5 — Remove incomplete `EditContent` feature

`BusinessEvent::EditContent` is defined with a conversion path from `InputMode::Edit`, but the handler is `Ok(Command::None)`. Unbuilt feature — remove entirely.

**`src/events/input.rs`:**
- Remove `EnterEditMode` variant from `UiEvent`
- Remove `(Some('i'), KeyCode::Char('e')) => Some(UiEvent::EnterEditMode)` keybinding

**`src/events/business.rs`:**
- Remove `EditContent { reviewable_id, new_content }` variant from `BusinessEvent`
- Remove `InputMode::Edit { .. } => Some(BusinessEvent::EditContent { ... })` arm in `ui_event_to_business_event()`

**`src/app.rs`:**
- Remove `UiEvent::EnterEditMode` handler arm from `handle_ui_event_impl()`
- Remove `BusinessEvent::EditContent { .. } => Ok(Command::None)` from `handle_business_event_impl()`

**`src/ui/components/input_modal.rs`:**
- Remove `InputMode::Edit { .. }` match arm from `get_modal_content()`

**`src/ui/components/status_bar.rs`:**
- Remove `InputMode::Edit { .. }` match arm

**`src/state.rs`:**
- Remove `Edit { reviewable_id: ReviewableDiffId }` variant from `InputMode`
- Remove `start_edit_mode()` method

---

### Step 6 — Remove dead navigation infrastructure

Three files and their orphaned state fields are entirely dead. `decision_tree` is the active navigation system.

**Delete files:**
- `src/navigation.rs`
- `src/ui/components/decision_list.rs`
- `src/ui/components/file_list.rs`

**`src/lib.rs`:**
- Remove `pub mod navigation;`

**`src/ui/components/mod.rs`:**
- Remove `pub mod decision_list;`
- Remove `pub mod file_list;`

**`src/state.rs`:**
- Remove `file_list_selection: usize` field + initializer
- Remove `expanded_files: HashSet<String>` field + initializer
- Remove `toggle_file_expansion()` method (only callers are deleted files)
- Remove `is_file_expanded()` method (only callers are deleted files)
- Remove `use std::collections::HashSet` if now unused

**`src/test_harness/snapshot.rs`:**
- Remove `file_list_selection` from `StateSnapshot` struct, `from_ui_state()`, and test constructor
- Remove `expanded_files` from `StateSnapshot` struct, `from_ui_state()`, and test constructor

---

## Verification

After all steps complete:
```bash
cargo fmt --all
cargo clippy --workspace -- -D warnings   # zero warnings
cargo check --workspace                   # clean compile
cargo test --workspace --all-features     # all tests pass
```

## Critical Files by Step

| File | Steps |
|------|-------|
| `src/app.rs` | 3, 5 |
| `src/state.rs` | 2, 5, 6 |
| `src/events/business.rs` | 5 |
| `src/events/input.rs` | 5 |
| `src/ui/components/renderable_diff_widget.rs` | 4 |
| `src/ui/components/diff_view.rs` | 4 |
| `src/ui/components/input_modal.rs` | 5 |
| `src/ui/components/status_bar.rs` | 5 |
| `src/test_harness/snapshot.rs` | 6 |
| `src/formatting/diff_formatter.rs` | 1 |
| `src/lib.rs` | 4, 6 |
| `src/ui/components/mod.rs` | 6 |
