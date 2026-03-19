# Implementation Contribution Artifacts

Schemas for the 2 mandatory files in every implementation, review, audit, or revision contribution.

---

## 1. `decision-log.yaml`

**Purpose**: Structured record of technical decisions made during this contribution, mapped to actual code changes.

**Schema**: Unified schema matching `diffviz-review::Decision` struct.

```yaml
commit: "abc123def456"  # Required: git hash of the commit containing these code changes

decisions:
  - number: 1
    title: "[Decision made in one sentence]"
    rationale: "[Why this choice was made — constraints, priorities, trade-offs]"  # optional
    code_impacts:
      - file: "[path/to/file.rs]"
        reasoning: "[Why this file is affected by this decision]"
        line_ranges:
          - start: 10
            end: 50

  - number: 2
    title: "[Next decision]"
    code_impacts:
      - file: "[path/to/another/file.rs]"
        reasoning: "[Why affected]"
        line_ranges:
          - start: 100
            end: 150
```

**Key Points:**
- Use `number` (u32) for decision ID, matching the struct
- Use `title` (not `decision`) — this is the struct field name
- `code_impacts` must reference actual code changes in this contribution
- `commit` must be populated with git hash of the commit containing code changes (dev-contribute Step 4.5)
- `rationale` is optional
- This is the **same schema** used in strategy decision-logs; see [strategy-artifacts.md](strategy-artifacts.md)

**Rules:**
- Document only NEW decisions made during this contribution
- Do not re-document decisions already in the dev-strategy decision log
- Every code_impact must have at least one line_range pointing to actual changes
- If a decision has no code impacts yet, use empty array: `code_impacts: []`
- Use YAML format (not markdown)

---

## 2. `context-handoff.md`

**Purpose**: Guide next agents with essential context and specific next steps.

**Schema:**
```markdown
# Context Handoff - Phase X [Contribution Type]

## 🎯 Core Result (What agents get from this work)
**Built**: [Main deliverable with key insight]
**Key insight**: [Most important technical discovery that affects future work]

## 🚦 Current State (Agent decision points)
**✅ Solid foundation**: [What works reliably for next phase]
**⚠️ Needs attention**: [Priority items next contributor should handle]
**⏸️ Deferred**: [What was postponed and why]

## 👥 Next Agent Guidance (Specific handoff)
**[Next Agent Type]**: [Actionable guidance - what they should focus on]
**[Future Agent Type]**: [Key context they'll need to know]

---
## 🔗 Integration Points (Technical context)
**Expects**: [Key assumptions/dependencies this work relies on]
**Provides**: [What this work makes available to others]

## 📋 Reference Links
- [decision-log.yaml](decision-log.yaml) - Technical choices made
```

**Rules:**
- Lead with what was built and the key insight
- "What works/fragile/missing" structure is the most important part
- Provide specific guidance for next contributors
- Progressive disclosure: this is the starting point for readers

---

## Optional Artifacts

For specialized contributions, additional files may be created:

**Performance:**
- `performance-report.md` — Baseline, results, bottlenecks, recommendations
- `optimization-recommendations.md` — High impact, quick wins, future work

**Security:**
- `security-scan-results.json` — Raw automated scan outputs
- `vulnerability-report.md` — Critical, medium, mitigated, monitoring
- `threat-model.md` — Attack vectors, mitigations, residual risk

**Architecture:**
- `integration-map.md` — Data flow, dependencies, failure points
- `api-contracts.md` — Endpoints, request/response, breaking changes

**Documentation:**
- `user-guide.md` — Getting started, common tasks, troubleshooting
- `developer-guide.md` — Setup, extension points, testing

