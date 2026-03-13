# Implementation Contribution Artifacts

Schemas for the 2 mandatory files in every implementation, review, audit, or revision contribution.

---

## 1. `decision-log.yaml`

**Purpose**: Structured record of technical decisions made during this contribution (YAML format).

**Schema:**
```yaml
decisions:
  - id: "001"
    type: implementation  # implementation | contribution
    decision: "[Decision made in one sentence]"
    rationale: "[Why this choice was made]"
    alternatives_rejected:
      - alternative: "[Option not chosen]"
        reason: "[Why rejected]"
    impact: "[Effect on future work]"
```

**Rules:**
- Document only NEW decisions made during this contribution
- Do not re-document decisions already in the dev-strategy decision log
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

See the [optional-artifacts-templates.md](../assets/templates/optional-artifacts-templates.md) for full schemas.
