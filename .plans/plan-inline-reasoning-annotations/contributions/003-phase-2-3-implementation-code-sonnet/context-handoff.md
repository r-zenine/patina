# Context Handoff — Contribution 003

**Phase**: Phase 2 (Widget Annotation Injection) + Phase 3 (View Integration)  
**Commit**: f19686c  
**Status**: Complete — all 8 reasoning annotation tests pass, zero warnings, clippy clean

---

## What was done

Phase 2 added the `ReasoningAnnotation` struct and widget infrastructure to `renderable_diff_widget.rs`:
- `ReasoningAnnotation { trigger_line, label, reasoning }` — `trigger_line` is **1-based relative** to the rendered boundary, not the absolute file line number
- `with_reasoning_annotations(&[ReasoningAnnotation])` builder on `RenderableDiffWidget`
- `annotation_line()` renders `  ◆ {label}  {reasoning}` with muted/italic styling
- O(1) HashMap lookup; annotations injected before their trigger line in both `show_all_context` paths

Phase 3 wired the data flow in `diff_view.rs`:
- `review_engine.get_decisions_for_diff(&reviewable_diff.id)` retrieves mapped decisions
- Title badge `  ◆ D1` appended when `show_reasoning=false` and decisions exist (collapsed state)
- When `show_reasoning=true`: annotations built from `decision.code_impacts`, absolute line converted to relative using `abs.saturating_sub(diff_start).saturating_add(1)` (see `doc/backlog/renderable-line-number-relative-vs-absolute.md`)
- Widget receives annotations via `.with_reasoning_annotations(&annotations)`

### Coordinate conversion (the core workaround)

`RenderableLine.line_number` is 1-based relative to the rendered boundary, not the file.  
`overall_line_range.start_line` on `RenderableDiff.metadata` is the absolute boundary start.  
Conversion: `relative = absolute.saturating_sub(diff_start).saturating_add(1)`

This is approximate for Myers-diff Modified boundaries when deletions precede the target line (each deleted line consumes a `line_number` slot). Good enough for this feature; the proper fix is tracked in `doc/backlog/renderable-line-number-relative-vs-absolute.md`.

### Test size choices

Three visual tests use `CombinedTestHarness::with_render_size(engine, 120, 40)` or `160×40` because:
- At default 80×24, diff panel inner area ≈ 58 chars — too narrow for a 79-char reasoning string (wraps across rows, `contains()` fails) and for the title badge when the boundary name is long.
- The wider sizes match what a real terminal would show for this feature.

---

## What was found / notable discoveries

1. **Phase 0 test design had two bugs** (not in the implementation):
   - `title_badge_hidden_when_reasoning_on` checked `!visual.contains("◆ D")` over the entire visual — but annotation body lines also contain `◆ D1`, so it always failed. Fixed: check only the first visual row (the title bar).
   - Group 3 tests used `new(engine)` (80×24), too narrow for the reasoning text. Fixed: `with_render_size`.

2. **`get_decisions_for_diff` was already implemented** in the ReviewEngine — no domain changes needed.

3. The `abs_trigger.max(reviewable_diff.id.line_range.start_line)` clamp ensures the trigger never maps below the boundary; without it, a decision whose impact starts before the rendered range would produce `trigger_line = 0`, which never matches any `line_number`.

---

## State for the next phase

According to the roadmap there is no Phase 4 — the plan is complete. All three phases (Phase 0 test design, Phase 1 toggle infra, Phase 2+3 annotation injection) are committed and passing.

If Phase 4 work is added (e.g., multi-decision collapsing, scrollable annotation popups, or fixing the coordinate approximation for Myers diffs), the relevant files are:
- `renderable_diff_widget.rs` — annotation rendering
- `diff_view.rs` — data flow and coordinate conversion
- `doc/backlog/renderable-line-number-relative-vs-absolute.md` — proper fix description
