# ADR Contribute Skill — Tactics Reference

Tactics available for reaching each adr-contribute outcome.
The agent chooses which tactics to apply and in what order based on context.

---

## HARVEST Outcome

Done when:
- All decisions have been assessed against the three-gate filter
- A candidate list has been presented to the user with reasoning per candidate
- Decisions that pass all three gates are marked "Recommended"
- Decisions that fail one or more gates are shown with the failing gate noted

### Context Detection — Choose a Harvest Strategy

**First action before any other work**: read the invocation prompt.

- **Plan name or path provided** (e.g. "wrap up plan-auth-refactor", "adr-contribute .plans/plan-auth-refactor") → load and follow [harvest-from-plan.md](harvest-from-plan.md)
- **No plan specified** → load and follow [harvest-from-codebase.md](harvest-from-codebase.md) (default)

Each strategy file is self-contained: it describes what to read, how to apply the three-gate filter, and how to present the candidate list.

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

### What Makes a Good ADR Title

- States the decision, not the topic: ✓ "Use event sourcing for the order aggregate" / ✗ "Order aggregate storage"
- Specific enough to be meaningful out of context
- Short enough to scan in a list (under 10 words)
