---
name: contribution-system
description: Canonical reference for all contribution artifact schemas, templates, folder naming conventions, file format rules, and workflow patterns (progressive disclosure, two-gate system, constraint discovery, routing) used across dev-strategy workflow skills.
---

# Contribution System

Standardized conventions for contribution artifacts, folder naming, directory structure, and workflow patterns used across all dev-strategy workflow skills.

## Core References

**Structure & Naming:**
- [artifact-system.md](references/artifact-system.md) — Directory structure and artifact organization
- [folder-naming.md](references/folder-naming.md) — Sequential naming convention for contributions
- [decision-schema.md](references/decision-schema.md) — Unified `decision-log.yaml` schema and why it's unified

**Artifact Schemas by Type:**
- [implementation-artifacts.md](references/implementation-artifacts.md) — Implementation/review contribution schemas
- [design-artifacts.md](references/design-artifacts.md) — Design contribution schemas (design-doc.md, decision-log.yaml, context-handoff.md)
- [strategy-artifacts.md](references/strategy-artifacts.md) — Strategy contribution schemas (planning phase output)

## Templates

All templates are in [assets/templates/](assets/templates/):

**Implementation templates:**
- [context-handoff-template.md](assets/templates/context-handoff-template.md)
- [decision-log-template.yaml](assets/templates/decision-log-template.yaml)

**Design templates:**
- [design-doc-template.md](assets/templates/design-doc-template.md)

**Strategy (planning) templates:**
- [code-context-template.md](assets/templates/code-context-template.md)
- [context-document-template.md](assets/templates/context-document-template.md)
- [implementation-roadmap-template.md](assets/templates/implementation-roadmap-template.md)
- [steel-thread-roadmap-template.md](assets/templates/steel-thread-roadmap-template.md)

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
