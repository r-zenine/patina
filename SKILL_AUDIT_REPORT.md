# Coding Framework Skills Audit Report

## Executive Summary

The coding-framework skill collection is **well-structured with strong architectural foundations**, but has **critical structural issues** that undermine usability and create maintenance risks. The framework exhibits thoughtful design philosophy but suffers from poor information hierarchy, naming confusion, and progressive disclosure violations.

**Severity: HIGH** — Fixes required to prevent skill misuse and context bloat.

---

## 1. Critical Issues

### 1.1 **CRITICAL: Confusing Naming — `dev-strategy` vs `dev-strategies`**

**Issue**: Two skills with nearly identical names cause cognitive friction:
- `dev-strategy` — Primary planning skill
- `dev-strategies` — Reference guide for execution strategies (TDD, Steel Thread, Core-then-Integrate)

**Why it's a problem**:
- Users will consistently pick the wrong skill when searching
- AI agents will struggle to determine which skill to invoke
- Skill metadata is the only trigger mechanism; similar names fail this mechanism
- Creates maintenance burden as both skills are referenced by other skills

**Recommendation**: Rename `dev-strategies` to something unambiguous:
- `execution-strategies` (clear: guides HOW to execute)
- `strategy-reference` (clear: reference material)
- `tdd-steel-thread-core` (explicit: list the strategies)

**Impact**: HIGH — affects all downstream skill usage

---

### 1.2 **CRITICAL: Progressive Disclosure Violations**

Multiple skills violate the skill-creator's progressive disclosure design principle by loading references into SKILL.md body when they should be loaded on-demand.

#### Problem Areas:

**1. `contribution-system/SKILL.md` (110 lines)**
- References 12+ markdown files in the body
- Users loading this skill get ALL cross-referenced content at once
- Creates massive context bloat when user only needs specific artifact schema

**2. `dev-strategy/SKILL.md` (78 lines)**
- The skill itself is short, but references cascade: contribution-system → 12 references, dev-strategies → 3 references
- When implementing dev-strategy, you're forced to load the entire chain

**3. `design-contribute/SKILL.md` (120+ lines)**
- Table of Contents present but not actionable (no navigation shortcuts)
- References 5 other skills/docs
- Prerequisite section overwhelms with cross-references before explaining what the skill does

**Recommendation**:
- Restructure skills to follow **"One level deep"** principle
- Only reference files that are directly needed for the current step
- Load strategy details from `dev-strategies` only when user is at "Choose Strategy" step
- Load `contribution-system` only when creating contributions, not upfront

**Impact**: HIGH — context window efficiency, reusability of skills

---

### 1.3 **STRUCTURAL: Redundant Skill Pair — `dev-strategy` and `dev-strategies`**

**Current design**:
```
dev-strategy/          # Primary skill (planning)
  SKILL.md
  references/guide.md

dev-strategies/        # Reference skill (execution strategies)
  SKILL.md
  references/tdd.md, steel-thread.md, core-then-integrate.md
```

**Issue**: `dev-strategy` offloads ALL strategy selection to `dev-strategies`, but:
- This creates a hard dependency on another skill
- The two skills are used in sequence (plan first, then select strategy)
- Users following the guide must load both skills to understand the workflow

**Better design**:
- Merge strategy selection guidance into `dev-strategy` directly (or reference one reference file)
- Keep `dev-strategies` as a detailed reference for implementers following dev-contribute

**Current footprint**:
- Both skills reference each other
- Both contribute to context bloat when planning
- Naming confusion makes it hard to know which to load

---

## 2. Moderate Issues

### 2.1 **Prerequisite Overload**

Skills list prerequisites *before* explaining what they do:

**Example** (`design-contribute/SKILL.md`):
```
## Prerequisites
Before using this skill, read [...10 cross-references...]

## What This Skill Does
[actual content]
```

**Problem**: User wants to know "What does this skill do?" but must read prerequisites first.

**Better approach**:
```
## What This Skill Does
[2-3 line summary]

## When to Use
[triggers and context]

## Learn More: Prerequisites
[For first-time users...]
```

**Impact**: MEDIUM — confusing user experience, prerequisite fatigue

---

### 2.2 **Execution Requirements Over-Specification**

All three primary skills (dev-strategy, design-contribute, dev-contribute) have **identical structure**:
```
## Execution Requirements
- UNDERSTANDING PHASE (outcome: ...)
- STRATEGY PHASE (outcome: ...)
- PLANNING PHASE (outcome: ...)
- How to achieve these outcomes: [6 steps]
```

**Problem**:
- Repetition across three skills (violates DRY principle)
- Not all phases apply to all skills (e.g., dev-contribute has "EXECUTION" not "STRATEGY")
- Creates maintenance burden: change one → must update three

**Better approach**:
- Define **one canonical template** in contribution-system
- Skills reference the template with skill-specific variations
- Reduces context bloat by moving common structure to reference

**Impact**: MEDIUM — maintenance burden, repetition

---

### 2.3 **DECISION LOG STRUCTURE COUPLING**

The CLAUDE.md notes:
> "The decision log artifacts use a unified YAML schema that directly matches the `diffviz-review::Decision` struct. This schema cannot be changed without corresponding updates to diffviz-cli's parsing logic."

**Problem**:
- Skills document this constraint in multiple places (contribution-system, dev-contribute, design-contribute)
- No single source of truth for the schema itself
- Skills reference the template location but don't embed schema details

**Recommendation**:
- Create `contribution-system/references/decision-schema.md` with actual schema definition
- Add schema validation instructions to `dev-contribute/references/guide.md`
- Remove redundant schema documentation from multiple skills

