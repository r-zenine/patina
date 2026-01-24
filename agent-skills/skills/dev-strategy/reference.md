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
Understand existing architecture, patterns, and constraints using the onboarding agent.

### Process
1. **Always use Task tool** with `subagent_type: "onboarding-agent"`
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

⚠️ **Research is expensive and should be avoided unless absolutely necessary. Most implementation questions can be answered with existing knowledge.**

The research step should ONLY be executed when ALL of these conditions are met:

1. **User Explicitly Mentions New Technology**
   - User specifically names a framework, library, or tool they want to use
   - User asks to evaluate or compare specific technology options (e.g., "should we use GraphQL or REST?")

2. **Technology is Not in Current Codebase**
   - Onboarding agent confirms the technology is not currently used
   - No existing patterns in the codebase for this technology

3. **Claude Lacks Implementation Knowledge**
   - Claude needs to research how to implement the specific technology
   - Integration patterns with the existing tech stack are unclear

**Do NOT trigger research for:**
- General performance, security, or scalability concerns
- "Best practices" without specific technology choices
- Complex integrations using existing technologies
- Architecture decisions that can be made with current knowledge
- Hypothetical technology evaluations ("what's the best database?")
- **Most common scenarios - prefer simple solutions over research**

### Research Process

1. **Identify Research Topics**
   - List specific technologies, patterns, or concepts that need investigation
   - Prioritize by impact on implementation strategy
   - Focus on practical implementation details, not general concepts

2. **Conduct Research**
   - Use WebFetch to access official documentation
   - Use WebSearch for best practices, tutorials, and community discussions
   - Look for implementation examples relevant to the existing codebase
   - Research integration patterns and compatibility considerations

3. **Document Findings**
   - Create structured research artifacts using the templates
   - Focus on actionable insights that will influence implementation
   - Include code examples and configuration requirements
   - Document pros/cons and recommendations

### Research Areas to Focus On

**For New Technologies**:
- How to install and configure with existing tech stack
- Basic implementation patterns and code examples
- Integration approach with current codebase patterns

**Focus Only On:**
- Immediate implementation needs for the current feature
- Concrete code examples and configuration
- Direct compatibility with existing codebase patterns

**Avoid Researching:**
- General performance or scalability implications
- Security considerations beyond basic implementation
- Enterprise concerns like rollback procedures or team adoption

### Research Focus Principle

Research should answer "How do I implement this?" not "What's the best approach?". Focus on:
- Specific setup and configuration steps
- Basic integration patterns with existing codebase
- Minimal working examples
- Immediate implementation requirements only

Avoid broad evaluations, performance comparisons, or enterprise considerations.

### Research Artifacts Generated

When research is conducted, create the following files in `./dev-strategy-[FEATURE-NAME]/research/`:

1. **research-summary.md** - High-level findings and recommendations
2. **technology-research.md** - Focused analysis of each technology researched
3. **implementation-examples.md** - Code patterns and practical examples

### Research Quality Guidelines

- **Focus on implementation** - How to install, configure, and integrate with existing codebase
- **Include practical examples** - Basic code snippets and configuration examples
- **Document setup requirements** - Installation steps and dependencies needed
- **Note integration points** - How it connects with current tech stack

### Example Research Scenarios

**Scenario 1**: User wants to add real-time notifications
- Research WebSocket vs Server-Sent Events vs polling approaches
- Investigate libraries like Socket.io, native WebSocket APIs
- Research scaling considerations for real-time features
- Document integration patterns with existing authentication

**Scenario 2**: User wants to implement GraphQL
- Research GraphQL server implementations (Apollo, GraphQL Yoga)
- Investigate schema design best practices
- Research client-side integration approaches
- Document migration strategy from existing REST endpoints

**Scenario 3**: User wants to add advanced caching
- Research Redis vs in-memory vs CDN caching strategies
- Investigate cache invalidation patterns
- Research performance monitoring for cache effectiveness
- Document integration with existing data access patterns

## Step 4: Gather Constraints & Requirements

### Goal
Collect user preferences, constraints, and requirements through targeted questioning.

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
- "Should this integrate with existing user accounts/data?" (only if user management is involved)
- "Must this work with current API formats?" (only if API changes are needed)

**Simple Constraints:**
- "Any specific technologies you want to use or avoid?"
- "Any existing patterns I should follow from the codebase?"

### 4.3 Open-Ended Constraints

- "Any specific preferences for how this should work?"
- "Anything else I should know about your setup or requirements?"

## Step 5: Select Implementation Strategy

### Goal
Analyze feature characteristics and recommend the most appropriate development strategy.

### The Three Core Strategies

**Test-Driven Development (TDD)**
- *Description*: Write sufficient tests first, then implement code to pass tests
- *Process*: Test Design → Test Validation → Implementation
- *Best for*: Clear requirements, small-medium features, well-defined business rules
- *Timeline*: Usually 3 contribution phases
- *Example*: User login feature - write auth tests, then build auth system

