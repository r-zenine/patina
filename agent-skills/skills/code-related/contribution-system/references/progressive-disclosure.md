# Progressive Disclosure

## The First Pass Rule

**Always start with `context-handoff.md`** — it contains the essential information needed by 80% of contributors 80% of the time.

## When to Read Additional Artifacts

**Read `decision-log.yaml` if:**
- You need to understand WHY specific technical choices were made
- You're implementing similar functionality and need precedent
- You're reviewing architectural consistency across contributions

**Read optional artifacts if:**
- You need specialized analysis (performance, security, architecture)
- You're doing compliance or audit work
- You're troubleshooting issues in specific domains

## Role-Specific Reading Strategies

### Implementation Agents

**Primary Goal**: Understand what to build next and avoid pitfalls.

**Reading order:**
1. `context-handoff.md` — focus on "What's fragile" and "For next contributors"
2. `decision-log.yaml` — only if you need technical patterns established

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
2. `decision-log.yaml` — critical: understand rationale behind choices; verify claims against the diff at its recorded `commit`
3. Optional artifacts — if specialized review needed (security, performance)

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
2. `decision-log.yaml` — document all technical choices for audit trail; verify completion claims against the diff at its recorded `commit`
3. All optional artifacts — essential for focused analysis
4. Cross-reference with dev-strategy plan and other contributions

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

**Note**: Design contributions have NO `context-handoff.md`.

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

**Strategy pattern recognition from folder names:**
- `design-[topic]-design-contribute` → design contributions precede implementation; read `design-doc.md` before implementing
- `test-design` → `test-validation` → `implementation` = TDD pattern
- `pathfinder` → `foundation` → `expansion` = Steel Thread pattern
- `domain-logic` → `port-design` → `adapter` → `integration` = Core-then-Integrate pattern

## Efficient Reading Techniques

### The 5-Minute Scan

**For context-handoff.md:**
1. **Read "Built:" line** (30 seconds) — understand the main deliverable
2. **Scan "What works/fragile/missing"** (2 minutes) — understand current state
3. **Read your role's guidance** (1 minute) — get specific direction
4. **Note "Expects/Deferred"** (1 minute) — understand constraints
5. **Check related docs links** (30 seconds) — decide if you need more

### The Deep Dive

**When 5-minute scan isn't enough:**
1. Full `context-handoff.md` read
2. Targeted `decision-log.yaml` scan — focus on decisions relevant to your work
3. Selective optional artifacts — only what you specifically need

## Common Reading Scenarios

**Starting a new phase implementation:**
1. Read previous phase's latest `context-handoff.md` (current state)
2. Scan previous phase's `decision-log.yaml` (established patterns)
3. Check dev-strategy plan (verify objectives)

**Implementing after a design contribution:**
1. Read `design-doc.md` in full (what to build)
2. Read `decision-log.md` (why this approach)
3. Read previous implementation `context-handoff.md` (current state)

**Debugging an issue:**
1. Start with most recent `context-handoff.md`
2. Look for relevant "What's fragile" entries
3. Check `decision-log.yaml` for related technical choices
4. Work backwards through contributions until you find relevant context

**Reviewing security across all contributions:**
1. Read all `context-handoff.md` files for security-related "What's fragile"
2. Read all `decision-log.yaml` files for security-related choices
3. Read any `security-*.md` optional artifacts

**Onboarding to a project:**
1. Read dev-strategy plan (overall approach)
2. Read Phase 1's final `context-handoff.md` (foundation)
3. Read latest contribution's `context-handoff.md` (current state)
4. Scan `decision-log.yaml` files for architectural patterns
