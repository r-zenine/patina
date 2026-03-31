# Coding Framework Overview

The coding-framework provides a structured workflow for planning and implementing complex software changes. It consists of **three primary skills** that work together sequentially, plus **two foundational reference skills** that encode design philosophy and contribution conventions.

## The Three Primary Skills

### 1. **dev-strategy** — Planning Skill

Creates sufficient implementation plans for complex coding projects by analyzing your codebase, gathering constraints, and proposing execution strategies.

**What it produces:**
- Code Context — Relevant classes/functions with line references
- Context Document — Behavioral spec + architecture summary + research findings
- Decision Log (YAML) — Technical decisions made during planning
- Implementation Roadmap — Phased execution plan with strategy selection
- Research artifacts — When technology research is needed

**Use this when:** Breaking down complex features, planning architectural changes, refactoring large systems, or understanding implementation trade-offs.

### 2. **design-contribute** — Design Skill

Enables collaborative, human-in-the-loop design sessions for roadmap phases with design objectives. Produces lean design docs (< 100 lines) through interactive exploration with the user.

**What it produces:**
- Design Doc — Lean specification for implementers
- Decision Log — Design decisions in unified YAML format
- Context Handoff — Problem solved + reading guide

**Use this when:** A roadmap phase has "Design" objectives, you need to determine architecture or approach before implementation, or previous implementation has revealed constraints that inform design choices.

### 3. **dev-contribute** — Development Skill

Enables structured contributions to dev-strategy implementation plans. Agents use this to implement phases, review code, audit security/performance, or handle specialized work while maintaining project continuity.

**What it produces:**
- Decision Log — Technical decisions made during work
- Context Handoff — Reasoning, exploration, and guidance for next phases
- Optional artifacts — Code examples, testing results, audit reports

**Use this when:** Implementing a dev-strategy phase, reviewing existing implementations, auditing code, testing implementations, or documenting dev-strategy work.

## The Two Supporting Skills

These skills are **canonical references** — not invoked directly, but fundamental to how the primary skills work:

- **contribution-system** — Defines artifact schemas, folder naming conventions, directory structure, templates, and workflow patterns (progressive disclosure, two-gate system, constraint discovery). All contributions must conform to its conventions.

- **design-principles** — Establishes the design philosophy underlying all skills: YAGNI (You Aren't Gonna Need It), Last Responsible Moment, Kent Beck's 4 Simple Design Rules, Sufficient Design/Implementation/Planning, and Just-In-Time Architecture.

## How They Work Together

1. Start with **dev-strategy** to create a structured plan
2. For phases with design objectives, use **design-contribute** to design collaboratively
3. Use **dev-contribute** to implement each phase following the roadmap
4. Repeat steps 2–3 for remaining phases
5. Reference **contribution-system** and **design-principles** throughout to maintain quality and consistency

## Important Context

The decision log artifacts use a unified YAML schema that directly matches the `diffviz-review::Decision` struct in the codebase. Get the canonical template via `diffviz templates decision-log`. This schema cannot be changed without corresponding updates to diffviz-cli's parsing logic.

