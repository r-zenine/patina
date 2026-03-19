---
name: dev-contribute
description: Enables structured contributions to dev-strategy implementation plans. Use when users say "contribute to dev-strategy", "implement this phase", "review the code", "audit the implementation", "add to the dev plan", or need to make any contribution to an existing dev-strategy with proper documentation and knowledge transfer.
allowed-tools: ["Read", "Write", "Grep", "Glob", "TodoWrite", "Edit", "MultiEdit", "Bash"]
---

# Dev Contribute Skill

Make structured contributions to dev-strategy implementation plans with proper documentation and knowledge transfer.

## What This Skill Does

Enables any agent to contribute to a dev-strategy by:
- Reading and understanding the existing plan (including research findings when relevant)
- Creating sequentially numbered contribution folders
- Generating mandatory documentation (decisions, context handoff)
- Maintaining project continuity and knowledge transfer
- Handling decision revisions when previous choices need revisiting

## ⚠️ MANDATORY Prerequisite: Understand contribution-system

**MANDATORY**: Before using this skill, load `contribution-system` skill to understand:
- Folder naming convention (NNN-phase-X-[type]-[specialty]-[agent])
- Artifact schemas (decision-log.yaml, context-handoff.md)
- Directory structure (contributions/ folder is required)
- Unified decision-log YAML schema
- What commit means and when to populate it (git hash of the commit containing code changes)
- Progressive disclosure strategy for reading contributions

All contributions created by this skill must conform to contribution-system conventions. This is not optional.

## When to Use This Skill

✅ **Use when:**
- Implementing code for a dev-strategy phase
- Reviewing existing implementations
- Auditing security, performance, or quality
- Testing implementations
- Documenting or analyzing dev-strategy work
- Any specialized contribution to an active dev-strategy

❌ **Don't use when:**
- Creating initial dev-strategy plans (use dev-strategy skill instead)
- Working on projects without dev-strategy plans
- Making quick fixes unrelated to strategic development

**💡 Tip**: Instead of manually invoking this skill, users can use **dev-continue** skill which automatically determines if design or implementation is needed next.

## Execution Requirements

To make a robust contribution to a dev-strategy plan, ensure these outcomes are achieved:

**FOUNDATION** (outcome: context is clear and phase is ready)
- Roadmap is understood (current phase objectives, what comes next)
- Prior decisions are known (what was decided in earlier phases, constraints they impose)
- Strategy is clear (TDD/Steel-Thread/Core-then-Integrate approach for this phase)
- Success criteria are identified (what deliverables complete this phase?)
- Phase is not blocked by prior decisions

**EXECUTION** (outcome: work is complete and quality bar is met)
- Phase deliverables are complete (all objectives for this phase achieved)
- Quality bar is met (code passes tests, reviews pass, security/performance requirements met)
- Decisions are captured (what trade-offs were made, why this approach was chosen?)
- Prior decisions still hold (no contradictions discovered with earlier choices)
- If prior decisions need revisiting, a revision contribution is created with reasoning

**KNOWLEDGE TRANSFER** (outcome: next phase is unblocked)
- Code is committed (all code changes committed before documentation is written — decision-log.yaml references commit hash)
- Decisions are recorded (decision-log.yaml populated with code_impacts showing what changed, commit field populated with git hash)
- Context is handed off (context-handoff.md tells next phase what to focus on and why)
- Contribution folder is committed (contribution folder with decision-log.yaml and context-handoff.md staged and committed)

**How to achieve these outcomes:**
1. Read existing dev-strategy plan (understand context, strategy, and current state)
2. Follow strategy approach (execute according to strategy → see [`dev-strategies` skill](../dev-strategies/SKILL.md))
3. Create numbered contribution folder (use sequential numbering → refer to [`contribution-system` skill](../contribution-system/SKILL.md) for naming)
4. Perform specialized work (implementation, review, audit, testing, or other contribution type)
5. **Commit code changes** (git commit with meaningful message — decision-log.yaml needs commit hash for code_impacts)
6. Generate mandatory documentation (decision-log.yaml and context-handoff.md + optional artifacts — populate commit field with git hash from step 5)
7. Handle decision revisions (if prior decisions need revisiting, create revision contribution with reasoning)

## Key Outputs

Every contribution generates 2 mandatory files (schemas in [`contribution-system/references/implementation-artifacts.md`](../contribution-system/references/implementation-artifacts.md)):
- **decision-log.yaml** - Technical decisions made during work (structured YAML format)
- **context-handoff.md** - Reasoning, exploration, and future guidance

## Core Principles

For all applicable principles (YAGNI, Sufficient Implementation, KISS, Last Responsible Moment), see [`design-principles` skill](../design-principles/SKILL.md).

**Phase Scoping**
- Each invocation must work on the **next incomplete phase** of the roadmap only
- Do not attempt multiple phases or skip ahead in the roadmap
- If a phase is blocked, communicate the blocker rather than proceeding anyway

**Minimize Code Generation in Documentation**
- **NEVER create code files** in contribution folders — these are for documentation only
- In documentation artifacts, explain concepts through clear descriptions
- Avoid code examples in documentation unless absolutely necessary

## Testing Guidelines

Follow Test Pyramid principles: many unit tests, some integration tests, few end-to-end tests.

For full Test Pyramid details, see [`dev-strategies/references/tdd.md`](../dev-strategies/references/tdd.md).

## Reading Contributions

Before making a contribution, read existing contributions using the progressive disclosure strategy in [`contribution-system/references/progressive-disclosure.md`](../contribution-system/references/progressive-disclosure.md).

## Additional Instructions

For essential instructions, strategy execution guidance, and templates, see [reference.md](reference.md).
