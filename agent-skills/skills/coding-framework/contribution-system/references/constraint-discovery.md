# Constraint Discovery

Interactive techniques for gathering real constraints before designing or planning.

## Core Principle

**Ask before analyzing.** Never assume constraints or infer priorities. Real constraints come from user knowledge and implementation experience — not from speculation.

## AskUserQuestion Patterns

### During design-contribute (Step 2 — Constraint Discovery)

**About implementation learnings:**
- "What did you learn during implementation that should inform this design?"
- "What patterns or approaches worked well so far?"
- "What integration challenges did you encounter?"

**About priorities:**
- "What's your priority: simplicity, performance, or flexibility?"
- "What concerns you most about this design decision?"
- "Are there any non-negotiable requirements?"

**Validating your analysis:**
- "I see pattern X emerged in the implementation — is that correct?"
- "I understand the constraint is Y — is that right?"

---

### During design-contribute (Step 3 — Option Exploration)

**Presenting options:**
- Present 2–3 options (simplest first) via AskUserQuestion
- Ask user to choose; include "Other" for custom input
- "Which approach fits your needs better?"

**Trade-off discussion:**
- "Any concerns about the recommended approach?"
- "What matters more to you: [trade-off A] or [trade-off B]?"

**Validation:**
- "Does this design doc clearly capture what you need?"
- "Any concerns about the implementation guidance?"

---

### During dev-strategy (Step 4 — Constraint Gathering)

**About codebase ambiguities:**
- "I found both React and Vue components — which patterns should I follow?"
- "I see both MongoDB and PostgreSQL — which should I use for new data?"
- "Multiple auth methods found — which should I extend?"

**Essential requirement questions (ask only when directly relevant):**
- "Should this integrate with existing user accounts/data?"
- "Must this work with current API formats?"
- "Any specific technologies you want to use or avoid?"

**Open-ended:**
- "Any specific preferences for how this should work?"
- "Anything else I should know about your setup or requirements?"

---

## Iteration Guidelines

- Show initial options → get feedback → refine → present again
- Maximum 2–3 iterations to avoid analysis paralysis
- Do NOT design in isolation then present a final complete design
- Confirm direction before investing in detailed design

## What NOT to Do

- ❌ Assume constraints without asking
- ❌ Design based on hypothetical future needs
- ❌ Infer priorities — ask explicitly
- ❌ Skip user interaction and go straight to design
- ❌ Make the decision for the user
- ❌ Present more than 3 options
