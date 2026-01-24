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
- [Quality Indicators](#quality-indicators)
- [Integration with Other Skills](#integration-with-other-skills)

## Progressive Disclosure Strategy

### The First Pass Rule
**Always start with context-handoff.md** - it contains the essential information needed by 80% of contributors 80% of the time.

### When to Read Additional Artifacts

**Read decision-log.md if:**
- You need to understand WHY specific technical choices were made
- You're implementing similar functionality and need precedent
- You're reviewing architectural consistency across contributions

**Read changelog.md if:**
- You need to verify what phase objectives were completed
- You're tracking project progress across multiple contributions
- You're preparing status updates or reports

**Read optional artifacts if:**
- You need specialized analysis (performance, security, architecture)
- You're doing compliance or audit work
- You're troubleshooting issues in specific domains

## Role-Specific Reading Strategies

### Implementation Agents

**Primary Goal**: Understand what to build next and avoid pitfalls

**Reading Strategy:**
1. **context-handoff.md**: Focus on "What's fragile" and "For next contributors" sections
2. **decision-log.md**: Only if you need to understand technical patterns established
3. **Skip**: changelog.md (unless verifying phase completion)

**Key Questions to Answer:**
- What patterns should I follow?
- What assumptions can I make?
- What are the known problem areas?
- Where are the integration points?

### Review Agents

**Primary Goal**: Assess quality, consistency, and architectural alignment

**Reading Strategy:**
1. **context-handoff.md**: Focus on "What works" patterns and architectural decisions
2. **decision-log.md**: Critical - understand rationale behind choices
3. **changelog.md**: Verify claimed accomplishments match evidence
4. **Optional artifacts**: If specialized review needed (security, performance)

**Key Questions to Answer:**
- Are patterns consistent with dev-strategy plan?
- Do decisions align with previous choices?
- Is the work truly complete?
- Are there quality or security concerns?

### Audit Agents

**Primary Goal**: Focused analysis for compliance, security, or governance

**Reading Strategy:**
1. **context-handoff.md**: Understand scope and approach
2. **decision-log.md**: Document all technical choices for audit trail
3. **changelog.md**: Verify completion claims
4. **All optional artifacts**: Essential for focused analysis
5. **Cross-reference**: Compare with dev-strategy plan and other contributions

**Key Questions to Answer:**
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

**Reading Strategy for Sequences:**
1. **Latest contribution first**: Most current state and context
2. **Work backwards**: Only if you need historical context
3. **Skip intermediate**: Unless they're directly relevant to your work

### Understanding Strategy Patterns

**Design Pattern Recognition (Any Strategy):**
- Look for `design-[topic]-design-contribute` contributions
- These appear when "Design: Determine [X]" objectives exist in roadmap
- Design contributions precede implementation contributions
- Example: `005-design-session-mgmt` → `006-implementation-session-validation`
- **Implementation agents**: Read design-doc.md before implementing
- **Review agents**: Verify implementation matches design specifications

**TDD Pattern Recognition:**
- Look for test-design → test-validation → implementation sequence
- May include design contributions when architectural decisions needed
- Implementation agents: Focus on test-validation contributions for requirements
- Review agents: Verify TDD discipline was maintained

**Steel Thread Pattern Recognition:**
- Look for pathfinder → foundation → expansion sequence
- May include design contributions before major expansions
- Implementation agents: Understand the core working flow before adding features
- Review agents: Verify working system was maintained throughout

**Core-then-Integrate Pattern Recognition:**
- Look for domain-logic → port-design → adapter → integration sequence
- May include design contributions for interface design decisions
- Implementation agents: Understand interface contracts before building adapters
- Review agents: Verify clean architecture principles were followed

## Efficient Reading Techniques

### The 5-Minute Scan

**For context-handoff.md:**
1. **Read "Built:" line** (30 seconds) - understand the main deliverable
2. **Scan "What works/fragile/missing"** (2 minutes) - understand current state
3. **Read your role's guidance** (1 minute) - get specific direction
4. **Note "Expects/Deferred"** (1 minute) - understand constraints
5. **Check related docs links** (30 seconds) - decide if you need more

### The Deep Dive

**When 5-minute scan isn't enough:**
1. **Full context-handoff.md read** (5-10 minutes)
2. **Targeted decision-log.md scan** (5 minutes) - focus on decisions relevant to your work
3. **Selective optional artifacts** (varies) - only what you specifically need

### Cross-Contribution Analysis

**Understanding project evolution:**
1. **Read latest context-handoff from each phase** - understand progression
2. **Look for decision patterns across contributions** - understand architectural evolution
3. **Identify recurring "What's fragile"** - understand systemic issues

## Common Reading Scenarios

### Scenario 1: "I'm starting Phase 3 implementation"
1. Read Phase 2's latest context-handoff.md (understand current state)
2. Scan Phase 2 decision-log.md (understand established patterns)
3. Check dev-strategy plan (verify Phase 3 objectives)

### Scenario 2: "I need to review security across all contributions"
1. Read all context-handoff.md files for security-related "What's fragile"
2. Read all decision-log.md files for security-related choices
3. Read any security-*.md optional artifacts
4. Cross-reference with dev-strategy security requirements

### Scenario 3: "I'm debugging an issue"
1. Start with most recent contribution's context-handoff.md
2. Look for relevant "What's fragile" entries
3. Check decision-log.md for related technical choices
4. Work backwards through contributions until you find relevant context

### Scenario 4: "I'm onboarding to this project"
1. Read dev-strategy plan first (understand overall approach)
2. Read Phase 1's final context-handoff.md (understand foundation)
3. Read latest contribution's context-handoff.md (understand current state)
4. Scan decision-log.md files for architectural patterns

### Scenario 5: "I'm implementing after a design contribution"
1. Read the design contribution's design-doc.md (understand what to build)
2. Focus on "How It Works" and "Implementation Guidance" sections
3. Note "What We're NOT Doing" to avoid scope creep
4. Read decision-log.md to understand why this approach was chosen
5. Read previous implementation context-handoff.md for current state

## Reading Design Contributions

Design contributions (created by design-contribute skill) have a **different structure** than implementation contributions:

### Design Contribution Structure
```
contributions/NNN-phase-X-design-[topic]-design-contribute/
├── design-doc.md          ← START HERE (replaces context-handoff.md)
└── decision-log.md        ← Design decisions and rationale
```

**Note**: Design contributions have NO changelog.md and NO context-handoff.md.

### Reading Strategy for Design Contributions

**Implementation Agents (Primary Users):**
1. **design-doc.md**: Read the entire document (< 100 lines)
   - Focus on "How It Works" - pattern to implement
   - Focus on "Implementation Guidance" - where to start
   - Note "What We're NOT Doing" - avoid scope creep
2. **decision-log.md**: Understand why this approach was chosen
   - See what alternatives were rejected and why
   - Understand constraints that led to this design

**Review Agents:**
1. **design-doc.md**: Focus on "Why This Design" section
   - Verify design matches constraints learned from implementation
   - Check "Simplicity Rationale" - ensure YAGNI principle followed
2. **decision-log.md**: Verify design decisions are well-justified

**Key Differences from Implementation Contributions:**
- **design-doc.md is the primary artifact** (not context-handoff.md)
- **Design contributions come BEFORE implementation** in the sequence
- **Focus is on "what to build"** not "what was built"
- **No code is implemented** in design contributions

### When to Read Design Contributions

**Before implementing:**
- Check if previous contribution is a design contribution
- Example: `005-phase-3-design-session-mgmt-design-contribute/` comes before `006-phase-3-implementation-session-validation/`
- Read design-doc.md to understand what to implement and how

**During review:**
- Verify implementation matches design-doc.md specifications
- Check if "Success Criteria" from design-doc.md are met

## Quality Indicators

### Well-Written Contributions
- **Context-handoff.md provides clear guidance** for next contributors (implementation contributions)
- **Design-doc.md is implementer-ready** (design contributions)
- **Decisions are justified with rationale** in decision-log.md
- **Related documentation is properly linked**
- **Assumptions and constraints are explicit**

### Red Flags
- **Vague "What's fragile" entries** - unclear what needs attention
- **Missing decision rationale** - can't understand why choices were made
- **No guidance for next contributors** - breaks knowledge transfer chain
- **Assumptions buried in implementation details** - creates hidden dependencies

## Integration with Other Skills

### Before Using Dev-Contribute
1. Use read-contribution to understand existing work
2. Check if previous contribution is a design contribution - if so, read design-doc.md
3. Identify your contribution type and phase
4. Understand established patterns and constraints

### Before Using Design-Contribute
1. Use read-contribution to understand implementation learnings
2. Read recent context-handoff.md files to understand current state
3. Identify constraints that emerged from implementation

### After Using Dev-Contribute
1. Your contribution becomes part of the reading chain for future contributors
2. Ensure your context-handoff.md follows progressive disclosure principles
3. Reference relevant previous contributions in your decision-log.md

### After Using Design-Contribute
1. Your design-doc.md becomes the specification for next implementation contribution
2. Ensure design-doc.md is implementer-ready (< 100 lines, clear guidance)
3. Reference implementation learnings that informed the design

### Cross-Skill Compatibility
This skill works with any *-contribute skill:
- **dev-contribute**: Read previous implementation contributions for established patterns
- **design-contribute**: Read design decisions before implementing (see Reading Design Contributions below)
- **qa-contribute**: Read previous QA contributions for established testing patterns
- **compliance-contribute**: Read all contributions for compliance assessment
- **security-contribute**: Focus on security-related decisions and artifacts