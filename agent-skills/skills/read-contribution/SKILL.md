---
name: read-contribution
description: Navigate and understand contribution artifacts using progressive disclosure principles. Use when you need to understand existing contributions, review implementation work, or continue from previous contributions.
allowed-tools: ["Read", "Glob", "Grep"]
---

# Read Contribution Skill

Navigate and understand contribution artifacts efficiently using progressive disclosure principles.

## What This Skill Does

Provides structured approaches to reading and understanding contributions made to dev-strategy implementations by:
- Following progressive disclosure: start with context-handoff.md, dive deeper only when needed
- Role-specific reading strategies for different types of contributors
- Efficient navigation between related artifacts
- Understanding contribution relationships and dependencies

## When to Use This Skill

✅ **Use when:**
- Understanding existing contributions before making your own
- Reviewing implementation work from other agents
- Continuing from a previous contribution phase
- Auditing or analyzing completed development work
- Need to understand project state and recent decisions

❌ **Don't use when:**
- Creating new contributions (use appropriate *-contribute skills instead)
- Reading initial dev-strategy plans (those have their own structure)
- Working on projects without contribution artifacts

## Progressive Disclosure Principle

For full progressive disclosure technique and role-specific reading strategies, see [`contribution-system/references/progressive-disclosure.md`](../contribution-system/references/progressive-disclosure.md).

**Start Here Always:** `context-handoff.md` — Contains everything most contributors need
**Dive Deeper Only If:** You need specific technical decisions or completion details

## Reading Strategies (Quick Reference)

Different roles have different information needs:

**Implementation Agents**: Focus on "What works/What's fragile" + technical guidance
**Review Agents**: Emphasize decision rationale + quality patterns established
**Audit Agents**: Deep dive into all artifacts + optional specialized reports

## Directory Structure

**Implementation contributions** follow this pattern:
```
contributions/NNN-phase-X-type-specialty-agent/
├── context-handoff.md     ← START HERE
├── decision-log.yaml      ← Read if you need technical choices
├── changelog.md           ← Read for completion summary
└── [optional artifacts]   ← Read for specialized analysis
```

**Design contributions** have a different structure:
```
contributions/NNN-phase-X-design-[topic]-design-contribute/
├── design-doc.md          ← START HERE (< 100 lines)
├── decision-log.md        ← Design decisions and rationale
└── context-handoff.md     ← Reading guide + problem + overview
```

For artifact schemas and quality indicators, see [`contribution-system` skill](../contribution-system/SKILL.md).

## Detailed Instructions

For complete reading strategies, navigation guides, and role-specific approaches, see [reference.md](reference.md).
