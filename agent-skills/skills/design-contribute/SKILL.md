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

## Two-Gate System Against Over-Engineering

**Gate 1 - dev-strategy**: Only defer design when implementation will reveal better constraints
- Don't defer if design is straightforward
- Defer when implementation learning will inform better choices

**Gate 2 - design-contribute (this skill)**: Design simplest viable approach with current learned constraints
- No hypothetical features
- No "what if" scenarios
- Only what current phase requires
- Always present simplest option first

## Human-in-the-Loop Design Philosophy

**Design is inherently collaborative and iterative:**
- User has context about business needs, team capabilities, future direction
- Design decisions benefit from dialogue, not just documentation
- Interactive exploration reveals constraints and preferences
- Quick feedback loops prevent wrong directions
- Present options → gather feedback → refine → validate

**This skill emphasizes:**
- Using AskUserQuestion to understand priorities and validate assumptions
- Presenting 2-3 options with clear trade-offs (simplest first)
- Iterative refinement based on user input
- Collaborative exploration of constraints
- Getting user choice through dialogue, not inference

## Process Overview

1. **Read and understand context** - Review dev-strategy plan and implementation learnings
2. **Interactive constraint discovery** - Ask user about learnings, priorities, and concerns
3. **Collaborative option exploration** - Present 2-3 options, discuss trade-offs, iterate based on feedback
4. **Create design contribution folder** - Sequential numbering for chronological order
5. **Document design decision** - Generate lean design doc (< 100 lines) and decision log

## Key Outputs

Every design contribution generates:
- **design-doc.md** - Target < 100 lines (combines design spec + implementer handoff)
- **decision-log.md** - Design decisions with rationale
- **context-handoff.md** - Target < 30 lines ( what problem did we solve ( 5 lines ), high level overview of the strategy ( 15 lines ), a short reading guide for `design-doc.md` 

**That's it. Just 3 files. No code, no prototypes, no comprehensive specs.**

## Directory Structure

Creates: `contributions/NNN-phase-X-design-[topic]-design-contribute/`
Example: `contributions/005-phase-3-design-session-mgmt-design-contribute/`

## Core Principles

**YAGNI (You Aren't Gonna Need It) - Primary Principle**
- Design only what the current phase explicitly requires
- Don't design for hypothetical future needs
- If it's not needed now, don't design it
- Choose the simplest design that solves the immediate problem

**Last Responsible Moment**
- You've deferred design to learn from implementation
- Now design with the constraints you've learned
- Don't defer further - you have the information you need

**KISS (Keep It Simple, Stupid)**
- Always present simplest option first with clear rationale
- Max 3 design options (if needing 4th → simplify instead)
- Choose the simplest design that works
- Avoid clever architectures or over-engineering

**Sufficient Design**
- Design enough to enable implementation, not comprehensive documentation
- Focus on "what to build" and "how it integrates"
- Target < 100 lines for design-doc.md
- Document decisions, don't over-explain

**Human-in-the-Loop**
- Design is collaborative - present options, gather feedback, iterate
- Use AskUserQuestion to understand priorities and validate assumptions
- Interactive exploration reveals constraints and preferences
- Quick feedback loops prevent wrong directions

**Interactive Exploration**
- Ask questions first, then generate options
- Present options with clear trade-offs
- Discuss which trade-offs matter most
- Refine based on user feedback
- Validate assumptions throughout

## Additional Instructions

For the complete 5-step workflow, interactive techniques, validation strategy, and templates, see [reference.md](reference.md).
