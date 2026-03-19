# Steel Thread

## Description

Build the thinnest viable end-to-end working version first, then expand capabilities while keeping the system functional at every phase.

**Process**: Implementation → Demonstration → Refactor (per phase loop)

## Core Principle

Each phase must clearly articulate the specific end-to-end path being built. Every phase maintains a fully working system. Users can observe progress after every phase.

**Example — User Authentication**:
- Phase 1: User enters hardcoded credentials, sees "Login Success" (no database, no sessions)
- Phase 2: User login checks real database, creates session token (no password validation yet)
- Phase 3: Full password validation, error handling, secure session management
- Phase 4: Add password reset flow while maintaining working login

## Phase Structure

Each Steel Thread phase follows this loop:

```
Implement → Demonstrate → Refactor for Next Phase
```

**Implement**: Build minimal working functionality for this phase — use intentional shortcuts (hardcoded values, stubs) where real integration is not yet needed.

**Demonstrate**: Prove end-to-end functionality works. What can the user actually do or see after this phase?

**Refactor for Next Phase (Just-In-Time only)**: Only refactor if the current structure actively blocks the next phase.

## Just-In-Time Refactor Guidelines

**When to refactor between phases:**
- Actual duplication: You are literally copying and pasting code
- Concrete integration needs: Next phase requires a specific interface that does not exist
- Breaking changes: Current structure actively prevents the next phase

**Never refactor for:**
- Hypothetical future needs
- Abstracting before you have 2–3 concrete use cases
- Preparing for "what if" scenarios

**Anti-patterns:**
- Creating interfaces before you have multiple implementations
- Adding configuration systems before you have actual variations to configure
- Building extension points before you know what will extend them

## Contribution Folder Naming

```
NNN-phase-X-pathfinder-steel-thread-[agent]
NNN-phase-X-foundation-steel-thread-[agent]
NNN-phase-X-expansion-capability-[agent]
NNN-phase-X-expansion-capability-[agent]
```

## Roadmap Format for Steel Thread

Each phase in the implementation roadmap should specify:

**Steel Thread Scope:**
- End-to-End Path Being Built: the specific user journey this phase enables
- User Experience After This Phase: what the user can actually do or see
- Intentional Shortcuts: what is hardcoded or mocked for speed

**Refactor section (at end of each phase):**
- Just-In-Time Changes: only what Phase N+1 actually requires
- Validation: all previous phase functionality still works after refactor
