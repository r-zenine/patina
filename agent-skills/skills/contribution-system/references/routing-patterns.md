# Routing Patterns (dev-continue)

How to determine the correct next skill to invoke by analyzing roadmap state and contribution history.

## Decision Tree

```
For each Phase in implementation-roadmap.md (in order):

  Does Phase have a "Design: Determine [X]" objective?
  ├─ Yes:
  │   Has a design contribution for this objective?
  │   ├─ Yes:
  │   │   Has an implementation contribution AFTER the design?
  │   │   ├─ Yes → Phase complete, check next phase
  │   │   └─ No → NEXT STEP: dev-contribute (implement the design)
  │   └─ No → NEXT STEP: design-contribute (create the design)
  │
  └─ No:
      Has an implementation contribution for this phase?
      ├─ Yes → Phase complete, check next phase
      └─ No → NEXT STEP: dev-contribute (implement the phase)

All phases complete → Inform user; do NOT auto-invoke
```

## Routing Rules

**Route to design-contribute if:**
- Next objective matches "Design: Determine [X]"
- Next objective matches "Design: Figure out [X]"
- Objective contains "Design:" or "Architecture decision:" keyword
- No design contribution exists for this objective yet

**Route to dev-contribute if:**
- Next objective is implementation work (no "Design:" keyword)
- Design contribution exists but implementation contribution does not follow it

**Do NOT auto-invoke if:**
- All phases are complete — inform user and ask what's next
- State is ambiguous — ask user for clarification

## Routing Confidence Levels

**High Confidence → auto-route:**
- Next objective is clear from roadmap
- Objective type (design vs implementation) is unambiguous
- No conflicting contributions found

**Medium Confidence → ask user:**
- Multiple valid next steps exist
- Objective type is ambiguous
- Partial or incomplete contribution found

**Low Confidence → inform only:**
- Roadmap structure is unclear or malformed
- Conflicting state detected (Phase 3 has contributions but Phase 2 doesn't)
- Missing critical artifacts

## Edge Cases

**Multiple objectives in current phase:**
- Work through objectives in the order listed in the roadmap
- If first objective is "Design:", do that first
- If design is done, move to the next implementation objective

**Unclear which phase is next:**
- Ask user which objective to tackle via AskUserQuestion

**Partial work found:**
- "Found incomplete contribution [NNN], should we continue that or start fresh?"
- Use AskUserQuestion to clarify

**Phase 2 skipped (Phase 3 has contributions but Phase 2 doesn't):**
- Inform user of the inconsistency
- Ask whether to go back and complete Phase 2 or continue with Phase 3

## How to Read the Completion Map

From the `contributions/` directory:

```
001-phase-1-implementation-auth-general-purpose/    → Phase 1 implementation done
002-phase-2-implementation-storage-general-purpose/ → Phase 2 implementation done
003-phase-3-design-session-mgmt-design-contribute/  → Phase 3 design done, implementation pending
```

**Build completion map:**
- Phase 1: ✅ Complete (implementation contribution found)
- Phase 2: ✅ Complete (implementation contribution found)
- Phase 3: ⏳ In Progress (design contribution exists, no implementation yet)
- Phase 4: ⬜ Not Started

## Output Format

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
```
