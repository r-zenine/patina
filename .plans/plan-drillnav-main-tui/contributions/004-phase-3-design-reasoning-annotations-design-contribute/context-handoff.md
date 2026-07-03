# Context Handoff - Phase 3 Design

## What Problem Are We Solving

Phase 3's roadmap left the exact visual treatment of reasoning annotations inside DrillNav chunk cards undecided: D5 (drillnav-main-tui) locked the tier (`CardTier::Content`) and general look (accent fg, no sigil) but deferred the specific color and, separately, whether the old diff_view's `◆` cross-decision badge still belongs anywhere in the new UI. This session resolved both with the user before implementation starts.

## Design Overview

Annotation lines render in **mauve**, injected before their trigger line using the unchanged trigger-line algorithm from `plan-inline-reasoning-annotations` D3 (`max(code_impact_range.start, diff_start_line)`). The `◆` badge is **dropped**, not relocated — it was a Browse-mode concern (surfacing that a chunk is reviewed under multiple decisions) that doesn't fit Drill mode, where the user has already committed to one decision; the per-annotation `D{n}` label already carries that information when `show_reasoning` is on. Rejected: peach (too warning-like next to red deletions) and keeping the badge in the chunk or file header (adds an unrequested feature).

## Reading Guide

Read `design-doc.md`'s "Implementation Guidance" section first — it names the exact port target (`diff_view.rs:129-158`), the injection point in `drillnav_drill.rs`'s code-row loop, and the test to un-ignore. "What We're NOT Doing" explains why the badge is gone, in case a future contributor is tempted to bring it back.
