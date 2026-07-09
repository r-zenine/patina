---
name: design-contribute
description: Enables collaborative design contributions to dev-strategy implementation plans through human-in-the-loop sessions. Use when roadmap phases have "Design" objectives, you need to "design the [component]", "figure out the [approach]", "determine the [architecture]", or refinement before implementation. Works with `.plans/plan-[FEATURE-NAME]/` directories only. Produces lean design docs (< 100 lines) through interactive exploration with ruthless focus on simplicity.
skills-depend-on:
  - contribution-system
  - design-principles
---

# Design Contribute Skill

Make structured design contributions to dev-strategy implementation plans through interactive, collaborative design sessions with ruthless focus on simplicity.

## What This Skill Does

Enables design at the Last Responsible Moment through human collaboration:
- Reading existing plan and implementation learnings
- Interactively exploring design options with the user
- Creating lean design documentation (< 100 lines target)
- Documenting design decisions for implementers
- Maintaining project continuity between design and implementation

**Works with**: Plans created by `dev-strategy` skill in `.plans/plan-[FEATURE-NAME]/`

## Context & Philosophy

### Two-Gate System Context

This skill operates at **Gate 2** of the Two-Gate System Against Over-Engineering. Design was deferred during planning (Gate 1) to gather implementation constraints. Now design with those learned constraints.

See [`contribution-system/references/two-gate-system.md`](../contribution-system/references/two-gate-system.md) for full explanation.

### Human-in-the-Loop Design Philosophy

Design is inherently collaborative and iterative. Use AskUserQuestion to:
- Understand priorities and validate assumptions
- Present 2-3 options with clear trade-offs (simplest first)
- Get explicit user choice through dialogue, not inference
- Iterate based on user feedback (max 2-3 rounds)

See [`contribution-system/references/constraint-discovery.md`](../contribution-system/references/constraint-discovery.md) for interactive techniques and patterns.

### Token Efficiency in Dialogue

Design sessions are conversation-heavy, and every reply is re-read on each subsequent turn:

- **Interactive replies: lead with the outcome, one level of detail.** No process narration, no teaching material — the user asks if they want more. When presenting options, state each in a few lines with its trade-off; save depth for the option the user picks.
- **Keep decisions, trade-offs, and failures in the reply.** Omitting one triggers a follow-up round-trip over the full context, costlier than the sentence that preempts it.
- **Artifacts are the opposite case.** design-doc.md and context-handoff.md are read by fresh-context implementers that can't ask follow-ups — the < 100 / < 30 line targets mean complete-but-lean, not minimal.

## Execution Overview

To create a robust design contribution, three outcomes must be achieved:

**Outcome 1 (UNDERSTANDING)**: Context is clear
- Current plan state is understood (roadmap, prior decisions, constraints visible)
- Implementation learnings are captured
- Design objective is clear and scoped
- Assumptions about design are validated

**Outcome 2 (EXPLORATION)**: Design is validated with user
- Options are presented (2-3 approaches, simplest first)
- Trade-offs are explicit
- User validates choice explicitly
- Design follows simplicity principles

**Outcome 3 (DOCUMENTATION)**: Design is durable and transferable
- Design doc exists (< 100 lines target)
- Decisions are recorded
- Context is handed off (< 30 lines)
- Contribution folder exists with sequential numbering

These criteria are owned here; references restate them for execution convenience only. For the generic outcome pattern shared by all skills, see [`contribution-system/references/execution-template.md`](../contribution-system/references/execution-template.md).

## Key Outputs

Every design contribution generates exactly 3 files:
- **design-doc.md** - Target < 100 lines (combines design spec + implementer handoff)
- **decision-log.yaml** - Design decisions in unified YAML schema
- **context-handoff.md** - Target < 30 lines (what was solved + reading guide)

**That's it. Just 3 files. No code, no prototypes, no comprehensive specs.**

For artifact schemas, see [`contribution-system/references/design-artifacts.md`](../contribution-system/references/design-artifacts.md).

## How to Use This Skill

For tactics available for each outcome, see [references/tactics.md](references/tactics.md).

## Core Principles

For all core principles (YAGNI, Last Responsible Moment, KISS, Sufficient Design), see [`design-principles` skill](../design-principles/SKILL.md).

## Prerequisites

Before using this skill, read [`contribution-system` skill](../contribution-system/SKILL.md) to understand:
- Design contribution folder naming (NNN-phase-X-design-[topic]-design-contribute)
- Design artifact schemas (design-doc.md, decision-log.yaml, context-handoff.md)
- Unified decision-log YAML schema (with empty code_impacts for design)
- Directory structure (contributions/ folder is required)
- Progressive disclosure strategy for reading design contributions

All design contributions created by this skill must conform to contribution-system conventions.
