---
name: adr-contribute
description: Wraps up a dev-strategy plan by harvesting decisions from all contributions, proposing ADR candidates through a human-in-the-loop selection, and writing lean Architecture Decision Records to docs/adr/. Use when a plan is done — whether fully implemented or deliberately stopped early. Works with .plans/plan-[FEATURE-NAME]/ directories only.
skills-depend-on:
  - contribution-system
  - design-principles
---

# ADR Contribute Skill

Wrap up a dev-strategy plan by surfacing the decisions that deserve to be remembered, getting human confirmation on which ones to record, and writing lean ADRs.

## What This Skill Does

Reads every `decision-log.yaml` across a plan's contributions (design and implementation) and applies a three-gate filter to identify which decisions are ADR-worthy:

1. **Hard to reverse** — changing your mind later would be costly
2. **Surprising without context** — a future reader would wonder "why did they do it this way?"
3. **Real trade-off** — genuine alternatives existed; one was chosen for specific reasons

The skill proposes candidates but the human selects which ones get written. It also captures plan completion status — which phases were implemented, which were intentionally skipped and why.

**Works with**: Plans created by `dev-strategy` skill in `.plans/plan-[FEATURE-NAME]/`

## Context & Philosophy

### This Skill Signals Plan Completion

Invoking this skill means the plan is done. Not necessarily fully implemented — a deliberate early stop is a valid outcome. The skill handles partial completion gracefully: it captures what was built, what was skipped, and why.

### Offer ADRs Sparingly

The three-gate filter is the only guard against ADR inflation. A decision that fails any gate should not become an ADR, even if it felt significant during implementation. The value of an ADR library degrades fast when it fills with obvious choices.

See the filter applied in detail in [references/tactics.md](references/tactics.md).

### Human-in-the-Loop Selection

The skill proposes; the human decides. This matters because the three-gate filter is heuristic, not mechanical — the human has context about what will genuinely surprise future readers that no automated analysis can match.

## Execution Overview

To wrap a plan with durable ADRs, three outcomes must be achieved:

**Outcome 1 (HARVEST)**: Candidate list is ready for human review
- All decision-log.yaml files have been read (root plan + all contributions)
- Each decision has been assessed against the three gates
- A ranked candidate list has been presented to the user with one-line justifications

**Outcome 2 (SELECTION)**: Human has confirmed what to write
- User has confirmed which phases were completed vs. intentionally skipped
- User has selected which ADR candidates to write (numbers, "all", or "none")
- Skipped-phase rationale captured (if any phases were left out)

**Outcome 3 (DOCUMENTATION)**: Plan is wrapped and committed
- Selected ADRs written to `docs/adr/` with correct sequential numbering
- Wrap-up contribution folder created with `wrap-summary.md`
- Everything committed to git

## Key Outputs

**ADR files** (one per selected candidate):
- Location: `docs/adr/NNNN-slug.md`
- Format: lean — title + 1–3 sentence summary; optional sections only when they add genuine value
- Numbering: scan existing `docs/adr/` and increment from highest

For the ADR format, see [assets/adr-template.md](assets/adr-template.md).

**Wrap-up contribution folder** (always created, even when no ADRs are written):
- Location: `.plans/plan-[FEATURE-NAME]/contributions/NNN-plan-wrap-adr-contribute/`
- Contains: `wrap-summary.md` — phase completion status + index of ADRs written

## How to Use This Skill

For tactics for each outcome, see [references/tactics.md](references/tactics.md).

## Core Principles

For design philosophy (YAGNI, Sufficient Documentation, Last Responsible Moment), see [`design-principles` skill](../design-principles/SKILL.md).

## Prerequisites

Before using this skill, read [`contribution-system` skill](../contribution-system/SKILL.md) to understand:
- Folder naming convention (NNN-phase-X-[type]-[specialty]-[agent])
- Unified decision-log YAML schema
- Directory structure (contributions/ folder required)
- Progressive disclosure reading strategy

**STOP if**: No `.plans/plan-[FEATURE-NAME]/` folder exists or no `contributions/` directory exists. This skill requires at least one completed contribution to have anything to harvest.
