# Design Document - Reasoning Annotations in Drill Chunk Cards

> **Target**: < 100 lines total
> **Note**: Captures the outcome of an interactive design session with the user.

## Decision: What We're Doing

Reasoning annotation lines render inside a chunk card's code section (`CardTier::Content`), positioned immediately before their trigger line, styled in **mauve** foreground with no leading sigil (distinguishing them from `+`/`-` code lines). The old title-bar `◆` badge that signaled "this chunk is also referenced by other decisions" is **dropped entirely** — not ported, not relocated. `Space-t-r` still toggles `show_reasoning`; when on, one annotation line is injected per decision that references the focused file, using the same trigger-line algorithm the old `diff_view.rs` used (D3 of `plan-inline-reasoning-annotations`): `trigger_line = max(code_impact_range.start, diff_start_line)`, converted to a chunk-relative row index.

## Why This Design

**Constraints That Led Here:**
- D5 (drillnav-main-tui) already fixed the tier (`CardTier::Content`) and general treatment (accent fg, no sigil) — this session only had to pick the actual color and settle the badge question.
- lavender, green, red, yellow are already claimed by focus/approval/deletion/notes in the drill card — the annotation color had to come from the unclaimed set (sky, mauve, peach).
- A chunk (`ReviewableDiffId`) can map to more than one `Decision` (`get_decisions_for_diff` returns a `Vec`); the old UI's `◆` badge existed to surface that cross-decision membership when annotations were collapsed.

**User Priorities:** Distinct-but-calm color that doesn't compete with existing accents; skepticism toward carrying over a Browse-mode concern (which decision to review) into Drill mode, where the user has already committed to reviewing one specific decision.

**Simplicity Rationale:** Dropping the badge removes a feature nobody asked for in the new UX — cross-decision membership is still visible through the annotation lines themselves (each carries its own `D{n}` label) whenever `show_reasoning` is on, so no information is lost, only the collapsed-state summary. This keeps Phase 3 to one visual concept (annotation lines) instead of two (annotation lines + badge slot).

## How It Works

**Key Interfaces:**
- `drillnav_drill.rs`'s per-chunk render loop gains an annotation-computation step (ported from `diff_view.rs:129-158`) that runs once `show_reasoning` is true, producing `Vec<ReasoningAnnotation { trigger_line, label, reasoning }>` per chunk.
- Annotation lines are interleaved into the existing code-line loop by comparing each visible line's chunk-relative index against `trigger_line`, inserting the annotation line just before it (same mechanic as the old `hidden_indicator` injection).

**Core Pattern:** Pure computation over already-available data (`engine.get_decisions_for_diff`, `decision.code_impacts`) — no new `ReviewEngine` methods, no domain changes, matching D1 of `plan-inline-reasoning-annotations`.

**Integration Points:**
- Reads `ui_state.show_reasoning` (existing field, toggle unchanged).
- Reads `engine.get_decisions_for_diff(chunk_id)` (existing method, unchanged signature).
- Renders via `HierarchicalCard::at(CardTier::Content, ...)` — the same builder `drillnav_drill.rs` already uses for code rows.

## What We're NOT Doing

**Rejected Alternatives:**
- **Peach for annotation color**: reads as a warning next to red deletions — rejected for being too visually loud for informational text.
- **Sky for annotation color**: viable runner-up, but mauve was chosen as closer to the old UI's "dim" feel while still being a distinct hue from every other accent in the card.
- **Keep the `◆` badge in the chunk header, next to the ✓ approval badge**: rejected — cross-decision membership is a Browse-mode concern; duplicating it per-chunk in Drill mode adds a feature the roadmap never asked for (YAGNI).
- **Aggregate the badge once per file header instead of per-chunk**: same rejection — no requirement drove this, and it reintroduces state (which decisions has this file's badge already shown) for no proven benefit.

**Out of Scope:**
- Refining the trigger-line algorithm beyond the ported D3 heuristic (`max(range.start, diff_start)`) — no evidence of imprecision has surfaced; revisit only if observed.

## Implementation Guidance

**For Next Contributor:**
- Start by porting the annotation-computation closure from `diff_view.rs:129-158` into a helper (e.g. in `drillnav_common.rs`) that takes `&ReviewEngine`, a `&ReviewableDiffId`, and the chunk's diff-start line, returning `Vec<ReasoningAnnotation>`.
- In `drillnav_drill.rs`'s code-row loop (around line 159), check each visible line's index against pending annotations' `trigger_line` before pushing the code row; push the annotation row first if it matches, styled `Style::default().fg(theme.accents.mauve)`.
- Un-ignore `annotation_lines_appear_when_reasoning_on` in `reasoning_annotation_tests.rs` once wired.

**Testing Strategy:** Reuse the existing `--test-full "<Space>tr"` visual assertions from the deferred test; verify annotation line position matches trigger line, mauve styling, and no sigil.

**Success Criteria:** `Space-t-r` toggles annotation lines on/off in drill visual output at the correct chunk-relative row; no `◆` badge appears anywhere in Drill mode; zero warnings.
