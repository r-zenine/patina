# Design Contribute Skill - Reference Guide

This document provides the complete workflow for making design contributions to dev-strategy implementation plans through interactive, human-in-the-loop sessions.

## Table of Contents

- [Overview](#overview)
- [Step 1: Read and Understand Context](#step-1-read-and-understand-context)
- [Step 2: Identify Constraints from Implementation (Interactive Discovery)](#step-2-identify-constraints-from-implementation-interactive-discovery)
- [Step 3: Interactive Option Exploration (Human-in-the-Loop)](#step-3-interactive-option-exploration-human-in-the-loop)
- [Step 4: Create Design Contribution Folder](#step-4-create-design-contribution-folder)
- [Step 5: Document Design Decision](#step-5-document-design-decision)
- [Validation Strategy](#validation-strategy)
- [Interactive Techniques Throughout Workflow](#interactive-techniques-throughout-workflow)
- [Integration with Other Skills](#integration-with-other-skills)
- [Example Flow](#example-flow)
- [Summary](#summary)

## Overview

The design-contribute skill enables collaborative design at the Last Responsible Moment. After implementation has revealed real constraints, this skill helps you work interactively with the user to design the simplest viable approach, then document it for implementers.

**Core Philosophy**: Design is collaborative. Present options, ask questions, gather feedback, iterate.

For principles applied here (YAGNI, LRM, Sufficient Design), see [`design-principles` skill](../design-principles/SKILL.md).

## Step 1: Read and Understand Context

### Goal
Understand what's been built and what needs to be designed.

### Process

1. **Read all dev-strategy artifacts:**
   - `context-document.md` - Behavioral spec and architecture
   - `decision-log.yaml` - Previous decisions made
   - `implementation-roadmap.md` - Planned phases and strategy
   - Look for the specific "Design: Determine [X]" objective

2. **Review existing contributions:**
   - Check `contributions/` directory for implementation work
   - Read recent context-handoff.md files to understand current state
   - Focus on what implementation revealed about constraints
   - Identify the next logical contribution number

3. **Locate the design objective:**
   - Find the specific "Design:" task in implementation-roadmap.md
   - Understand why design was deferred (what needed to be learned first)

For progressive disclosure reading strategy, see [`contribution-system/references/progressive-disclosure.md`](../contribution-system/references/progressive-disclosure.md).

## Step 2: Identify Constraints from Implementation (Interactive Discovery)

### Goal
Work with the user to understand what implementation revealed and what matters most.

**This is the most critical step. Design collaboratively, not in isolation.**

For complete AskUserQuestion patterns and interactive techniques, see [`contribution-system/references/constraint-discovery.md`](../contribution-system/references/constraint-discovery.md).

### Process - Ask First, Then Analyze

1. **Ask about implementation learnings** via AskUserQuestion:
   - "What did you learn during implementation that should inform this design?"
   - "What patterns or approaches worked well so far?"
   - "What integration challenges did you encounter?"

2. **Validate your understanding:**
   - Review the implementation code yourself
   - Confirm: "I see pattern X emerged - is that correct?"

3. **Clarify priorities** via AskUserQuestion:
   - "What's your priority: simplicity, performance, or flexibility?"
   - "What concerns you most about this design decision?"
   - "Are there any non-negotiable requirements?"

4. **Document concrete constraints:**
   - What integration points already exist?
   - What patterns are already established?
   - What technical constraints emerged?
   - What user priorities were revealed?

### What NOT to Do

- ❌ Don't assume constraints without asking
- ❌ Don't design based on hypothetical future needs
- ❌ Don't infer priorities — ask explicitly
- ❌ Don't skip user interaction and go straight to design

## Step 3: Interactive Option Exploration (Human-in-the-Loop)

### Goal
Collaboratively explore design options with the user, presenting simplest first.

**Never design in isolation. Always involve the user in option exploration.**

### Process

1. **Generate 2-3 options maximum:**
   - **Option 1**: Simplest approach (always first) — mark as "(Recommended)"
   - **Option 2**: Alternative with different trade-offs
   - **Option 3**: Only if significantly different approach exists
   - **If considering a 4th**: Stop. You're over-engineering. Simplify instead.

2. **For each option, document:**
   - **Approach**: What is it? (1-2 sentences)
   - **How it works**: Brief explanation
   - **Pros**: What makes it good
   - **Cons**: What are the trade-offs
   - **Complexity**: Low/Medium/High

3. **Present to user via AskUserQuestion** — ask user to choose, include "Other"

4. **Discuss trade-offs** — answer questions, clarify what each option enables/prevents

5. **Iterate if needed** — max 2-3 rounds only

### Option Presentation Format

```markdown
### Option 1: [Simple Approach] (Recommended)
**Approach**: [What is it]
**How it works**: [Brief explanation]
**Pros**: [Benefits]
**Cons**: [Limitations]
**Complexity**: Low
**Why recommended**: [Simplest approach that solves the current need]

### Option 2: [Alternative Approach]
**Approach**: [What is it]
**Pros**: [Benefits]
**Cons**: More complex; [other trade-offs]
**Complexity**: Medium
```

### What NOT to Do

- ❌ Don't design in isolation then present final design
- ❌ Don't present more than 3 options
- ❌ Don't make the decision for the user
- ❌ Don't skip getting explicit user choice
- ❌ Don't recommend complex options over simple ones

## Step 4: Create Design Contribution Folder

### Goal
Create sequentially numbered folder for this design contribution.

### Process

1. **Check existing contributions:**
   ```bash
   ls contributions/
   ```

2. **Pick next number and create folder:**
   ```bash
   mkdir contributions/005-phase-3-design-session-mgmt-design-contribute/
   ```

3. **Folder naming convention:**
   `NNN-phase-X-design-[topic]-design-contribute`

For folder naming details and topic examples, see [`contribution-system` skill](../contribution-system/SKILL.md).

## Step 5: Document Design Decision

### Goal
Create lean design documentation that enables implementation.

For full artifact schemas, see [`contribution-system/references/design-artifacts.md`](../contribution-system/references/design-artifacts.md).
For templates, see [`contribution-system/assets/templates/`](../contribution-system/assets/templates/).

### Files to Create

**1. design-doc.md** (< 100 lines target)
- Use [design-doc-template.md](../contribution-system/assets/templates/design-doc-template.md)
- Document the chosen approach
- Explain why (constraints + user priorities)
- Describe how it works
- Specify what we're NOT doing
- Provide implementation guidance

**2. decision-log.md**
- Use [decision-log-design-template.md](../contribution-system/assets/templates/decision-log-design-template.md)
- Document primary design decision with rationale
- Include supporting decisions made during design
- List rejected alternatives

**3. context-handoff.md** (< 30 lines target)
- Document 3 things only:
  1. What problem are we solving with this design (5–10 lines)
  2. High-level overview of the design-doc (15 lines)
  3. Reading guide for design-doc.md (5 lines)

### Quality Checks

**Implementer Readiness:**
- ✅ Can someone implement from design-doc.md alone?
- ✅ Are integration points clearly specified?
- ✅ Are success criteria defined?

**Simplicity Enforcement:**
- ✅ Is design-doc.md under 100 lines?
- ✅ Did we choose the simplest option?
- ✅ Is "Simplicity Rationale" section clear?
- ✅ Did we avoid designing for hypothetical futures?

**Scope Boundaries:**
- ✅ Does design only address current phase objective?
- ✅ Did we document what we're NOT doing?
- ✅ Is YAGNI principle applied?

**Interactive Design Quality:**
- ✅ Did we ask user about implementation learnings?
- ✅ Did we present options and get user choice?
- ✅ Did we validate assumptions?

### What NOT to Include

- ❌ No code files (documentation only)
- ❌ No comprehensive specifications
- ❌ No prototypes or proof-of-concepts
- ❌ No design for future phases
- ❌ No "nice to have" features

### Commit the design artifacts

After the design document is created and all quality checks pass:
```bash
git add <contribution-folder>/
git commit -m "design(NNN): <description>"
```

## Validation Strategy

For complete anti-patterns (over-engineering and under-engineering signals), see [`design-principles/references/anti-patterns.md`](../design-principles/references/anti-patterns.md).

### Key Enforcement Rules

**Always present simplest option first** — Option 1 must be simplest; mark as "Recommended" with rationale.

**Maximum 3 options** — If considering 4th, you're over-engineering; stop and simplify.

**design-doc.md < 100 lines** — If exceeding 100 lines, focus on "what" and "why" not exhaustive "how".

**Only current phase objective** — Design addresses specific "Design:" task from roadmap only.

## Interactive Techniques Throughout Workflow

For the complete AskUserQuestion pattern reference, see [`contribution-system/references/constraint-discovery.md`](../contribution-system/references/constraint-discovery.md).

**Quick reference — when to use AskUserQuestion:**

**Step 2 — Constraint Discovery:**
- "What did you learn during implementation that should inform this design?"
- "What's your priority: simplicity, performance, or flexibility?"

**Step 3 — Option Exploration:**
- Present 2-3 options and ask user to choose
- "Any concerns about the recommended approach?"

**Step 5 — Validation:**
- "Does this design doc clearly capture what you need?"

## Integration with Other Skills

### Reading from dev-strategy
- `implementation-roadmap.md` — Find "Design:" objectives
- `context-document.md` — Understand behavioral spec
- `decision-log.yaml` — Previous decisions and constraints

### Handing off to dev-contribute
- design-doc.md becomes the specification for the next implementation contribution
- dev-contribute reads design-doc.md and implements according to the guidance
- Reference design contribution number in implementation context-handoff.md

### Progressive disclosure for readers
- Implementation agents: Start with design-doc.md (replaces context-handoff.md for design contributions)
- Review agents: Focus on "Why This Design" section and decision-log.md

## Example Flow

```
Roadmap Phase 3: "Design: Determine session management approach"

[Existing contributions:]
001-phase-1-implementation-basic-auth/
002-phase-2-implementation-user-storage/
003-phase-3-implementation-jwt-auth/

[design-contribute invoked:]
Step 1: Read context and contributions 001-003
Step 2: Ask user about JWT implementation learnings
        User: "Need stateless approach, short tokens"
Step 3: Generate 2 options: JWT-only (recommended) vs JWT+Redis
        User chooses JWT-only (simplest)
Step 4: Create: 004-phase-3-design-session-mgmt-design-contribute/
Step 5: Document design-doc.md, decision-log.md, context-handoff.md

[Next dev-contribute:]
005-phase-3-implementation-session-validation/
↓ Reads design-doc.md from contribution 004
```

## Summary

design-contribute is about **collaborative design at the Last Responsible Moment**:

1. **Wait until implementation reveals constraints** (dev-strategy deferred it at Gate 1)
2. **Work interactively with user** to understand learnings and priorities
3. **Present simple options** (max 3, simplest first)
4. **Get user choice** through dialogue, not inference
5. **Document lean design** (< 100 lines) for implementers
