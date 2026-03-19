# Design Contribute Skill - Reference Guide

This document provides the complete workflow for making design contributions to dev-strategy implementation plans through interactive, human-in-the-loop sessions. It's organized by outcome phases that define what success looks like at each stage.

## Overview

The design-contribute skill enables collaborative design at the Last Responsible Moment. After implementation has revealed real constraints, this skill helps you work interactively with the user to design the simplest viable approach, then document it for implementers.

The process is organized into **three outcome phases**. Each phase has specific success criteria that must be verified before proceeding to the next phase.

**Core Philosophy:** Design is collaborative. Present options, ask questions, gather feedback, iterate.

For principles applied here (YAGNI, LRM, Sufficient Design), see [`design-principles` skill](../design-principles/SKILL.md).

---

## OUTCOME 1: Context is Clear

**What this outcome means:**
You understand what's been built so far, what constraints have emerged from implementation, what specifically needs to be designed, and why design was deferred at planning time. You have the context needed to make informed design decisions.

### What You Must Verify

Before proceeding to exploration, confirm all of these:

- [ ] Dev-strategy plan exists at `.plans/plan-[FEATURE-NAME]/`?
- [ ] Can you articulate the current behavioral spec and architecture without looking at artifacts?
- [ ] Can you identify which roadmap phase has the "Design:" objective?
- [ ] Do you understand what implementation has revealed so far?
- [ ] Can you explain why design was deferred to this point?

### How to Achieve This Outcome

#### Verify Dev-Strategy Plan Exists

**Goal:** Ensure this skill can proceed (it only works with dev-strategy plans).

**STOP if any of these are true:**
- No `.plans/plan-[FEATURE-NAME]/` folder exists
- The plan was not created by the `dev-strategy` skill
- No `implementation-roadmap.md` exists in the plan folder

If the plan doesn't exist, tell the user to run the `dev-strategy` skill first. This skill ONLY works with plans created by `dev-strategy`.

#### Read and Understand Context

**Goal:** Understand what's been built and what needs to be designed.

**Process:**

1. **Locate the dev-strategy plan** in `.plans/plan-[FEATURE-NAME]/`

2. **Read all dev-strategy artifacts** (located in `.plans/plan-[FEATURE-NAME]/`):
   - `context-document.md` - Behavioral spec and architecture
   - `decision-log.yaml` - Previous decisions made during planning
   - `implementation-roadmap.md` - Planned phases and strategy
   - Look for the specific "Design: Determine [X]" objective

3. **Review existing contributions:**
   - Check `.plans/plan-[FEATURE-NAME]/contributions/` directory for implementation work
   - Read recent `context-handoff.md` files to understand current state
   - **Focus on what implementation revealed about constraints**
   - Identify the next logical contribution number

4. **Locate the design objective:**
   - Find the specific "Design:" task in `implementation-roadmap.md`
   - Understand why design was deferred (what needed to be learned first)

For progressive disclosure reading strategy, see [`contribution-system/references/progressive-disclosure.md`](../contribution-system/references/progressive-disclosure.md).

---

## OUTCOME 2: Design is Validated with User

**What this outcome means:**
You've worked interactively with the user to understand what implementation has taught you, what matters most to them, and you've presented design options. The user has explicitly chosen a direction, and you've validated that choice addresses the design objective.

### What You Must Verify

Before proceeding to documentation, confirm all of these:

- [ ] Did you ask the user about implementation learnings (not assumed)?
- [ ] Did you validate your understanding of constraints and patterns?
- [ ] Did you ask about user priorities (not inferred)?
- [ ] Did you generate 2-3 options (simplest first)?
- [ ] Did you present options to user via AskUserQuestion?
- [ ] Did the user explicitly choose one option?
- [ ] Does the chosen design address the "Design:" objective from the roadmap?

### How to Achieve This Outcome

#### Identify Constraints from Implementation (Interactive Discovery)

**Goal:** Work with the user to understand what implementation revealed and what matters most.

