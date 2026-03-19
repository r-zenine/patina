---
name: design-contribute
description: Enables collaborative design contributions to dev-strategy implementation plans through human-in-the-loop sessions. Use when roadmap phases have "Design" objectives, you need to "design the [component]", "figure out the [approach]", "determine the [architecture]", or refinement before implementation. Works with `.plans/plan-[FEATURE-NAME]/` directories only. Produces lean design docs (< 100 lines) through interactive exploration with ruthless focus on simplicity.
---

# Design Contribute Skill

Make structured design contributions to dev-strategy implementation plans through interactive, collaborative design sessions with ruthless focus on simplicity.

## Table of Contents

- [What This Skill Does](#what-this-skill-does)
- [Prerequisite: Understand contribution-system](#-prerequisite-understand-contribution-system)
- [When to Use This Skill](#when-to-use-this-skill)
- [Two-Gate Context](#two-gate-context)
- [Human-in-the-Loop Design Philosophy](#human-in-the-loop-design-philosophy)
- [Execution Requirements](#execution-requirements)
- [Key Outputs](#key-outputs)
- [Directory Structure](#directory-structure)
- [Core Principles](#core-principles)
- [Additional Instructions](#additional-instructions)

## What This Skill Does

Enables design at the Last Responsible Moment through human collaboration:
- Reading existing plan and implementation learnings
- Interactively exploring design options with the user
- Creating lean design documentation (< 100 lines target)
- Documenting design decisions for implementers
- Maintaining project continuity between design and implementation

## Prerequisites

Before using this skill, read [`contribution-system` skill](../contribution-system/SKILL.md) to understand:
- Design contribution folder naming (NNN-phase-X-design-[topic]-design-contribute)
- Design artifact schemas (design-doc.md, decision-log.yaml, context-handoff.md)
- Unified decision-log YAML schema (with empty code_impacts for design)
- Directory structure (contributions/ folder is required)
- Why design decisions don't have code_impacts (code comes in implementation phase)
- Progressive disclosure strategy for reading design contributions

All design contributions created by this skill must conform to contribution-system conventions.

**Note**: This skill works only with plans created by `dev-strategy` skill in `.plans/plan-[FEATURE-NAME]/`.

## Two-Gate Context

This skill operates at **Gate 2** of the Two-Gate System Against Over-Engineering. Design was deferred during planning (Gate 1) to gather implementation constraints. Now design with those learned constraints.

For the full Two-Gate System explanation, see [`contribution-system/references/two-gate-system.md`](../contribution-system/references/two-gate-system.md).

## Human-in-the-Loop Design Philosophy

Design is inherently collaborative and iterative. Use AskUserQuestion to:
- Understand priorities and validate assumptions
- Present 2-3 options with clear trade-offs (simplest first)
- Get explicit user choice through dialogue, not inference
- Iterate based on user feedback (max 2-3 rounds)

For interactive techniques and AskUserQuestion patterns, see [`contribution-system/references/constraint-discovery.md`](../contribution-system/references/constraint-discovery.md).

## Execution Requirements

To create a robust design contribution, ensure these outcomes are achieved:

**UNDERSTANDING PHASE** (outcome: context is clear)
- Current plan state is understood (roadmap, prior decisions, constraints visible)
- Implementation learnings are captured (what did prior phases reveal?)
- Design objective is clear and scoped (what specifically needs designing?)
- Assumptions about design are validated (don't assume, ask)

**EXPLORATION PHASE** (outcome: design is validated with user)
- Options are presented (2-3 approaches, simplest first)
- Trade-offs are explicit (why each option exists, what you give up)
- User validates choice explicitly (confirmation of preference, not inference)
- Design decision is sufficiently detailed for implementation handoff
- Design follows simplicity principles (YAGNI, Sufficient Design, KISS)

**DOCUMENTATION PHASE** (outcome: design is durable and transferable)
- Design doc exists (< 100 lines target, clear enough for implementers)
- Decisions are recorded (decision-log.yaml populated with reasoning)
- Context is handed off (context-handoff.md < 30 lines, tells next phase what was solved)
- Contribution folder exists with sequential numbering (NNN-phase-X-design-[topic]-design-contribute)

**How to achieve these outcomes:**
1. Read and understand context (review dev-strategy plan and prior implementation learnings)
2. Conduct interactive constraint discovery (ask user about learnings, priorities, and concerns)
3. Explore design options collaboratively (present options, discuss trade-offs, iterate based on feedback)
4. Create design contribution folder (use sequential numbering for chronological order)
5. Document design decision (generate lean design doc and decision log)
6. Commit the contribution (stage explicitly and commit with message matching folder name)

## Key Outputs

Every design contribution generates exactly 3 files (schemas in [`contribution-system/references/design-artifacts.md`](../contribution-system/references/design-artifacts.md)):
- **design-doc.md** - Target < 100 lines (combines design spec + implementer handoff)
- **decision-log.yaml** - Design decisions in unified YAML schema (matching `diffviz-review::Decision` struct)
- **context-handoff.md** - Target < 30 lines (problem solved + design overview + reading guide)

**That's it. Just 3 files. No code, no prototypes, no comprehensive specs.**

## Directory Structure

Creates: `.plans/plan-[FEATURE-NAME]/contributions/NNN-phase-X-design-[topic]-design-contribute/`
Example: `.plans/plan-myfeature/contributions/005-phase-3-design-session-mgmt-design-contribute/`

## Core Principles

For all core principles (YAGNI, Last Responsible Moment, KISS, Sufficient Design, Human-in-the-Loop), see [`design-principles` skill](../design-principles/SKILL.md).

## Additional Instructions

For the complete 5-step workflow, interactive techniques, validation strategy, and templates, see [references/guide.md](references/guide.md).
