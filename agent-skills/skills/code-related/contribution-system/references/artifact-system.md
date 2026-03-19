# Artifact System Overview

Every contribution creates files in a numbered folder under `.plans/plan-[FEATURE-NAME]/contributions/`. The folder name encodes the phase, type, specialty, and agent — enabling any agent to understand contribution history by reading folder names alone.

## Full Directory Structure

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

## Critical Points

- **`contributions/` folder is REQUIRED** — all design and implementation work goes inside it
- Initial plan artifacts (`code-context.md`, `decision-log.yaml`, etc.) are **at plan root** only (created by dev-strategy)
- Contributions are numbered sequentially (`001`, `002`, `003`...) regardless of type or phase
- **All contributions MUST use full naming convention**: `NNN-phase-X-[type]-[specialty]-[agent]`
- The plan folder must exist before contributions can be created