**This is the most critical step. Design collaboratively, not in isolation.**

For complete AskUserQuestion patterns and interactive techniques, see [`contribution-system/references/constraint-discovery.md`](../contribution-system/references/constraint-discovery.md).

**Process — Ask First, Then Analyze:**

1. **Ask about implementation learnings** via AskUserQuestion:
   - "What did you learn during implementation that should inform this design?"
   - "What patterns or approaches worked well so far?"
   - "What integration challenges did you encounter?"

2. **Validate your understanding:**
   - Review the implementation code yourself
   - Confirm: "I see pattern X emerged — is that correct?"

3. **Clarify priorities** via AskUserQuestion:
   - "What's your priority: simplicity, performance, or flexibility?"
   - "What concerns you most about this design decision?"
   - "Are there any non-negotiable requirements?"

4. **Document concrete constraints:**
   - What integration points already exist?
   - What patterns are already established?
   - What technical constraints emerged?
   - What user priorities were revealed?

**What NOT to Do:**
- ❌ Don't assume constraints without asking
- ❌ Don't design based on hypothetical future needs
- ❌ Don't infer priorities — ask explicitly
- ❌ Don't skip user interaction and go straight to design

#### Interactive Option Exploration (Human-in-the-Loop)

**Goal:** Collaboratively explore design options with the user, presenting simplest first.

**Never design in isolation. Always involve the user in option exploration.**

**Process:**

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

**Option Presentation Format:**

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

**What NOT to Do:**
- ❌ Don't design in isolation then present final design
- ❌ Don't present more than 3 options
- ❌ Don't make the decision for the user
- ❌ Don't skip getting explicit user choice
- ❌ Don't recommend complex options over simple ones

---

## OUTCOME 3: Design is Durable and Transferable

**What this outcome means:**
Design documentation is complete, decisions are recorded with reasoning, context is handed off clearly, and the design is ready for implementers to use immediately without ambiguity.

### What You Must Verify

Before finalizing design contribution, confirm all of these:

- [ ] Is design-doc.md under 100 lines?
- [ ] Can an implementer build from design-doc.md alone without asking questions?
- [ ] Are integration points clearly specified?
- [ ] Are success criteria defined?
- [ ] Does decision-log.yaml document the primary design decision with rationale?
- [ ] Does context-handoff.md explain the problem, design overview, and reading guide (< 30 lines)?
- [ ] Are assumptions and limitations clearly documented?
- [ ] Does the design only address the current phase objective (YAGNI applied)?

### How to Achieve This Outcome

#### Create Design Contribution Folder

**Goal:** Create sequentially numbered folder for this design contribution under the plan's contributions directory.

**IMPORTANT: Contributions are ALWAYS saved in `.plans/plan-[FEATURE-NAME]/contributions/`**

All contributions for a dev-strategy plan are stored together in the plan's dedicated contributions directory. This ensures contributions remain organized and linked to their parent plan.

**Process:**

1. **Navigate to the plan's contributions directory:**
   ```bash
   cd .plans/plan-[FEATURE-NAME]/contributions/
   ls
   ```

2. **Pick next number and create folder:**
   ```bash
   mkdir 005-phase-3-design-session-mgmt-design-contribute/
   ```

3. **Folder naming convention:**
   `NNN-phase-X-design-[topic]-design-contribute`

For folder naming details and topic examples, see [`contribution-system` skill](../contribution-system/SKILL.md).

#### Document Design Decision

**Goal:** Create lean design documentation that enables implementation.

For full artifact schemas, see [`contribution-system/references/design-artifacts.md`](../contribution-system/references/design-artifacts.md).
For templates, see [`contribution-system/assets/templates/`](../contribution-system/assets/templates/).

**Create exactly three files:**

**1. design-doc.md** (< 100 lines target)

Use [design-doc-template.md](../contribution-system/assets/templates/design-doc-template.md)

Document the chosen approach:
- Explain why this design (constraints + user priorities)
- Describe how it works
- Specify what we're NOT doing (scope boundaries)
- Provide implementation guidance

