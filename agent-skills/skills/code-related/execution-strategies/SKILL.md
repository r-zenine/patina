---
name: execution-strategies
description: Reference guide for three development execution strategies: TDD, Steel Thread, and Core-then-Integrate. Load this when you need strategy definitions, selection criteria, execution phases, test pyramid guidance, contribution folder naming patterns, or help choosing a strategy. Referenced during dev-strategy planning and dev-contribute execution.
---

# Execution Strategies

Canonical reference for the three development execution strategies used across the dev-strategy workflow skills.

## What This Skill Contains

- Definitions and selection criteria for TDD, Steel Thread, and Core-then-Integrate
- Execution phases and contribution sequences for each strategy
- Test pyramid guidance
- Contribution folder naming patterns per strategy

## Strategy Overview

### Test-Driven Development (TDD)

Write sufficient tests first, then implement code to pass those tests.

**Best for**: Clear requirements, small-medium features, well-defined business rules

**Contribution sequence**:
1. Test Designer → creates failing tests
2. Test Validator → validates coverage and edge cases
3. Implementation Agent → writes minimal code to pass tests

**Folder pattern**:
```
001-phase-X-test-design-tdd-[agent]
002-phase-X-test-validation-tdd-[agent]
003-phase-X-implementation-code-[agent]
```

---

### Steel Thread

Build the thinnest viable end-to-end working version first, then expand capabilities while keeping the system functional.

**Best for**: Uncertain requirements, multiple integration points, need early feedback

**Key requirement**: Each phase must clearly articulate the specific end-to-end path being built. Every phase maintains a fully functional system.

**Contribution sequence**:
1. Pathfinder → maps minimal viable path
2. Foundation Builder → implements basic working flow
3. Capability Expander → adds features iteratively

**Folder pattern**:
```
001-phase-X-pathfinder-steel-thread-[agent]
002-phase-X-foundation-steel-thread-[agent]
003-phase-X-expansion-capability-[agent]
004-phase-X-expansion-capability-[agent]
```

---

### Core-then-Integrate

Build pure business logic first, then add external integrations layer by layer.

**Best for**: Complex business rules, clean architecture goals, heavy external dependencies

**Contribution sequence**:
1. Domain Modeler → pure business logic with unit tests
2. Port Designer → interface contracts and integration points
3. Adapter Builder → external system integrations
4. Integration Orchestrator → wires everything together

**Folder pattern**:
```
001-phase-X-domain-logic-core-[agent]
002-phase-X-port-design-core-[agent]
003-phase-X-adapter-database-[agent]
004-phase-X-adapter-api-[agent]
005-phase-X-integration-orchestrator-[agent]
```

---

## Strategy Selection Guide

| Signal | Recommended Strategy |
|--------|---------------------|
| Clear requirements, small-medium feature | TDD |
| Uncertain requirements, need feedback | Steel Thread |
| Complex business rules, clean architecture | Core-then-Integrate |
| Multiple external integrations | Core-then-Integrate |
| User-facing feature with unknown flow | Steel Thread |

**Apply Last Responsible Moment**: Don't commit to complex approaches without evidence they're needed.

## Detailed References

- [TDD](references/tdd.md) — Phases, test pyramid, contribution sequence detail
- [Steel Thread](references/steel-thread.md) — Loop structure, JIT refactor guidelines
- [Core-then-Integrate](references/core-then-integrate.md) — Domain → Port → Adapter → Integration phases

## See Also

**Related Skills:**
- **[dev-strategy](../dev-strategy/SKILL.md)** — Uses this skill during strategy selection (Outcome 2)
- **[dev-contribute](../dev-contribute/SKILL.md)** — References this skill when executing phases according to chosen strategy
- **[design-principles](../design-principles/SKILL.md)** — Applies Last Responsible Moment principle when selecting strategies
- **[contribution-system](../contribution-system/SKILL.md)** — For folder naming patterns specific to each strategy
