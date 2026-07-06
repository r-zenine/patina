# Execution Requirements Template

This document defines the standard outcome-based execution structure used across all dev-strategy workflow skills. Each skill applies these phases with skill-specific variations.

## Universal Structure

All execution requirements follow this three-outcome pattern:

### Outcome 1: Foundation Phase (context is clear)
The first outcome establishes clarity about what needs to be done and what has already been done.

**Purpose**: Ensure all prerequisites are met and no blockers exist before proceeding.

**Generic Checklist** (adapt to skill-specific context):
- [ ] Prior context/decisions are understood
- [ ] Current state is clear
- [ ] No blockers exist
- [ ] Success criteria are identified

### Outcome 2: Execution Phase (work is complete)
The second outcome describes completion of the primary work and validation of quality.

**Purpose**: Ensure deliverables are complete and meet quality standards.

**Generic Checklist** (adapt to skill-specific context):
- [ ] All deliverables are complete
- [ ] Quality bar is met
- [ ] Decisions are captured
- [ ] Prior decisions still hold (or revision is needed)

### Outcome 3: Transfer Phase (knowledge is handed off)
The third outcome ensures documentation and continuity for the next phase.

**Purpose**: Enable the next phase to begin without gaps.

**Generic Checklist** (adapt to skill-specific context):
- [ ] All work is documented
- [ ] Decisions are recorded
- [ ] Context is clear for next phase
- [ ] Contribution artifacts are left uncommitted (`.plans/` is planning scratch, not part of git history)

## Criteria Ownership

Each skill's `SKILL.md` is the **sole owner** of its skill-specific done-criteria. This template defines only the generic three-outcome pattern above — it does not restate per-skill criteria, so there is exactly one place per skill to read or change them. References within a skill (tactics.md, guide.md) may restate the criteria for execution convenience but must not add to or alter them.

Criteria must be written so each one is either **confirmed by the user** or **checkable against an artifact** (a file that exists with required content, a validator that passes, a statement visible in the transcript). Avoid self-assessed criteria ("X is understood", "knowledge is sufficient") — an outcome without an observable trace didn't happen.

## How to Apply This Template

When a skill defines "Execution Requirements":

1. **State the template outcome names** (using skill-specific names when clearer)
2. **List the specific criteria** for that skill
3. **Provide concrete "How to achieve these outcomes" steps**
4. **Reference relevant supporting skills** only when needed for that specific step

This approach ensures:
- Consistent structure across all skills
- Clear outcome-based thinking
- Single source of truth for the template
- Skills can vary while maintaining coherent patterns
