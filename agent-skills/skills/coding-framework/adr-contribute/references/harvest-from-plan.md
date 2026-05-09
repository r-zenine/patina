# Harvest Strategy: From Dev-Strategy Plan

Use this strategy when a `.plans/plan-[FEATURE-NAME]/` directory exists with at least one contribution.

Reads every `decision-log.yaml` across the plan's contributions and applies the three-gate filter to surface ADR candidates.

---

## Tactic: Read All Decision Logs

Read decision logs in this order:

1. **Root plan decision-log**: `.plans/plan-[FEATURE-NAME]/decision-log.yaml`
   — Strategy-level decisions (e.g. execution strategy choice, architecture framing)

2. **All contribution decision logs**: `.plans/plan-[FEATURE-NAME]/contributions/*/decision-log.yaml`
   — Read in sequential order (001, 002, 003, ...)
   — Note whether each contribution is design type (code_impacts: []) or implementation type (code_impacts populated)

For each decision extracted, record:
- Source (root plan / contribution folder name)
- Decision number and title
- Rationale text
- Whether code_impacts is populated (implementation decision) or empty (design/strategy decision)
- Affected files and line ranges (from code_impacts, if present)

**Do not read context-handoff.md files** — decision-log.yaml is the authoritative source. Context handoffs are for progressive disclosure during implementation, not for decision harvesting.

---

## Tactic: Apply Three-Gate Filter

Apply all three gates to each extracted decision. A decision must pass ALL THREE to be recommended as an ADR candidate.

### Gate 1: Hard to Reverse

The decision would be costly or time-consuming to change later.

**Signals that pass Gate 1:**
- Touches core abstractions or shared interfaces (visible in code_impacts spanning multiple files)
- Commits to a technology, protocol, or data format with lock-in
- Establishes a structural pattern others will build on
- Changes a domain boundary or ownership rule

**Signals that fail Gate 1:**
- Implementation detail confined to a single module
- Decision is easy to reverse with a focused refactor
- Choice of one equivalent library over another (if either could be swapped in a day)

### Gate 2: Surprising Without Context

A future reader looking at the code would wonder "why did they do it this way?" — the code alone doesn't explain it.

**Signals that pass Gate 2:**
- Deliberately avoided the obvious approach for a non-obvious reason
- Constraint came from outside the codebase (compliance, partner API, deployment target)
- The code looks suboptimal without knowing the trade-off that was made
- Rejected a pattern that a reasonable engineer would expect to see here

**Signals that fail Gate 2:**
- The code is self-explanatory or follows an obvious pattern
- The decision is well documented in comments or README already
- Any competent reader would make the same choice without being told why

### Gate 3: Real Trade-Off

There were genuine alternatives and one was chosen for specific reasons.

**Signals that pass Gate 3:**
- Rationale explicitly mentions alternatives considered
- A design contribution (design-doc.md) shows options were explored
- The rejected option has real merit — it wasn't obviously wrong

**Signals that fail Gate 3:**
- "We did the obvious thing" — there was no meaningful alternative
- The rationale only explains what was done, not why over alternatives
- One option was clearly superior; no trade-off was made

---

## Tactic: Present Candidate List

**When to use**: Three-gate filter applied to all decisions.

Present a table of all decisions assessed, then a summary of recommended candidates.

**Format:**

```
## ADR Candidates — [Plan Name]

| # | Decision | Source | Gate 1 | Gate 2 | Gate 3 | Verdict |
|---|----------|--------|--------|--------|--------|---------|
| 1 | [title]  | 002-phase-1-... | ✓ | ✓ | ✓ | Recommended |
| 2 | [title]  | root plan | ✓ | ✓ | - | Optional (Gate 3 weak) |
| 3 | [title]  | 004-phase-2-... | - | ✓ | ✓ | Skip (not hard to reverse) |
| 4 | [title]  | 001-phase-1-... | - | - | - | Skip |

**Recommended for ADR**: 1
**Optional** (judgment call): 2
**Not recommended**: 3, 4

Which would you like to write? Enter numbers (e.g. "1, 2"), "all recommended", or "none".
```

**Rules:**
- Show ALL decisions assessed, not just recommended ones — the human may disagree with the filter
- For each failing gate, note briefly why it failed (one clause)
- Never pre-write ADRs before user confirms selection
- Don't ask multiple questions at once — present the table, then ask for selection

---

## Quality Reference

### Three-Gate Filter Quick Reference

| Gate | Question | Fail if... |
|------|----------|-----------|
| Hard to reverse | Would changing this cost meaningful time? | Easy refactor, local change |
| Surprising without context | Would a reader wonder "why?" without this ADR? | Code is self-explanatory |
| Real trade-off | Were genuine alternatives rejected? | Only one option existed |

### Integration With Other Skills

**Reading from dev-strategy:**
- `implementation-roadmap.md` — Phase list for completion status
- `decision-log.yaml` — Root-level strategy decisions

**Reading from design-contribute contributions:**
- `decision-log.yaml` — Design decisions (code_impacts empty)
- `design-doc.md` — Source material for "Considered Options" section

**Reading from dev-contribute contributions:**
- `decision-log.yaml` — Implementation decisions (code_impacts populated)

**Not reading:**
- `context-handoff.md` files — not needed for ADR harvesting
- `code-context.md` / `context-document.md` — background only; decision-logs are authoritative
