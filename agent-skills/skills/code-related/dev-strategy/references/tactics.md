# Dev Strategy Skill — Tactics Reference

Tactics available for reaching each dev-strategy outcome.
The agent chooses which tactics to apply and in what order based on context.

Done-when criteria are owned by [SKILL.md](../SKILL.md) — restated here for convenience; do not add or alter them here.

---

## UNDERSTANDING Outcome

Done when:
- Behavioral spec is confirmed by the user
- Draft `code-context.md` exists, naming the files/functions the change will touch, with line references
- Constraint list is recorded in `context-document.md` and explicitly confirmed by the user

### Tactic: Clarify Behavioral Specification

**When to use**: Requirements are ambiguous, vague, or leave key behaviors undefined.

Ask clarifying questions to surface unclear aspects (edge cases, users, expected outcomes). Then produce a 2–3 sentence plain description of what to build. Focus on WHAT, not HOW. Avoid technical jargon.

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

**Keep the exploration out of the planning session:** work from the Explore agent's report. If a specific detail is missing, read only the targeted line range (or send a follow-up question to the same agent) — do not re-read whole files into the planning context to double-check findings.

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

**Delegate the research:** spawn a general-purpose sub-agent that performs the web searches and doc reading, writes the three research artifacts directly, and returns only its `research-summary.md` content. Search transcripts and fetched pages are the bulkiest content a planning session can accumulate and are worthless once distilled — they must not pass through the main context.

---

### Tactic: Constraint Discovery Questioning

**When to use**: Constraints are unclear after codebase analysis, or codebase analysis surfaced ambiguous choices.

Ask specific questions grounded in what codebase analysis actually found — ambiguous framework/storage/auth choices, integration compatibility, technologies to use or avoid ("I see both X and Y — which should new code follow?"). Close with an open-ended catch-all.

For interactive questioning patterns, see [`contribution-system/references/constraint-discovery.md`](../contribution-system/references/constraint-discovery.md).

---

## STRATEGY Outcome

Done when:
- User selected a strategy
- `decision-log.yaml` records the choice, rationale, and rejected alternatives with one-line reasons

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
- First phase lists concrete file paths and testing criteria
- Future phases are deferred appropriately per Last Responsible Moment principle

### Tactic: Create Plan Artifacts

**When to use**: STRATEGY outcome is met. Create all 4 core artifacts.

**Write lean — every line is paid for once per phase.** Downstream, each dev-contribute invocation re-reads these artifacts, so an N-phase plan pays for their length N times. Distill instead of transcribing: reference code by path and line range rather than quoting it, and keep the behavioral spec and architecture summary to what an implementer actually needs.

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

### Tactic: Leave the Plan Uncommitted

**When to use**: All 4 core artifacts are written and the PLANNING outcome is met.

`.plans/plan-<feature-name>/` is planning scratch, not part of the codebase history — do not `git add` or `git commit` it. Leave it as local, uncommitted files. Contributions made later under `dev-contribute` will commit source code changes, but never the `.plans/` tree itself.

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

- Each phase must be completable in **one focused session with a fresh context window** — dev-contribute runs one contribution per session, and a phase that would push a session past ~100k tokens of working context should be split into two phases
- Make each phase self-contained in the roadmap: its entry carries the file paths, relevant constraints, and success criteria the implementer needs, so a contribution can start from a compact brief instead of re-reading the whole plan
- Include specific file paths that need modification
- Provide clear success criteria
- Consider dependencies between phases
- Separate **Implementation** objectives (ready to execute) from **Design** objectives (require further design decisions)
