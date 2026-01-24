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

## Step 1: Read and Understand Context

### Goal
Understand what's been built and what needs to be designed.

### Process

1. **Read all dev-strategy artifacts:**
   - `context-document.md` - Behavioral spec and architecture
   - `decision-log.md` - Previous decisions made
   - `implementation-roadmap.md` - Planned phases and strategy
   - Look for the specific "Design: Determine [X]" objective

2. **Review existing contributions:**
   - Check `contributions/` directory for implementation work
   - Read recent context-handoff.md files to understand current state
   - Focus on what implementation revealed about constraints
   - Identify the next logical contribution number

3. **Locate the design objective:**
   - Find the specific "Design:" task in implementation-roadmap.md
   - Example: "Phase 3: Design: Determine session management approach"
   - Understand why design was deferred (what needed to be learned first)

### What to Look For

- What has been implemented so far?
- What patterns have emerged from implementation?
- What integration points are already established?
- What did implementation reveal about the problem?

## Step 2: Identify Constraints from Implementation (Interactive Discovery)

### Goal
Work with the user to understand what implementation revealed and what matters most.

### Process - Ask First, Then Analyze

**This is the most critical step. Design collaboratively, not in isolation.**

1. **Ask about implementation learnings:**
   ```
   Use AskUserQuestion to ask:
   - "What did you learn during implementation that should inform this design?"
   - "What patterns or approaches worked well so far?"
   - "What integration challenges did you encounter?"
   ```

2. **Validate your understanding:**
   - Review the implementation code yourself
   - Compare what you found with what user said
   - Use AskUserQuestion to confirm: "I see pattern X emerged - is that correct?"

3. **Clarify priorities:**
   ```
   Use AskUserQuestion to ask:
   - "What's your priority: simplicity, performance, or flexibility?"
   - "What concerns you most about this design decision?"
   - "Are there any non-negotiable requirements?"
   ```

4. **Document concrete constraints:**
   - What integration points already exist?
   - What patterns are already established?
   - What technical constraints emerged? (performance, security, etc.)
   - What user priorities were revealed?

### What NOT to Do

- ❌ Don't assume constraints without asking
- ❌ Don't design based on hypothetical future needs
- ❌ Don't infer priorities - ask explicitly
- ❌ Don't skip user interaction and go straight to design

### Example Interactive Discovery

```
Agent: "I see the roadmap says 'Design: Determine session management approach'.
        What did you learn during the authentication implementation that should
        inform how we handle sessions?"

User: "We built JWT-based auth, and I want sessions to be stateless to avoid
       database lookups on every request."

Agent: "Got it. So the constraint is: session approach must work with stateless
        JWT tokens. What's your priority: simplicity of implementation or
        flexibility for future features?"

User: "Simplicity. Just need basic session validation."
```

## Step 3: Interactive Option Exploration (Human-in-the-Loop)

### Goal
Collaboratively explore design options with the user, presenting simplest first.

### Process - Always Interactive

**Never design in isolation. Always involve the user in option exploration.**

1. **Generate 2-3 options maximum:**
   - **Option 1**: Simplest approach (always first)
   - **Option 2**: Alternative with different trade-offs
   - **Option 3**: Only if significantly different approach exists
   - **If considering a 4th**: Stop. You're over-engineering. Simplify instead.

2. **For each option, document:**
   - **Approach**: What is it? (1-2 sentences)
   - **How it works**: Brief explanation
   - **Pros**: What makes it good
   - **Cons**: What are the trade-offs
   - **Complexity**: Relative complexity (Low/Medium/High)

3. **Explicitly recommend the simplest:**
   - Mark Option 1 as "(Recommended)"
   - Provide clear rationale: "Simplest because..."
   - Explain why it's sufficient for current needs

4. **Present to user interactively:**
   ```
   Use AskUserQuestion to:
   - Present the 2-3 options with clear descriptions
   - Ask user to choose
   - Include "Other" for user to provide alternative
   ```

5. **Discuss trade-offs:**
   - Answer user questions about each option
   - Clarify what each option enables/prevents
   - Help user understand implications