**Impact**: LOW-MEDIUM — documentation quality, schema governance

---

## 3. Design & Architecture

### 3.1 What Works Well ✓

- **Clear layer separation**: Support skills (contribution-system, design-principles) vs. primary skills (dev-strategy, design-contribute, dev-contribute)
- **Excellent principle documentation**: `design-principles` clearly articulates YAGNI, LRM, Kent Beck's rules, etc.
- **Strong outcome-focused execution**: All skills define clear "outcomes" rather than prescriptive steps
- **Artifact-centric design**: Emphasis on what gets created (decision logs, context handoffs) vs. process steps
- **Two-gate system**: Thoughtful design gate (LRM) prevents over-engineering

### 3.2 What Needs Fixing ✗

1. **Naming clarity** (dev-strategy vs dev-strategies)
2. **Progressive disclosure** (too much loaded upfront)
3. **Prerequisite ordering** (prerequisites before purpose)
4. **Repetition** (execution requirements template across 3 skills)
5. **Schema governance** (decision log schema not centralized)

---

## 4. Skill-by-Skill Audit

| Skill | Status | Issues | Priority |
|-------|--------|--------|----------|
| **contribution-system** | 🟡 MODERATE | Progressive disclosure; 12 cross-refs in body | HIGH |
| **design-principles** | ✓ GOOD | Clear, concise; anti-patterns reference is helpful | — |
| **dev-strategy** | 🟡 MODERATE | Naming confusion (vs dev-strategies); prerequisite overload | HIGH |
| **design-contribute** | 🟡 MODERATE | Prerequisites before purpose; execution requirements repetition | MEDIUM |
| **dev-contribute** | 🟡 MODERATE | Prerequisite overload; missing code commit guidance in body | MEDIUM |
| **dev-strategies** | 🟡 MODERATE | Name collision; references embedded in strategy definitions | HIGH |

---

## 5. Recommendations (Prioritized)

### Phase 1: Structural Fixes (CRITICAL)

1. **Rename `dev-strategies` to `execution-strategies`**
   - Update all cross-references in other skills
   - Reason: Eliminates naming confusion; clarifies that this is a reference guide for HOW to execute
   - Effort: 20 mins (regex rename + reference updates)

2. **Extract execution requirements template to contribution-system**
   - Create `contribution-system/references/execution-template.md`
   - Skills reference with skill-specific variations
   - Reason: Eliminates repetition; single source of truth for outcome-focused execution
   - Effort: 30 mins

3. **Restructure skill bodies per progressive disclosure**
   - Move non-essential references to separate reference files
   - Reorder: "What it does" → "When to use" → "How to use" → "Prerequisites"
   - Focus on skills: contribution-system, design-contribute, dev-strategy
   - Reason: Reduces context bloat; improves user experience
   - Effort: 1-2 hours

### Phase 2: Information Architecture (MEDIUM)

4. **Centralize decision-log schema in contribution-system**
   - Create `contribution-system/references/decision-log-schema.md` with actual YAML
   - Add schema validation checklist to dev-contribute guide
   - Reason: Single source of truth; easier to maintain and version
   - Effort: 30 mins

5. **Create "First Time Using This Framework?" guide**
   - New reference file in root or contribution-system
   - Explains: which skill to start with, what each produces, how they work together
   - Reason: Onboarding clarity; reduces need for prerequisites in individual skills
   - Effort: 1 hour

### Phase 3: Polish (LOW)

6. **Add "See also" navigation links** between related skills
   - Help users understand skill relationships
   - Reason: Improves discoverability; reduces "which skill do I need?" confusion
   - Effort: 30 mins

7. **Consolidate anti-patterns reference**
   - Add `design-principles/references/anti-patterns.md` examples to main body
   - Or create single `anti-patterns.md` shared by design-principles and dev-contribute
   - Reason: Reduces redundant files; improves access
   - Effort: 20 mins

---

## 6. Context Window Impact Analysis

### Current State (Estimated)

When invoking **dev-strategy** workflow:
- dev-strategy SKILL.md: ~2kb
- Cross-references triggered: dev-strategies (entire), contribution-system (entire)
- **Total context loaded: ~50-70kb** (rough estimate with all references)

### After Recommended Changes

- dev-strategy SKILL.md: ~2kb
- Cross-references triggered (on-demand): contribution-system guide section only
- **Total context loaded: ~15-20kb**

**Efficiency gain: 60-70% reduction in upfront context bloat**

---

## 7. Validation Checklist

After implementing fixes:

- [ ] `dev-strategy` and `execution-strategies` names cause no confusion
- [ ] Invoking any skill doesn't auto-load all cross-referenced skills
- [ ] Each skill body explains "What" and "When" before "Prerequisites"
- [ ] Execution requirements template is in ONE place (contribution-system)
- [ ] Decision log schema is accessible from contribution-system
- [ ] All cross-references are resolved (no broken links)
- [ ] No skill body exceeds 500 lines (progressive disclosure limit)
- [ ] Skill frontmatter descriptions accurately trigger on actual user requests

---

## 8. Conclusion

The coding-framework skills represent solid architectural thinking with clear principles and strong outcomes-driven design. However, **information architecture issues** create friction for both users and AI agents. The fixes are structural (reordering, renaming, extracting) rather than conceptual, making them low-risk but high-impact improvements.

**Recommendation**: Implement Phase 1 (structural fixes) as a single atomic refactor to avoid partial states. Phases 2-3 can follow incrementally as usage reveals pain points.
