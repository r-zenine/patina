---
name: contribution-system
description: Canonical reference for all contribution artifact schemas, templates, folder naming conventions, file format rules, and workflow patterns (progressive disclosure, two-gate system, constraint discovery, routing) used across dev-strategy workflow skills.
---

# Contribution System

Canonical reference for artifact schemas, templates, folder naming conventions, file format rules, and workflow patterns used across all dev-strategy workflow contributions.

## Table of Contents

- [Artifact System Overview](#artifact-system-overview)
- [Folder Naming Convention](#folder-naming-convention)
- [Unified Decision Schema](#unified-decision-schema)
- [Artifact Schemas by Contribution Type](#artifact-schemas-by-contribution-type)
- [Templates](#templates)
- [Why a Unified Decision Schema?](#why-a-unified-decision-schema)
- [Finding the Right Artifact](#finding-the-right-artifact)
- [Workflow Patterns](#workflow-patterns)

## Artifact System Overview

Every contribution creates files in a numbered folder under `.plans/plan-[FEATURE-NAME]/contributions/`. The folder name encodes the phase, type, specialty, and agent — enabling any agent to understand contribution history by reading folder names alone.

### Full Directory Structure

```
.plans/
└── plan-[FEATURE-NAME]/                          ← Created by dev-strategy skill
    ├── code-context.md
    ├── context-document.md
    ├── decision-log.yaml
    ├── implementation-roadmap.md
    ├── research/                                 ← Optional, research artifacts
    │   ├── research-summary.md
    │   ├── technology-research.md
    │   └── implementation-examples.md
    │
    └── contributions/                            ← ⚠️ REQUIRED: All work goes here
        ├── 001-phase-1-implementation-code-general-purpose/
        │   ├── decision-log.yaml
        │   ├── context-handoff.md
        │   └── [optional artifacts]
        │
        ├── 002-phase-1-review-security-code-simplifier/
        │   ├── decision-log.yaml
        │   ├── context-handoff.md
        │   └── [optional artifacts]
        │
        ├── 003-phase-2-design-error-handling-design-contribute/
        │   ├── design-doc.md
        │   ├── decision-log.yaml
        │   └── context-handoff.md
        │
        └── 004-phase-2-implementation-code-general-purpose/
            ├── decision-log.yaml
            ├── context-handoff.md
            └── [optional artifacts]
```

**Critical Points:**
- ⚠️ **`contributions/` folder is REQUIRED** — all design and implementation work goes inside it
- Initial plan artifacts (`code-context.md`, `decision-log.yaml`, etc.) are **at plan root** only (created by dev-strategy)
- Contributions are numbered sequentially (`001`, `002`, `003`...) regardless of type or phase
- **All contributions MUST use full naming convention**: `NNN-phase-X-[type]-[specialty]-[agent]`
- The plan folder must exist before contributions can be created

## Folder Naming Convention

### Implementation and Review Contributions

```
NNN-phase-X-[type]-[specialty]-[agent]
```

**Contribution Types:**
- `implementation` — Code, features, infrastructure
- `review` — Code review, architecture review, quality assessment
- `audit` — Security, performance, compliance analysis
- `revision` — Revisiting a previous decision with code updates
- `test-design` — Test creation (TDD Test Designer)
- `test-validation` — Test review (TDD Test Validator)
- `foundation` — Basic working implementation (Steel Thread)
- `expansion` — Feature additions (Steel Thread Capability Expander)
- `domain-logic` — Business logic implementation (Core-then-Integrate)
- `port-design` — Interface design (Core-then-Integrate)
- `adapter` — External integrations (Core-then-Integrate)
- `integration` — System integration (Core-then-Integrate)

**Specialties:**
- `code` — General coding
- `frontend` — UI/UX implementation
- `backend` — Server-side logic
- `database` — Data layer
- `security` — Security-focused work
- `performance` — Optimization-focused work
- `tdd` — Test-driven development
- `api` — API design/implementation
- `steel-thread` — Steel Thread strategy
- `core` — Core-then-Integrate strategy
- `capability` — Capability expansion

**Examples:**
```
.plans/plan-myfeature/contributions/001-phase-1-implementation-code-general-purpose/
.plans/plan-myfeature/contributions/002-phase-1-review-security-code-simplifier/
.plans/plan-myfeature/contributions/003-phase-2-test-design-tdd-general-purpose/
.plans/plan-myfeature/contributions/004-phase-2-expansion-capability-general-purpose/
.plans/plan-myfeature/contributions/005-phase-3-adapter-database-general-purpose/
```

### Design Contributions

```
NNN-phase-X-design-[topic]-design-contribute
```

**Topic examples:** `session-mgmt`, `error-handling`, `caching-strategy`, `api-structure`, `data-model`, `auth-flow`

**Example:**
```
.plans/plan-myfeature/contributions/005-phase-3-design-session-mgmt-design-contribute/
```

### Revision Contributions

```
NNN-phase-X-revision-[original-specialty]-[agent]
```

**Example:** Revising a decision from contribution 002:
```
.plans/plan-myfeature/contributions/004-phase-1-revision-code-general-purpose/
```

## Unified Decision Schema

All `decision-log.yaml` files (strategy, implementation, design) use the **same schema**, matching the `diffviz-review::Decision` struct:

```yaml
commit: null  # Strategy: null. Implementation: git hash of commit containing code changes.

decisions:
  - number: 1                          # Decision identifier (u32)
    title: "[One sentence summary]"    # What was decided (required)
    rationale: "[Why...]"              # Why this choice (optional)
    code_impacts: []                   # Code changes (empty for planning, populated for implementation)
      # - file: "path/to/file.rs"
      # - reasoning: "[Why affected]"
      # - line_ranges:
      #     - start: 10
      #       end: 50
```

**Key insight**: The distinction between strategy and implementation is the `code_impacts`:
- **Strategy-level** (`dev-strategy` output): `code_impacts: []` — empty, decisions made before coding
- **Implementation-level** (contributions): `code_impacts: [...]` — populated, decisions + actual code changes
- **Design-level** (design contributions): See [design-artifacts.md](references/design-artifacts.md) for design-specific variant

---

## Artifact Schemas by Contribution Type

### Implementation / Review Contributions

See [implementation-artifacts.md](references/implementation-artifacts.md) for:
- `decision-log.yaml` schema (unified)
- `context-handoff.md` schema
- Optional artifact schemas

### Design Contributions

See [design-artifacts.md](references/design-artifacts.md) for:
- `design-doc.md` schema (< 100 lines target)
- `decision-log.yaml` schema (unified, code_impacts empty)
- `context-handoff.md` schema (< 30 lines target)

### Strategy Contributions (dev-strategy output)

See [strategy-artifacts.md](references/strategy-artifacts.md) for:
- `code-context.md` schema
- `context-document.md` schema
- `decision-log.yaml` schema (unified, code_impacts empty)
- `implementation-roadmap.md` schema
- Research artifacts schemas

## Templates

All templates are in [assets/templates/](assets/templates/):

**Implementation templates:**
- [changelog-template.md](assets/templates/changelog-template.md)
- [context-handoff-template.md](assets/templates/context-handoff-template.md)
- [decision-log-template.yaml](assets/templates/decision-log-template.yaml)

**Design templates:**
- [design-doc-template.md](assets/templates/design-doc-template.md)
- [decision-log-template.yaml](assets/templates/decision-log-template.yaml)

**Strategy (planning) templates:**
- [code-context-template.md](assets/templates/code-context-template.md)
- [context-document-template.md](assets/templates/context-document-template.md)
- [implementation-roadmap-template.md](assets/templates/implementation-roadmap-template.md)
- [steel-thread-roadmap-template.md](assets/templates/steel-thread-roadmap-template.md)
- [decision-log-template.yaml](assets/templates/decision-log-template.yaml)
- [research-summary-template.md](assets/templates/research-summary-template.md)
- [technology-research-template.md](assets/templates/technology-research-template.md)

---

## Why a Unified Decision Schema?

The `decision-log.yaml` schema is unified across strategy, implementation, and review contributions because it directly matches the `diffviz-review::Decision` struct in the codebase:

```rust
pub struct Decision {
    pub number: u32,
    pub title: String,
    pub rationale: Option<String>,
    pub code_impacts: Vec<CodeImpact>,
}
```

**Benefits:**
1. **Direct deserialization** — YAML parses straight to Rust struct, no translation
2. **Same structure everywhere** — reduces cognitive load; one schema to learn
3. **Progressive population** — strategy decisions start with `code_impacts: []`, implementation fills them in
4. **Testability** — the struct defines the contract; tests verify it works
5. **Traceability** — decisions flow from planning → implementation without reformatting

**Distinction is semantic, not structural:**
- **Strategy decisions** (from `dev-strategy`): `code_impacts: []` — decisions made before coding
- **Design decisions** (from `design-contribute`): `code_impacts: []` — design specs with no code yet
- **Implementation decisions** (from `dev-contribute`): `code_impacts: [...]` — decisions + actual code changes
- Same YAML, same struct, different content determined by phase

## Finding the Right Artifact

When you need information about a feature, use this table to find the right artifact:

| Question | Answer From | Location |
|----------|------------|----------|
| **"What am I building?"** | `context-document.md` | Plan root |
| **"What's the overall execution strategy?"** | `implementation-roadmap.md` | Plan root |
| **"What functions/classes are relevant?"** | `code-context.md` | Plan root |
| **"Why was this architecture chosen?"** | `decision-log.yaml` | Plan root |
| **"What changed in this contribution?"** | `decision-log.yaml` | `contributions/NNN-.../` |
| **"Which files were affected by decision #3?"** | `decision-log.yaml` → `code_impacts` | `contributions/NNN-.../` |
| **"What should the next phase focus on?"** | `context-handoff.md` | `contributions/NNN-.../` |
| **"What's the design for error handling?"** | `design-doc.md` | `contributions/NNN-design-.../` |
| **"What design constraints exist?"** | `decision-log.yaml` (design variant) | `contributions/NNN-design-.../` |
| **"How do I get oriented quickly?"** | `context-handoff.md` | `contributions/NNN-.../` |

**Progressive Disclosure Rule**: Always start with context-handoff.md in a contribution, then dive into decision-log.yaml for details.

---

## Workflow Patterns

### Onboarding Guide Generation

How to generate and update `onboarding.md` orientation guides using the Explore agent.

See [onboarding-generation.md](references/onboarding-generation.md) for:
- When to invoke (new crate, git diffs, major changes)
- Generate vs update detection
- Explore agent invocation template
- Expected output structure
- What to include and exclude

### Progressive Disclosure

How to read contribution artifacts efficiently — start with minimum context, dive deeper only when needed.

See [progressive-disclosure.md](references/progressive-disclosure.md) for:
- First Pass Rule (always start with `context-handoff.md`)
- Role-specific reading strategies (Implementation, Review, Audit agents)
- Reading design contributions (different structure)
- Understanding contribution sequences

### Two-Gate System Against Over-Engineering

Two checkpoints ensure design happens at the right time — not too early, not too late.

See [two-gate-system.md](references/two-gate-system.md) for:
- Gate 1 (dev-strategy): when to defer a design decision
- Gate 2 (design-contribute): design at the Last Responsible Moment
- How the two gates work together

### Constraint Discovery

Interactive techniques for gathering real constraints before designing or planning.

See [constraint-discovery.md](references/constraint-discovery.md) for:
- AskUserQuestion patterns for design-contribute (Steps 2 and 3)
- AskUserQuestion patterns for dev-strategy (Step 4)
- Iteration guidelines and what NOT to do

### Routing Patterns (dev-continue)

How to determine the correct next skill by reading roadmap state and contribution history.

See [routing-patterns.md](references/routing-patterns.md) for:
- Decision tree for design vs implementation routing
- Routing confidence levels (high/medium/low)
- Edge case handling
- Completion map output format
