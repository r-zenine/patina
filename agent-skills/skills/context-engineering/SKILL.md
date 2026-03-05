---
name: context-engineering
description: Reference for context gathering and routing techniques used across dev-strategy workflow skills. Contains progressive disclosure, two-gate system, constraint discovery, routing confidence levels, and contribution chain reading.
allowed-tools: ["Read"]
---

# Context Engineering

Canonical reference for the techniques agents use to gather, organize, and hand off context across the dev-strategy workflow.

## The 5 Core Techniques

### 1. Progressive Disclosure

Start with the minimum context needed, dive deeper only when required.

**First Pass Rule**: Always start with `context-handoff.md` — it contains what 80% of contributors need 80% of the time.

**Dive deeper only if:**
- You need to understand WHY a specific technical choice was made → read `decision-log.md`
- You need to verify what phase objectives were completed → read `changelog.md`
- You need specialized analysis (performance, security) → read optional artifacts

See [progressive-disclosure.md](references/progressive-disclosure.md) for role-specific reading strategies.

---

### 2. Two-Gate System Against Over-Engineering

Two checkpoints prevent design work happening too early or too late.

**Gate 1 — dev-strategy**: Decide whether to defer a design decision.
- Don't defer if design is straightforward
- Defer when implementation learning will inform better choices

**Gate 2 — design-contribute**: Design at the Last Responsible Moment.
- Only design what the current phase requires
- No hypothetical features, no "what if" scenarios
- Present simplest option first

See [two-gate-system.md](references/two-gate-system.md) for detailed gate criteria.

---

### 3. Constraint Discovery

Gather real constraints through interactive questioning before designing or planning.

**Ask before analyzing:**
- "What did you learn during implementation that should inform this design?"
- "What patterns or approaches worked well so far?"
- "What's your priority: simplicity, performance, or flexibility?"
- "Are there any non-negotiable requirements?"

**Validate, don't assume:**
- Review the implementation yourself, then confirm with user: "I see pattern X emerged — is that correct?"
- Never infer priorities — ask explicitly

See [constraint-discovery.md](references/constraint-discovery.md) for AskUserQuestion patterns.

---

### 4. Routing Patterns (dev-continue)

Determine the next correct action by reading roadmap state and contribution history.

**Decision tree:**
- Next objective is "Design: Determine [X]" and no design contribution exists → route to `design-contribute`
- Design contribution exists but no implementation follows → route to `dev-contribute`
- No "Design:" keyword → route to `dev-contribute`
- All phases complete → inform user, do not auto-invoke

**Routing confidence levels:**
- High confidence (auto-route): Next objective is clear, type is obvious, no conflicts
- Medium confidence (ask user): Multiple valid next steps, partial work found, ambiguous objective type
- Low confidence (inform only): Roadmap structure unclear, conflicting state, missing artifacts

See [routing-patterns.md](references/routing-patterns.md) for the full decision tree and edge cases.

---

### 5. The 5-Minute Scan

Efficient reading technique for `context-handoff.md`:

1. **Read "Built:" line** (30 seconds) — understand the main deliverable
2. **Scan "What works/fragile/missing"** (2 minutes) — understand current state
3. **Read your role's guidance** (1 minute) — get specific direction
4. **Note "Expects/Deferred"** (1 minute) — understand constraints
5. **Check related doc links** (30 seconds) — decide if you need more

## Reference Files

- [progressive-disclosure.md](references/progressive-disclosure.md) — First-pass rule, role-specific reading strategies
- [two-gate-system.md](references/two-gate-system.md) — Gate 1 (dev-strategy deferral) + Gate 2 (design-contribute)
- [constraint-discovery.md](references/constraint-discovery.md) — AskUserQuestion patterns, interactive design techniques
- [routing-patterns.md](references/routing-patterns.md) — Decision trees, confidence levels, edge case handling
