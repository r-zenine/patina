# Decision Log

## Project Scope

**Two-track implementation:**
- **Track A (Phases 1-3):** Modify diffviz-review Rust structs for code review system
- **Track B (Phase 4):** Create agent-skills YAML templates for contribution documentation

Both tracks enable better YAML support but serve different purposes. This decision log covers decisions for both tracks.

---

## Track A Decisions (diffviz-review structs)

## D1: Remove summary field, add optional rationale field

**Choice:** Replace Decision.summary (String) with Decision.rationale (Option<String>)

**Rationale:**
- `summary` was a brief description, `rationale` provides full context (why, alternatives, trade-offs)
- Optional field allows concise decisions (just title) or verbose (with full rationale)
- Reduces redundancy between title and summary
- Better aligns with YAML artifact philosophy: structured metadata + free-form content

**Alternatives considered:**
- Keep both summary and rationale: Rejected, creates confusion about which to use
- Make rationale required: Rejected, forces verbosity even for simple decisions
- Rename summary to rationale: Rejected, changes semantics (summary=brief, rationale=detailed)

**Impact:**
- Breaking change for serialization (old YAML with summary won't deserialize)
- Medium effort: Update TUI rendering + all test fixtures
- Acceptable because pre-production (no existing data to migrate)

## D2: Remove change_type field from CodeImpact

**Choice:** Remove CodeImpact.change_type (ChangeType enum: Addition | Modification | Deletion)

**Rationale:**
- Git diffs already show whether code is added, modified, or deleted
- Redundant information that adds no value to decision-to-code mapping
- Simplifies CodeImpact structure
- Reduces boilerplate in YAML files

**Alternatives considered:**
- Keep as optional field: Rejected, still adds complexity for marginal value
- Make it a string instead of enum: Rejected, doesn't solve the redundancy problem

**Impact:**
- Breaking change for serialization
- TUI currently displays change_type as "Addition/Modification/Deletion" string
- Update: Remove rendering logic, rely on diff context instead
- Low risk: Not used by indexing algorithm

## D3: Remove confidence field from CodeImpact

**Choice:** Remove CodeImpact.confidence (Confidence enum: High | Medium | Low)

**Rationale:**
- Confidence level is subjective and not essential for review workflow
- Adds cognitive overhead (what's "medium" vs "high"?)
- Not used by decision indexing algorithm
- Simplifies CodeImpact structure

**Alternatives considered:**
- Keep as optional: Rejected, still creates question of "should I add this?"
- Make it boolean (confident/not confident): Rejected, too reductive

**Impact:**
- Breaking change for serialization
- TUI displays confidence with color coding (High=green, Medium=yellow, Low=red)
- Update: Remove color styling logic
- Low risk: Not used by core functionality

## D4: Use serde_yaml for YAML serialization

**Choice:** Use serde_yaml crate (already in dependencies) for Decision serialization

**Rationale:**
- Already part of project dependencies
- Seamless integration with existing Serde derives
- Supports Option<T> naturally for optional fields
- Proven and widely used in Rust ecosystem

**Alternatives considered:**
- Manual YAML parsing: Rejected, reinvents wheel and error-prone
- yaml-rust crate: Rejected, serde_yaml is more idiomatic with Serde

**Impact:**
- No new dependencies needed
- Zero risk: Already using serde for JSON serialization

## D5: Phased implementation approach

**Choice:** Use 3-phase implementation: Entity updates → TUI updates → Test updates

**Rationale:**
- Phase 1 (entities) isolates structural changes
- Phase 2 (TUI) updates rendering for new structure
- Phase 3 (tests) verifies everything works end-to-end
- Each phase can be verified independently
- Follows "Technical And Functional never change together" principle from CLAUDE.md

**Alternatives considered:**
- Big bang: Change everything at once: Rejected, too risky and hard to debug
- Test-first approach: Rejected, tests would fail during entity updates

**Impact:**
- Slightly longer implementation time
- Much easier to debug and verify
- Each phase produces compilable code (even with failing tests)

## D6: Update TUI to gracefully handle missing fields (Track A)

**Choice:** TUI displays rationale if present, shows nothing if absent; removes change_type/confidence rendering entirely

**Rationale:**
- Optional rationale field means some decisions may not have it
- Better UX to hide empty section than show "None" or placeholder
- Removing change_type/confidence simplifies UI, reduces visual clutter

**Alternatives considered:**
- Show placeholder text: Rejected, adds noise without value
- Make rationale required in TUI but optional in struct: Rejected, creates inconsistency

**Impact:**
- Cleaner, simpler TUI rendering
- Slightly less information displayed, but redundant information wasn't valuable

---

## Track B Decisions (agent-skills YAML templates)

## D7: Create minimal YAML structure for agent-skills artifacts

**Choice:** Define minimal required structure with free-form content sections

**Rationale:**
- Balances machine-parseability (for stats) with agent flexibility (for content)
- Only specify what's needed for indexing and tooling
- Agents can choose verbosity level (minimal vs detailed)
- Reduces token usage vs markdown (40-50% savings)

**Required fields:**
- `contribution` block: number, phase, type (for indexing)
- Top-level sections: deliverables, quality_assessment, etc.
- Content within sections: FREE-FORM (string, list, or structured)

**Alternatives considered:**
- Fully structured (every field defined): Rejected, too rigid, loses agent flexibility
- Pure free-form (no structure): Rejected, can't build tooling or compute stats
- Keep markdown: Rejected, can't parse for stats, wastes tokens

**Impact:**
- Enables stats tooling (autonomous decision count, approval rates, etc.)
- 40-50% token savings over markdown
- Agents retain flexibility in how they describe work

## D8: Reuse same decision-log schema for dev-strategy and dev-contribute

**Choice:** Both skills use identical decision-log.yaml template structure

**Rationale:**
- Decision documentation needs are the same regardless of skill
- Reduces cognitive load (one schema to learn)
- Enables consistent tooling across both skills
- Simplifies maintenance (one template to update)

**Alternatives considered:**
- Separate schemas: Rejected, creates unnecessary divergence
- Dev-strategy has more fields: Rejected, no clear need for different fields

**Impact:**
- Single template to maintain
- Tools work across both skills
- Consistent agent experience

## D9: Optional stats block in all templates

**Choice:** Add optional `stats` block to all artifact templates for metrics

**Rationale:**
- Enables machine computation of contribution metrics
- Optional: agents include if relevant, omit if not
- Structured data: easy to parse and aggregate
- Examples: lines_added, lines_removed, autonomous_decisions, test_failures

**Alternatives considered:**
- Required stats: Rejected, forces agents to fill in potentially irrelevant numbers
- No stats support: Rejected, loses value of YAML (can't compute metrics)
- Stats as separate file: Rejected, splits related information

**Impact:**
- Enables contribution analytics
- Optional nature preserves simplicity
- Agents can evolve stats fields over time

## D10: Update skill.md instructions to generate YAML

**Choice:** Modify dev-contribute and dev-strategy skill instructions to reference YAML templates instead of markdown

**Rationale:**
- Skills need to know to generate YAML instead of markdown
- Instructions should reference template files for structure
- Examples should show YAML instead of markdown

**Alternatives considered:**
- Support both markdown and YAML: Rejected, creates inconsistency
- Don't update instructions: Rejected, agents would continue generating markdown

**Impact:**
- Skills automatically generate YAML artifacts
- Consistent experience across all contributions
- Templates serve as documentation for agents