**Steel Thread**
- *Description*: Build minimal end-to-end working version first, then expand capabilities
- *Process*: Implementation → Demonstration → Refactor (per phase)
- *Best for*: Uncertain requirements, multiple integration points, need early feedback
- *Timeline*: Usually 4-5 contribution phases
- *Example*: Basic login only, then add password reset, 2FA, session management
- *Key Requirement*: Each phase must clearly articulate the specific end-to-end path being built
- *Innovation*: Built-in refactor step prevents architectural debt accumulation

**Steel Thread Example - User Authentication**:
- *Phase 1*: User enters hardcoded credentials, sees "Login Success" (no database, no sessions)
- *Phase 2*: User login checks real database, creates session token (no password validation yet)
- *Phase 3*: Full password validation, error handling, secure session management
- *Phase 4*: Add password reset flow while maintaining working login

Each phase maintains a working end-to-end user experience.

### Just-In-Time Architecture Guidelines

**Purpose**: Build what you need now, refactor when complexity actually emerges.

**Core Principles**:
1. **Build for Current Phase Only**: Solve the immediate problem simply and directly
2. **Refactor When Patterns Emerge**: Wait until you have 2-3 concrete use cases before abstracting
3. **Let Usage Drive Design**: Architecture decisions should be informed by real usage, not speculation
4. **Simple Solutions Last Longer**: Resist the urge to prepare for hypothetical future requirements

**When to Refactor Between Phases**:
- **Actual Duplication**: When you're literally copying and pasting code
- **Concrete Integration Needs**: When the next phase requires a specific interface that doesn't exist
- **Breaking Changes**: When the current structure actively prevents the next phase
- **Never for Hypothetical Futures**: Don't refactor based on "what if" scenarios

**Just-In-Time Refactor Examples**:
- **Phase 1 → 2**: Only extract hardcoded values if Phase 2 actually needs different values
- **Phase 2 → 3**: Only add service layers if Phase 3 requires multiple implementations
- **Phase 3 → Production**: Only add monitoring if there are specific observability requirements

**Anti-Patterns to Avoid**:
- **Don't**: Create interfaces before you have multiple implementations
- **Don't**: Add configuration systems before you have actual variations to configure
- **Don't**: Build extension points before you know what will extend them
- **Do**: Keep it simple until complexity is actually needed

**Core-then-Integrate**
- *Description*: Build pure business logic first, then add external integrations
- *Process*: Domain Logic → Interface Design → Adapter Implementation → Integration
- *Best for*: Complex business rules, clean architecture goals, heavy external dependencies
- *Timeline*: Usually 4-6 contribution phases
- *Example*: Auth business logic + unit tests, then database, API, UI layers

### Strategy Selection Process

Apply these principles during strategy selection:

**Last Responsible Moment**: Don't commit to complex approaches without evidence they're needed
**Simple Design Rules**: Choose strategies that promote working, clear, non-duplicated, minimal solutions

1. **Analyze feature characteristics:**
   - **Requirements clarity**: Clear/essential → TDD; Uncertain → Steel Thread
   - **Business logic complexity**: High → Core-then-Integrate; Medium → TDD; Low → Steel Thread
   - **Integration complexity**: Heavy external deps → Core-then-Integrate; Multiple touchpoints → Steel Thread
   - **Risk level**: Critical business logic → Core-then-Integrate; User-facing → Steel Thread; Low risk → TDD

2. **Recommend 1-2 most suitable strategies** with rationale

3. **Present strategy comparison:**
   ```markdown
   ## Recommended Implementation Strategies

   ### Option 1: [Strategy Name]
   - **Rationale**: [Why this fits the feature]
   - **Process**: [Phase breakdown]
   - **Timeline**: [Number of contribution phases]
   - **Pros**: [Key advantages for this context]
   - **Cons**: [Potential drawbacks]
   ```

4. **Ask user to select preferred strategy**
5. **Document choice in decision log** with selected strategy and rationale

## Step 6: Create Actionable Plan

### Goal
Generate sufficient artifacts that provide everything needed for implementation, including research findings when applicable.

### Artifact 1: Code Context (`code-context.md`)

**Purpose**: Quick reference for relevant existing code

**Format**:
```markdown
# Code Context for [FEATURE NAME]

## Relevant Classes and Functions

- **ClassName.methodName** (`file_path:line_number`) - Brief description of functionality
- **UserService.createUser** (`src/services/user.js:42`) - Handles user creation with validation
- **AuthMiddleware.validateToken** (`src/middleware/auth.js:15`) - JWT token validation logic

## Key Files to Reference

- `src/models/user.js` - User data model and validation
- `src/config/database.js` - Database connection configuration
- `tests/auth.test.js` - Existing authentication tests
```

### Artifact 2: Context Document (`context-document.md`)

