---
name: contribution-system
description: Canonical reference for all contribution artifact schemas, templates, folder naming conventions, file format rules, and workflow patterns (progressive disclosure, two-gate system, constraint discovery, routing) used across dev-strategy workflow skills.
---

# Contribution System

Standardized conventions for contribution artifacts, folder naming, directory structure, and workflow patterns used across all dev-strategy workflow skills.

## What This Skill Provides

This skill is the canonical reference for:
- **Artifact schemas** — Exact structure of decision-log.yaml, context-handoff.md, design-doc.md
- **Folder naming conventions** — Sequential numbering (NNN-phase-X-[type]-[specialty]-[agent])
- **Directory structure** — Where artifacts live and how they relate
- **Templates** — Ready-to-use starting points for all artifact types
- **Lookup table** — Find the right artifact for any question

Load this skill when:
- Creating or contributing to a dev-strategy plan
- Unsure where to find specific information about a feature
- Starting a new contribution and need folder naming rules
- Writing decision logs, context handoffs, or design docs

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

## Workflow Patterns Reference

This skill also defines workflow patterns used across all dev-strategy skills. Load these references as needed:

- **[Execution Requirements Template](references/execution-template.md)** — Standard outcome-based structure for all skills (UNDERSTANDING → EXECUTION → TRANSFER)
- **[Progressive Disclosure](references/progressive-disclosure.md)** — How to read contributions efficiently (start with context-handoff.md)
- **[Onboarding Guide Generation](references/onboarding-generation.md)** — Generating codebase orientation guides with Explore agent
- **[Two-Gate System](references/two-gate-system.md)** — Design gates in planning and implementation (Last Responsible Moment)
- **[Constraint Discovery](references/constraint-discovery.md)** — AskUserQuestion patterns for gathering real constraints
- **[Routing Patterns](references/routing-patterns.md)** — Determining next skill by reading roadmap state
