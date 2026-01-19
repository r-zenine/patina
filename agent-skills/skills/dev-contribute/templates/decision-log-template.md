# Technical Decisions - Phase X [Contribution Type]

> **Purpose**: Record technical choices for future agents to understand context and constraints

## ⚡ Critical Decisions (Agents must know)
**[Decision name]**: [Choice made] → **Impact**: [How this affects future work]

## 🔧 Implementation Choices (Reference for next contributors)
**[Technical approach]**: [Reasoning in brief] → **Result**: [What pattern to follow]

## 🔍 Alternatives Considered (Background context)
**[Rejected option]**: [Why not chosen] → **Trade-off**: [What we gained/lost]

---
## 📚 Human Context (Detailed reasoning)
**Decision Process**: [How decisions were made]
**Constraints**: [What limited the choices]

## Example
```markdown
# Technical Decisions - Phase 2 Test Design

## ⚡ Critical Decisions
**Jest + custom matchers**: Standard test framework + auth helpers → **Impact**: All future auth tests must use toBeAuthenticated() pattern

## 🔧 Implementation Choices
**Factory functions for users**: Avoid test coupling → **Result**: Always use UserFactory.create() in tests
**Three-tier test structure**: Unit→Integration→E2E → **Result**: Follow pyramid, focus on unit tests

## 🔍 Alternatives Considered
**Mocha instead of Jest**: More flexibility → **Trade-off**: Chose Jest for consistency with existing codebase

## 📚 Human Context
**Decision Process**: Followed existing codebase patterns, prioritized maintainability
**Constraints**: Must integrate with current CI/CD setup and team familiarity
```