---
name: dev-strategy
description: Creates sufficient implementation plans for complex coding projects. Use when users say "plan how to build", "create implementation roadmap", "break down this feature", "how should I approach building", or need structured development strategies for multi-phase projects, refactoring, or architectural changes.
allowed-tools: ["Task", "Read", "Write", "Grep", "Glob", "TodoWrite"]
---

# Dev Strategy Skill

Transform user requirements into actionable development roadmaps through structured planning and codebase analysis.

## What This Skill Does

Creates sufficient implementation plans by:
- Analyzing your codebase architecture and patterns
- Gathering constraints and requirements
- Proposing execution strategies
- Generating 4 structured artifacts for execution

## 6-Step Process

1. **Generate Behavioral Spec** - Plain description of what to build
2. **Analyze Codebase** - Use onboarding agent for context
3. **Technical Research** - Research new/unfamiliar technologies and approaches
4. **Gather Constraints** - Interactive requirements collection
5. **Select Implementation Strategy** - Choose development approach → consult [`dev-strategies` skill](../dev-strategies/SKILL.md)
6. **Create Plan** - Generate implementation artifacts → schemas in [`contribution-system` skill](../contribution-system/SKILL.md)

## Guiding Principles

For principles applied during planning (YAGNI, Last Responsible Moment, Kent Beck's 4 Rules, Sufficient Planning), see [`design-principles` skill](../design-principles/SKILL.md).

## Key Outputs

The skill generates structured artifacts in `./dev-strategy-[FEATURE-NAME]/`:

**Core Artifacts:**
- **Code Context** - Relevant classes/functions with line references
- **Context Document** - Behavioral spec + architecture summary + research findings
- **Decision Log** (YAML) - Technical decisions made during strategy development
- **Implementation Roadmap** - Phased execution plan with strategy

**Research Artifacts** (when research step is triggered):
- **Research Summary** - Key findings and recommendations
- **Technology Research** - Deep-dive analysis of new technologies
- **Implementation Examples** - Code patterns and usage examples

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

For step-by-step execution instructions, see [reference.md](reference.md).
