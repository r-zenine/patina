# Dev Continue Skill - Reference Guide

This document provides the complete workflow for analyzing dev-strategy state and routing to the appropriate skill.

## Table of Contents

- [Overview](#overview)
- [Step 1: Locate Dev-Strategy Plan](#step-1-locate-dev-strategy-plan)
- [Step 2: Read Current State](#step-2-read-current-state)
- [Step 3: Determine Next Objective](#step-3-determine-next-objective)
- [Step 4: Route to Appropriate Skill](#step-4-route-to-appropriate-skill)
- [Step 5: Handle Edge Cases](#step-5-handle-edge-cases)
- [Output Format](#output-format)
- [Integration with Other Skills](#integration-with-other-skills)
- [Validation Strategy](#validation-strategy)
- [Examples of Full Workflow](#examples-of-full-workflow)
- [Summary](#summary)

## Overview

The dev-continue skill is a smart router that eliminates the cognitive overhead of manually determining which skill to use next. It reads the current state of a dev-strategy project and automatically invokes the appropriate skill for the next step.

## Step 1: Locate Dev-Strategy Plan

### Goal
Find and validate the dev-strategy project the user wants to continue.

### Process

1. **Check current directory for dev-strategy artifacts:**
   ```bash
   # Look for dev-strategy plan in current directory
   ls -la | grep -E "(context-document|implementation-roadmap|decision-log).md"
   ```

2. **If not in current directory, search for dev-strategy projects:**
   ```bash
   # Search for dev-strategy plans in common locations
   find . -name "implementation-roadmap.md" -type f
   ```

3. **If multiple dev-strategy projects found:**
   - List them for the user
   - Ask which one they want to continue
   - Use AskUserQuestion if needed

4. **If no dev-strategy plan found:**
   - Inform user: "No dev-strategy plan found in current directory"
   - Suggest: "Would you like to create a new dev-strategy plan? Use the dev-strategy skill"
   - Exit gracefully

### Success Criteria
- Located valid dev-strategy plan directory
- Confirmed this is the project user wants to continue

## Step 2: Read Current State

### Goal
Understand what's been done and what's next.

### Process

1. **Read mandatory dev-strategy artifacts:**
   - `implementation-roadmap.md` - THE SOURCE OF TRUTH for phases and objectives
   - `context-document.md` - Project context and behavioral spec
   - `decision-log.md` - Decisions made so far

2. **Read existing contributions:**
   ```bash
   # List all contributions in chronological order
   ls -d contributions/*/

   # Examples:
   # 001-phase-1-implementation-auth-general-purpose/
   # 002-phase-2-implementation-storage-general-purpose/
   # 003-phase-2-design-session-mgmt-design-contribute/
   ```

3. **For each contribution, determine:**
   - Which phase does it belong to?
   - What type of contribution? (implementation, design, etc.)
   - What objective did it address?

4. **Build completion map:**
   - Phase 1: ✅ Complete (found implementation contribution)
   - Phase 2: ✅ Complete (found implementation contribution)
   - Phase 3: ⏳ In Progress (found design contribution, no implementation yet)
   - Phase 4: ⬜ Not Started

### What to Extract

**From implementation-roadmap.md:**
- List of all phases
- Objectives for each phase
- Which objectives are "Design: Determine [X]" vs implementation

**From contributions/:**
- Highest contribution number (to know what's latest)
- Which phases have been addressed
- Which specific objectives have been completed

## Step 3: Determine Next Objective

### Goal
Identify the specific next objective to work on.

### Decision Tree

**Check Phase by Phase in Order:**

```
For each Phase in implementation-roadmap.md:

  Check if Phase has "Design: Determine [X]" objective:
    ├─ Yes: Has design contribution for this objective?
    │   ├─ Yes: Has implementation contribution after design?
    │   │   ├─ Yes: Phase complete, check next phase
    │   │   └─ No: NEXT STEP = Implement the design (dev-contribute)
    │   └─ No: NEXT STEP = Create design (design-contribute)
    │
    └─ No: Has implementation contribution for this phase?
        ├─ Yes: Phase complete, check next phase
        └─ No: NEXT STEP = Implement phase (dev-contribute)

If all Phases complete:
  DONE = Inform user all objectives complete
```

### Examples

**Example 1: Design needed**
```
Phase 3 objective: "Design: Determine session management approach"
Contributions: 001-phase-1-..., 002-phase-2-...
Analysis: No phase 3 contributions exist yet
Decision: Next step = design-contribute for session management
```

**Example 2: Implementation needed after design**
```
Phase 3 objectives:
  1. "Design: Determine session management approach"
  2. "Implement session validation"

Contributions:
  001-phase-1-...
  002-phase-2-...
  003-phase-3-design-session-mgmt-design-contribute/

Analysis: Design exists, but no implementation yet
Decision: Next step = dev-contribute to implement session validation
```

**Example 3: Implementation needed (no design)**
```
Phase 2 objective: "Implement user storage"
Contributions: 001-phase-1-...
Analysis: Phase 1 complete, phase 2 not started, no design needed
Decision: Next step = dev-contribute for user storage
```

**Example 4: All done**
```
All phases have corresponding contributions
Analysis: Roadmap complete
Decision: Inform user, ask what's next
```

### Edge Cases

**Multiple objectives in current phase:**
- Work on them in order listed in roadmap
- If first objective is design, do that first
- If design is done, move to implementation

**Unclear which phase is next:**
- Ask user which objective they want to tackle
- Use AskUserQuestion with options

**Found partial work:**
- If contribution exists but seems incomplete
- Inform user: "Found incomplete contribution [NNN], should we continue that or start fresh?"
- Use AskUserQuestion to clarify

## Step 4: Route to Appropriate Skill

### Goal
Automatically invoke the correct skill based on next objective.

### Routing Logic

**Route to design-contribute if:**
- Next objective matches pattern "Design: Determine [X]"
- Next objective matches pattern "Design: Figure out [X]"
- Next objective contains keyword "Design:" or "Architecture decision:"
- No design contribution exists for this objective yet

**Route to dev-contribute if:**
- Next objective is implementation work
- Design contribution exists but implementation doesn't
- No "Design:" keyword in objective

### Invocation

**For design-contribute:**
```
Tell user:
"Next objective: Design: Determine [topic]
Invoking design-contribute skill to work on this design decision..."

Invoke:
Skill tool with skill="design-contribute"
```

**For dev-contribute:**
```
Tell user:
"Next objective: Implement [feature]
Invoking dev-contribute skill to implement this phase..."

Invoke:
Skill tool with skill="dev-contribute"
```

**For completion:**
```
Tell user:
"🎉 All roadmap objectives complete!

Completed phases:
- Phase 1: [objective]
- Phase 2: [objective]
- Phase 3: [objective]

What would you like to do next?
- Extend the roadmap with new phases
- Review and audit the implementation
- Start a new dev-strategy project"

Do NOT auto-invoke anything
```

## Step 5: Handle Edge Cases

### No Dev-Strategy Plan Found

**Response:**
```
"I couldn't find a dev-strategy plan in the current directory.

Would you like to:
1. Create a new dev-strategy plan (use dev-strategy skill)
2. Navigate to an existing dev-strategy project directory
3. Search for dev-strategy plans in subdirectories"
```

### Multiple Dev-Strategy Plans Found

**Response:**
```
"I found multiple dev-strategy projects:
1. ./project-a/implementation-roadmap.md
2. ./project-b/implementation-roadmap.md
3. ./features/auth/implementation-roadmap.md

Which project would you like to continue?"

Use AskUserQuestion to get user choice
```

### Roadmap Incomplete or Malformed

**Response:**
```
"I found a dev-strategy plan but couldn't determine the next step because:
- implementation-roadmap.md is missing or empty
- No clear phase structure found

Would you like me to help fix the roadmap structure?"
```

### Conflicting State

**Response:**
```
"I found an inconsistency:
- Phase 2 has no contributions yet
- But Phase 3 has contributions

This suggests Phase 2 might have been skipped. What would you like to do:
1. Go back and complete Phase 2
2. Continue with Phase 3 (if Phase 2 is truly not needed)
3. Review the roadmap to understand the situation"

Use AskUserQuestion
```

## Output Format

### Standard Output

The skill should provide clear, structured output:

```
**Dev-Strategy Project Found**
Project: [name from context-document.md]
Location: [directory path]

**Current State**
✅ Phase 1: [objective] - Complete (contribution 001)
✅ Phase 2: [objective] - Complete (contribution 002)
⏳ Phase 3: [objective] - Design complete (contribution 003)
⬜ Phase 4: [objective] - Not started

**Next Step**
Phase 3: Implement [feature based on design]

**Routing Decision**
Invoking dev-contribute skill to implement this phase...

[Skill tool invocation happens]
```

## Integration with Other Skills

### With dev-strategy
- Reads artifacts created by dev-strategy
- Does NOT modify dev-strategy artifacts
- If roadmap is complete, can suggest extending via dev-strategy

### With design-contribute
- Routes to design-contribute when "Design:" objectives found
- design-contribute creates design contribution
- Next invocation of dev-continue will detect design and route to dev-contribute

### With dev-contribute
- Routes to dev-contribute for implementation objectives
- dev-contribute creates implementation contribution
- Next invocation of dev-continue will move to next phase

### With read-contribution
- Uses similar logic to read-contribution for understanding contribution structure
- Does NOT invoke read-contribution (just uses same patterns)

## Validation Strategy

### Before Routing

**Verify:**
- ✅ Dev-strategy plan exists and is readable
- ✅ implementation-roadmap.md has clear phase structure
- ✅ Contributions directory structure matches expected format
- ✅ Next objective is unambiguously determined

**If any validation fails:**
- Inform user clearly what's wrong
- Provide actionable suggestions
- Do NOT auto-invoke skills with uncertain state

### Routing Decision Confidence

**High Confidence (auto-route):**
- Next objective is clear from roadmap
- Objective type (design vs implementation) is obvious
- No conflicting contributions found

**Medium Confidence (ask user):**
- Multiple valid next steps
- Objective type is ambiguous
- Partial work found

**Low Confidence (inform only):**
- Roadmap structure unclear
- Conflicting state detected
- Missing critical artifacts

## Examples of Full Workflow

### Example 1: Clean Happy Path

```
User: "continue work on auth-system"

Step 1: Found implementation-roadmap.md in ./auth-system/
Step 2: Read state:
  - Phase 1 complete (001-phase-1-implementation-basic-auth/)
  - Phase 2 complete (002-phase-2-implementation-user-storage/)
  - Phase 3 not started, objective: "Design: Determine session approach"
Step 3: Next = Phase 3 design objective
Step 4: Route to design-contribute
Output: "Invoking design-contribute for Phase 3 session design..."
[design-contribute skill starts]
```

### Example 2: Implementation After Design

```
User: "continue work on auth-system"

Step 1: Found implementation-roadmap.md in ./auth-system/
Step 2: Read state:
  - 001-phase-1-implementation-basic-auth/
  - 002-phase-2-implementation-user-storage/
  - 003-phase-3-design-session-mgmt-design-contribute/
  - Phase 3 has more objectives: "Implement session validation"
Step 3: Next = Implement session validation (design already done)
Step 4: Route to dev-contribute
Output: "Design complete. Invoking dev-contribute to implement session validation..."
[dev-contribute skill starts]
```

### Example 3: All Complete

```
User: "continue work on auth-system"

Step 1: Found implementation-roadmap.md in ./auth-system/
Step 2: Read state - all phases have contributions
Step 3: All objectives complete
Step 4: Celebrate and ask what's next
Output:
"🎉 All roadmap objectives complete!

Completed:
- Phase 1: Basic authentication ✅
- Phase 2: User storage ✅
- Phase 3: Session management ✅

What would you like to do next?"
```

### Example 4: Ambiguous State

```
User: "continue work"

Step 1: Found 2 dev-strategy plans
Step 2: Can't determine which one user wants
Step 3: N/A
Step 4: Ask user
Output:
"Found multiple dev-strategy projects:
1. ./auth-system
2. ./payment-system

Which would you like to continue?"
[Wait for user response]
```

## Summary

dev-continue is a smart router that:
1. **Locates** dev-strategy plan
2. **Analyzes** current state from roadmap + contributions
3. **Determines** next objective using decision tree
4. **Routes** automatically to design-contribute or dev-contribute
5. **Handles** edge cases gracefully with user clarification

**Key principles:**
- Zero user overhead - one command to continue
- Smart analysis of roadmap state
- Transparent about what it's doing
- Fails gracefully with clear guidance
