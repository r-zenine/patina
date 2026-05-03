# ADR Contribute Skill — Tactics Reference

Tactics available for reaching each adr-contribute outcome.
The agent chooses which tactics to apply and in what order based on context.

---

## HARVEST Outcome

Done when:
- All decision-log.yaml files across the plan have been read
- Each decision has been assessed against the three gates
- A candidate list has been presented to the user with reasoning per candidate
- Decisions that pass all three gates are marked "Recommended"
- Decisions that fail one or more gates are shown with the failing gate noted

### Tactic: Verify Plan Exists

**When to use**: Always — first tactic before any other work.

**STOP if any of these are true:**
- No `.plans/plan-[FEATURE-NAME]/` folder exists
- No `contributions/` directory exists inside the plan folder
- The plan was not created by the `dev-strategy` skill

If the plan doesn't exist, tell the user to run `dev-strategy` first. This skill ONLY works with plans that have at least one contribution.

---

### Tactic: Read All Decision Logs

**When to use**: Plan exists and contributions/ directory is non-empty.

Read decision logs in this order:

1. **Root plan decision-log**: `.plans/plan-[FEATURE-NAME]/decision-log.yaml`
   — Strategy-level decisions (e.g. execution strategy choice, architecture framing)

2. **All contribution decision logs**: `.plans/plan-[FEATURE-NAME]/contributions/*/decision-log.yaml`
   — Read in sequential order (001, 002, 003, ...)
   — Note whether each contribution is design type (code_impacts: []) or implementation type (code_impacts populated)

For each decision extracted, record:
- Source (root plan / contribution folder name)
- Decision number and title
- Rationale text
- Whether code_impacts is populated (implementation decision) or empty (design/strategy decision)
- Affected files and line ranges (from code_impacts, if present)

**Do not read context-handoff.md files for this tactic** — decision-log.yaml is the authoritative source. Context handoffs are for progressive disclosure during implementation, not for decision harvesting.

---

### Tactic: Apply Three-Gate Filter

**When to use**: All decision logs are read — assess each decision against the three gates.

Apply all three gates to each extracted decision. A decision must pass ALL THREE to be recommended as an ADR candidate.

#### Gate 1: Hard to Reverse

The decision would be costly or time-consuming to change later.

**Signals that pass Gate 1:**
- Touches core abstractions or shared interfaces (visible in code_impacts spanning multiple files)
- Commits to a technology, protocol, or data format with lock-in
- Establishes a structural pattern others will build on
- Changes a domain boundary or ownership rule

**Signals that fail Gate 1:**
- Implementation detail confined to a single module
- Decision is easy to reverse with a focused refactor
- Choice of one equivalent library over another (if either could be swapped in a day)

#### Gate 2: Surprising Without Context

A future reader looking at the code would wonder "why did they do it this way?" — the code alone doesn't explain it.

**Signals that pass Gate 2:**
- Deliberately avoided the obvious approach for a non-obvious reason
- Constraint came from outside the codebase (compliance, partner API, deployment target)
- The code looks suboptimal without knowing the trade-off that was made
- Rejected a pattern that a reasonable engineer would expect to see here

**Signals that fail Gate 2:**
- The code is self-explanatory or follows an obvious pattern
- The decision is well documented in comments or README already
- Any competent reader would make the same choice without being told why

#### Gate 3: Real Trade-Off

There were genuine alternatives and one was chosen for specific reasons.

**Signals that pass Gate 3:**
- Rationale explicitly mentions alternatives considered
- A design contribution (design-doc.md) shows options were explored
- The rejected option has real merit — it wasn't obviously wrong

**Signals that fail Gate 3:**
- "We did the obvious thing" — there was no meaningful alternative
- The rationale only explains what was done, not why over alternatives
- One option was clearly superior; no trade-off was made

---

### Tactic: Present Candidate List

**When to use**: Three-gate filter applied to all decisions.

Present a table of all decisions assessed, then a summary of recommended candidates.

**Format:**

```
## ADR Candidates — [Plan Name]

| # | Decision | Source | Gate 1 | Gate 2 | Gate 3 | Verdict |
|---|----------|--------|--------|--------|--------|---------|
| 1 | [title]  | 002-phase-1-... | ✓ | ✓ | ✓ | Recommended |
| 2 | [title]  | root plan | ✓ | ✓ | - | Optional (Gate 3 weak) |
| 3 | [title]  | 004-phase-2-... | - | ✓ | ✓ | Skip (not hard to reverse) |
| 4 | [title]  | 001-phase-1-... | - | - | - | Skip |

**Recommended for ADR**: 1
**Optional** (judgment call): 2
**Not recommended**: 3, 4

Which would you like to write? Enter numbers (e.g. "1, 2"), "all recommended", or "none".
```

**Rules:**
- Show ALL decisions assessed, not just recommended ones — the human may disagree with the filter
- For each failing gate, note briefly why it failed (one clause)
- Never pre-write ADRs before user confirms selection
- Don't ask multiple questions at once — present the table, then ask for selection

---

## SELECTION Outcome

Done when:
- User has confirmed which phases were completed vs. intentionally skipped
- User has selected which ADR candidates to write
- If phases were skipped: brief rationale is captured (one sentence per skipped phase)

### Tactic: Confirm Plan Completion Status

**When to use**: After candidate list is presented, before finalising ADR selection.

Ask via AskUserQuestion:

```
Before I write the ADRs, I want to capture what was built and what was left out.

Looking at the roadmap:
- Phase 1: [objective] — contribution 001 exists ✓
- Phase 2: [objective] — contribution 003 exists ✓
- Phase 3: [objective] — no contribution found

Was Phase 3 intentionally skipped? If so, briefly why?
(e.g. "scope reduced", "deferred to next iteration", "dependency not ready")
```