**Purpose**: Immutable reference information including research findings (doesn't change during implementation)

**Format**:
```markdown
# Context Document for [FEATURE NAME]

## Behavioral Specification
[Plain description of what to build]

## Codebase Patterns to Follow
- [e.g., Services in src/services/ with dependency injection]
- [e.g., Models use Joi for validation]
- [e.g., Tests follow Given/When/Then structure]

## Technical Constraints
- [e.g., Must use TypeScript for new code]
- [e.g., Database schema changes require migration scripts]
- [e.g., All endpoints must include rate limiting]

## Research Findings (if research step was executed)
- **Technologies Researched**: [List of new technologies investigated]
- **Key Insights**: [Important findings that influence implementation]
- **Recommended Approach**: [Research-informed recommendations]
- **Integration Considerations**: [How new technologies fit with existing codebase]
- **Risk Factors**: [Identified risks and mitigation strategies]

[For essential research findings, see ./research/ directory]
```

### Artifact 3: Decision Log (`decision-log.md`)

**Purpose**: Mutable record of decisions made (can be extended during implementation)

**Format**:
```markdown
# Decision Log for [FEATURE NAME]

## Strategy: Steel Thread
**Why**: Uncertain requirements, need early feedback
**Rejected**: TDD (requirements not clear enough)

## Tech: Use existing JWT + refresh tokens
**Why**: Maintains current auth patterns
**Rejected**: OAuth (adds complexity), Sessions (doesn't scale)

## Constraints: Maintain API compatibility
**Why**: Zero-downtime deployment required
**Impact**: Must version new endpoints

[Additional decisions added chronologically during implementation]
```

### Artifact 4: Implementation Roadmap (`implementation-roadmap.md`)

**Purpose**: Phase-by-phase execution plan with clear separation between implementation and design tasks

**Template Selection**:
- **Steel Thread Strategy**: Use `steel-thread-roadmap-template.md` for enhanced steel thread articulation
- **Other Strategies**: Use standard `implementation-roadmap-template.md`

**Format**:
```markdown
# Implementation Roadmap for [FEATURE NAME]

## Execution Strategy
[Selected strategy name and brief description]

## Phase 1: [Phase Title]
**Description**: [What to implement in this phase]

**Objectives**:
- **Implementation**: [Clear execution step following existing patterns]
- **Design**: [Design decision needed - to be refined based on learnings]

**Testing Criteria**:
- [How to verify objective 1 works]
- [How to verify objective 2 works]

**Dependencies**: None (or list phase numbers)

**Files to Modify**:
- `src/services/auth.js` - Add refresh token logic
- `src/middleware/auth.js` - Update token validation
- `tests/auth.test.js` - Add refresh token tests

## Phase 2: [Next Phase Title]
[Same structure as Phase 1]
```

**Steel Thread Enhanced Format**:
For Steel Thread strategy, each phase can contain multiple 3-step loops:

```markdown
## Phase X: [Phase Title]

### Steel Thread Scope
**End-to-End Path Being Built**: [Specific user journey this phase enables]
**User Experience After This Phase**: [What user can actually do/see]
**Phase Complexity**: [Simple (1 loop) or Complex (multiple loops)]

### Loop 1: [Loop Name]
#### Step 1: Implementation - [Build specific capability]
#### Step 2: Working Demonstration - [Prove functionality works]
#### Step 3: Refactor for Loop 2 - [Or "Refactor for Next Phase" if only one loop]

### Loop 2: [Loop Name] (if complex phase)
#### Step 1: Implementation - [Build next capability]
#### Step 2: Working Demonstration - [Prove enhanced functionality]
#### Step 3: Production Readiness - [If final loop of final phase]
```

**Phase Types**:
- **Simple Phase**: 1 loop, Step 3 = "Refactor for Next Phase"
- **Complex Phase**: Multiple loops, intermediate Step 3 = "Refactor for Loop N+1"
- **Final Phase**: Last loop Step 3 = "Production Readiness"

### File Organization

Create directory structure:
```
./dev-strategy-[FEATURE-NAME]/
├── code-context.md
├── context-document.md
├── decision-log.md
├── implementation-roadmap.md (use steel-thread-roadmap-template.md for Steel Thread strategy)
└── research/ (created when research step is executed)
    ├── research-summary.md
    ├── technology-research.md
    └── implementation-examples.md
```

Use a clear, descriptive feature name:
- ✅ `dev-strategy-user-authentication/`
- ✅ `dev-strategy-real-time-notifications/`
- ❌ `dev-strategy-2024-01-15/`
- ❌ `dev-strategy-feature1/`

## Implementation Tips

### Making Phases Actionable
- Each phase should be completable in 1-3 development sessions
- Include specific file paths that need modification
- Provide clear success criteria
- Consider dependencies between phases
- Separate **Implementation** objectives (ready to execute) from **Design** objectives (require further design decisions)

**Sufficient Planning Principle**: Provide enough detail for implementation without over-specifying. Perfect plans aren't needed - working software is the goal.

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
