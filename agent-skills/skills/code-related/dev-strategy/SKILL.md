---
name: dev-strategy
description: Creates sufficient implementation plans for complex coding projects. Use when users say "plan how to build", "create implementation roadmap", "break down this feature", "how should I approach building", or need structured development strategies for multi-phase projects, refactoring, or architectural changes.
---

# Dev Strategy Skill

Transform user requirements into actionable development roadmaps through structured planning and codebase analysis.

## What This Skill Does

Creates sufficient implementation plans by:
- Analyzing your codebase architecture and patterns
- Gathering constraints and requirements
- Proposing execution strategies
- Generating 4 structured artifacts for execution

## Execution Requirements

To produce a sufficient implementation plan, ensure these outcomes are achieved:

**UNDERSTANDING PHASE** (outcome: context is clear)
- Behavioral spec is clear (user confirms what to build)
- Codebase architecture is understood (relevant classes/functions identified)
- Technology knowledge is sufficient (research done only if unfamiliar tech is involved)
- Constraints are discovered (user confirms project boundaries, dependencies, deadlines)

**STRATEGY PHASE** (outcome: approach is selected)
- Best strategy is chosen based on project characteristics (consult [`dev-strategies` skill](../dev-strategies/SKILL.md))
- Strategy aligns with constraints and requirements
- Trade-offs between strategies are understood

**PLANNING PHASE** (outcome: execution is clear)
- All 4 core artifacts exist (code-context.md, context-document.md, decision-log.yaml, implementation-roadmap.md)
- Roadmap is phased appropriately (each phase is a complete deliverable)
- First phase has sufficient detail to begin work
- Future phases are deferred appropriately per Last Responsible Moment principle

**How to achieve these outcomes:**
1. Generate Behavioral Spec through user discussion or codebase analysis
2. Analyze Codebase using Explore agent with onboarding prompt
3. Conduct Technical Research only if new/unfamiliar technologies are involved
4. Gather Constraints through interactive discovery (see [`constraint-discovery.md`](../contribution-system/references/constraint-discovery.md))
5. Select Implementation Strategy based on findings → refer to [`dev-strategies` skill](../dev-strategies/SKILL.md)
6. Create Plan artifacts following schemas in [`contribution-system` skill](../contribution-system/SKILL.md)

## Guiding Principles

For principles applied during planning (YAGNI, Last Responsible Moment, Kent Beck's 4 Rules, Sufficient Planning), see [`design-principles` skill](../design-principles/SKILL.md).

## Key Outputs

The skill generates structured artifacts in `.plans/plan-[FEATURE-NAME]/`:

**Core Artifacts:**
- **Code Context** - Relevant classes/functions with line references
- **Context Document** - Behavioral spec + architecture summary + research findings
- **Decision Log** (YAML) - Technical decisions made during strategy development
- **Implementation Roadmap** - Phased execution plan with strategy

**Research Artifacts** (when research step is triggered):
- **Research Summary** - Key findings and recommendations
- **Technology Research** - Deep-dive analysis of new technologies
- **Implementation Examples** - Code patterns and usage examples

## ⚠️ Prerequisite: Understand contribution-system

**MANDATORY**: Before using this skill, read [`contribution-system` skill](../contribution-system/SKILL.md) to understand:
- Artifact schemas (code-context.md, context-document.md, decision-log.yaml, implementation-roadmap.md)
- How decisions are documented (unified YAML schema)
- How contributions are structured (contributions/ folder, NNN-phase-X naming)
- Where artifacts are created (plan root vs contribution folders)

All artifacts created by this skill must conform to contribution-system conventions.

## When to Use This Skill

✅ **Use for:**
- Complex features requiring multiple phases
- Architectural changes or refactoring
- Breaking down large development tasks
- Understanding implementation trade-offs
- Projects needing structured planning

❌ **Don't use for:**
- Simple, single-step tasks
- Pure research or documentation questions
- Quick bug fixes or minor changes

## Additional Instructions

For step-by-step execution instructions, see [references/guide.md](references/guide.md).
