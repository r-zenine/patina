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
- [ ] Contributions are committed

## Skill-Specific Variations

### Dev-Strategy Skill

**Outcome 1: UNDERSTANDING PHASE** (context is clear)
- Behavioral spec is clear (user confirms what to build)
- Codebase architecture is understood (relevant classes/functions identified)
- Technology knowledge is sufficient (research done only if unfamiliar tech is involved)
- Constraints are discovered (user confirms project boundaries, dependencies, deadlines)

**Outcome 2: STRATEGY PHASE** (approach is selected)
- Best strategy is chosen based on project characteristics
- Strategy aligns with constraints and requirements
- Trade-offs between strategies are understood

**Outcome 3: PLANNING PHASE** (execution is clear)
- All 4 core artifacts exist (code-context.md, context-document.md, decision-log.yaml, implementation-roadmap.md)
- Roadmap is phased appropriately (each phase is a complete deliverable)
- First phase has sufficient detail to begin work
- Future phases are deferred appropriately per Last Responsible Moment principle

### Design-Contribute Skill

**Outcome 1: UNDERSTANDING PHASE** (context is clear)
- Current plan state is understood (roadmap, prior decisions, constraints visible)
- Implementation learnings are captured (what did prior phases reveal?)
- Design objective is clear and scoped (what specifically needs designing?)
- Assumptions about design are validated (don't assume, ask)

**Outcome 2: EXPLORATION PHASE** (design is validated with user)
- Options are presented (2-3 approaches, simplest first)
- Trade-offs are explicit (why each option exists, what you give up)
- User validates choice explicitly (confirmation of preference, not inference)
- Design decision is sufficiently detailed for implementation handoff
- Design follows simplicity principles (YAGNI, Sufficient Design, KISS)

**Outcome 3: DOCUMENTATION PHASE** (design is durable and transferable)
- Design doc exists (< 100 lines target, clear enough for implementers)
- Decisions are recorded (decision-log.yaml populated with reasoning)
- Context is handed off (context-handoff.md < 30 lines, tells next phase what was solved)
- Contribution folder exists with sequential numbering (NNN-phase-X-design-[topic]-design-contribute)

### Dev-Contribute Skill

**Outcome 1: FOUNDATION** (context is clear and phase is ready)
- Roadmap is understood (current phase objectives, what comes next)
- Prior decisions are known (what was decided in earlier phases, constraints they impose)
- Strategy is clear (TDD/Steel-Thread/Core-then-Integrate approach for this phase)
- Success criteria are identified (what deliverables complete this phase?)
- Phase is not blocked by prior decisions

**Outcome 2: EXECUTION** (work is complete and quality bar is met)
- Phase deliverables are complete (all objectives for this phase achieved)
- Quality bar is met (code passes tests, reviews pass, security/performance requirements met)
- Decisions are captured (what trade-offs were made, why this approach was chosen?)
- Prior decisions still hold (no contradictions discovered with earlier choices)
- If prior decisions need revisiting, a revision contribution is created with reasoning

**Outcome 3: KNOWLEDGE TRANSFER** (next phase is unblocked)
- Code is committed (all code changes committed before documentation is written — decision-log.yaml references commit hash)
- Decisions are recorded (decision-log.yaml with code_impacts marking only the critical impacts a reviewer must scrutinize, commit field populated with git hash)
- Context is handed off (context-handoff.md tells next phase what to focus on and why)
- Contribution folder is committed (contribution folder with decision-log.yaml and context-handoff.md staged and committed)

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
