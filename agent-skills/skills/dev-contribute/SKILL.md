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

## Process Overview

1. **Read existing dev-strategy plan** - Understand context, strategy, and current state
2. **Follow strategy approach** - Execute according to TDD, Steel Thread, or Core-then-Integrate
3. **Create numbered contribution folder** - Sequential numbering for chronological order
4. **Perform specialized work** - Implementation, review, audit, testing, etc.
5. **Generate mandatory documentation** - 3 required files + optional artifacts

## Key Outputs

Every contribution generates:
- **changelog.md** - High-level impact and accomplishments
- **decision-log.md** - New decisions made during work
- **context-handoff.md** - Reasoning, exploration, and future guidance

## Directory Structure

Creates: `contributions/NNN-phase-X-[type]-[specialty]-[agent]/`
Example: `contributions/003-phase-2-implementation-frontend-general-purpose/`

## Core Principles for All Contributions

**YAGNI (You Aren't Gonna Need It) - Primary Principle**
- Build only what the current phase explicitly requires
- Don't add features, abstractions, or complexity for hypothetical future needs
- If it's not needed now, don't implement it
- Choose the simplest solution that solves the immediate problem

**Minimize Code Generation in Documentation**
- **NEVER create code files** in contribution folders - these are for documentation only
- In documentation artifacts (changelog, decision-log, context-handoff), explain concepts through clear descriptions
- Avoid code examples in documentation unless absolutely necessary to explain something that cannot be described in words
- Focus on "what was built", "why it was built that way", and "what comes next" rather than implementation details

**Sufficient Implementation**
- Focus on making it work correctly, not perfect
- Avoid over-polishing or adding unnecessary refinements
- Document decisions, don't over-explain obvious choices

**KISS (Keep It Simple, Stupid)**
- Choose the simplest solution that works
- Avoid clever code, complex patterns, or over-engineering
- Prefer clarity and directness over sophistication

**Last Responsible Moment**
- Defer decisions until you have maximum information
- Don't commit to approaches before you understand the full context
- Wait to see patterns before abstracting

## Implementation Strategies

Contributions must follow the strategy defined in the dev-strategy plan:

**TDD**: Test Design → Test Validation → Implementation
**Steel Thread**: Foundation → Basic Flow → Capability Expansion
**Core-then-Integrate**: Domain Logic → Interface Design → Adapter Implementation

## Testing Guidelines

When creating tests in any contribution, follow Test Pyramid principles (Martin Fowler): many unit tests, some integration tests, few end-to-end tests.

**Sufficient Coverage Principle**: Write enough tests to verify the feature works correctly. More tests are not always better - focus on testing the behavior that matters, not achieving perfect coverage percentages.

See [reference.md](reference.md) for essential guidelines.

## Reading Contributions

To understand existing contributions, use the [read-contribution skill](../read-contribution/SKILL.md) which provides progressive disclosure strategies for different roles and contexts.

## Additional Instructions

For essential instructions, strategy execution guidance, and templates, see [reference.md](reference.md).

For strategy definitions and selection criteria, see [dev-strategy skill documentation](../dev-strategy/SKILL.md).
