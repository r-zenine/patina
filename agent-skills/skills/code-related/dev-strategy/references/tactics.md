# Dev Strategy Skill — Tactics Reference

Tactics available for reaching each dev-strategy outcome.
The agent chooses which tactics to apply and in what order based on context.

---

## UNDERSTANDING Outcome

Done when:
- Behavioral spec is confirmed by the user
- Codebase architecture is understood (relevant classes/functions identified)
- Technology knowledge is sufficient
- Constraints are discovered and confirmed

### Tactic: Clarify Behavioral Specification

**When to use**: Requirements are ambiguous, vague, or leave key behaviors undefined.

Ask clarifying questions to surface unclear aspects:
- "What should happen when [edge case]?"
- "Who are the main users of this feature?"
- "What's the expected outcome?"

Then produce a 2–3 sentence plain description of what to build. Focus on WHAT, not HOW. Avoid technical jargon.

**Example output:**
```
Build a user authentication system that allows users to register with email/password,
log in securely, and maintain their session across browser refreshes. The system should
integrate with existing user profiles and respect current security requirements.
```

---

### Tactic: Codebase Analysis (Explore Agent)

**When to use**: Architecture and relevant code are not yet understood.

Use the Explore agent to generate or update the crate's `onboarding.md` — see [`contribution-system/references/onboarding-generation.md`](../contribution-system/references/onboarding-generation.md) for invocation details.

Request analysis focused on:
- Architecture patterns and frameworks in use
- Key files, classes, and functions relevant to the task
- Existing code conventions and patterns
- Technical constraints from current implementation
- Local repository-specific skills (in `.claude/skills/`, `skills/`, `.skills/`, etc.) with their descriptions

Key information to extract:
- **Architecture type** (monolith, microservices, serverless, etc.)
- **Frameworks** (React, Vue, Express, Django, etc.)
- **Patterns** (Repository, Service Layer, MVC, etc.)
- **Key files** with line numbers for relevant code
- **Technical constraints** (TypeScript usage, database limitations, etc.)

---

### Tactic: Domain Grilling Session

**When to use**: The user's stated mental model may conflict with what the code shows, or terminology in the requirements is fuzzy or overloaded (e.g., "account" could mean two different things).

Challenge the user's understanding against codebase reality:
- Ask one question at a time, wait for the answer before continuing
- If a question can be answered by reading the code, read the code instead of asking
- When the user states how something works, verify against code and surface contradictions: "Your description says X owns Y, but the code has Z owning Y — which is right?"
- When a term is fuzzy or overloaded, propose a precise canonical name: "You said 'account' — do you mean `User` or `Profile`? They're different structs"
- Stop when UNDERSTANDING done-criteria are met, not when all possible questions are exhausted

---

### Tactic: Technical Research

**When to use**: ALL THREE trigger conditions are met:
1. The user explicitly names a specific framework, library, or tool
2. That technology is not present in the current codebase (confirmed by Explore agent)
3. Integration patterns with the existing stack are unclear

**Do NOT trigger for**: general performance/security concerns, "best practices" without specific technology choices, architecture decisions resolvable with current knowledge, hypothetical technology evaluations.

See [`design-principles/references/anti-patterns.md`](../design-principles/references/anti-patterns.md) for the full list of research anti-patterns.

**Research focus**: Answer "How do I implement this?" not "What's the best approach?". Focus on specific setup steps, minimal integration patterns, and immediate implementation requirements only.

**Research artifacts** — create in `.plans/plan-[FEATURE-NAME]/research/`:
- `research-summary.md` — high-level findings and recommendations
- `technology-research.md` — focused analysis of each technology researched
- `implementation-examples.md` — code patterns and practical examples

Use templates from [`contribution-system/assets/templates/`](../contribution-system/assets/templates/).

---

### Tactic: Constraint Discovery Questioning

**When to use**: Constraints are unclear after codebase analysis, or codebase analysis surfaced ambiguous choices.

Ask specific questions based on codebase findings:

**Framework choices:**
- "I found both React and Vue components — which patterns should I follow?"
- "I see Express and FastAPI endpoints — which style should I use?"

**Database/Storage:**
- "I see both MongoDB and PostgreSQL — which should I use for new data?"
- "I notice existing migration scripts — are schema changes allowed?"

**Authentication:**
- "Multiple authentication methods found — which should I extend?"

**Integration compatibility:**
- "Should this integrate with existing user accounts/data?"
- "Must this work with current API formats?"

**Open-ended:**
- "Any specific technologies you want to use or avoid?"
- "Anything else I should know about your setup or requirements?"

For interactive questioning patterns, see [`contribution-system/references/constraint-discovery.md`](../contribution-system/references/constraint-discovery.md).

---

## STRATEGY Outcome

Done when:
- Best strategy is chosen based on project characteristics
- Strategy aligns with constraints and requirements
- Trade-offs are understood and documented in the decision log
- User has confirmed the choice

### Tactic: Strategy Selection

**When to use**: UNDERSTANDING outcome is met and an implementation approach must be chosen.

Quick selection guide:

