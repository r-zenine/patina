# Context Handoff - Phase X [Contribution Type]

> **Purpose**: Guide next agents with essential context and specific next steps
>
> ⚠️ **Be Concise**: Only include information the next agent actually needs. Skip obvious details or lengthy explanations.

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
- [decision-log.md](decision-log.md) - Technical choices made
- [changelog.md](changelog.md) - Phase completion summary

## Example
```markdown
# Context Handoff - Phase 2 Test Design

## 🎯 Core Result
**Built**: Layered auth test suite with custom matchers
**Key insight**: toBeAuthenticated() pattern enables consistent auth testing across all test types

## 🚦 Current State
**✅ Solid foundation**: Factory pattern for users, test coverage hierarchy, JWT structure alignment
**⚠️ Needs attention**: Integration DB setup, password validation edge cases
**⏸️ Deferred**: Performance tests (future phase), password reset (MVP scope)

## 👥 Next Agent Guidance
**Test Validator**: Focus on password validation edge cases + DB setup refinement first
**Implementation Agent**: JWT expectations in tests/fixtures/, error constants in tests/fixtures/auth-errors.js

## 🔗 Integration Points
**Expects**: Redis for sessions, JWT structure per context doc
**Provides**: Working test suite ready for implementation validation
```