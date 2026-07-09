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

Each criterion below is either confirmed by the user or checkable against an artifact. These criteria are owned here; references restate them for execution convenience only.

**Outcome 1 (UNDERSTANDING)**: Context is clear
- Behavioral spec is confirmed by the user
- Draft `code-context.md` exists, naming the files/functions the change will touch, with line references
- Constraint list is recorded in `context-document.md` and explicitly confirmed by the user
- (Technical research is governed by the trigger conditions in [references/tactics.md](references/tactics.md) — there is no standing "research done" criterion)

**Outcome 2 (STRATEGY)**: Approach is selected
- User selected a strategy (consult [`execution-strategies` skill](../execution-strategies/SKILL.md))
- `decision-log.yaml` records the choice, rationale, and rejected alternatives with one-line reasons

**Outcome 3 (PLANNING)**: Execution is clear
- All 4 core artifacts exist (code-context.md, context-document.md, decision-log.yaml, implementation-roadmap.md) and respect their templates
- Roadmap is phased appropriately (each phase is a complete deliverable)
- First phase lists concrete file paths and testing criteria
- Future phases are deferred appropriately per Last Responsible Moment principle

For the generic outcome pattern shared by all skills, see [`contribution-system/references/execution-template.md`](../contribution-system/references/execution-template.md).

## Key Outputs

The skill generates only the following structured artifacts in `.plans/plan-[FEATURE-NAME]/`:

**Core Artifacts:**
- **Code Context** - Relevant classes/functions with line references
- **Context Document** - Behavioral spec + architecture summary + research findings
- **Decision Log** (YAML) - Technical decisions made during strategy development
- **Implementation Roadmap** - Phased execution plan with strategy

**Research Artifacts** (when research tactic is applied):
- **Research Summary** - Key findings and recommendations
- **Technology Research** - Deep-dive analysis of new technologies
- **Implementation Examples** - Code patterns and usage examples

## Guiding Principles

For principles applied during planning (YAGNI, Last Responsible Moment, Kent Beck's 4 Rules, Sufficient Planning), see [`design-principles` skill](../design-principles/SKILL.md).

**Token Efficiency** — a plan's cost is not just the planning session; every artifact is re-read by every downstream contribution:

- **Delegate bulk reading.** Codebase analysis and technical research go through sub-agents (Explore for code, general-purpose for research) that return distilled findings — raw file contents and web-search transcripts must not accumulate in the planning session.
- **Artifacts are paid for once per phase.** A line in `context-document.md` or `code-context.md` is re-read by every contribution, so an N-phase plan pays for it N times. Write artifacts lean: distill, don't transcribe; link to code by path and line range instead of quoting it.
- **Size phases for a single fresh session.** Each phase must be completable by one dev-contribute invocation in a fresh context window without approaching the context limit (~100k tokens of working room). A phase that needs more is two phases.
- **Make phases self-contained in the roadmap.** Each phase entry should carry the file paths, constraints, and success criteria the implementer needs, so a contribution can start from a compact brief instead of re-reading the whole plan.

## How to Use This Skill

For tactics available for each outcome, see [references/tactics.md](references/tactics.md).

## See Also

**Related Skills:**
- **[execution-strategies](../execution-strategies/SKILL.md)** — To compare TDD, Steel Thread, and Core-then-Integrate approaches during strategy selection
- **[dev-contribute](../dev-contribute/SKILL.md)** — To implement the planned phases once dev-strategy is complete
- **[design-contribute](../design-contribute/SKILL.md)** — When a planned phase has design objectives that need collaborative refinement
- **[design-principles](../design-principles/SKILL.md)** — To understand YAGNI, Last Responsible Moment, and other principles applied during planning
- **[contribution-system](../contribution-system/SKILL.md)** — For artifact schemas and folder naming conventions


