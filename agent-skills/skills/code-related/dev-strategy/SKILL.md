---
name: dev-strategy
description: Creates sufficient implementation plans for complex coding projects by analyzing your codebase, gathering constraints, and proposing execution strategies. Use when you need to "plan how to build", "create implementation roadmap", "break down this feature", "how should I approach building", or structure multi-phase development, refactoring, or architectural changes. Produces code context, behavioral specs, decision logs, and implementation roadmaps.
skills-depend-on:
  - contribution-system
  - design-principles
  - execution-strategies
---

# Dev Strategy Skill

Transform user requirements into actionable development roadmaps through structured planning and codebase analysis.

## What This Skill Does

Creates sufficient implementation plans by:
- Analyzing your codebase architecture and patterns
- Gathering constraints and requirements
- Proposing execution strategies
- Generating structured artifacts for implementation teams

## Execution Overview

To produce a sufficient implementation plan, this skill progresses through three outcomes:

**Outcome 1 (UNDERSTANDING)**: Context is clear
- Behavioral spec is clear (user confirms what to build)
- Codebase architecture is understood
- Technology knowledge is sufficient
- Constraints are discovered

**Outcome 2 (STRATEGY)**: Approach is selected
- Best strategy is chosen based on project characteristics
- Strategy aligns with constraints and requirements
- Trade-offs between strategies are understood

**Outcome 3 (PLANNING)**: Execution is clear
- All 4 core artifacts exist and are ready
- Roadmap is phased appropriately (each phase is a complete deliverable)
- First phase has sufficient detail to begin work
- Future phases are deferred appropriately per Last Responsible Moment principle

For the canonical execution requirements template with skill-specific variations, see [`contribution-system/references/execution-template.md`](../contribution-system/references/execution-template.md).

## Detailed Execution Requirements

To produce a sufficient implementation plan, ensure these outcomes are achieved:

**UNDERSTANDING PHASE** (outcome: context is clear)
- Behavioral spec is clear (user confirms what to build)
- Codebase architecture is understood (relevant classes/functions identified)
- Technology knowledge is sufficient (research done only if unfamiliar tech is involved)
- Constraints are discovered (user confirms project boundaries, dependencies, deadlines)

**STRATEGY PHASE** (outcome: approach is selected)
- Best strategy is chosen based on project characteristics (consult [`execution-strategies` skill](../execution-strategies/SKILL.md))
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
5. Select Implementation Strategy based on findings → refer to [`execution-strategies` skill](../execution-strategies/SKILL.md)
6. Create Plan artifacts following schemas in [`contribution-system` skill](../contribution-system/SKILL.md)

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

## Guiding Principles

For principles applied during planning (YAGNI, Last Responsible Moment, Kent Beck's 4 Rules, Sufficient Planning), see [`design-principles` skill](../design-principles/SKILL.md).

## How to Use This Skill

For step-by-step execution instructions, see [references/guide.md](references/guide.md).

## See Also

**Related Skills:**
- **[execution-strategies](../execution-strategies/SKILL.md)** — To compare TDD, Steel Thread, and Core-then-Integrate approaches during strategy selection
- **[dev-contribute](../dev-contribute/SKILL.md)** — To implement the planned phases once dev-strategy is complete
- **[design-contribute](../design-contribute/SKILL.md)** — When a planned phase has design objectives that need collaborative refinement
- **[design-principles](../design-principles/SKILL.md)** — To understand YAGNI, Last Responsible Moment, and other principles applied during planning
- **[contribution-system](../contribution-system/SKILL.md)** — For artifact schemas and folder naming conventions


