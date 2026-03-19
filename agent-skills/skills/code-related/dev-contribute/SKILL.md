---
name: dev-contribute
description: Enables structured contributions to dev-strategy implementation plans with proper documentation and knowledge transfer. Use when you need to "contribute to dev-strategy", "implement this phase", "review the code", "audit the implementation", "add to the dev plan", or make specialized contributions (testing, auditing, documenting) to existing dev-strategy roadmaps. Generates decision logs and context handoffs with mandatory documentation.
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

## Execution Overview

To make a robust contribution to a dev-strategy plan, three outcomes must be achieved:

**Outcome 1 (FOUNDATION)**: Context is clear and phase is ready
- Roadmap is understood (current phase objectives, what comes next)
- Prior decisions are known (what was decided in earlier phases, constraints they impose)
- Strategy is clear (TDD/Steel-Thread/Core-then-Integrate approach for this phase)
- Success criteria are identified (what deliverables complete this phase?)
- Phase is not blocked by prior decisions

**Outcome 2 (EXECUTION)**: Work is complete and quality bar is met
- Phase deliverables are complete (all objectives for this phase achieved)
- Quality bar is met (code passes tests, reviews pass, security/performance requirements met)
- Decisions are captured (what trade-offs were made, why this approach was chosen?)
- Prior decisions still hold (or a revision contribution is created with reasoning)

**Outcome 3 (KNOWLEDGE TRANSFER)**: Next phase is unblocked
- Code is committed (all code changes committed before documentation is written)
- Decisions are recorded (decision-log.yaml populated with code_impacts and commit hash)
- Context is handed off (context-handoff.md tells next phase what to focus on)
- Contribution folder is committed (staged and committed to git)

For the canonical execution requirements template with dev-contribute specifics, see [`contribution-system/references/execution-template.md`](../contribution-system/references/execution-template.md).

## Key Outputs

Every contribution generates 2 mandatory files:
- **decision-log.yaml** - Technical decisions made during work (structured YAML format)
- **context-handoff.md** - Reasoning, exploration, and future guidance

For artifact schemas, see [`contribution-system/references/implementation-artifacts.md`](../contribution-system/references/implementation-artifacts.md).

## How to Use This Skill

For essential instructions, strategy execution guidance, and templates, see [references/guide.md](references/guide.md).

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

For full Test Pyramid details, see [`execution-strategies/references/tdd.md`](../execution-strategies/references/tdd.md).

## See Also

**Related Skills:**
- **[dev-strategy](../dev-strategy/SKILL.md)** — To understand the plan and roadmap this contribution works on
- **[design-contribute](../design-contribute/SKILL.md)** — When a phase requires design work before implementation
- **[execution-strategies](../execution-strategies/SKILL.md)** — For strategy-specific implementation guidance (TDD, Steel Thread, Core-then-Integrate)
- **[design-principles](../design-principles/SKILL.md)** — To understand YAGNI, Sufficient Implementation, and other principles applied during work
- **[contribution-system](../contribution-system/SKILL.md)** — For artifact schemas, folder naming, and progressive disclosure patterns

## Prerequisites

Before using this skill, load [`contribution-system` skill](../contribution-system/SKILL.md) to understand:
- Folder naming convention (NNN-phase-X-[type]-[specialty]-[agent])
- Artifact schemas (decision-log.yaml, context-handoff.md)
- Directory structure (contributions/ folder is required)
- Unified decision-log YAML schema
- What commit means and when to populate it (git hash of the commit containing code changes)
- Progressive disclosure strategy for reading contributions (see [`contribution-system/references/progressive-disclosure.md`](../contribution-system/references/progressive-disclosure.md))

All contributions created by this skill must conform to contribution-system conventions.