6. **Iterate if needed:**
   - If user has concerns, refine the options
   - Ask clarifying questions
   - Present refined options
   - **But**: Don't iterate endlessly. 2-3 rounds maximum.

### Option Presentation Format

```markdown
## Design Options

### Option 1: [Simple Approach] (Recommended)
**Approach**: [What is it]
**How it works**: [Brief explanation]
**Pros**:
- Simple to implement
- Meets current requirements
- [Other benefits]
**Cons**:
- [Limitations]
**Complexity**: Low

**Why recommended**: Simplest approach that solves the current need. No extra complexity.

### Option 2: [Alternative Approach]
**Approach**: [What is it]
**How it works**: [Brief explanation]
**Pros**: [Benefits]
**Cons**:
- More complex
- [Other trade-offs]
**Complexity**: Medium

### Option 3: [Only if needed]
[Similar structure]
```

### Interactive Techniques

**Use AskUserQuestion for:**
- Presenting options and getting user choice
- Validating assumptions: "Is [X] a correct understanding?"
- Clarifying priorities: "What matters more: [A] or [B]?"
- Addressing concerns: "You mentioned [X] - how important is that?"
- Confirming direction: "Does this approach address your concern about [Y]?"

**Example Interactive Option Exploration:**

```
Agent presents:
"Based on your constraint of stateless sessions, here are 2 approaches:

Option 1 (Recommended): JWT-only validation
- Simplest: Decode JWT, validate signature, check expiry
- Pros: No database, stateless, fast
- Cons: Can't revoke sessions until token expires
- Complexity: Low

Option 2: JWT + Redis revocation list
- Check JWT first, then verify against Redis revocation list
- Pros: Can revoke sessions immediately
- Cons: Adds Redis dependency, adds latency
- Complexity: Medium

Which approach fits your needs?"

User: "Option 1. We don't need immediate revocation right now."

Agent: "Perfect. Any concerns about not being able to revoke sessions immediately?"

User: "No, token expiry is short enough."
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
   # 001-phase-1-implementation-auth-general-purpose/
   # 002-phase-2-implementation-storage-general-purpose/
   # 003-phase-2-review-security-code-simplifier/
   # 004-phase-3-implementation-api-general-purpose/
   ```

2. **Pick next number and create folder:**
   ```bash
   mkdir contributions/005-phase-3-design-session-mgmt-design-contribute/
   ```

3. **Folder naming convention:**
   `NNN-phase-X-design-[topic]-design-contribute`

   **Topics** (brief, descriptive):
   - `session-mgmt` - Session management approach
   - `error-handling` - Error handling strategy
   - `caching-strategy` - Caching approach
   - `api-structure` - API design
   - `data-model` - Data modeling decisions
   - `integration-approach` - Integration strategy
   - `auth-flow` - Authentication flow
   - `state-management` - State management approach

## Step 5: Document Design Decision

### Goal
Create lean design documentation that enables implementation.

### Process

**Create exactly 2 files:**

1. **design-doc.md** (< 100 lines target)
   - Use [design-doc-template.md](templates/design-doc-template.md)
   - Document the chosen approach
   - Explain why (constraints + user priorities)
   - Describe how it works
   - Specify what we're NOT doing
   - Provide implementation guidance

2. **decision-log.md** (inline format)
   - Use [decision-log-template.md](templates/decision-log-template.md)
   - Document primary design decision with rationale
   - Include supporting decisions made during design
   - List rejected alternatives

3 **context-handoff.md** - Target < 30 lines, keep only critical information, Document 3 things: 
   - What problem are we trying to solve with this design ( 10 lines ) 
   - High level overview of the design-doc, ( this of it as a high level summary of design-doc.md , 15 lines)
   - Reading guide of design-doc.md in the last 5 lines

### Quality Checks

Before finalizing, verify:

**Implementer Readiness:**
- ✅ Can someone implement from design-doc.md alone?
- ✅ Are integration points clearly specified?
- ✅ Is the pattern described clearly?
- ✅ Are success criteria defined?

