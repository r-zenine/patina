# Code Context — TUI Simplification

## Files Being Deleted

- `diffviz-review-tui/src/navigation.rs` — deprecated legacy flat-list navigation, no live callers
- `diffviz-review-tui/src/ui/components/decision_list.rs` — never called in active draw path
- `diffviz-review-tui/src/ui/components/file_list.rs` — replaced by decision_tree, no live callers
- `diffviz-review-tui/src/diff/inline.rs` — `derive_inline_diff_map` always returns empty; `derive_inline_segments` also dead
- `diffviz-review-tui/src/diff/mod.rs` — becomes empty after inline.rs deletion

## Key Functions Being Removed

- **`ReviewTuiApp::into_review_engine`** (`src/app.rs:77-88`) — replaced with safe `Option::take` pattern
- **`UiState::toggle_file_expansion`** (`src/state.rs:359-365`) — only called from deleted navigation files
- **`UiState::is_file_expanded`** (`src/state.rs:368-370`) — only called from deleted navigation files
- **`UiState::start_edit_mode`** (`src/state.rs:163-167`) — only called from deleted `EnterEditMode` handler
- **`derive_inline_diff_map`** (`src/diff/inline.rs:23`) — no-op stub, always returns empty
- **`create_inline_old_line`** (`src/ui/components/renderable_diff_widget.rs:382-442`) — dead code path

## Key Types Being Removed

- **`InputMode::Edit`** (`src/state.rs:23`) — unbuilt feature variant
- **`BusinessEvent::EditContent`** (`src/events/business.rs:29-32`) — no-op handler `Ok(Command::None)`
- **`UiEvent::EnterEditMode`** (`src/events/input.rs`) — keybinding `Space → i → e`
- **`InlineDiffMap`**, **`InlineOldLine`**, **`InlineOldSegment`** (`src/diff/inline.rs:16`) — re-exported from `renderable_diff_widget.rs:14`
- **`UiState.file_list_selection: usize`** (`src/state.rs:60`) — only read by deleted navigation files
- **`UiState.expanded_files: HashSet<String>`** (`src/state.rs:57`) — only used by deleted methods

## Fields Being Removed from `StateSnapshot` (test harness)

- `file_list_selection: usize` (`src/test_harness/snapshot.rs:37,86,128`) — mirrors deleted state field
- `expanded_files: Vec<String>` (`src/test_harness/snapshot.rs:61,94,136`) — mirrors deleted state field

## Key Patterns in Active Code (do not break)

- **`DecisionNavigationTree`** (`src/decision_navigation.rs`) — active navigation system; untouched
- **`GutterBracketMap`** (`src/ui/components/renderable_diff_widget.rs:30`) — active gutter indicator type; keep as-is (separate from `InlineDiffMap`)
- **`RenderableDiffWidget` builder chain** (`src/ui/components/diff_view.rs:109-120`) — remove only `.show_inline_old(true)`; other builder methods stay
- **`Drop for ReviewTuiApp`** (`src/app.rs:166-173`) — terminal cleanup; untouched, handles cleanup when `into_review_engine` drops `self`

## Call Chains Affected by Phase 4 (inline removal)

```
diff_view.rs                     → RenderableDiffWidget::new()
  .show_inline_old(true)           → REMOVE this call
  → widget.render()
    → derives InlineDiffMap        → REMOVE Cow block (~lines 142-151)
    → append_line()
      → if ctx.show_inline_old     → REMOVE this block (~lines 347-353)
        → create_inline_old_line() → REMOVE this function (~lines 382-442)
```
