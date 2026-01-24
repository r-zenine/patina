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

## Example (Regular Contribution)
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

## Example (Revision Contribution)
```markdown
# Technical Decisions - Phase 1 Revision

## ⚡ Critical Decisions
**Revision of Contribution 002 - REST API approach**: Switched to GraphQL → **Impact**: All future API endpoints use GraphQL schema, REST endpoints deprecated

## 🔧 Implementation Choices
**GraphQL with Apollo Server**: Better type safety and client flexibility → **Result**: Use schema-first approach, all types defined in schema.graphql
**Rationale for revision**: Original REST design led to over-fetching and multiple endpoints for related data. GraphQL solves both issues.

## 🔍 Alternatives Considered
**Keep REST and optimize**: Add field filtering → **Trade-off**: Would require complex query parameters, chose GraphQL for cleaner solution

## 📚 Human Context
**Decision Process**: Reviewed contribution 002 decision-log.md, identified REST limitations in practice
**Constraints**: Must migrate existing 3 endpoints gradually, maintain backwards compatibility during transition
```