**Simplicity Enforcement:**
- ✅ Is design-doc.md under 100 lines?
- ✅ Did we choose the simplest option?
- ✅ Is "Simplicity Rationale" section clear?
- ✅ Did we avoid designing for hypothetical futures?

**Scope Boundaries:**
- ✅ Does design only address current phase objective?
- ✅ Did we document what we're NOT doing?
- ✅ Did we avoid future phase design?
- ✅ Is YAGNI principle applied?

**Interactive Design Quality:**
- ✅ Did we ask user about implementation learnings?
- ✅ Did we clarify user priorities?
- ✅ Did we present options and get user choice?
- ✅ Did we validate assumptions?
- ✅ Does design-doc note it captures interactive session outcome?

### What NOT to Include

- ❌ No code files (documentation only)
- ❌ No comprehensive specifications
- ❌ No prototypes or proof-of-concepts
- ❌ No design for future phases
- ❌ No "nice to have" features

## Validation Strategy

### Simplicity Enforcement

**Always present simplest option first:**
- Option 1 must be the simplest viable approach
- Explicitly state "Recommended" with rationale
- If Option 1 isn't simplest, reorder the options

**Maximum 3 options:**
- If considering 4th option → you're over-engineering
- Stop, simplify, reduce to 3 or fewer

**Design-doc < 100 lines enforced:**
- Target is under 100 lines
- If exceeding 100 lines, you're over-documenting
- Focus on "what" and "why", not exhaustive "how"

**Articulate "Simplest Because:"**
- Must have clear rationale for why approach is simplest
- Compare complexity to alternatives
- Explain why simple approach is sufficient

### Scope Boundaries (YAGNI Validation)

**Only current phase objective:**
- Design addresses specific "Design:" task from roadmap
- No design beyond current phase scope
- Document what we're NOT doing

**No future phase design:**
- Don't design for hypothetical phase 5 when in phase 3
- Don't add flexibility "just in case"
- Wait for implementation to reveal actual needs

**Document what we're NOT doing:**
- "What We're NOT Doing" section required
- List rejected alternatives with brief rationale
- List out-of-scope considerations for future

### Quality Gates

**Before completing contribution, verify:**

1. **Handoff Quality:**
   - Can implementer start coding from design-doc.md alone?
   - Are integration points specified?
   - Is guidance actionable?

2. **Documentation Complete:**
   - Both mandatory docs present (design-doc.md + decision-log.md)?
   - Design-doc < 100 lines?
   - Decision-log documents primary decision + supporting choices?

3. **Interactive Design:**
   - Did we use AskUserQuestion appropriately?
   - Did we validate assumptions with user?
   - Did we get explicit user choice on approach?

4. **Simplicity:**
   - Is this the simplest approach that works?
   - Did we avoid over-engineering?
   - Is rationale for simplicity clear?

### Anti-Patterns Detection

**Over-Engineering Signals:**
- Design-doc exceeds 100 lines → too comprehensive
- More than 3 design options → too many choices
- Designing for future phases → violates YAGNI
- Complex patterns when simple would work → over-engineering

**Under-Engineering Signals:**
- No integration points specified → implementer can't connect
- No success criteria → can't verify completion
- Missing key constraints → implementer lacks context

**Non-Interactive Design:**
- No AskUserQuestion used → missing user input
- Assumptions not validated → wrong constraints
- User priorities unknown → wrong trade-offs

## Interactive Techniques Throughout Workflow

### When to Use AskUserQuestion

**Step 2 - Constraint Discovery:**
- "What did you learn during implementation that should inform this design?"
- "What patterns or approaches worked well so far?"
- "What integration challenges did you encounter?"
- "What's your priority: simplicity, performance, or flexibility?"
- "Are there any non-negotiable requirements?"

**Step 3 - Option Exploration:**
- Present 2-3 design options and ask user to choose
- "Which approach fits your needs better?"
- "Any concerns about the recommended approach?"
- "What matters more to you: [trade-off A] or [trade-off B]?"

