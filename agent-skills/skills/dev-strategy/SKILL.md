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
5. **Select Implementation Strategy** - Choose development approach (TDD, Steel Thread, Core-then-Integrate)
6. **Create Plan** - Generate implementation artifacts with strategy and research findings

## Core Planning Principles

**YAGNI (You Aren't Gonna Need It) - Primary Principle**
- Build only what is explicitly required for the current feature
- Don't add functionality, abstractions, or complexity for hypothetical future needs
- If it's not needed now, don't plan for it
- Prefer simple solutions that solve the immediate problem

**Minimize Code Generation**
- **NEVER generate code examples** in context documents, implementation roadmaps, or research artifacts
- Explain concepts through clear descriptions, architectural patterns, and integration approaches
- Only include code snippets if a concept absolutely cannot be explained in plain language (extremely rare)
- Focus on "what to build" and "how it connects" rather than "exact implementation details"

**Last Responsible Moment**
- Defer architectural and technology decisions until you have maximum information
- Don't commit to complex solutions before understanding the full problem
- Wait for patterns to emerge naturally before abstracting

**Simple Design Rules (Kent Beck)**
1. **Passes tests** - The code works as required
2. **Reveals intent** - Code clearly expresses what it does
3. **No duplication** - Don't repeat logic or structure
4. **Minimal elements** - Use the fewest classes, methods, and abstractions possible

## Development Strategies

**Test-Driven Development (TDD)**
- Write tests first, implement to pass tests
- Example: User login - write login test, then build auth system
- Best for: Clear requirements, small-medium features

**Steel Thread**
- Build minimal end-to-end working version, then expand
- Example: Phase 1: Hardcoded login → Phase 2: Real database → Phase 3: Validation
- Best for: Uncertain requirements, need early feedback
- Key: Each phase clearly defines the specific end-to-end path being built
- Innovation: Built-in refactor step prevents architectural debt

**Core-then-Integrate**
- Build pure business logic first, then add external integrations
- Example: Auth logic + unit tests, then database, API, UI layers
- Best for: Complex business rules, clean architecture goals

## Key Outputs

The skill generates structured artifacts in `./dev-strategy-[FEATURE-NAME]/`:

**Core Artifacts:**
- **Code Context** - Relevant classes/functions with line references
- **Context Document** - Behavioral spec + architecture summary + research findings
- **Decision Log** (YAML) - Technical decisions made during strategy development (structured YAML format)
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

## Planning Philosophy

**Build Sufficient Plans, Not Perfect Plans**: The goal is actionable guidance that enables implementation, not comprehensive documentation. Focus on decisions and information that directly impact building the feature.

## Additional Instructions

For essential instructions, examples, and templates, see [reference.md](reference.md).
