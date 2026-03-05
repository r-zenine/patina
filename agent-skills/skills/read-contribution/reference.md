# Read Contribution Skill - Reference Guide

This document provides strategies for efficiently reading and understanding contribution artifacts.

## Table of Contents

- [Progressive Disclosure Strategy](#progressive-disclosure-strategy)
- [Role-Specific Reading Strategies](#role-specific-reading-strategies)
  - [Implementation Agents](#implementation-agents)
  - [Review Agents](#review-agents)
  - [Audit Agents](#audit-agents)
- [Navigation Patterns](#navigation-patterns)
- [Efficient Reading Techniques](#efficient-reading-techniques)
- [Common Reading Scenarios](#common-reading-scenarios)
- [Reading Design Contributions](#reading-design-contributions)
- [Integration with Other Skills](#integration-with-other-skills)

## Progressive Disclosure Strategy

For the complete progressive disclosure technique, see [`context-engineering/references/progressive-disclosure.md`](../context-engineering/references/progressive-disclosure.md).

**The First Pass Rule**: Always start with `context-handoff.md` — it contains the essential information needed by 80% of contributors 80% of the time.

**When to read additional artifacts:**
- `decision-log.yaml` → when you need to understand WHY specific technical choices were made
- `changelog.md` → when you need to verify what phase objectives were completed
- Optional artifacts → when you need specialized analysis (performance, security, architecture)

## Role-Specific Reading Strategies

### Implementation Agents

**Primary Goal**: Understand what to build next and avoid pitfalls.

**Reading order:**
1. `context-handoff.md` — focus on "What's fragile" and "For next contributors"
2. `decision-log.yaml` — only if you need technical patterns established
3. Skip `changelog.md` unless verifying phase completion

**Key questions to answer:**
- What patterns should I follow?
- What assumptions can I make?
- What are the known problem areas?
- Where are the integration points?

---

### Review Agents

**Primary Goal**: Assess quality, consistency, and architectural alignment.

**Reading order:**
1. `context-handoff.md` — focus on "What works" patterns and architectural decisions
2. `decision-log.yaml` — critical: understand rationale behind choices
3. `changelog.md` — verify claimed accomplishments match evidence
4. Optional artifacts — if specialized review needed (security, performance)

**Key questions to answer:**
- Are patterns consistent with the dev-strategy plan?
- Do decisions align with previous choices?
- Is the work truly complete?
- Are there quality or security concerns?

---

### Audit Agents

**Primary Goal**: Focused analysis for compliance, security, or governance.

**Reading order:**
1. `context-handoff.md` — understand scope and approach
2. `decision-log.yaml` — document all technical choices for audit trail
3. `changelog.md` — verify completion claims
4. All optional artifacts — essential for focused analysis
5. Cross-reference with dev-strategy plan and other contributions

**Key questions to answer:**
- Does this comply with organizational standards?
- Are security/performance requirements met?
- Is the documentation complete and accurate?
- Are there gaps or risks introduced?

## Navigation Patterns

### Understanding Contribution Sequences

**Contributions are numbered chronologically:**
```
001-phase-1-implementation-code-general-purpose/
002-phase-1-review-security-code-simplifier/
003-phase-2-test-design-tdd-general-purpose/
```

**Reading strategy for sequences:**
1. Latest contribution first — most current state and context
2. Work backwards only if you need historical context
3. Skip intermediate contributions unless directly relevant

### Understanding Strategy Patterns from Folder Names

**Design Pattern Recognition (Any Strategy):**
- Look for `design-[topic]-design-contribute` contributions
- These appear when "Design: Determine [X]" objectives exist in roadmap
- Design contributions precede implementation contributions
- **Implementation agents**: Read design-doc.md before implementing
- **Review agents**: Verify implementation matches design specifications

**TDD Pattern Recognition:**
- Look for `test-design` → `test-validation` → `implementation` sequence

**Steel Thread Pattern Recognition:**
- Look for `pathfinder` → `foundation` → `expansion` sequence

**Core-then-Integrate Pattern Recognition:**
- Look for `domain-logic` → `port-design` → `adapter` → `integration` sequence

For strategy execution details and contribution type definitions, see [`dev-strategies` skill](../dev-strategies/SKILL.md) and [`contribution-artifacts` skill](../contribution-artifacts/SKILL.md).

## Efficient Reading Techniques

### The 5-Minute Scan

**For context-handoff.md:**
1. **Read "Built:" line** (30 seconds) — understand the main deliverable
2. **Scan "What works/fragile/missing"** (2 minutes) — understand current state
3. **Read your role's guidance** (1 minute) — get specific direction
4. **Note "Expects/Deferred"** (1 minute) — understand constraints
5. **Check related docs links** (30 seconds) — decide if you need more

### The Deep Dive

**When 5-minute scan isn't enough:**
1. **Full context-handoff.md read** (5-10 minutes)
2. **Targeted decision-log.yaml scan** (5 minutes) — focus on decisions relevant to your work
3. **Selective optional artifacts** (varies) — only what you specifically need

### Cross-Contribution Analysis

**Understanding project evolution:**
1. Read latest context-handoff from each phase — understand progression
2. Look for decision patterns across contributions — understand architectural evolution
3. Identify recurring "What's fragile" — understand systemic issues

## Common Reading Scenarios

### Scenario 1: "I'm starting Phase 3 implementation"
1. Read Phase 2's latest context-handoff.md (understand current state)
2. Scan Phase 2 decision-log.yaml (understand established patterns)
3. Check dev-strategy plan (verify Phase 3 objectives)

### Scenario 2: "I need to review security across all contributions"
1. Read all context-handoff.md files for security-related "What's fragile"
2. Read all decision-log.yaml files for security-related choices
3. Read any security-*.md optional artifacts
4. Cross-reference with dev-strategy security requirements

### Scenario 3: "I'm debugging an issue"
1. Start with most recent contribution's context-handoff.md
2. Look for relevant "What's fragile" entries
3. Check decision-log.yaml for related technical choices
4. Work backwards through contributions until you find relevant context

### Scenario 4: "I'm onboarding to this project"
1. Read dev-strategy plan first (understand overall approach)
2. Read Phase 1's final context-handoff.md (understand foundation)
3. Read latest contribution's context-handoff.md (understand current state)
4. Scan decision-log.yaml files for architectural patterns

### Scenario 5: "I'm implementing after a design contribution"
1. Read the design contribution's design-doc.md (understand what to build)
2. Focus on "How It Works" and "Implementation Guidance" sections
3. Note "What We're NOT Doing" to avoid scope creep
4. Read decision-log.md to understand why this approach was chosen
5. Read previous implementation context-handoff.md for current state

## Reading Design Contributions

Design contributions (created by design-contribute skill) have a **different structure** than implementation contributions:

```
contributions/NNN-phase-X-design-[topic]-design-contribute/
├── design-doc.md          ← START HERE (replaces context-handoff.md)
├── decision-log.md        ← Design decisions and rationale
└── context-handoff.md     ← Problem + overview + reading guide (< 30 lines)
```

### Reading Strategy for Design Contributions

**Implementation Agents (Primary Users):**
1. **design-doc.md**: Read the entire document (< 100 lines)
   - Focus on "How It Works" — pattern to implement
   - Focus on "Implementation Guidance" — where to start
   - Note "What We're NOT Doing" — avoid scope creep
2. **decision-log.md**: Understand why this approach was chosen
3. **context-handoff.md**: Quick reading guide if needed

**Review Agents:**
1. **design-doc.md**: Focus on "Why This Design" section
   - Verify design matches constraints learned from implementation
   - Check "Simplicity Rationale" — ensure YAGNI principle followed
2. **decision-log.md**: Verify design decisions are well-justified

**Key Differences from Implementation Contributions:**
- **design-doc.md is the primary artifact** (not context-handoff.md)
- **Design contributions come BEFORE implementation** in the sequence
- **Focus is on "what to build"** not "what was built"
- **No code is implemented** in design contributions

For artifact schemas and quality indicators, see [`contribution-artifacts/references/design-artifacts.md`](../contribution-artifacts/references/design-artifacts.md).

## Integration with Other Skills

### Before Using Dev-Contribute
1. Use read-contribution to understand existing work
2. Check if previous contribution is a design contribution — if so, read design-doc.md
3. Identify your contribution type and phase
4. Understand established patterns and constraints

### Before Using Design-Contribute
1. Use read-contribution to understand implementation learnings
2. Read recent context-handoff.md files to understand current state
3. Identify constraints that emerged from implementation

### After Using Dev-Contribute
1. Your contribution becomes part of the reading chain for future contributors
2. Ensure your context-handoff.md follows progressive disclosure principles
3. Reference relevant previous contributions in your decision-log.yaml

### After Using Design-Contribute
1. Your design-doc.md becomes the specification for next implementation contribution
2. Ensure design-doc.md is implementer-ready (< 100 lines, clear guidance)
3. Reference implementation learnings that informed the design
