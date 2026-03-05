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
- Generating mandatory documentation (changelog, decisions, context handoff)
- Maintaining project continuity and knowledge transfer
- Handling decision revisions when previous choices need revisiting

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

## Process Overview

1. **Read existing dev-strategy plan** - Understand context, strategy, and current state
2. **Follow strategy approach** - Execute according to strategy → see [`dev-strategies` skill](../dev-strategies/SKILL.md)
3. **Create numbered contribution folder** - Sequential numbering for chronological order → folder naming in [`contribution-artifacts` skill](../contribution-artifacts/SKILL.md)
4. **Perform specialized work** - Implementation, review, audit, testing, etc.
5. **Generate mandatory documentation** - 3 required files + optional artifacts
6. **Handle decision revisions** - When decisions need revisiting, create new revision contribution

## Key Outputs

Every contribution generates 3 mandatory files (schemas in [`contribution-artifacts/references/implementation-artifacts.md`](../contribution-artifacts/references/implementation-artifacts.md)):
- **changelog.md** - High-level impact and accomplishments
- **decision-log.yaml** - Technical decisions made during work (structured YAML format)
- **context-handoff.md** - Reasoning, exploration, and future guidance

## Core Principles

For all applicable principles (YAGNI, Sufficient Implementation, KISS, Last Responsible Moment), see [`design-principles` skill](../design-principles/SKILL.md).

**Minimize Code Generation in Documentation**
- **NEVER create code files** in contribution folders — these are for documentation only
- In documentation artifacts, explain concepts through clear descriptions
- Avoid code examples in documentation unless absolutely necessary

## Testing Guidelines

Follow Test Pyramid principles: many unit tests, some integration tests, few end-to-end tests.

For full Test Pyramid details, see [`dev-strategies/references/tdd.md`](../dev-strategies/references/tdd.md).

## Reading Contributions

To understand existing contributions, use the [read-contribution skill](../read-contribution/SKILL.md) which provides progressive disclosure strategies for different roles.

## Additional Instructions

For essential instructions, strategy execution guidance, and templates, see [reference.md](reference.md).
