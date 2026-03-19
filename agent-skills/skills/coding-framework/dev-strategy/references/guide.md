# Dev Strategy Skill - Reference Guide

This document provides step-by-step instructions for executing the 6-step dev-strategy skill.

## Table of Contents

- [Step 1: Generate Behavioral Specification](#step-1-generate-behavioral-specification)
- [Step 2: Analyze Codebase](#step-2-analyze-codebase)
- [Step 3: Technical Research](#step-3-technical-research)
- [Step 4: Gather Constraints & Requirements](#step-4-gather-constraints--requirements)
- [Step 5: Select Implementation Strategy](#step-5-select-implementation-strategy)
- [Step 6: Create Actionable Plan](#step-6-create-actionable-plan)
- [Implementation Tips](#implementation-tips)
- [Common Patterns](#common-patterns)

## Step 1: Generate Behavioral Specification

### Goal
Create a plain, clear description of what needs to be built without technical implementation details.

### Process
1. **Parse user requirements** - Extract key functionality from user input
2. **Ask clarifying questions** to refine unclear aspects:
   - "What should happen when [edge case]?"
   - "Who are the main users of this feature?"
   - "What's the expected outcome?"
3. **Generate plain description** - Write 2-3 sentences describing what to build
4. **Avoid technical jargon** - Focus on WHAT, not HOW

### Example Output
```
Build a user authentication system that allows users to register with email/password,
log in securely, and maintain their session across browser refreshes. The system should
integrate with existing user profiles and respect current security requirements.
```

## Step 2: Analyze Codebase

### Goal
Understand existing architecture, patterns, and constraints using the Explore agent.

### Process
1. **Use Explore agent** to generate/update the crate's `onboarding.md` — see [`contribution-system/references/onboarding-generation.md`](../contribution-system/references/onboarding-generation.md) for invocation details
2. **Request specific analysis**:
   - Architecture patterns and frameworks in use
   - Key files, classes, and functions relevant to the task
   - Existing code conventions and patterns
   - Technical constraints from current implementation
   - Local repository-specific skills (in .claude/skills/, skills/, .skills/, etc.) with their descriptions and when to use them

### Example Task Prompt
```
Analyze this codebase focusing on [FEATURE AREA]. I need to understand:
- Current architecture patterns (MVC, microservices, etc.)
- Frameworks and libraries in use
- Authentication/authorization patterns
- Database and data access patterns
- Key classes and functions related to [SPECIFIC AREA]
- Code organization and naming conventions
- Any existing [FEATURE TYPE] implementations
```

### Key Information to Extract
- **Architecture type** (monolith, microservices, serverless, etc.)
- **Frameworks** (React, Vue, Express, Django, etc.)
- **Patterns** (Repository, Service Layer, MVC, etc.)
- **Key files** with line numbers for relevant code
- **Technical constraints** (TypeScript usage, database limitations, etc.)

## Step 3: Technical Research

### Goal
Research new technologies, unfamiliar concepts, or approaches that will be needed for implementation but aren't covered by existing codebase analysis.

### Research Trigger Conditions

⚠️ **Research is expensive and should be avoided unless absolutely necessary.**

The research step should ONLY be executed when ALL of these conditions are met:

1. **User Explicitly Mentions New Technology** - User specifically names a framework, library, or tool
2. **Technology is Not in Current Codebase** - Confirmed by Explore agent
3. **Claude Lacks Implementation Knowledge** - Integration patterns with existing stack are unclear

**Do NOT trigger research for:** general performance/security concerns, "best practices" without specific technology choices, architecture decisions that can be made with current knowledge, hypothetical technology evaluations.

See [`design-principles/references/anti-patterns.md`](../design-principles/references/anti-patterns.md) for the full list of research anti-patterns.

### Research Process

1. **Identify Research Topics** - List specific technologies that need investigation
2. **Conduct Research** - Use WebFetch for official docs, WebSearch for best practices
3. **Document Findings** - Create structured research artifacts using templates

### Research Focus Principle

Research should answer "How do I implement this?" not "What's the best approach?". Focus on:
- Specific setup and configuration steps
- Basic integration patterns with existing codebase
- Minimal working examples
- Immediate implementation requirements only

### Research Artifacts Generated

When research is conducted, create in `.plans/plan-[FEATURE-NAME]/research/`:
1. **research-summary.md** - High-level findings and recommendations
2. **technology-research.md** - Focused analysis of each technology researched
3. **implementation-examples.md** - Code patterns and practical examples

Use templates from [`contribution-system/assets/templates/`](../contribution-system/assets/templates/).

## Step 4: Gather Constraints & Requirements

### Goal
Collect user preferences, constraints, and requirements through targeted questioning.

For interactive questioning patterns, see [`contribution-system/references/constraint-discovery.md`](../contribution-system/references/constraint-discovery.md).

### 4.1 Structured Questions (Based on Codebase Findings)

Ask specific questions about choices discovered in the codebase:

**Framework Choices:**
- "I found both React and Vue components - which patterns should I follow?"
- "I see Express and FastAPI endpoints - which style should I use?"

**Database/Storage:**
- "I see both MongoDB and PostgreSQL - which should I use for new data?"
- "I notice existing migration scripts - are schema changes allowed?"

**Authentication:**
- "Multiple authentication methods found - which should I extend?"
- "I see JWT and session-based auth - which approach for new features?"

### 4.2 Essential Requirements Questions

Ask only when directly relevant to the feature:

**Integration Compatibility:**
- "Should this integrate with existing user accounts/data?"
- "Must this work with current API formats?"

**Simple Constraints:**
- "Any specific technologies you want to use or avoid?"
- "Any existing patterns I should follow from the codebase?"

### 4.3 Open-Ended Constraints

- "Any specific preferences for how this should work?"
- "Anything else I should know about your setup or requirements?"

## Step 5: Select Implementation Strategy

### Goal
Analyze feature characteristics and recommend the most appropriate development strategy.

For full strategy definitions, selection criteria, execution phases, and contribution folder naming patterns, see the [`execution-strategies` skill](../execution-strategies/SKILL.md).

**Quick selection guide:**

| Signal | Recommended Strategy |
|--------|---------------------|
| Clear requirements, small-medium feature | TDD |
| Uncertain requirements, need early feedback | Steel Thread |
| Complex business rules, clean architecture | Core-then-Integrate |

### Strategy Selection Process

1. **Analyze feature characteristics** using the selection guide above
2. **Recommend 1-2 most suitable strategies** with rationale
3. **Present strategy comparison** to user
4. **Ask user to select preferred strategy**
5. **Document choice in decision log** with rationale

## Step 6: Create Actionable Plan

### Goal
Generate sufficient artifacts that provide everything needed for implementation.

For artifact schemas and templates, see [`contribution-system` skill](../contribution-system/SKILL.md).

### Artifact 1: Code Context (`code-context.md`)

Use template: [`contribution-system/assets/templates/code-context-template.md`](../contribution-system/assets/templates/code-context-template.md)

**Purpose**: Quick reference for relevant existing code — classes, functions, key files with line numbers.

### Artifact 2: Context Document (`context-document.md`)

Use template: [`contribution-system/assets/templates/context-document-template.md`](../contribution-system/assets/templates/context-document-template.md)

**Purpose**: Immutable reference — behavioral spec, architecture summary, constraints, research findings.

### Artifact 3: Decision Log (`decision-log.yaml`)

Use template: [`contribution-system/assets/templates/decision-log-template.yaml`](../contribution-system/assets/templates/decision-log-template.yaml)

**Purpose**: Mutable record of decisions made. Document: selected strategy + rationale, rejected alternatives, key technology/architecture choices.

### Artifact 4: Implementation Roadmap (`implementation-roadmap.md`)

**Template selection:**
- Steel Thread strategy → use [`steel-thread-roadmap-template.md`](../contribution-system/assets/templates/steel-thread-roadmap-template.md)
- Other strategies → use [`implementation-roadmap-template.md`](../contribution-system/assets/templates/implementation-roadmap-template.md)

**Purpose**: Phase-by-phase execution plan. Each phase includes objectives (Implementation and Design), testing criteria, dependencies, and files to modify.

**Key**: Design objectives are deferred to `design-contribute` via the Two-Gate System — see [`contribution-system/references/two-gate-system.md`](../contribution-system/references/two-gate-system.md).

### File Organization

```
.plans/plan-[FEATURE-NAME]/
├── code-context.md
├── context-document.md
├── decision-log.yaml
├── implementation-roadmap.md
└── research/ (only when research step is executed)
    ├── research-summary.md
    ├── technology-research.md
    └── implementation-examples.md
```

Use clear, descriptive feature names with `plan-` prefix:
- ✅ `.plans/plan-user-authentication/`
- ✅ `.plans/plan-real-time-notifications/`
- ❌ `.plans/plan-2024-01-15/`
- ❌ `.plans/plan-feature1/`

### Commit the strategy artifacts

After all artifacts are created in `.plans/plan-<feature-name>/`:
```bash
git add .plans/plan-<feature-name>/
git commit -m "plan: <feature name>"
```

## Implementation Tips

### Making Phases Actionable
- Each phase should be completable in 1-3 development sessions
- Include specific file paths that need modification
- Provide clear success criteria
- Consider dependencies between phases
- Separate **Implementation** objectives (ready to execute) from **Design** objectives (require further design decisions)

### Writing Good Objectives

**Implementation Objectives** (clear execution following existing patterns):
- ✅ "Add JWT refresh token endpoint to AuthController"
- ✅ "Update frontend login form to handle refresh tokens"
- ❌ "Improve authentication"

**Design Objectives** (require design decisions to be refined later):
- ✅ "Design error handling strategy for auth failures"
- ✅ "Determine optimal session management approach"
- ❌ "Make it better"

### Effective Testing Criteria
- ✅ "User can log in and receive both access and refresh tokens"
- ✅ "Expired access token automatically refreshes without user action"
- ❌ "Authentication works"
- ❌ "No bugs"

## Common Patterns

### For New Features
1. **Setup/Infrastructure** - Add necessary dependencies, config
2. **Core Logic** - Implement main functionality
3. **Integration** - Connect to existing systems
4. **UI/API** - Add user-facing interfaces
5. **Testing & Polish** - Sufficient testing, error handling

### For Refactoring
1. **Preparation** - Add tests for existing behavior
2. **Extract** - Move code to new structure
3. **Replace** - Update callers to use new structure
4. **Cleanup** - Remove old code
5. **Validation** - Ensure no regressions

### For Bug Fixes
1. **Reproduce** - Create test that reproduces the bug
2. **Investigate** - Understand root cause
3. **Fix** - Implement solution
4. **Test** - Verify fix and no regressions
5. **Monitor** - Plan for monitoring in production
