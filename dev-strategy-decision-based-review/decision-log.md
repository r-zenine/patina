# Decision Log for Decision-Based Review

## Strategy: Steel Thread
**Why**: Multiple integration points (diffviz-review, diffviz-review-tui, dev-contribute), need early UX validation with mock data before committing to skill changes
**Rejected**: TDD (requirements clear but need UX feedback first), Core-then-Integrate (not heavy external deps)

## Architecture: Decisions in diffviz-review only
**Why**: Keep diffviz-core pure for semantic analysis. Decisions are review workflow metadata, not core domain concepts.
**Rejected**: diffviz-core (would pollute semantic analysis layer), Both layers (unnecessary complexity)

## Overlapping Code: Review twice under both decisions
**Why**: User preference - ensures each decision gets proper review attention
**Rejected**: Toggle (adds UI complexity), Split ranges (loses context)

## Precision: Function-level mapping
**Why**: Simpler to generate, matches semantic analysis granularity in diffviz-core
**Rejected**: Hunk-level (harder to generate accurately), Both (unnecessary complexity for MVP)

## Confidence: Three-level (high/medium/low)
**Why**: Balanced granularity for filtering without over-engineering
**Rejected**: Binary (too coarse), Numeric (over-engineered)

## No-code Decisions: Include with empty code_impacts
**Why**: Complete decision context even for architectural decisions
**Rejected**: Exclude (loses important context)

## Backward Compatibility: None
**Why**: Decision-based review only for new contributions with mapping files
**Rejected**: Graceful degradation (adds complexity, per CLAUDE.md no-fallback rule)

## Phasing: DiffViz first, then dev-contribute
**Why**: Validate TUI UX with mock/static data before touching skill system. Use existing mock binary for fast iteration.
**Rejected**: dev-contribute first (would build mapping without knowing if TUI works)

## Navigation Pattern: Decision-first hierarchy (not dual-view toggle)
**Why**: Aligns with core vision ("decision-based review"), simpler mental model, complete UX validates concept better
**Details**: Navigation hierarchy: Decision List → Modal → File View → Chunk Detail. No file-first mode.
**Rejected**: Decision badges in file view (treats decisions as secondary), Dual-view toggle (complexity, unclear primary pattern)

## Unmapped Code: Synthetic Decision 0
**Why**: Ensures all code accessible through decision navigation, no special fallback modes needed
**Rejected**: Separate file view for unmapped only (maintains dual navigation), Require all code mapped (too strict)

## Decision Detail: Modal view
**Why**: Minimizes TUI architecture changes, reuses existing modal pattern, quick context without navigation state complexity
**Rejected**: Full panel (larger TUI changes), Inline expansion (harder to scan impacts)

[Additional decisions added chronologically during implementation]
