# Anti-Patterns to Avoid

## Over-Engineering Signals

### In Planning (dev-strategy)
- Planning phases for hypothetical future features not in current requirements
- Choosing Steel Thread or Core-then-Integrate without evidence of uncertain requirements or complex business rules
- Creating research artifacts for technologies not explicitly mentioned by the user
- Over-specifying implementation details in the roadmap

### In Design (design-contribute)
- design-doc.md exceeds 100 lines → over-documenting
- Presenting more than 3 design options → over-thinking
- Designing for future phases instead of the current "Design:" objective
- Adding flexibility "just in case" instead of designing for current constraints
- Designing comprehensive specifications instead of enabling handoffs

### In Implementation (dev-contribute)
- Creating interfaces before you have multiple implementations
- Adding configuration systems before you have actual variations to configure
- Building extension points before you know what will extend them
- Adding error handling for scenarios that cannot happen in the current phase
- Abstracting after the first instance instead of waiting for the pattern to appear twice more

---

## Under-Engineering Signals

### In Design
- No integration points specified → implementer cannot connect components
- No success criteria → cannot verify completion
- Missing key constraints discovered during implementation → implementer lacks context

### In Implementation
- No tests for the behavior the phase explicitly requires
- No documentation of decisions made
- Skipping pre-work validation (build, lint, tests) before contributing

---

## Research Anti-Patterns (dev-strategy)

**Do NOT trigger research for:**
- General performance, security, or scalability concerns (not asked for)
- "Best practices" without specific technology choices
- Hypothetical technology evaluations ("what's the best database?")
- Complex integrations using existing, well-known technologies
- Architecture decisions that can be made with current knowledge

**Only research when ALL three conditions are true:**
1. User explicitly names a technology they want to use
2. That technology is not currently in the codebase
3. Integration patterns with the existing stack are unclear

---

## Documentation Anti-Patterns (dev-contribute)

- Creating code files in contribution folders (documentation only)
- Writing code examples in decision-log or context-handoff when description suffices
- Over-explaining obvious choices in the decision log
- Vague "What's fragile" entries that don't tell the next contributor what to actually watch out for
- Missing decision rationale — next contributor cannot understand why choices were made