**Step 5 - Validation:**
- "Does this design doc clearly capture what you need?"
- "Any concerns about the implementation guidance?"

### How to Make Design Interactive

**Use AskUserQuestion tool:**
- Present design choices with clear options
- Gather user preferences and priorities
- Validate assumptions before finalizing

**Iterative refinement:**
- Show initial options → get feedback → refine → present again
- Max 2-3 iterations to avoid analysis paralysis

**Collaborative exploration:**
- "What concerns you most about approach X?"
- "What's your priority: simplicity or flexibility?"
- "Does this address your concern about Y?"

**Assumption validation:**
- "I understand the constraint is Y - is that correct?"
- "You mentioned X worked well - should we use that pattern?"

**Trade-off discussion:**
- Present options with clear pros/cons
- Discuss which trade-offs matter most for this context
- Get explicit user choice, don't infer

**Quick feedback loops:**
- Don't design in isolation for long periods
- Check in frequently during option generation
- Validate direction before investing in detailed design

### Example Interactive Session Flow

```
1. Agent reads context and contributions
2. Agent asks: "What did implementation reveal about session needs?"
3. User explains constraints learned
4. Agent confirms: "So constraint is X, correct?"
5. Agent asks: "Priority: simplicity or flexibility?"
6. User chooses simplicity
7. Agent generates 2 options (simplest first)
8. Agent presents options via AskUserQuestion
9. User chooses Option 1 (simplest)
10. Agent asks: "Any concerns about this approach?"
11. User confirms no concerns
12. Agent creates design-doc.md capturing choice + rationale
13. Agent creates decision-log.md with decision record
```

## Integration with Other Skills

### Integration with dev-strategy

**Reads from dev-strategy:**
- `implementation-roadmap.md` - Find "Design:" objectives
- `context-document.md` - Understand behavioral spec
- `decision-log.md` - Previous decisions and constraints

**Can update dev-strategy:**
- Optionally append design decision to dev-strategy's `decision-log.md`
- Mark "Design:" objective as completed in roadmap

### Integration with dev-contribute

**Before dev-contribute:**
- design-contribute creates design contribution folder
- Produces design-doc.md that dev-contribute reads
- Specifies interfaces and patterns to follow

**dev-contribute reads:**
- design-doc.md from design contribution folder
- Follows patterns and interfaces specified
- Implements according to design guidance

### Integration with read-contribution

**read-contribution can:**
- Read design contributions to understand design decisions
- Progressive disclosure for design reasoning
- Context for implementers

## Example Flow

```
Roadmap Phase 3: "Design: Determine session management approach"
↓
[Existing contributions:]
001-phase-1-implementation-basic-auth/
002-phase-2-implementation-user-storage/
003-phase-3-implementation-jwt-auth/
004-phase-3-implementation-api-endpoints/

[design-contribute invoked:]
↓
Step 1: Read context and contributions 001-004
Step 2: Ask user about learnings from JWT implementation
        User reveals: "Need stateless approach, short tokens"
Step 3: Generate 2 options: JWT-only vs JWT+Redis
        Present to user, user chooses JWT-only (simplest)
Step 4: Create folder: 005-phase-3-design-session-mgmt-design-contribute/
Step 5: Document in design-doc.md and decision-log.md

[Next dev-contribute invoked:]
006-phase-3-implementation-session-validation/
↓
Reads design-doc.md from contribution 005
Implements session validation following design
```

## Summary

design-contribute is about **collaborative design at the Last Responsible Moment**:

1. **Wait until implementation reveals constraints** (dev-strategy deferred it)
2. **Work interactively with user** to understand learnings and priorities
3. **Present simple options** (max 3, simplest first)
4. **Get user choice** through dialogue, not inference
5. **Document lean design** (< 100 lines) for implementers

**Key Success Factors:**
- Use AskUserQuestion liberally throughout
- Always present simplest option first with clear rationale
- Never design in isolation - make it collaborative
- Keep design-doc under 100 lines
- Focus on current phase only (YAGNI)
- Validate assumptions, don't infer priorities
