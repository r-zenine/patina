# Two-Gate System Against Over-Engineering

Two checkpoints ensure design work happens at the right time — not too early (before you have real constraints) and not too late (after you've built something that doesn't fit).

## Gate 1 — dev-strategy: Should We Defer This Design?

During planning (dev-strategy skill), for each design decision in the roadmap:

**Do NOT defer if:**
- The design is straightforward and requirements are clear
- Implementation will not reveal meaningfully better constraints
- The decision is low risk and easily reversible

**DO defer (mark as "Design: Determine [X]" in roadmap) if:**
- Implementation will reveal actual constraints that should inform the design
- The decision is complex and user has not yet formed a concrete preference
- Getting implementation learnings first will produce a meaningfully simpler design
- Deciding now would require speculative design

**Gate 1 principle**: Only defer design to Gate 2 when the deferral will result in better design. If design is clear now, make it now.

---

## Gate 2 — design-contribute: Design With What We've Learned

When a "Design: Determine [X]" objective is reached during execution:

**Design constraints:**
- Only design what the current phase explicitly requires (YAGNI)
- No hypothetical features, no "what if" scenarios
- Maximum 3 design options; Option 1 must be the simplest
- design-doc.md target: < 100 lines
- Always present simplest option first with explicit "Why recommended" rationale

**Gate 2 principle**: You've deferred design to gather constraints. Now design with those constraints — don't defer further.

---

## How the Two Gates Work Together

```
dev-strategy planning
  └─ Phase N: "Implementation: Add JWT auth"
  └─ Phase N: "Design: Determine session management approach"  ← Gate 1 deferred this
  └─ Phase N+1: "Implementation: Implement session validation"

Execution:
  001-phase-N-implementation-jwt-auth/         ← reveals real constraints
  002-phase-N-design-session-mgmt/             ← Gate 2: design with learned constraints
  003-phase-N-implementation-session-validation/  ← implements the design
```

The two-gate system ensures:
- Planning is not blocked by premature design decisions
- Design is informed by real implementation constraints
- Implementation is guided by a lean, concrete design doc
