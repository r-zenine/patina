# Progressive Disclosure

## The First Pass Rule

**Always start with `context-handoff.md`** — it contains the essential information needed by 80% of contributors 80% of the time.

## When to Read Additional Artifacts

**Read `decision-log.md` if:**
- You need to understand WHY specific technical choices were made
- You're implementing similar functionality and need precedent
- You're reviewing architectural consistency across contributions

**Read `changelog.md` if:**
- You need to verify what phase objectives were completed
- You're tracking project progress across multiple contributions
- You're preparing status updates or reports

**Read optional artifacts if:**
- You need specialized analysis (performance, security, architecture)
- You're doing compliance or audit work
- You're troubleshooting issues in specific domains

## Role-Specific Reading Strategies

### Implementation Agents

**Primary Goal**: Understand what to build next and avoid pitfalls.

**Reading order:**
1. `context-handoff.md` — focus on "What's fragile" and "For next contributors"
2. `decision-log.md` — only if you need technical patterns established
3. Skip `changelog.md` unless verifying phase completion

**Key questions to answer:**
- What patterns should I follow?
- What assumptions can I make?
- What are the known problem areas?
- Where are the integration points?

---

### Review Agents

**Primary Goal**: Assess quality, consistency, and architectural alignment.

**Reading order:**
1. `context-handoff.md` — focus on "What works" patterns and architectural decisions
2. `decision-log.md` — critical: understand rationale behind choices
3. `changelog.md` — verify claimed accomplishments match evidence
4. Optional artifacts — if specialized review needed (security, performance)

**Key questions to answer:**
- Are patterns consistent with the dev-strategy plan?
- Do decisions align with previous choices?
- Is the work truly complete?
- Are there quality or security concerns?

---

### Audit Agents

**Primary Goal**: Focused analysis for compliance, security, or governance.

**Reading order:**
1. `context-handoff.md` — understand scope and approach
2. `decision-log.md` — document all technical choices for audit trail
3. `changelog.md` — verify completion claims
4. All optional artifacts — essential for focused analysis
5. Cross-reference with dev-strategy plan and other contributions

**Key questions to answer:**
- Does this comply with organizational standards?
- Are security/performance requirements met?
- Is the documentation complete and accurate?
- Are there gaps or risks introduced?

## Reading Design Contributions

Design contributions have a **different structure** than implementation contributions:

```
contributions/NNN-phase-X-design-[topic]-design-contribute/
├── design-doc.md          ← START HERE (replaces context-handoff.md)
└── decision-log.md        ← Design decisions and rationale
```

**Note**: Design contributions have NO `changelog.md` and NO `context-handoff.md`.

**For implementation agents:**
1. Read `design-doc.md` in full (< 100 lines)
   - Focus on "How It Works" — the pattern to implement
   - Focus on "Implementation Guidance" — where to start
   - Note "What We're NOT Doing" — avoid scope creep
2. Read `decision-log.md` to understand why this approach was chosen

## Understanding Contribution Sequences

**Contributions are numbered chronologically:**
```
001-phase-1-implementation-code-general-purpose/
002-phase-1-review-security-code-simplifier/
003-phase-2-test-design-tdd-general-purpose/
```

**Reading strategy for sequences:**
1. Latest contribution first — most current state and context
2. Work backwards only if you need historical context
3. Skip intermediate contributions unless directly relevant
