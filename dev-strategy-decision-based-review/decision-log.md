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

[Additional decisions added chronologically during implementation]
