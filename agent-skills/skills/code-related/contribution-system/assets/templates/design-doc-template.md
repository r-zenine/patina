# Design Document - [Topic Name]

> **Target**: < 100 lines total
> **Purpose**: Document design decision to enable implementation
> **Note**: This captures the outcome of interactive design session with user

## Decision: [What We're Doing]

[One paragraph summary of chosen approach. Example: "We're implementing JWT-only session validation with no server-side session storage. Sessions are validated by decoding the JWT, verifying the signature, and checking expiry."]

## Why This Design

**Constraints That Led Here:**
- [Implementation constraint discovered] Example: "JWT authentication already implemented in phase 3"
- [User requirement clarified through discussion] Example: "User requires stateless approach to avoid database lookups"
- [Technical constraint validated] Example: "Token expiry set to 1 hour, short enough to not need revocation"

**User Priorities:**
[What user valued most in the interactive session]
Example: "User prioritized simplicity over flexibility. No need for immediate session revocation."

**Simplicity Rationale:**
[Why this is simplest approach given priorities and constraints]
Example: "Simplest because it leverages existing JWT infrastructure with no additional dependencies. No Redis, no database, no caching layer. Pure stateless validation."

## How It Works

**Key Interfaces:**
- [Interface 1]: [Purpose and what it exposes]
  Example: "SessionValidator interface: validateSession(token) → SessionData | Error"
- [Interface 2 if needed]: [Purpose]
  Example: "SessionData interface: { userId, email, roles, expiresAt }"

**Core Pattern:**
[Brief architectural pattern description - no code, just concept]
Example: "Request → Extract JWT from header → Decode & validate signature → Check expiry → Return session data. No external dependencies, purely computational validation."

**Integration Points:**
- [Connection to existing component A]
  Example: "Integrates with existing JWT signing service from 003-phase-3-implementation-jwt-auth/"
- [Connection to existing component B]
  Example: "Used by API middleware in 004-phase-3-implementation-api-endpoints/"

## What We're NOT Doing

**Rejected Alternatives:**
- **[Option 1]**: [Why not chosen]
  Example: "JWT + Redis revocation list: Adds complexity and Redis dependency for feature not currently needed"
- **[Option 2]**: [Why not chosen]
  Example: "Server-side sessions: Violates stateless requirement and adds database lookups"

**Out of Scope:**
- [Future consideration that's explicitly deferred]
  Example: "Immediate session revocation (not needed with 1-hour token expiry)"
- [Another deferred concern]
  Example: "Session refresh tokens (can add in future phase if needed)"

## Implementation Guidance

**For Next Contributor:**
- Start by implementing [specific component]
  Example: "Start by implementing SessionValidator class in src/auth/session.ts"
- Follow [pattern] from [file]
  Example: "Follow validation pattern from JWTService in src/auth/jwt.ts"
- [Key implementation note]
  Example: "Use same JWT library (jsonwebtoken) already in dependencies"

**Testing Strategy:**
[What to test - brief, actionable]
Example:
- Unit tests: Valid token → returns session data
- Unit tests: Expired token → returns error
- Unit tests: Invalid signature → returns error
- Integration test: Full request flow with valid/invalid tokens

**Success Criteria:**
[How to verify this design is complete and working]
Example:
- Valid JWT tokens are validated without database calls
- Invalid/expired tokens are rejected with clear errors
- API endpoints use SessionValidator middleware
- All tests passing
