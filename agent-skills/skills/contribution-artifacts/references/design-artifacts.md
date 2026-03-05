# Design Contribution Artifacts

Schemas for the 3 files in every design contribution (created by the design-contribute skill).

**Design contributions create exactly 3 files. No code, no prototypes, no comprehensive specs.**

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

## 2. `decision-log.md` (design variant)

**Purpose**: Record the design choice for implementers to understand context and constraints.

**Schema:**
```markdown
# Design Decisions - Phase X Design

## Primary Design Decision

**[Design Topic]**: [Chosen approach in one sentence]
**Why**: [Constraints and user priorities that led here]
**Rejected**: [Alternatives considered and why not chosen]
**Impact**: [Effect on implementation — what this enables/constrains]

## Supporting Decisions

**[Choice 1]**: [Decision made] → **Impact**: [Effect on implementation]
**[Choice 2]**: [Decision made] → **Impact**: [Effect on implementation]
```

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
