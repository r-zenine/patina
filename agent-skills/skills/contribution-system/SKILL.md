---
name: contribution-system
description: Canonical reference for all contribution artifact schemas, templates, folder naming conventions, file format rules, and workflow patterns (progressive disclosure, two-gate system, constraint discovery, routing) used across dev-strategy workflow skills.
allowed-tools: ["Read"]
---

# Contribution System

Canonical reference for artifact schemas, templates, folder naming conventions, file format rules, and workflow patterns used across all dev-strategy workflow contributions.

## Artifact System Overview

Every contribution creates files in a numbered folder under `contributions/`. The folder name encodes the phase, type, specialty, and agent — enabling any agent to understand contribution history by reading folder names alone.

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
001-phase-1-implementation-code-general-purpose/
002-phase-1-review-security-code-simplifier/
003-phase-2-test-design-tdd-general-purpose/
004-phase-2-expansion-capability-general-purpose/
005-phase-3-adapter-database-general-purpose/
```

### Design Contributions

```
NNN-phase-X-design-[topic]-design-contribute
```

**Topic examples:** `session-mgmt`, `error-handling`, `caching-strategy`, `api-structure`, `data-model`, `auth-flow`

**Example:**
```
005-phase-3-design-session-mgmt-design-contribute/
```

### Revision Contributions

```
NNN-phase-X-revision-[original-specialty]-[agent]
```

**Example:** Revising a decision from contribution 002:
```
004-phase-1-revision-code-general-purpose/
```

## Artifact Schemas by Contribution Type

### Implementation / Review Contributions

See [implementation-artifacts.md](references/implementation-artifacts.md) for:
- `changelog.md` schema
- `decision-log.yaml` schema
- `context-handoff.md` schema
- Optional artifact schemas

### Design Contributions

See [design-artifacts.md](references/design-artifacts.md) for:
- `design-doc.md` schema (< 100 lines target)
- `decision-log.md` schema (design variant)
- `context-handoff.md` schema (< 30 lines target)

### Strategy Contributions (dev-strategy output)

See [strategy-artifacts.md](references/strategy-artifacts.md) for:
- `code-context.md` schema
- `context-document.md` schema
- `decision-log.yaml` schema (planning variant)
- `implementation-roadmap.md` schema
- Research artifacts schemas

## Templates

All templates are in [assets/templates/](assets/templates/):

**Implementation templates:**
- [changelog-template.md](assets/templates/changelog-template.md)
- [context-handoff-template.md](assets/templates/context-handoff-template.md)
- [decision-log-template.yaml](assets/templates/decision-log-template.yaml)
- [optional-artifacts-templates.md](assets/templates/optional-artifacts-templates.md)

**Design templates:**
- [design-doc-template.md](assets/templates/design-doc-template.md)
- [decision-log-design-template.md](assets/templates/decision-log-design-template.md)

**Strategy (planning) templates:**
- [code-context-template.md](assets/templates/code-context-template.md)
- [context-document-template.md](assets/templates/context-document-template.md)
- [implementation-roadmap-template.md](assets/templates/implementation-roadmap-template.md)
- [steel-thread-roadmap-template.md](assets/templates/steel-thread-roadmap-template.md)
- [decision-log-template-strategy.yaml](assets/templates/decision-log-template-strategy.yaml)
- [research-summary-template.md](assets/templates/research-summary-template.md)
- [technology-research-template.md](assets/templates/technology-research-template.md)
- [implementation-examples-template.md](assets/templates/implementation-examples-template.md)

## Workflow Patterns

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
