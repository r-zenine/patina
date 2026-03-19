# Design Contribution Artifacts

Schemas for the 3 files in every design contribution (created by the design-contribute skill).

**Design contributions create exactly 3 files. No code, no prototypes, no comprehensive specs.**

**Files:**
1. `design-doc.md` — Design specification (< 100 lines)
2. `decision-log.yaml` — Design decisions in unified YAML schema
3. `context-handoff.md` — Orientation for next contributors (< 30 lines)

---

## 1. `design-doc.md`

**Target**: < 100 lines
**Purpose**: Document the design decision to enable implementation. This is the primary artifact implementers read.

**Schema:**
```markdown
# Design Document - [Topic Name]

> **Target**: < 100 lines total
> **Note**: This captures the outcome of an interactive design session with user

## Decision: [What We're Doing]
[One paragraph summary of chosen approach]

## Why This Design

**Constraints That Led Here:**
- [Implementation constraint discovered]
- [User requirement clarified through discussion]

**User Priorities:**
[What user valued most in the interactive session]

**Simplicity Rationale:**
[Why this is the simplest approach given priorities and constraints]

## How It Works

**Key Interfaces:**
- [Interface name]: [Purpose and what it exposes]

**Core Pattern:**
[Brief architectural pattern description - no code, just concept]

**Integration Points:**
- [Connection to existing component]

## What We're NOT Doing

**Rejected Alternatives:**
- **[Option]**: [Why not chosen]

**Out of Scope:**
- [Future consideration that's explicitly deferred]

## Implementation Guidance

**For Next Contributor:**
- Start by implementing [specific component]
- Follow [pattern] from [file]

**Testing Strategy:**
- [What to test - brief, actionable]

**Success Criteria:**
- [How to verify this design is complete and working]
```

**Enforcement rules:**
- Must be under 100 lines; if exceeding, you are over-documenting
- No code files in the contribution folder
- No comprehensive specifications — focus on "what" and "why"
- Must have a "What We're NOT Doing" section

---

## 2. `decision-log.yaml` (design variant)

**Purpose**: Record design decisions made during this contribution, using the unified Decision schema.

**Schema**: Same unified schema as strategy and implementation, matching `diffviz-review::Decision` struct.

```yaml
commit: null  # Design decisions typically have no code_impacts (code comes in implementation phase)

decisions:
  - number: 1
    title: "[Design decision in one sentence]"
    rationale: "[Why this design — constraints and user priorities that led here]"
    code_impacts: []  # Empty: design phase doesn't produce code, only spec

  - number: 2
    title: "[Supporting design decision]"
    rationale: "[Why this choice was made]"
    code_impacts: []
```

**Key Points:**
- Use the same YAML schema as strategy and implementation decisions
- `number`: Sequential decision ID (1, 2, 3...)
- `title`: Design choice in one sentence
- `rationale`: Must explain constraints and user priorities discovered during design session
- `code_impacts`: Empty array (design phase doesn't produce code)
- This allows design decisions to flow seamlessly into implementation, where `code_impacts` will be populated

---

## 3. `context-handoff.md` (design variant)

**Target**: < 30 lines
**Purpose**: Rapid orientation for any agent reading the design contribution.

**Schema (3 sections only):**
```markdown
# Context Handoff - Phase X Design

## What Problem Are We Solving
[5–10 lines: the specific design question this session resolved + why it was deferred]

## Design Overview
[~15 lines: high-level summary of the approach chosen, key constraints, and what was rejected]

## Reading Guide
[5 lines: how to read design-doc.md — which sections matter most for implementers]
```

**Enforcement rules:**
- Must be under 30 lines
- Only 3 sections: problem, overview, reading guide
- No implementation details — that belongs in design-doc.md
