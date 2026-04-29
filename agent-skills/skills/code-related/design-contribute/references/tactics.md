# Design Contribute Skill — Tactics Reference

Tactics available for reaching each design-contribute outcome.
The agent chooses which tactics to apply and in what order based on context.

---

## UNDERSTANDING Outcome

Done when:
- Dev-strategy plan is confirmed to exist and be valid
- Current behavioral spec and architecture are understood
- Relevant implementation contributions have been reviewed
- Design objective from the roadmap is clearly identified
- Reason for deferral (what implementation was meant to reveal) is understood

### Tactic: Verify Plan Exists

**When to use**: Always — first tactic before any other work.

**STOP if any of these are true:**
- No `.plans/plan-[FEATURE-NAME]/` folder exists
- The plan was not created by the `dev-strategy` skill
- No `implementation-roadmap.md` exists in the plan folder

If the plan doesn't exist, tell the user to run the `dev-strategy` skill first. This skill ONLY works with plans created by `dev-strategy`.

---

### Tactic: Read Plan Context

**When to use**: Plan exists — read all dev-strategy artifacts before any interaction.

Read in order:
1. `context-document.md` — Behavioral spec and architecture
2. `decision-log.yaml` — Previous decisions made during planning
3. `implementation-roadmap.md` — Find the specific "Design:" objective for this contribution

Key information to extract:
- The specific "Design: Determine [X]" objective in the roadmap
- Why design was deferred (what needed to be learned from implementation first)
- Architecture patterns and constraints established at planning time

---

### Tactic: Review Implementation Contributions

**When to use**: A `contributions/` directory exists in the plan folder.

Read enough recent contributions to understand what implementation has revealed:

1. List `.plans/plan-[FEATURE-NAME]/contributions/` and identify the next sequential number
2. Read `context-handoff.md` files from recent contributions (focus on the most recent)
3. Browse implementation code if the handoff doesn't surface concrete constraints

Key information to extract:
- What patterns emerged during implementation
- What integration constraints were discovered
- What the current state of the codebase is

For progressive disclosure reading strategy, see [`contribution-system/references/progressive-disclosure.md`](../contribution-system/references/progressive-disclosure.md).

---

## EXPLORATION Outcome

Done when:
- Implementation learnings and constraints are captured (from user + code review)
- User priorities are understood (explicitly asked, not inferred)
- 2–3 design options were presented (simplest first)
- Design options respect [`design-principles/SKILL.md`](../../design-principles/SKILL.md)
- No anti-patterns listed under [`design-principles/references/anti-patterns.md`](../../design-principles/references/anti-patterns.md) has be used in the design proposals
- User explicitly chose one option
- Chosen design addresses the "Design:" objective from the roadmap

### Tactic: Constraint Discovery Questioning

**When to use**: UNDERSTANDING outcome is met — ask before analyzing to surface constraints implementation revealed.

**Ask first, then validate against code. Never assume constraints without asking.**

Ask via AskUserQuestion:
- "What did you learn during implementation that should inform this design?"
- "What patterns or approaches worked well so far?"
- "What integration challenges did you encounter?"
- "What's your priority: simplicity, performance, or flexibility?"
- "Are there any non-negotiable requirements?"

Then validate against the code:
- Review implementation code yourself
- Confirm: "I see pattern X emerged — is that correct?"
- Document concrete constraints: integration points, established patterns, technical constraints, user priorities

For complete AskUserQuestion patterns, see [`contribution-system/references/constraint-discovery.md`](../contribution-system/references/constraint-discovery.md).

**What NOT to do:**
- ❌ Don't assume constraints without asking
- ❌ Don't design based on hypothetical future needs
- ❌ Don't infer priorities — ask explicitly

---

### Tactic: Interactive Option Exploration

**When to use**: Constraints are understood — present design options and get explicit user choice.

**Never design in isolation. Always involve the user in option exploration.**

1. **Generate 2–3 options maximum:**
   - **Option 1**: Simplest approach (always first) — mark as "(Recommended)"
   - **Option 2**: Alternative with different trade-offs
   - **Option 3**: Only if a significantly different approach exists
   - **If considering a 4th**: Stop. You're over-engineering. Simplify instead.

2. **For each option, document:**
   - **Approach**: What is it? (1–2 sentences)
   - **How it works**: Brief explanation
   - **Pros / Cons**
   - **Complexity**: Low / Medium / High

3. **Present to user via AskUserQuestion** — ask user to choose, include "Other"

4. **Discuss trade-offs** — answer questions, clarify what each option enables or prevents

5. **Iterate if needed** — max 2–3 rounds

**Option presentation format:**

```markdown
### Option 1: [Simple Approach] (Recommended)
**Approach**: [What is it]
**How it works**: [Brief explanation]
**Pros**: [Benefits]
**Cons**: [Limitations]
**Complexity**: Low
**Why recommended**: [Simplest approach that solves the current need]

### Option 2: [Alternative Approach]
**Approach**: [What is it]
**Pros**: [Benefits]
**Cons**: More complex; [other trade-offs]
**Complexity**: Medium
```

**What NOT to do:**
- ❌ Don't design in isolation then present final design
- ❌ Don't present more than 3 options
- ❌ Don't make the decision for the user
- ❌ Don't skip getting explicit user choice
- ❌ Don't recommend complex options over simple ones

---

## DOCUMENTATION Outcome

