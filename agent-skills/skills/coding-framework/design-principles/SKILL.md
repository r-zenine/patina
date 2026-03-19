---
name: design-principles
description: Reference guide for design philosophy applied across all dev-strategy workflow skills. Use when you need to understand YAGNI, Last Responsible Moment, Kent Beck's 4 Simple Design Rules, Sufficient Design, and Just-In-Time Architecture principles. Provides context for planning, design, and implementation decisions.
---

# Design Principles

Canonical reference for the design philosophy applied across all planning, implementation, and design contributions.

## The 5 Core Principles

### 1. YAGNI — You Aren't Gonna Need It (Primary Principle)

Build only what is explicitly required for the current feature or phase.

- Don't add functionality, abstractions, or complexity for hypothetical future needs
- If it's not needed now, don't plan for it, design it, or implement it
- Prefer simple solutions that solve the immediate problem
- Applies equally to planning (don't plan hypothetical phases), design (don't design for future scenarios), and implementation (don't implement unused abstractions)

---

### 2. Last Responsible Moment (LRM)

Defer decisions until you have maximum information — not until it's too late, but not before you need to decide.

- Don't commit to complex solutions before understanding the full problem
- Wait for patterns to emerge naturally before abstracting
- In planning: only design what Phase N requires; defer Phase N+1 design to Phase N+1
- In implementation: only extract abstractions when you have 2–3 concrete use cases

---

### 3. Kent Beck's 4 Simple Design Rules

A design is sufficient when it (in order of priority):

1. **Passes tests** — The code works as required
2. **Reveals intent** — Code clearly expresses what it does
3. **No duplication** — Don't repeat logic or structure
4. **Minimal elements** — Use the fewest classes, methods, and abstractions possible

---

### 4. Sufficient Design / Implementation / Planning

The goal is actionable guidance that enables execution — not comprehensive documentation or perfect plans.

- **Sufficient Planning**: Provide enough detail for implementation without over-specifying
- **Sufficient Implementation**: Make it work correctly, not perfectly; avoid over-polishing
- **Sufficient Design**: Design enough to enable implementation; target < 100 lines for design docs
- Focus on "what to build" and "how it connects" rather than exhaustive "how"

---

### 5. Just-In-Time Architecture

Build what you need now. Refactor when complexity actually emerges.

**Refactor when:**
- You are literally copying and pasting code (actual duplication)
- The next phase requires a specific interface that does not exist
- The current structure actively prevents the next step

**Never refactor for:**
- Hypothetical future requirements
- "What if" scenarios
- Abstractions without 2–3 concrete use cases

## Anti-Patterns Reference

See [anti-patterns.md](references/anti-patterns.md) for a list of over-engineering and under-engineering signals with examples.

## How These Principles Apply

| Context | Primary Principle Applied |
|---------|--------------------------|
| dev-strategy planning | YAGNI + LRM: plan current feature only |
| design-contribute sessions | Sufficient Design + YAGNI: design current phase only |
| dev-contribute implementation | YAGNI + Kent Beck: implement minimum to pass tests |
| Strategy selection | LRM: don't choose complex strategy without evidence |
| Between-phase refactoring | JIT Architecture: only refactor what next phase requires |
