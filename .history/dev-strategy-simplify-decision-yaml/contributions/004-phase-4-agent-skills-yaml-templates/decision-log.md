# Technical Decisions - Phase 4: Agent Skills YAML Templates

## ⚡ Critical Decisions

**YAML over Markdown for decision-log**: YAML provides machine-parseable structure while preserving agent flexibility for rationale content → **Impact**: Future tools can aggregate and analyze decisions, agents still write free-form explanations

**Single template for both skills**: decision-log-template.yaml used by both dev-contribute and dev-strategy → **Impact**: Agents see consistent decision-log format across skills, easier knowledge transfer

**Only decision-log in YAML**: context-handoff and changelog remain markdown → **Impact**: Minimal disruption to existing workflows, only the "decisions" aspect is structured

## 🔧 Implementation Choices

**Minimal YAML schema**: Required (contribution, decisions), Optional (rationale, context) → **Reasoning**: Agents choose verbosity level, don't force rigid structure

**Free-form rationale field**: No enforced sub-fields → **Result**: Agents write prose, bullets, or structured format as they prefer

**Inline documentation with examples**: Each template shows minimal, verbose, and edge case styles → **Result**: Agents understand flexibility without reading external docs

## 🔍 Alternatives Considered

**Fully structured YAML with sub-fields**: Every decision requires title, choice, rationale, impact, assumptions → **Trade-off**: More queryable but restrictive, less agent autonomy

**Separate templates for each skill**: Different templates for dev-strategy vs dev-contribute → **Trade-off**: More customization but harder to maintain consistency

**Convert all three artifacts to YAML**: decision-log, context-handoff, changelog all YAML → **Trade-off**: Bigger change, more disruption to existing skills

## 📚 Human Context

**Decision Process**:
- Reviewed existing decision-log-template.md format (emoji sections, examples)
- Analyzed Phases 1-3 to understand what decisions were actually captured
- Recognized that only decisions are truly "structured" content (id, title, choice)
- Rationale field needs to be flexible to capture alternatives, trade-offs, constraints

**Constraints**:
- Must not break existing skill workflows
- Templates should feel natural to agents (not overly rigid)
- Schema should enable future tooling (parsing, stats) without forcing it

**Key Learnings**:
- The "decision-log" in agent-skills is different from diffviz-review Decision entity
- Agent decision-logs need flexibility; entity definitions need structure
- YAML is good for capturing "what was decided" but rationale benefits from prose