Done when:
- Contribution folder exists with correct sequential numbering
- `design-doc.md` exists and is under 100 lines
- `decision-log.yaml` documents the primary design decision with rationale
- `context-handoff.md` exists and is under 30 lines
- An implementer can build from `design-doc.md` alone without asking questions
- Design addresses only the current phase objective (YAGNI applied)
- Changes are committed to git

### Tactic: Create Design Contribution Folder

**When to use**: EXPLORATION outcome is met — before writing any files.

**IMPORTANT: Contributions are ALWAYS saved in `.plans/plan-[FEATURE-NAME]/contributions/`**

1. Check existing contribution numbers:
   ```bash
   ls .plans/plan-[FEATURE-NAME]/contributions/
   ```

2. Pick the next number and create folder:
   ```bash
   mkdir .plans/plan-[FEATURE-NAME]/contributions/005-phase-3-design-session-mgmt-design-contribute/
   ```

3. Folder naming convention: `NNN-phase-X-design-[topic]-design-contribute`

For folder naming details, see [`contribution-system` skill](../contribution-system/SKILL.md).

---

### Tactic: Document Design Decision

**When to use**: Contribution folder exists — create the 3 artifacts.

For full artifact schemas, see [`contribution-system/references/design-artifacts.md`](../contribution-system/references/design-artifacts.md).
For templates, see [`contribution-system/assets/templates/`](../contribution-system/assets/templates/).

**Create exactly three files:**

**1. design-doc.md** (< 100 lines target)

Use [design-doc-template.md](../contribution-system/assets/templates/design-doc-template.md)

Document:
- Why this design (constraints + user priorities)
- How it works
- What we're NOT doing (scope boundaries)
- Implementation guidance

**2. decision-log.yaml**

Get the template by running: `diffviz templates decision-log`

```yaml
commit: null  # Design decisions don't have code yet

decisions:
  - number: 1
    title: "[One sentence summary]"
    rationale: "[Why...]"
    code_impacts: []  # Empty for design phase; filled in by implementation
```

**3. context-handoff.md** (< 30 lines target)

Document three things only:
1. What problem are we solving with this design (5–10 lines)
2. High-level overview of the design-doc (15 lines)
3. Reading guide for design-doc.md (5 lines)

**Quality checks before committing:**

| Check | Criterion |
|-------|-----------|
| Implementer readiness | Can someone implement from design-doc.md alone? |
| Integration points | Are they clearly specified? |
| Simplicity | Is design-doc.md under 100 lines? |
| Scope | Does design only address the current phase objective? |
| YAGNI | Did we document what we're NOT doing? |
| Interactive quality | Did we ask user about learnings and present options? |

**What NOT to include:**
- ❌ No code files (documentation only)
- ❌ No comprehensive specifications
- ❌ No prototypes or proof-of-concepts
- ❌ No design for future phases

---

### Tactic: Commit the Design Contribution

**When to use**: All 3 files are written and quality checks pass.

```bash
git add .plans/plan-[FEATURE-NAME]/contributions/<contribution-folder>/
git commit -m "design(NNN): <description matching contribution folder name>"
```

**Rules:**
- Do NOT use `git add -A` or `git add .`
- Use the full path when staging
- The commit message number (NNN) must match the contribution folder number
- The description must match the contribution folder name

---

## Quality Reference

### Enforcement Rules

**Always present simplest option first** — Option 1 must be simplest; mark as "Recommended" with rationale.

**Maximum 3 options** — If considering a 4th, you're over-engineering; stop and simplify.

**design-doc.md < 100 lines** — If exceeding 100 lines, focus on "what" and "why", not exhaustive "how".

**Only current phase objective** — Design addresses the specific "Design:" task from the roadmap only.

For complete anti-patterns (over-engineering and under-engineering signals), see [`design-principles/references/anti-patterns.md`](../design-principles/references/anti-patterns.md).

### Integration with Other Skills

**Reading from dev-strategy:**
- `implementation-roadmap.md` — Find "Design:" objectives
- `context-document.md` — Understand behavioral spec
- `decision-log.yaml` — Previous decisions and constraints

**Handing off to dev-contribute:**
- `design-doc.md` becomes the specification for the next implementation contribution
- dev-contribute reads design-doc.md and implements according to the guidance
- Reference the design contribution number in the implementation context-handoff.md

### Example Flow

```
Roadmap Phase 3: "Design: Determine session management approach"

[Existing contributions:]
001-phase-1-implementation-basic-auth/
002-phase-2-implementation-user-storage/
003-phase-3-implementation-jwt-auth/

[design-contribute invoked:]
Tactic: Verify Plan Exists              → plan confirmed
Tactic: Read Plan Context               → objective: "Design: session management"
Tactic: Review Contributions            → JWT work revealed stateless constraint
Tactic: Constraint Discovery Questioning → User: "Need stateless, short tokens"
Tactic: Interactive Option Exploration  → JWT-only vs JWT+Redis; user picks JWT-only
Tactic: Create Contribution Folder      → 004-phase-3-design-session-mgmt-design-contribute/
Tactic: Document Design Decision        → design-doc.md, decision-log.yaml, context-handoff.md
Tactic: Commit                          → "design(004): phase-3-design-session-mgmt-design-contribute"

[Next dev-contribute:]
005-phase-3-implementation-session-validation/
↓ Reads design-doc.md from contribution 004
```
