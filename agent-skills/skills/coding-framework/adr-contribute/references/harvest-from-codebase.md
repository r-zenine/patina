# Harvest Strategy: From Codebase

Use this strategy when no `.plans/` directory exists — the codebase evolved without a dev-strategy plan, or the skill is called on a bare repository.

Decisions must be inferred from evidence rather than read from structured logs. The same three-gate filter applies, but confidence is lower — flag inferred rationale clearly so the human can fill gaps.

---

## Before You Start: Confirm Depth Limit

Ask the user before harvesting:

```
No dev-strategy plan found — I'll harvest decisions from git history and code.

Default scope: last 100 commits or 6 months, whichever is smaller.
Is that the right window, or would you like to adjust?
```

Accept the user's answer and proceed. Don't debate the choice.

---

## Tactic: Mine Git History

Scan commit messages for decision language. Run:

```bash
git log --oneline --since="6 months ago" | head -100
```

Collect commits whose messages contain signals of a deliberate choice:
- Keywords: "chose", "use", "switch to", "instead of", "because", "trade-off", "decided", "avoid", "replace", "revert to"
- Merge commit messages that summarize a feature or refactor
- Commits with "why" language rather than just "what"

For each candidate commit, retrieve the full message:
```bash
git show --no-patch --format="%B" <hash>
```

Record: commit hash, short message, any rationale text, files touched (from `git show --stat`).

---

## Tactic: Check PR Descriptions

If a GitHub remote is configured, scan merged PR descriptions for decision context:

```bash
gh pr list --state merged --limit 50 --json number,title,body
```

PR descriptions often contain the "why" that commit messages omit. Flag PRs where the body explains a deliberate choice or rejected alternative.

Skip this tactic if `gh` is not available or no remote is configured.

---

## Tactic: Scan Code Comments

Search for inline decision markers:

```bash
git grep -n "DECISION:\|HACK:\|NOTE:\|FIXME:" -- '*.rs' '*.go' '*.ts' '*.py'
```

Adjust file extensions to match the repo's languages. For each match, read a few lines of surrounding context to assess whether it records a real decision or just a to-do.

---

## Tactic: Read Existing Docs

Check for prose that captures past choices:

- `README.md` — architecture or design rationale sections
- `CHANGELOG.md` or `CHANGELOG/` — breaking changes and migration notes
- Any files under `docs/` or `doc/`

Skim for decisions, not for feature descriptions.

---

## Tactic: Apply Three-Gate Filter

Apply the same three gates as the plan-based strategy, with codebase-specific signals.

### Gate 1: Hard to Reverse

**Signals that pass:**
- Touches core abstractions, shared interfaces, or public API surface (many files in `git show --stat`)
- Commits to a technology, serialization format, or protocol
- Structural patterns that other code builds on

**Signals that fail:**
- Isolated to a single module or function
- Clearly reversible in a day's work

### Gate 2: Surprising Without Context

**Signals that pass:**
- Commit message or PR description says "instead of the obvious X, we did Y because..."
- Code comment explicitly documents a counter-intuitive choice
- The code looks suboptimal unless you know the constraint

**Signals that fail:**
- Commit message only says what changed, not why
- The approach is conventional for the language/framework

### Gate 3: Real Trade-Off

**Note:** This gate is hardest to assess from git history alone. Look for:

**Signals that pass:**
- PR description or commit message explicitly mentions alternatives considered
- A revert followed by a different approach (two commits together = a trade-off was made)
- Comment says "we tried X first"

**Signals that fail:**
- Single commit, no mention of alternatives
- Rationale is "best practice" or "standard approach" — no genuine trade-off

**When confidence is low on Gate 3:** mark the candidate as "inferred — rationale unclear" in the table. The human can confirm or discard.

---

## Tactic: Present Candidate List

Same format as the plan-based strategy, with one addition — flag inferred rationale:

```
## ADR Candidates — [Repo / Feature Area]

| # | Decision | Source | Gate 1 | Gate 2 | Gate 3 | Verdict |
|---|----------|--------|--------|--------|--------|---------|
| 1 | [title]  | commit abc1234 | ✓ | ✓ | ✓ | Recommended |
| 2 | [title]  | PR #42 | ✓ | ✓ | ~ | Optional (Gate 3 inferred) |
| 3 | [title]  | README.md | - | ✓ | ✓ | Skip (not hard to reverse) |

~ = rationale inferred from context, not explicitly stated

**Recommended for ADR**: 1
**Optional** (judgment call): 2
**Not recommended**: 3

Which would you like to write? Enter numbers, "all recommended", or "none".
```

**Rules:**
- Show ALL decisions assessed
- Use `~` for inferred gate assessments; note the inference briefly
- Never pre-write ADRs before user confirms selection
