# Read Contribution Skill - Reference Guide

This document provides strategies for efficiently reading and understanding contribution artifacts.

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

**TDD Pattern Recognition:**
- Look for test-design → test-validation → implementation sequence
- Implementation agents: Focus on test-validation contributions for requirements
- Review agents: Verify TDD discipline was maintained

**Steel Thread Pattern Recognition:**
- Look for pathfinder → foundation → expansion sequence
- Implementation agents: Understand the core working flow before adding features
- Review agents: Verify working system was maintained throughout

**Core-then-Integrate Pattern Recognition:**
- Look for domain-logic → port-design → adapter → integration sequence
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

## Quality Indicators

### Well-Written Contributions
- **Context-handoff.md provides clear guidance** for next contributors
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
2. Identify your contribution type and phase
3. Understand established patterns and constraints

### After Using Dev-Contribute
1. Your contribution becomes part of the reading chain for future contributors
2. Ensure your context-handoff.md follows progressive disclosure principles
3. Reference relevant previous contributions in your decision-log.md

### Cross-Skill Compatibility
This skill works with any *-contribute skill:
- **qa-contribute**: Read previous QA contributions for established testing patterns
- **compliance-contribute**: Read all contributions for compliance assessment
- **security-contribute**: Focus on security-related decisions and artifacts