**2. decision-log.yaml**

Use [decision-log-template.yaml](../contribution-system/assets/templates/decision-log-template.yaml)

Document primary design decision with rationale:
- What was decided
- Why this choice over alternatives
- Include supporting decisions made during design
- List rejected alternatives

**Schema:**
```yaml
commit: null  # Design decisions don't have code yet

decisions:
  - number: 1
    title: "[One sentence summary]"
    rationale: "[Why...]"
    code_impacts: []  # Empty for design phase; filled in by implementation
```

**3. context-handoff.md** (< 30 lines target)

Document three things only:
1. **What problem are we solving with this design** (5–10 lines)
2. **High-level overview of the design-doc** (15 lines)
3. **Reading guide for design-doc.md** (5 lines)

**Quality Checks:**

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

**What NOT to Include:**
- ❌ No code files (documentation only)
- ❌ No comprehensive specifications
- ❌ No prototypes or proof-of-concepts
- ❌ No design for future phases
- ❌ No "nice to have" features

#### Commit the Design Contribution

**Goal:** Persist all design artifacts to git with clear commit message.

After all quality checks pass and files are created:

```bash
git add .plans/plan-[FEATURE-NAME]/contributions/<contribution-folder>/
git commit -m "design(NNN): <description matching contribution folder name>"
```

**Rules:**
- Do NOT use `git add -A` or `git add .`
- Use the full path `.plans/plan-[FEATURE-NAME]/contributions/<contribution-folder>/` when staging
- The commit message number (NNN) must match the contribution folder number
- The description must match the contribution folder name (e.g., `phase-3-design-session-mgmt-design-contribute`)

---

## Reference: Key Enforcement Rules

For complete anti-patterns (over-engineering and under-engineering signals), see [`design-principles/references/anti-patterns.md`](../design-principles/references/anti-patterns.md).

**Always present simplest option first** — Option 1 must be simplest; mark as "Recommended" with rationale.

**Maximum 3 options** — If considering 4th, you're over-engineering; stop and simplify.

**design-doc.md < 100 lines** — If exceeding 100 lines, focus on "what" and "why" not exhaustive "how".

**Only current phase objective** — Design addresses specific "Design:" task from roadmap only.

---

## Reference: Interactive Techniques Throughout Workflow

For the complete AskUserQuestion pattern reference, see [`contribution-system/references/constraint-discovery.md`](../contribution-system/references/constraint-discovery.md).

**When to use AskUserQuestion:**

**OUTCOME 1 → OUTCOME 2 Transition (Constraint Discovery):**
- "What did you learn during implementation that should inform this design?"
- "What's your priority: simplicity, performance, or flexibility?"

**OUTCOME 2 (Option Exploration):**
- Present 2-3 options and ask user to choose
- "Any concerns about the recommended approach?"

**OUTCOME 3 (Validation):**
- "Does this design doc clearly capture what you need?"

---

## Reference: Integration with Other Skills

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
- Review agents: Focus on "Why This Design" section and decision-log.yaml

---

## Reference: Example Flow

```
Roadmap Phase 3: "Design: Determine session management approach"

[Existing contributions:]
001-phase-1-implementation-basic-auth/
002-phase-2-implementation-user-storage/
003-phase-3-implementation-jwt-auth/

[design-contribute invoked:]
Step 1 (UNDERSTANDING): Read context and contributions 001-003
Step 2 (EXPLORATION): Ask user about JWT implementation learnings
                      User: "Need stateless approach, short tokens"
Step 3 (EXPLORATION): Generate 2 options: JWT-only (recommended) vs JWT+Redis
                      User chooses JWT-only (simplest)
Step 4 (DOCUMENTATION): Create: 004-phase-3-design-session-mgmt-design-contribute/
Step 5 (DOCUMENTATION): Document design-doc.md, decision-log.yaml, context-handoff.md

[Next dev-contribute:]
005-phase-3-implementation-session-validation/
↓ Reads design-doc.md from contribution 004
```
