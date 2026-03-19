# Folder Naming Convention

All contributions follow a sequential naming scheme: `NNN-phase-X-[type]-[specialty]-[agent]`

This standardized naming enables any agent to understand contribution history without reading folder contents.

## Implementation and Review Contributions

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

## Design Contributions

```
NNN-phase-X-design-[topic]-design-contribute
```

**Topic examples:** `session-mgmt`, `error-handling`, `caching-strategy`, `api-structure`, `data-model`, `auth-flow`

**Example:**
```
.plans/plan-myfeature/contributions/005-phase-3-design-session-mgmt-design-contribute/
```

## Revision Contributions

```
NNN-phase-X-revision-[original-specialty]-[agent]
```

**Example:** Revising a decision from contribution 002:
```
.plans/plan-myfeature/contributions/004-phase-1-revision-code-general-purpose/
```