Only ask about phases with no contributions — don't make the user re-confirm completed work.

If all roadmap phases have contributions, skip this tactic entirely.

---

### Tactic: Confirm ADR Selection

**When to use**: Completion status confirmed — get final ADR selection.

If not already captured from the HARVEST step:

```
Which candidates would you like to write as ADRs?
- Enter numbers: "1, 3"
- "all recommended" — write only recommended candidates
- "all" — write everything in the table including optional
- "none" — wrap up without writing ADRs
```

Accept the answer and proceed. Don't debate the user's choices.

---

## DOCUMENTATION Outcome

Done when:
- Selected ADRs are written to docs/adr/ with correct sequential numbering
- Wrap-up contribution folder exists with wrap-summary.md
- All new files are committed to git

### Tactic: Scan Existing ADRs

**When to use**: User has selected at least one ADR to write — before writing any file.

```bash
ls docs/adr/ 2>/dev/null | sort | tail -1
```

If `docs/adr/` does not exist, the first ADR will be `0001-[slug].md` — create the directory lazily when writing the first file.

Determine the next available number by finding the highest existing `NNNN-` prefix and incrementing by one. Pad to 4 digits.

---

### Tactic: Write Selected ADRs

**When to use**: Next ADR number is known — write one file per selected candidate.

For each selected candidate:

1. Derive a slug from the decision title: lowercase, words separated by hyphens, max 5 words
   - Example: "Use Steel Thread execution strategy" → `steel-thread-execution-strategy`

2. Write `docs/adr/NNNN-[slug].md` using [assets/adr-template.md](../assets/adr-template.md)

3. Mandatory content:
   - Title: `# [Short title of the decision]`
   - Summary paragraph: 1–3 sentences — what's the context, what did we decide, and why

4. Optional sections (only include when they add genuine value):
   - **Considered Options** — only when rejected alternatives are worth remembering
   - **Consequences** — only when non-obvious downstream effects need calling out
   - **Status** frontmatter — only when this ADR supersedes or is superseded by another

**Size target**: Most ADRs should be under 15 lines. An ADR can be a single paragraph — that's enough.

**Source material to draw from:**
- The decision's `rationale` field from the decision-log.yaml
- The `design-doc.md` from the corresponding design contribution (for Considered Options)
- The `code_impacts` (for understanding scope — don't paste file lists into the ADR)

**What NOT to include:**
- ❌ No file lists or line number references
- ❌ No implementation details (the code holds those)
- ❌ No paraphrasing of what the code does — only why it was done this way

---

### Tactic: Create Wrap-Up Contribution Folder

**When to use**: All selected ADRs are written — before committing.

1. Determine the next sequential contribution number:
   ```bash
   ls .plans/plan-[FEATURE-NAME]/contributions/ | sort | tail -1
   ```

2. Create the folder:
   ```bash
   mkdir .plans/plan-[FEATURE-NAME]/contributions/NNN-plan-wrap-adr-contribute/
   ```

3. Write `wrap-summary.md` with three sections:

```markdown
# Plan Wrap-Up: [Plan Name]

## Implementation Status

| Phase | Objective | Status | Contributions |
|-------|-----------|--------|---------------|
| 1     | [objective] | Complete | 001, 002 |
| 2     | [objective] | Complete | 003 |
| 3     | [objective] | Intentionally skipped — [one-sentence reason] | — |

## ADRs Written

| ADR | Decision |
|-----|----------|
| [0001-slug.md](../../../../docs/adr/0001-slug.md) | [Decision title] |

## ADR Candidates Not Written

| Decision | Reason skipped |
|----------|----------------|
| [Decision title] | [Which gate failed and why] |
```

Omit the "ADRs Written" section if no ADRs were written. Omit the "Candidates Not Written" section if all candidates were written or no candidates existed.

---

### Tactic: Commit Everything

**When to use**: All ADR files and wrap-summary.md are written and reviewed.

Stage and commit in one commit:

```bash
git add docs/adr/
git add .plans/plan-[FEATURE-NAME]/contributions/NNN-plan-wrap-adr-contribute/
git commit -m "wrap(NNN): [plan-name] — [N] ADR(s) written"
```

**Rules:**
- Do NOT use `git add -A` or `git add .`
- Stage only the new ADR files and the wrap contribution folder
- Commit message format: `wrap(NNN): [plan-name] — [N] ADR(s) written`
  - NNN is the contribution folder number
  - If no ADRs were written: `wrap(NNN): [plan-name] — plan complete, no ADRs`

---

## Quality Reference

### Three-Gate Filter Quick Reference

| Gate | Question | Fail if... |
|------|----------|-----------|
| Hard to reverse | Would changing this cost meaningful time? | Easy refactor, local change |
| Surprising without context | Would a reader wonder "why?" without this ADR? | Code is self-explanatory |
| Real trade-off | Were genuine alternatives rejected? | Only one option existed |

### What Makes a Good ADR Title

- States the decision, not the topic: ✓ "Use event sourcing for the order aggregate" / ✗ "Order aggregate storage"
- Specific enough to be meaningful out of context
- Short enough to scan in a list (under 10 words)

### Integration With Other Skills

**Reading from dev-strategy:**
- `implementation-roadmap.md` — Phase list for completion status
- `decision-log.yaml` — Root-level strategy decisions

**Reading from design-contribute contributions:**
- `decision-log.yaml` — Design decisions (code_impacts empty)
- `design-doc.md` — Source material for "Considered Options" section

**Reading from dev-contribute contributions:**
- `decision-log.yaml` — Implementation decisions (code_impacts populated)

**Not reading:**
- `context-handoff.md` files — not needed for ADR harvesting
- `code-context.md` / `context-document.md` — background only; decision-logs are authoritative
