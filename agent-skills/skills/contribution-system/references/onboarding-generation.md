# Onboarding Guide Generation with Explore Agent

Generate and update `onboarding.md` orientation guides for code modules using the Explore agent.

## When to Invoke

Use the Explore agent to generate/update onboarding guides before analyzing a crate or when the codebase has changed:

- **Generate new guide** — Starting analysis of a crate with no existing `onboarding.md`
- **Update existing guide** — Git diffs exist in the crate since the guide was last written
- **Refresh after major changes** — Directory structure, public APIs, or architectural constraints have shifted

## Generate vs Update Detection

**Check for existing documentation first:**

1. **Check for existing `onboarding.md`** in the crate root or documentation directory
2. **If found:** Pass it to the Explore agent as context for incremental updates
3. **If not found:** Invoke Explore agent in generate mode

**Update mode (file exists):**
- Read the existing `onboarding.md`
- Pass to Explore agent with context: "Here is the current onboarding guide. Check for changes and update only what has changed since this was written."
- Preserve manually-added sections; focus on changed capabilities, types, and constraints

**Generate mode (no file):**
- Invoke Explore agent to discover and document module patterns from scratch

## Explore Agent Invocation Template

Use the Explore agent with `thoroughness: "medium"` (standard codebase exploration, not deep analysis):

### Generate Mode

```
subagent_type: "Explore"
description: "Generate onboarding guide"
prompt: """
Create an onboarding guide for the [CRATE_NAME] crate that prevents code agents from reinventing existing patterns and missing reusable abstractions.

Analyze the crate and document:

1. **What This Module Does** — Single sentence describing the crate's primary responsibility in the system.

2. **Before You Code Here** — 2-3 established patterns agents should follow, key reusable types/DTOs to use instead of creating new ones, and how this crate connects to others (don't duplicate).

3. **Key Abstractions to Reuse** — 3-4 core types, traits, or modules that solve common problems. Include what problems each abstraction solves.

4. **Architectural Constraints** — Non-negotiable rules that prevent contradictory implementations. Examples: module layering rules, forbidden patterns, integration contracts.

5. **Directory Map** — Only directories that indicate where different types of changes belong (src/, tests/, benches/, examples/, etc.). Skip detailed file lists.

Organize as markdown with these exact H2 headings. Output a guide under 200 lines. Focus on answering: What patterns exist that should be reused? What types should not be reinvented? What constraints must be respected?

Do NOT include: line numbers, implementation details, historical commit context, or "nice-to-know" information.
"""
thoroughness: "medium"
```

### Update Mode

```
subagent_type: "Explore"
description: "Update onboarding guide"
prompt: """
Review and update the onboarding guide for the [CRATE_NAME] crate. Here is the current guide:

[PASTE EXISTING onboarding.md HERE]

Check the current codebase for:
1. New types or abstractions added since this guide was written
2. Changed or removed patterns
3. New architectural constraints
4. Shifted directory organization

Update ONLY the sections that have changed. Preserve user customizations. Add a brief note at the top: "Updated: [date] — [summary of changes]".

If nothing has meaningfully changed, return the guide unchanged with the date updated.

Focus on: What new patterns exist? What types were added? What constraints changed?
Do NOT include: line numbers, implementation details, or historical context.
"""
thoroughness: "medium"
```

## Expected Output Structure

The generated `onboarding.md` should follow this structure:

```markdown
# [CRATE_NAME] - Orientation Guide

## What This Module Does
[Single sentence: primary responsibility]

## Before You Code Here
**Existing Patterns:** [2-3 established patterns]
**Reusable DTOs/Types:** [Key shared types]
**Integration Points:** [How this connects to other modules]

## Key Abstractions to Reuse
[3-4 core types/interfaces with brief descriptions]

## Architectural Constraints
[Non-negotiable rules that prevent contradictory implementations]

## Directory Map
```
[Only directories that indicate where changes belong]
```
```

## What NOT to Include

The guide should **exclude**:

- Line number references — use conceptual descriptions only
- Implementation details or internal algorithm descriptions
- Historical context or past design decisions
- Nice-to-know information that doesn't prevent reinvention
- Code snippets (unless directly showing a pattern to follow)
- Names of individual team members or historical contributors

The guide should **include only** what prevents agents from:
- Reinventing existing abstractions
- Violating architectural constraints
- Duplicating integration contracts
- Following contradictory patterns

## Verification

After Explore agent generates/updates the guide:

1. **Save to correct location** — Place at `[CRATE_ROOT]/onboarding.md`
2. **Read before deep analysis** — Verify the guide answers "What exists that should be reused?"
3. **Refresh before major work** — Regenerate if git diffs indicate structural changes

## Integration with CLAUDE.md Onboarding Requirement

The CLAUDE.md project instructions mandate reading `onboarding.md` before code analysis. This reference documents how to generate that guide using the Explore agent instead of a dedicated onboarding-agent, reducing maintenance overhead while preserving the same quality output.