| Signal | Recommended Strategy |
|--------|---------------------|
| Clear requirements, small-medium feature | TDD |
| Uncertain requirements, need early feedback | Steel Thread |
| Complex business rules, clean architecture | Core-then-Integrate |

For full strategy definitions, selection criteria, execution phases, and contribution folder naming patterns, see the [`execution-strategies` skill](../execution-strategies/SKILL.md).

**Selection process:**
1. Analyze feature characteristics using the table above
2. Recommend 1–2 most suitable strategies with rationale
3. Present the comparison to the user
4. Ask the user to select their preferred strategy
5. Document the choice in `decision-log.yaml` with rationale

---

## PLANNING Outcome

Done when:
- All 4 core artifacts exist and respect the templates documented below (`code-context.md`, `context-document.md`, `decision-log.yaml`, `implementation-roadmap.md`)
- Roadmap is phased appropriately (each phase is a complete deliverable)
- First phase has sufficient detail to begin work
- Future phases are deferred appropriately per Last Responsible Moment principle

### Tactic: Create Plan Artifacts

**When to use**: STRATEGY outcome is met. Create all 4 core artifacts.

**Artifact 1 — Code Context** (`code-context.md`)

Use template: [`contribution-system/assets/templates/code-context-template.md`](../contribution-system/assets/templates/code-context-template.md)

Purpose: Quick reference for relevant existing code — classes, functions, key files with line numbers.

**Artifact 2 — Context Document** (`context-document.md`)

Use template: [`contribution-system/assets/templates/context-document-template.md`](../contribution-system/assets/templates/context-document-template.md)

Purpose: Immutable reference — behavioral spec, architecture summary, constraints, research findings.

**Artifact 3 — Decision Log** (`decision-log.yaml`)

Get the template by running: `diffviz templates decision-log`

Purpose: Mutable record of decisions made. Document: selected strategy + rationale, rejected alternatives, key technology/architecture choices.

**Artifact 4 — Implementation Roadmap** (`implementation-roadmap.md`)

Template selection:
- Steel Thread strategy → use [`steel-thread-roadmap-template.md`](../contribution-system/assets/templates/steel-thread-roadmap-template.md)
- Other strategies → use [`implementation-roadmap-template.md`](../contribution-system/assets/templates/implementation-roadmap-template.md)

Purpose: Phase-by-phase execution plan. Each phase includes objectives (Implementation and Design), testing criteria, dependencies, and files to modify.

Design objectives are deferred to `design-contribute` via the Two-Gate System — see [`contribution-system/references/two-gate-system.md`](../contribution-system/references/two-gate-system.md).

**File organization:**

```
.plans/plan-[FEATURE-NAME]/
├── code-context.md
├── context-document.md
├── decision-log.yaml
├── implementation-roadmap.md
└── research/                    (only when research tactic was applied)
    ├── research-summary.md
    ├── technology-research.md
    └── implementation-examples.md
```

Use clear, descriptive feature names with `plan-` prefix:
- ✅ `.plans/plan-user-authentication/`
- ✅ `.plans/plan-real-time-notifications/`
- ❌ `.plans/plan-2024-01-15/`
- ❌ `.plans/plan-feature1/`

---

### Tactic: Commit the Plan

**When to use**: All 4 core artifacts are written and the PLANNING outcome is met.

```bash
git add .plans/plan-<feature-name>/
git commit -m "plan: <feature name>"
```

---

## Quality Reference

### Writing Good Objectives

**Implementation Objectives** (clear execution following existing patterns):
- ✅ "Add JWT refresh token endpoint to AuthController"
- ✅ "Update frontend login form to handle refresh tokens"
- ❌ "Improve authentication"

**Design Objectives** (require design decisions to be refined later via design-contribute):
- ✅ "Design error handling strategy for auth failures"
- ✅ "Determine optimal session management approach"
- ❌ "Make it better"

### Effective Testing Criteria

- ✅ "User can log in and receive both access and refresh tokens"
- ✅ "Expired access token automatically refreshes without user action"
- ❌ "Authentication works"
- ❌ "No bugs"

### Making Phases Actionable

- Each phase should be completable in 1–3 development sessions
- Include specific file paths that need modification
- Provide clear success criteria
- Consider dependencies between phases
- Separate **Implementation** objectives (ready to execute) from **Design** objectives (require further design decisions)

### Common Patterns

**For new features:**
1. Setup/Infrastructure — Add necessary dependencies, config
2. Core Logic — Implement main functionality
3. Integration — Connect to existing systems
4. UI/API — Add user-facing interfaces
5. Testing & Polish — Sufficient testing, error handling

**For refactoring:**
1. Preparation — Add tests for existing behavior
2. Extract — Move code to new structure
3. Replace — Update callers to use new structure
4. Cleanup — Remove old code
5. Validation — Ensure no regressions

**For bug fixes:**
1. Reproduce — Create test that reproduces the bug
2. Investigate — Understand root cause
3. Fix — Implement solution
4. Test — Verify fix and no regressions
5. Monitor — Plan for monitoring in production
