# Dev Contribute Skill - Reference Guide

This document provides instructions for contributing to dev-strategy implementation plans, organized by the outcome phases that define success.

## Table of Contents

- [Overview](#overview)
- [OUTCOME 1: Context is Clear and Phase is Ready](#outcome-1-context-is-clear-and-phase-is-ready)
  - [What You Must Verify](#what-you-must-verify)
  - [How to Achieve This Outcome](#how-to-achieve-this-outcome)
- [OUTCOME 2: Work is Complete and Quality Bar is Met](#outcome-2-work-is-complete-and-quality-bar-is-met)
  - [What You Must Verify (Outcome 2)](#what-you-must-verify-1)
  - [How to Achieve This Outcome (Outcome 2)](#how-to-achieve-this-outcome-1)
- [OUTCOME 3: Next Phase is Unblocked](#outcome-3-next-phase-is-unblocked)
  - [What You Must Verify (Outcome 3)](#what-you-must-verify-2)
  - [How to Achieve This Outcome (Outcome 3)](#how-to-achieve-this-outcome-2)
- [Reference: Common Patterns by Strategy](#reference-common-patterns-by-strategy)
- [Reference: Research Integration Examples](#reference-research-integration-examples)

## Overview

The dev-contribute skill enables structured contributions to existing dev-strategy plans. Each contribution follows the selected implementation strategy and creates proper documentation for knowledge transfer.

The process is organized into **three outcome phases**. Each phase has specific success criteria that must be verified before proceeding to the next phase.

---

## OUTCOME 1: Context is Clear and Phase is Ready

**What this outcome means:**
You understand the current roadmap state, the constraints from previous decisions, the execution strategy, and what success looks like for this phase. The phase is not blocked by prior work.

### What You Must Verify

Before proceeding to execution work, confirm all of these:

- [ ] Can you articulate the current phase objective without looking at the roadmap?
- [ ] Can you explain how this phase depends on previous decisions?
- [ ] Can you identify what success looks like for this phase?
- [ ] Can you confirm this phase is not blocked by prior work?
- [ ] Does the codebase baseline have zero build errors, zero linter warnings, zero test failures?

### How to Achieve This Outcome

#### Read and Understand Dev-Strategy Plan

**Goal:** Understand the context and strategic direction before contributing.

**Process:**

1. **Locate the dev-strategy plan** in `.plans/plan-[FEATURE-NAME]/`

2. **Read all dev-strategy artifacts** (located in `.plans/plan-[FEATURE-NAME]/`):
   - `code-context.md` - Relevant code references with line numbers
   - `context-document.md` - Behavioral spec and architecture (includes research findings if available)
   - `decision-log.yaml` - Previous decisions made during strategy
   - `implementation-roadmap.md` - Planned phases and strategy selection
   - `research/` directory (if present) - Technical research findings and recommendations

3. **Identify the implementation strategy:**
   - Look for strategy choice in `decision-log.yaml`
   - Understand the planned approach (TDD, Steel Thread, Core-then-Integrate)
   - For strategy details → see [`execution-strategies` skill](../execution-strategies/SKILL.md)

4. **Review research findings (when relevant):**
   - If implementing new technologies mentioned in research/, review those artifacts
   - For early-phase work, research artifacts are often valuable for implementation guidance
   - For later-phase work, research may be less relevant — use judgment

5. **Review existing contributions:**
   - Check `.plans/plan-[FEATURE-NAME]/contributions/` directory for previous work
   - Read recent `context-handoff.md` files to understand current state
   - Identify the next logical contribution number

6. **Check for reviewer instructions (review-state.json):**
   After reading existing contributions, look in the most recent contribution folder for `review-state.json`.

   If present:
   1. Read `instructions.instructions[]`
   2. Filter for entries where `status: "active"`
   3. Each active instruction contains: `file`, `line_range`, `content`, `query`
   4. **Active instructions MUST be addressed in this contribution**
   5. Use the `query` field with `git show <query>:<file>` or `git diff <query> <file>` to retrieve file content at the right git ref for context

   If absent or no active instructions, proceed normally.

#### Identify Your Phase (Phase Scoping)

**Goal:** Ensure each contribution focuses on exactly one phase of the roadmap.

**Mandatory Requirement:**
**Each invocation of dev-contribute must work on the NEXT INCOMPLETE PHASE ONLY.** Do not attempt multiple phases or skip ahead.

**Process:**

1. **Identify the next incomplete phase** in `implementation-roadmap.md`
2. **Confirm no previous contributions exist** for later phases
3. **Limit all work to this single phase** — do not proceed into subsequent phases
4. **If the phase is blocked**, communicate the blocker rather than skipping to the next phase

**Rationale:**
- Sequential workflow ensures predictable progress through the roadmap
- Single-phase contributions prevent scope creep
- Clear phase boundaries make progress visible and auditable

#### Validate Clean Baseline (Pre-Work Validation)

**Goal:** Ensure a clean, working baseline before starting contribution work.

**Mandatory Requirement:**
**The contribution CANNOT proceed if any of these checks fail.**

**Process:**

1. **Run compilation/build:** All compilation must succeed with zero errors
2. **Run linter:** All linting must pass with zero warnings
3. **Run test suite:** All tests must pass with zero failures

**If Validation Fails:**
- STOP immediately
- Document the failures found
- Inform the user that the codebase baseline is not clean
- Do NOT proceed with contribution until baseline is clean

**Rationale:**
Contributions must start from a clean, working baseline to ensure:
- New issues can be clearly attributed to the contribution
- Quality degradation is prevented
- Technical debt is not accumulated

---

## OUTCOME 2: Work is Complete and Quality Bar is Met

**What this outcome means:**
All phase deliverables are complete, code passes tests and linting, security/performance requirements are met, and decisions made during work are documented. Prior decisions still hold (or a revision contribution was created with reasoning).

### What You Must Verify

Before proceeding to documentation, confirm all of these:

- [ ] All phase deliverables from the roadmap are complete?
- [ ] Do all tests pass with zero failures?
- [ ] Does linting pass with zero warnings?
- [ ] Do code changes follow existing patterns from the codebase?
- [ ] Are prior decisions still valid, or have you identified ones needing revision?
- [ ] Is the implementation "sufficient" (works correctly, not over-polished)?

### How to Achieve This Outcome

#### Follow Strategy Approach

**Goal:** Execute according to the selected implementation strategy.

For complete strategy execution details, contribution sequences, and contribution type definitions, see the [`execution-strategies` skill](../execution-strategies/SKILL.md):

**Quick reference:**
- **TDD**: Test Design → Test Validation → Implementation (see [`execution-strategies/references/tdd.md`](../execution-strategies/references/tdd.md))
- **Steel Thread**: Pathfinder → Foundation Builder → Capability Expander (see [`execution-strategies/references/steel-thread.md`](../execution-strategies/references/steel-thread.md))
- **Core-then-Integrate**: Domain Modeler → Port Designer → Adapter Builder → Integration Orchestrator (see [`execution-strategies/references/core-then-integrate.md`](../execution-strategies/references/core-then-integrate.md))

#### Create Numbered Contribution Folder

**Goal:** Create sequentially numbered folder for chronological ordering under the plan's contributions directory.

**IMPORTANT: Contributions are ALWAYS saved in `.plans/plan-[FEATURE-NAME]/contributions/`**

All contributions for a dev-strategy plan are stored together in the plan's dedicated contributions directory. This ensures contributions remain organized and linked to their parent plan.

**Process:**

1. **Navigate to the plan's contributions directory:**
   ```bash
   cd .plans/plan-[FEATURE-NAME]/contributions/
   ls
   # 001-phase-1-implementation-code-general-purpose/
   # 002-phase-1-review-security-code-simplifier/
   ```

2. **Pick next number and create folder:**
   ```bash
   mkdir 003-phase-2-test-design-tdd-general-purpose/
   ```

For complete folder naming convention, contribution type definitions, and specialty list, see [`contribution-system` skill](../contribution-system/SKILL.md).

#### Perform Specialized Work

**Goal:** Execute the phase according to the selected strategy.

**Implementation Guidelines by Strategy:**
- **TDD** → [`execution-strategies/references/tdd.md`](../execution-strategies/references/tdd.md)
- **Steel Thread** → [`execution-strategies/references/steel-thread.md`](../execution-strategies/references/steel-thread.md)
- **Core-then-Integrate** → [`execution-strategies/references/core-then-integrate.md`](../execution-strategies/references/core-then-integrate.md)

**Quality Standards — All Contributions Must:**
- Follow existing code patterns from `context-document.md`
- Respect constraints from `decision-log.yaml`
- Maintain or improve test coverage
- Handle expected errors for the current scope
- Follow basic security practices from existing code

**Sufficient Implementation Principle:**
Focus on making it work correctly, not perfectly. Avoid over-polishing, excessive error handling for edge cases that won't occur, or adding features not required for the current phase. See [`design-principles` skill](../design-principles/SKILL.md) for full principles.

**When Research Artifacts Are Most Valuable:**
- First implementation of new technologies mentioned in research/
- Early phases where research decisions need implementation
- When encountering integration challenges covered in research findings
- Skip research review for routine work on well-established patterns

#### Commit Code and Record Commit Hash

**Goal:** Persist code changes with a commit hash for decision documentation.

After implementing changes and passing all quality checks:

```bash
git add .
git commit -m "Implementation of phase..."
```

After committing, get the commit hash and record it for the next step:

```bash
git rev-parse HEAD
```

**Important:** You will write this commit hash to `decision-log.yaml` in the next outcome phase. The commit hash references the exact commit containing your code changes, so diffviz can analyze the permanent diff correctly.

**Mandatory Requirement:** Commit all code changes before creating `decision-log.yaml`. Do not document decisions until code is committed.

---

## OUTCOME 3: Next Phase is Unblocked

**What this outcome means:**
Code is committed, all decisions (including those that need revision) are documented with reasoning and code impacts, and context is handed off clearly to enable the next phase to begin immediately.

### What You Must Verify

Before finalizing contribution, confirm all of these:

- [ ] Can another agent understand what was done and why?
- [ ] Are assumptions and blockers clearly documented?
- [ ] Does decision-log.yaml reference the commit hash with all code_impacts populated?
- [ ] Does context-handoff.md explain what works, what's fragile, and what's next?
- [ ] If a prior decision needed revision, is there a revision contribution with clear reasoning?

### How to Achieve This Outcome

#### Generate Mandatory Documentation

**Goal:** Create documentation for knowledge transfer and project continuity.

For complete artifact schemas and templates, see [`contribution-system/references/implementation-artifacts.md`](../contribution-system/references/implementation-artifacts.md).

**Create two required files:**

**1. Create decision-log.yaml**

First, generate the template by running:
```bash
diffviz templates decision-log > decision-log.yaml
```

Then fill in the decisions you made during this contribution:
- What trade-offs were made?
- Why was this approach chosen over alternatives?
- What code impacts resulted from this decision?

The template will look like this:
```yaml
commit: "<git-hash>"  # Git hash from Step 4.5

decisions:
  - number: 1
    title: "[One sentence summary]"
    rationale: "[Why...]"
    code_impacts:
      - file: "path/to/file.rs"
        reasoning: "[Why affected]"
        line_ranges:
          - start: 10
            end: 50
```

After filling in your decisions, validate with:
```bash
diffviz validate decision-log decision-log.yaml
```

**2. Create context-handoff.md**

Use [context-handoff-template.md](../contribution-system/assets/templates/context-handoff-template.md)

Lead with what you built and key insights. Structure as:
- **What works:** What was successfully completed
- **What's fragile:** What areas need care in next phase
- **What's missing:** What's intentionally deferred
- **Guidance for next:** Specific direction for next contributors

**Quality Checks:**

- Can next contributor understand what was done and why?
- Are assumptions and blockers clearly documented?
- Does each file serve its distinct purpose without overlap?

#### Handle Decision Revisions (if needed)

**Goal:** Provide clear workflow when you disagree with logged decisions from previous contributions.

**When to Use This:**
After reviewing a previous contribution's `decision-log.yaml`, if you discover a decision that needs revisiting.

**Process:**

1. **Create new revision contribution folder:**
   - Format: `NNN-phase-X-revision-[original-specialty]-[agent]`
   - Example: If revising contribution `002-phase-1-implementation-code-general-purpose/`, create `004-phase-1-revision-code-general-purpose/`
   - Use next sequential number, maintain phase and specialty context

2. **Update code to match revised decision:**
   - Make all necessary code changes to implement the new decision
   - Ensure changes are complete and align with the revised approach
   - Follow same quality standards as original contribution

3. **Document in new contribution's decision-log.yaml:**
   - Reference original contribution number
   - Explain what decision is being revised and why

4. **Generate all mandatory documentation:**
   - **decision-log.yaml**: New decision with revision context and reference to original
   - **context-handoff.md**: Explain impact of revision on existing work and future contributions

5. **Run final validation checks:**
   - Same mandatory quality gates as any contribution
   - Ensure build, lint, and tests all pass

**Key Principles:**
- **Always create new folder** — Don't edit previous contribution folders
- **Always update code** — Code changes are part of the revision contribution
- **Clear audit trail** — Decision evolution is visible through contribution history
- **Sequential numbering** — Revisions follow chronological order like any contribution

#### Final Quality Gate (Mandatory)

**Goal:** Ensure all work meets quality standards before completion.

**MANDATORY: The contribution CANNOT be completed if any of these checks fail.**

1. **Run compilation/build:** All compilation must succeed with zero errors
2. **Run linter:** All linting must pass with zero warnings
3. **Run test suite:** All tests must pass with zero failures

If any check fails: fix all issues before completing the contribution.

**Additional Validation:**

1. **Strategy Compliance Check:**
   - Does this follow the chosen strategy approach?
   - Are we in the right phase of the strategy?
   - Does this build logically on previous contributions?

2. **Quality Validation:**
   - All mandatory files created and complete?
   - Context handoff explains reasoning and alternatives?
   - Decision log captures new choices made?

3. **Knowledge Transfer:**
   - Would another agent understand what was done and why?
   - Are assumptions and limitations clearly documented?
   - Is guidance provided for next contributors?

#### Commit the Contribution

**Goal:** Persist all contribution artifacts to git with clear commit message.

After all validation checks pass (build, lint, tests green):

```bash
git add .plans/plan-[FEATURE-NAME]/contributions/<contribution-folder>/ <each modified source file by explicit path>
git commit -m "contrib(NNN): <description matching contribution folder name>"
```

**Rules:**
- Do NOT use `git add -A` or `git add .`
- Use the full path `.plans/plan-[FEATURE-NAME]/contributions/<contribution-folder>/` when staging
- Stage each changed source file explicitly by its full path
- The commit message number (NNN) must match the contribution folder number
- The description must match the contribution folder name (e.g., `phase-2-implementation-code-general-purpose`)

---

## Reference: Common Patterns by Strategy

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

## Reference: Research Integration Examples

**Early Phase (Research Relevant):**
- Contribution 001 implementing GraphQL for first time → Review research/technology-research.md
- Contribution 002 setting up Redis caching → Reference research/implementation-examples.md

**Later Phase (Research Less Relevant):**
- Contribution 012 adding another CRUD endpoint → Skip research review, focus on established patterns
- Contribution 015 refining existing UI components → Research likely not needed
