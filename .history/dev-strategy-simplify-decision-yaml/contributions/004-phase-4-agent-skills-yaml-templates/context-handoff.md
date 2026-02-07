# Context Handoff - Phase 4: Agent Skills YAML Templates

## What Was Accomplished

**Primary objective**: Create decision-log-template.yaml for both dev-contribute and dev-strategy skills to enable structured decision logging in agent contributions.

**Summary of work:**
- Created decision-log-template.yaml in `agent-skills/skills/dev-contribute/templates/`
- Created decision-log-template.yaml in `agent-skills/skills/dev-strategy/templates/`
- Both templates use identical schema for consistency across skills
- Included inline documentation and examples (minimal, verbose, revision styles)
- Clarified that YAML structure is for decision-logs only (not context-handoff/changelog)

**Results:**
- ✅ Both YAML templates are valid and well-documented
- ✅ Templates support agent flexibility while enabling structure
- ✅ Examples demonstrate realistic usage patterns
- ✅ Zero breaking changes to existing skill workflows

## Key Learnings

### 1. YAML Enables Structure Without Rigidity

The key insight: agents need **flexibility** in rationale but **structure** in metadata.

YAML approach:
- **Structured**: contribution metadata (number, phase, type), decision id/title/choice
- **Flexible**: rationale field is free-form (prose, bullets, or nested)
- This balances machine-parseability with human creativity

Alternative rejected:
- Fully structured YAML with required sub-fields (alternatives, assumptions, impact)
- Agents found this restrictive in practice; they want to explain trade-offs their way

### 2. Same Schema Across Skills Reduces Cognitive Load

Both dev-contribute and dev-strategy use identical decision-log schema.

Benefits:
- Agents learn once, use everywhere
- Templates are easy to maintain (one schema, two locations)
- Future tooling only needs to parse one format

This differs from dev-strategy (which has many template types: roadmap, context-document, code-context) but decision-logs benefit from consistency.

### 3. The Two "Decision Logs" Serve Different Purposes

Important distinction clarified during Phase 4:

- **diffviz-review Decision** (Phases 1-3): Runtime entity mapping architectural decisions → code changes in code review
- **agent-skills decision-log** (Phase 4): Documentation artifact recording decisions made during contribution

Both happen to be called "decision log" but are independent. Potential future integration (agent-skills could generate diffviz-review Decision YAML files) but not in scope.

### 4. Only decision-log Needed YAML Conversion

Original roadmap mentioned all three (decision-log, context-handoff, changelog) but user clarified:
- Only decision-log requires YAML structure (machines need to parse decisions)
- context-handoff and changelog are prose-heavy, markdown is fine
- This minimizes disruption to existing skills

## For Next Contributors

### Understanding Phase 4's Role in the Dev-Strategy

This dev-strategy consists of four coordinated phases:

**Phase 1-3: diffviz-review refactoring** (✅ Complete)
- Simplified Decision/CodeImpact entities
- Updated TUI rendering
- Fixed test fixtures
- **Impact:** Code change, breaking API changes

**Phase 4: agent-skills templates** (✅ Complete)
- Created decision-log-template.yaml for structured decision logging
- Applies to agent-skills contribution workflow
- **Impact:** Documentation improvement, no code changes

### Where to Use These Templates

The decision-log-template.yaml is for:
- **dev-contribute**: When an agent implements a phase, it records decisions made
- **dev-strategy**: When planning a new strategy, record design decisions

Example usage:
```yaml
# In a dev-contribute contribution
contribution:
  number: 1
  phase: "Phase 1"
  type: "implementation"
decisions:
  - id: "D1"
    title: "Use tree-sitter for parsing"
    choice: "Adopted tree-sitter library"
    rationale: |
      Needed robust parsing without regex.
      Alternatives: regex (too fragile), hand-written parser (too slow).
      Tree-sitter provides good AST analysis for semantic understanding.
```

### If You Need to Modify These Templates

Key principle: **Structure is minimal, flexibility is maximum**

When considering changes:
1. Don't add required sub-fields (keeps template flexible)
2. Do expand examples section if new patterns emerge
3. Consider adding context section if meta-decisions become important

Example of good change:
- Adding "revision" example when agents frequently revise prior decisions

Example of bad change:
- Making rationale sub-fields required (impact, assumptions, alternatives)
- Agents will find workarounds, defeats the purpose

### Testing the Templates

No automated tests (YAML is prose-based). Validate by:
1. Reading template documentation (clear?)
2. Checking examples (realistic and useful?)
3. Asking: "Could I use this right now?" (templates should feel natural)

### Connection to Future Work

Potential future opportunities:
- Build YAML parser to extract decision metrics (decisions per phase, decision velocity)
- Create CLI viewer to render decision-logs nicely
- Connect agent-skills decision-logs to diffviz-review Decision generation (one YAML format)
- Build decision traceability (link decisions to commits, code review outcomes)

None of this is in scope for Phase 4, but architecture should enable these.

## Quality Assurance Checklist

For any future modifications to decision-log templates:

- [ ] Template is valid YAML
- [ ] Examples are realistic (based on actual contributions)
- [ ] Flexibility is preserved (agents can vary structure)
- [ ] Documentation is clear (comments explain intent, not just syntax)
- [ ] Both dev-contribute and dev-strategy versions stay in sync
- [ ] No breaking changes to existing contribution artifacts

## Files Changed Summary

**Files created:** 2 YAML templates
- `agent-skills/skills/dev-contribute/templates/decision-log-template.yaml`
- `agent-skills/skills/dev-strategy/templates/decision-log-template.yaml`

**Files not modified:** Existing .md templates remain unchanged (changelog, context-handoff)

**Impact:** Zero disruption to existing workflows, pure addition

## Rollback Information

If Phase 4 needs to be reverted:
- Delete both decision-log-template.yaml files
- No other dependencies or migrations needed
- Existing markdown templates still available
- Phases 1-3 code changes unaffected
