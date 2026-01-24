---
name: dev-continue
description: Smart router that analyzes dev-strategy project state and automatically invokes the appropriate skill (design-contribute or dev-contribute) for the next phase. Use when users say "continue work on [project]", "what's next for [project]", or "keep going on [project]". Eliminates need for users to manually determine which skill to use.
allowed-tools: ["Read", "Glob", "Grep", "Skill"]
---

# Dev Continue Skill

Intelligent router that determines the next step in a dev-strategy implementation and automatically invokes the correct skill.

## What This Skill Does

Analyzes dev-strategy project state and automatically routes to the appropriate skill:
- Reads implementation roadmap to identify current phase and next objective
- Reviews existing contributions to determine what's been completed
- Identifies whether next step is design or implementation
- Automatically invokes `design-contribute` or `dev-contribute` as needed
- Informs user when all objectives are complete

## When to Use This Skill

✅ **Use when:**
- User wants to continue work on an existing dev-strategy project
- User asks "continue work on [project]", "what's next?", "keep going"
- User wants to know what step comes next without manually checking roadmap
- Resuming work after a break and need to figure out where you left off

❌ **Don't use when:**
- Creating a new dev-strategy plan (use `dev-strategy` skill instead)
- User explicitly specifies which skill to use (use that skill directly)
- Working on non-dev-strategy projects
- User wants to review/audit only (not continue implementation)

## How It Works

**Simple 3-step process:**

1. **Analyze State**: Read dev-strategy plan and existing contributions
2. **Determine Next Step**: Identify next incomplete objective from roadmap
3. **Route**: Automatically invoke appropriate skill:
   - "Design: Determine [X]" → invoke `design-contribute`
   - Implementation objective → invoke `dev-contribute`
   - All done → inform user and ask what's next

## User Experience

**Before (manual):**
```
User: "I want to keep working on my auth system"
Agent: "Let me check the roadmap... you need to design session management...
        you should use the design-contribute skill"
User: "ok, run design-contribute"
[skill starts]
```

**After (with dev-continue):**
```
User: "continue work on auth system"
[dev-continue automatically reads state and invokes design-contribute]
```

## Core Principles

**Zero User Overhead**
- User shouldn't need to know which skill to use
- User shouldn't need to read roadmap manually
- One command to continue work

**Smart Routing**
- Analyze roadmap state accurately
- Detect design vs implementation objectives
- Handle edge cases (nothing done yet, all done, blocked)

**Transparency**
- Tell user what was detected: "Next objective: Design session management approach"
- Tell user which skill is being invoked: "Invoking design-contribute..."
- If unclear, ask user for clarification

**Fail Gracefully**
- If roadmap unclear or missing, inform user
- If multiple paths possible, ask user to choose
- If all done, celebrate and ask what's next

## Expected Output

The skill will:
1. Output what it detected: "Found dev-strategy plan: [project name]"
2. Output current state: "Completed: Phase 1, Phase 2. Next: Phase 3 - Design session management"
3. Output routing decision: "Next step requires design. Invoking design-contribute skill..."
4. Invoke the appropriate skill automatically

## Additional Instructions

For detailed workflow, routing logic, and edge case handling, see [reference.md](reference.md).
