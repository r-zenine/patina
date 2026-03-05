# Dev Contribute Skill - Reference Guide

This document provides instructions for contributing to dev-strategy implementation plans.

## Table of Contents

- [Overview](#overview)
- [Step 1: Read and Understand Dev-Strategy Plan](#step-1-read-and-understand-dev-strategy-plan)
- [Step 1.5: Pre-Work Validation](#step-15-pre-work-validation)
- [Step 2: Follow Strategy Approach](#step-2-follow-strategy-approach)
- [Step 3: Create Numbered Contribution Folder](#step-3-create-numbered-contribution-folder)
- [Step 4: Perform Specialized Work](#step-4-perform-specialized-work)
- [Step 5: Generate Mandatory Documentation](#step-5-generate-mandatory-documentation)
- [Step 6: Handling Decision Revisions](#step-6-handling-decision-revisions)
- [Contribution Validation](#contribution-validation)
- [Common Patterns by Strategy](#common-patterns-by-strategy)
- [Research Integration Examples](#research-integration-examples)

## Overview

The dev-contribute skill enables structured contributions to existing dev-strategy plans. Each contribution follows the selected implementation strategy and creates proper documentation for knowledge transfer.

## Step 1: Read and Understand Dev-Strategy Plan

### Goal
Understand the context before contributing.

### Process
1. **Read all dev-strategy artifacts:**
   - `code-context.md` - Relevant code references
   - `context-document.md` - Behavioral spec and architecture (includes research findings if available)
   - `decision-log.yaml` - Previous decisions made
   - `implementation-roadmap.md` - Planned phases and strategy
   - `research/` directory (if present) - Technical research findings and recommendations

2. **Identify the implementation strategy:**
   - Look for strategy choice in decision-log.yaml
   - Understand the planned approach (TDD, Steel Thread, Core-then-Integrate)
   - For strategy details → see [`dev-strategies` skill](../dev-strategies/SKILL.md)

3. **Review research findings (when relevant):**
   - If implementing new technologies or unfamiliar patterns, check for `research/` directory
   - For early-phase work, review research artifacts for implementation guidance
   - For later-phase work, research may be less relevant - use judgment

4. **Review existing contributions:**
   - Check `contributions/` directory for previous work
   - Read recent context-handoff.md files for current state
   - Identify the next logical contribution number

## Step 1.5: Pre-Work Validation

### Goal
Ensure a clean baseline before starting contribution work.

### Process
**MANDATORY: The contribution CANNOT proceed if any of these checks fail.**

1. **Run compilation/build:** All compilation must succeed with zero errors
2. **Run linter:** All linting must pass with zero warnings
3. **Run test suite:** All tests must pass with zero failures

### If Validation Fails
- STOP immediately
- Document the failures found
- Inform the user that the codebase baseline is not clean
- Do NOT proceed with contribution until baseline is clean

### Rationale
Contributions must start from a clean, working baseline to ensure:
- New issues can be clearly attributed to the contribution
- Quality degradation is prevented
- Technical debt is not accumulated

## Step 2: Follow Strategy Approach

For complete strategy execution details, contribution sequences, and contribution type definitions, see the [`dev-strategies` skill](../dev-strategies/SKILL.md):
- TDD → [`dev-strategies/references/tdd.md`](../dev-strategies/references/tdd.md)
- Steel Thread → [`dev-strategies/references/steel-thread.md`](../dev-strategies/references/steel-thread.md)
- Core-then-Integrate → [`dev-strategies/references/core-then-integrate.md`](../dev-strategies/references/core-then-integrate.md)

**Quick reference:**
- **TDD**: Test Design → Test Validation → Implementation
- **Steel Thread**: Pathfinder → Foundation Builder → Capability Expander
- **Core-then-Integrate**: Domain Modeler → Port Designer → Adapter Builder → Integration Orchestrator

## Step 3: Create Numbered Contribution Folder

### Goal
Create sequentially numbered folder for chronological ordering.

### Process
1. **Check existing contributions:**
   ```bash
   ls contributions/
   # 001-phase-1-implementation-code-general-purpose/
   # 002-phase-1-review-security-code-simplifier/
   ```

2. **Pick next number:**
   ```bash
   mkdir contributions/003-phase-2-test-design-tdd-general-purpose/
   ```

For complete folder naming convention, contribution type definitions, and specialty list, see [`contribution-artifacts` skill](../contribution-artifacts/SKILL.md).

## Step 4: Perform Specialized Work

### Implementation Guidelines by Strategy

Follow the execution guidance for your strategy:
- **TDD** → [`dev-strategies/references/tdd.md`](../dev-strategies/references/tdd.md)
- **Steel Thread** → [`dev-strategies/references/steel-thread.md`](../dev-strategies/references/steel-thread.md)
- **Core-then-Integrate** → [`dev-strategies/references/core-then-integrate.md`](../dev-strategies/references/core-then-integrate.md)

### Quality Standards

**All Contributions Must:**
- Follow existing code patterns from context-document.md
- Respect constraints from decision-log.yaml
- Maintain or improve test coverage
- Handle expected errors for the current scope
- Follow basic security practices from existing code

**Sufficient Implementation Principle**: Focus on making it work correctly, not perfectly. Avoid over-polishing, excessive error handling for edge cases that won't occur, or adding features not required for the current phase.

See [`design-principles` skill](../design-principles/SKILL.md) for full principles.

**When Research Artifacts Are Most Valuable:**
- First implementation of new technologies mentioned in research/
- Early phases where research decisions need implementation
- When encountering integration challenges covered in research findings
- Skip research review for routine work on well-established patterns

## Step 5: Generate Mandatory Documentation

### Goal
Create three required artifacts for knowledge transfer and project continuity.

For complete artifact schemas and templates, see [`contribution-artifacts/references/implementation-artifacts.md`](../contribution-artifacts/references/implementation-artifacts.md).

### Process

**1. Create changelog.md**
- Use [changelog-template.md](../contribution-artifacts/assets/templates/changelog-template.md)
- Focus on delivered value and what this enables
- Keep to 3-4 lines maximum

**2. Create decision-log.yaml**
- Use [decision-log-template.yaml](../contribution-artifacts/assets/templates/decision-log-template.yaml)
- Document only NEW decisions made during this contribution

**3. Create context-handoff.md**
- Use [context-handoff-template.md](../contribution-artifacts/assets/templates/context-handoff-template.md)
- Lead with what you built and key insights
- Structure as: What works/What's fragile/What's missing
- Provide specific guidance for next contributors

### Quality Check
- Can next contributor understand what was done and why?
- Are assumptions and blockers clearly documented?
- Does each file serve its distinct purpose without overlap?

### Optional Artifacts
For specialized contributions, see [optional-artifacts-templates.md](../contribution-artifacts/assets/templates/optional-artifacts-templates.md) for additional documentation types.

## Step 6: Handling Decision Revisions

### Goal
Provide clear workflow when users disagree with logged decisions from previous contributions.

### When to Use This
After reviewing a contribution's decision-log.yaml, if the user disagrees with a decision and wants to revisit it.

### Process

**1. Create new revision contribution folder:**
- Format: `NNN-phase-X-revision-[original-specialty]-[agent]`
- Example: If revising contribution `002-phase-1-implementation-code-general-purpose/`, create `004-phase-1-revision-code-general-purpose/`
- Use next sequential number, maintain phase and specialty context

**2. Update code to match revised decision:**
- Make all necessary code changes to implement the new decision
- Ensure changes are complete and align with the revised approach
- Follow same quality standards as original contribution

**3. Document in new contribution's decision-log.yaml:**
- Reference original contribution number
- Explain what decision is being revised and why

**4. Generate all mandatory documentation:**
- **changelog.md**: Indicate this revises contribution NNN, explain what changed
- **decision-log.yaml**: New decision with revision context and reference to original
- **context-handoff.md**: Explain impact of revision on existing work and future contributions

**5. Run validation checks:**
- Same mandatory quality gates as any contribution
- Ensure build, lint, and tests all pass

### Key Principles
- **Always create new folder** - Don't edit previous contribution folders
- **Always update code** - Code changes are part of the revision contribution
- **Clear audit trail** - Decision evolution is visible through contribution history
- **Sequential numbering** - Revisions follow chronological order like any contribution

## Contribution Validation

Before finalizing your contribution:

0. **Final Quality Gate (MANDATORY):**
   **The contribution CANNOT be completed if any of these checks fail.**

   - **Run compilation/build:** All compilation must succeed with zero errors
   - **Run linter:** All linting must pass with zero warnings
   - **Run test suite:** All tests must pass with zero failures

   If any check fails: fix all issues before completing the contribution.

1. **Strategy Compliance Check:**
   - Does this follow the chosen strategy approach?
   - Are we in the right phase of the strategy?
   - Does this build logically on previous contributions?

2. **Quality Validation:**
   - All mandatory files created and complete?
   - Context handoff explains reasoning and alternatives?
   - Decision log captures new choices made?
   - Changelog summarizes impact clearly?

3. **Knowledge Transfer:**
   - Would another agent understand what was done and why?
   - Are assumptions and limitations clearly documented?
   - Is guidance provided for next contributors?

## Common Patterns by Strategy

### TDD Pattern
```
001-phase-X-test-design-tdd-[agent]
002-phase-X-test-validation-tdd-[agent]
003-phase-X-implementation-code-[agent]
```

### Steel Thread Pattern
```
001-phase-X-pathfinder-steel-thread-[agent]
002-phase-X-foundation-steel-thread-[agent]
003-phase-X-expansion-capability-[agent]
004-phase-X-expansion-capability-[agent]
```

### Core-then-Integrate Pattern
```
001-phase-X-domain-logic-core-[agent]
002-phase-X-port-design-core-[agent]
003-phase-X-adapter-database-[agent]
004-phase-X-adapter-api-[agent]
005-phase-X-integration-orchestrator-[agent]
```

### Revision Pattern
```
001-phase-X-implementation-code-[agent]
002-phase-X-review-security-[agent]
003-phase-Y-test-design-tdd-[agent]
004-phase-X-revision-code-[agent]  # Revises decision from 001
005-phase-Y-test-validation-tdd-[agent]
```

## Research Integration Examples

**Early Phase (Research Relevant):**
- Contribution 001 implementing GraphQL for first time → Review research/technology-research.md
- Contribution 002 setting up Redis caching → Reference research/implementation-examples.md

**Later Phase (Research Less Relevant):**
- Contribution 012 adding another CRUD endpoint → Skip research review, focus on established patterns
- Contribution 015 refining existing UI components → Research likely not needed
