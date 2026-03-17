---
name: design-contribute
description: Enables structured design contributions to dev-strategy implementation plans through interactive, human-in-the-loop sessions. Use when roadmap phases have "Design" objectives, users say "design the [component]", "figure out the [approach]", "determine the [architecture]", or architectural decisions need refinement before implementation. Produces lean design docs through collaborative exploration with ruthless focus on simplicity.
allowed-tools: ["Read", "Write", "Grep", "Glob", "TodoWrite", "Edit", "Bash", "AskUserQuestion"]
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

## When to Use This Skill

✅ **Use when:**
- Roadmap phases have "Design: Determine [X]" objectives
- Users say "design the [component]", "figure out the [approach]", "determine the [architecture]"
- Implementation has revealed constraints that inform design choices
- Architectural decisions need refinement before implementation
- Need to choose between multiple viable approaches interactively

❌ **Don't use when:**
- Creating initial dev-strategy plans (use dev-strategy skill instead)
- Design can be deferred further (wait for more implementation learnings)
- Implementation approach is already clear (just use dev-contribute)
- Designing hypothetical future features (violates YAGNI)

**💡 Tip**: Instead of manually invoking this skill, users can use **dev-continue** skill which automatically determines if design or implementation is needed next.

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

## Process Overview

1. **Read and understand context** - Review dev-strategy plan and implementation learnings
2. **Interactive constraint discovery** - Ask user about learnings, priorities, and concerns
3. **Collaborative option exploration** - Present 2-3 options, discuss trade-offs, iterate based on feedback
4. **Create design contribution folder** - Sequential numbering for chronological order
5. **Document design decision** - Generate lean design doc (< 100 lines) and decision log

## Key Outputs

Every design contribution generates exactly 3 files (schemas in [`contribution-system/references/design-artifacts.md`](../contribution-system/references/design-artifacts.md)):
- **design-doc.md** - Target < 100 lines (combines design spec + implementer handoff)
- **decision-log.md** - Design decisions with rationale
- **context-handoff.md** - Target < 30 lines (problem solved + design overview + reading guide)

**That's it. Just 3 files. No code, no prototypes, no comprehensive specs.**

## Directory Structure

Creates: `contributions/NNN-phase-X-design-[topic]-design-contribute/`
Example: `contributions/005-phase-3-design-session-mgmt-design-contribute/`

## Core Principles

For all core principles (YAGNI, Last Responsible Moment, KISS, Sufficient Design, Human-in-the-Loop), see [`design-principles` skill](../design-principles/SKILL.md).

## Additional Instructions

For the complete 5-step workflow, interactive techniques, validation strategy, and templates, see [reference.md](reference.md).
