# ADR Format

ADRs live in `docs/adr/` and use 4-digit sequential numbering: `0001-slug.md`, `0002-slug.md`, etc.

Create the `docs/adr/` directory lazily — only when the first ADR is needed.

## Template

```md
# {Short title of the decision}

{1–3 sentences: what's the context, what did we decide, and why.}
```

That's it. An ADR can be a single paragraph. The value is in recording *that* a decision was made and *why* — not in filling out sections.

## Optional Sections

Only include these when they add genuine value. Most ADRs won't need them.

**Considered Options** — only when the rejected alternatives are worth remembering. If someone might suggest the rejected option again in six months, record it here.

```md
## Considered Options

- **[Option A]**: [one sentence] — rejected because [specific reason]
- **[Option B]**: [one sentence] — chosen because [specific reason]
```

**Consequences** — only when non-obvious downstream effects need calling out.

```md
## Consequences

[One or two sentences on what this decision enables or constrains downstream.]
```

**Status frontmatter** — only when this ADR revises or supersedes another.

```md
---
status: superseded by ADR-0004
---
```

Valid statuses: `proposed | accepted | deprecated | superseded by ADR-NNNN`

