# Implementation Roadmap

## Strategy: Core-then-Integrate

**Rationale:** Update core entity definitions first, then update consumers (TUI + tests), then create agent-skills YAML templates. This follows the "Technical And Functional never change together" principle - we refactor the structure without changing behavior, then update usage sites, then add new capabilities.

**Two-Track Implementation:**

**Track A: diffviz-review (Code Review System)**
1. **Phase 1: Entity Structure Updates** - Modify Decision/CodeImpact structs (diffviz-review)
2. **Phase 2: TUI Rendering Updates** - Update decision details panel (diffviz-review-tui)
3. **Phase 3: Test Fixture Updates** - Update all test construction (diffviz-review + diffviz-review-tui)

**Track B: agent-skills (Contribution Documentation)**
4. **Phase 4: Agent Skills YAML Templates** - Create YAML artifact templates for dev-strategy and dev-contribute

---

## Phase 1: Entity Structure Updates

**Objective:** Update Decision and CodeImpact structs to simplified schema

**Scope:** diffviz-review/src/entities/decision.rs only

**Tasks:**

### 1.1: Remove ChangeType and Confidence enums
- Delete lines 12-28 (ChangeType and Confidence definitions)
- These are public types, so this is a breaking change
- **Files modified:** decision.rs

### 1.2: Update CodeImpact struct
- Remove `change_type: ChangeType` field
- Remove `confidence: Confidence` field
- Keep: file, line_ranges, reasoning
- **Files modified:** decision.rs (lines 40-46)

### 1.3: Update Decision struct
- Remove `summary: String` field
- Add `rationale: Option<String>` field with serde attributes:
  ```rust
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub rationale: Option<String>,
  ```
- Keep: number, title, decision_log_line, code_impacts
- **Files modified:** decision.rs (lines 50-57)

### 1.4: Update decision.rs test helpers
- Update `create_test_decision()` helper (line ~283)
- Remove change_type, confidence, summary from construction
- Add optional rationale field
- **Files modified:** decision.rs (tests section)

### 1.5: Update all unit tests in decision.rs
- Find all Decision/CodeImpact construction in tests
- Remove change_type, confidence fields from CodeImpact construction
- Replace summary with rationale in Decision construction
- Expected locations: Lines 270-923 (test suite)
- **Files modified:** decision.rs (tests section)

**Verification:**
- Run `cargo check --package diffviz-review`
- Run `cargo test --package diffviz-review` (will fail - expected)
- Run `cargo clippy --package diffviz-review` (zero warnings)

**Expected state after Phase 1:**
- ✅ diffviz-review compiles with warnings (unused imports)
- ❌ diffviz-review tests fail (some tests reference old fields)
- ❌ diffviz-review-tui fails to compile (references removed types)

---

## Phase 2: TUI Rendering Updates

**Objective:** Update TUI to render simplified Decision structure

**Scope:** diffviz-review-tui/src/ui/components/decision_details_panel.rs

**Tasks:**

### 2.1: Update import statements
- Remove `ChangeType` and `Confidence` from imports (if present)
- **Files modified:** decision_details_panel.rs (top of file)

### 2.2: Update summary rendering
- Change lines 75-79 from rendering `decision.summary` to `decision.rationale`
- Handle Option<String>: only render if rationale is Some
- **Current code:**
  ```rust
  lines.push(Line::from(vec![Span::styled(
      &decision.summary,
      Styles::primary(),
  )]));
  ```
- **New code:**
  ```rust
  if let Some(rationale) = &decision.rationale {
      lines.push(Line::from(vec![Span::styled(
          rationale,
          Styles::primary(),
      )]));
      lines.push(Line::from("")); // Spacer
  }
  ```
- **Files modified:** decision_details_panel.rs (lines ~75-82)

### 2.3: Remove change_type rendering
- Delete lines 139-143 (change_type match statement)
- Remove line showing change_type_str (line ~159)
- **Files modified:** decision_details_panel.rs (lines ~139-159)

### 2.4: Remove confidence rendering
- Delete lines 145-155 (confidence match statements)
- Remove confidence_str and confidence_style lines (lines ~157-162)
- **Files modified:** decision_details_panel.rs (lines ~145-162)

### 2.5: Simplify code impact rendering
- Update impact rendering section to show only: file, line ranges, reasoning
- Remove change_type and confidence display logic entirely
- Keep file path, line ranges, reasoning intact
- **Files modified:** decision_details_panel.rs (lines ~124-174)

**Verification:**
- Run `cargo check --package diffviz-review-tui`
- Run `cargo clippy --package diffviz-review-tui` (zero warnings)
- TUI tests will still fail (they construct Decisions with old schema)

**Expected state after Phase 2:**
- ✅ diffviz-review-tui compiles
- ❌ diffviz-review-tui tests fail (test fixtures use old schema)

---

## Phase 3: Test Fixture Updates

**Objective:** Update all test construction to use new Decision/CodeImpact schema

**Scope:** Both diffviz-review and diffviz-review-tui test files

### Phase 3a: diffviz-review test updates

**Tasks:**

### 3a.1: Fix remaining decision.rs tests
- Any tests still failing after Phase 1.5
- Search for `ChangeType::`, `Confidence::`, `.summary` patterns
- Update to new schema
- **Files modified:** diffviz-review/src/entities/decision.rs (tests)

**Verification:**
- Run `cargo test --package diffviz-review`
- All tests should pass

### Phase 3b: diffviz-review-tui test updates

**Files to update (10 test files):**
1. decision_approval_tests.rs
2. decision_tree_expansion_tests.rs
3. keybinding_tests.rs
4. panel_management_tests.rs
5. leader_key_tests.rs
6. input_mode_tests.rs
7. core_navigation_tests.rs
8. (3 more test files from Grep results)

**Tasks for each file:**

### 3b.1: Update imports
- Remove `ChangeType` and `Confidence` from import statements
- Keep `CodeImpact`, `Decision`, `DecisionLineRange`
- **Pattern to find:** `use diffviz_review::{ChangeType, ...}`

### 3b.2: Update Decision construction
- Find all `Decision { ... }` construction
- Remove `summary: ...` field
- Add `rationale: Some("...")` or `rationale: None` field
- **Pattern to find:** `Decision {`

### 3b.3: Update CodeImpact construction
- Find all `CodeImpact { ... }` construction
- Remove `change_type: ChangeType::...` field
- Remove `confidence: Confidence::...` field
- Keep: file, line_ranges, reasoning
- **Pattern to find:** `CodeImpact {`

### 3b.4: Update test helper functions
- Example: `create_test_engine()` in decision_approval_tests.rs (lines 28-82)
- Example: `create_enriched_test_engine()` in same file
- Update all Decision/CodeImpact construction within helpers

**Verification after each file:**
- Run `cargo test --package diffviz-review-tui --test [filename]`
- Verify tests pass for that file

**Final verification:**
- Run `cargo test --workspace`
- Run `cargo clippy --workspace`
- Run `cargo check --workspace`
- All should pass with zero warnings

---

## Phase 4: Agent Skills YAML Templates

**Objective:** Create YAML templates and convert existing markdown artifacts to YAML for agent-skills

**Scope:** agent-skills/skills/dev-contribute and agent-skills/skills/dev-strategy templates

**Important Distinction:**
- Phase 4 creates **contribution documentation** templates (for agent-to-agent communication)
- This is SEPARATE from Phase 1-3 which modified **code review** Decision structs
- Both happen to use the name "decision log" but serve different purposes:
  - **diffviz-review Decision** = mapping architectural decisions to code changes for review
  - **agent-skills decision-log** = documenting decisions made during a contribution

### 4.1: Create decision-log.yaml template for dev-contribute

**File:** `agent-skills/skills/dev-contribute/templates/decision-log-template.yaml`

**Minimal structure:**
```yaml
# Contribution identification
contribution:
  number: <number>
  phase: "<phase>"
  title: "<title>"

# List of decisions
decisions:
  - id: "D1"
    title: "<decision title>"
    choice: "<what was chosen>"

    # FREE-FORM: rationale can be structured or unstructured
    rationale: |
      <explanation of why, alternatives considered, trade-offs>

    # FREE-FORM: Optional additional context
    impact: |
      <files affected, breaking changes, etc>

  - id: "D2"
    title: "<another decision>"
    choice: "<choice>"
    rationale: |
      <explanation>
```

**Key principles:**
- Only `id`, `title`, `choice` are required per decision
- `rationale` and `impact` are free-form strings (agents choose format)
- `contribution` block provides metadata for indexing

**Tasks:**
- Create template file with comments explaining each section
- Add example showing both minimal and verbose decision styles
- **Files created:** decision-log-template.yaml

### 4.2: Create decision-log.yaml template for dev-strategy

**File:** `agent-skills/skills/dev-strategy/templates/decision-log-template.yaml`

**Same structure as dev-contribute template** (reuse the schema)

**Tasks:**
- Copy template from dev-contribute (or symlink if possible)
- Add dev-strategy-specific examples in comments
- **Files created:** decision-log-template.yaml

### 4.3: Create context-handoff.yaml template

**File:** `agent-skills/skills/dev-contribute/templates/context-handoff-template.yaml`

**Minimal structure:**
```yaml
# REQUIRED: Contribution metadata
contribution:
  number: <number>
  phase: "<phase>"
  type: "<implementation|design|cleanup|refactor>"

# REQUIRED: What was delivered (simple list or structured)
deliverables:
  - "<description>"
  - "<description>"

# REQUIRED: Quality assessment (free-form)
quality_assessment:
  what_works: |
    <description of what's solid>

  what_is_fragile: |
    <description of risks/fragile areas>

# REQUIRED: Guidance for next contributors (free-form)
for_next_contributors: |
  <entry points, key learnings, blockers>

# OPTIONAL: Stats for metrics
stats:
  lines_added: <number>
  lines_removed: <number>
  autonomous_decisions: <number>
```

**Key principles:**
- Only 4 top-level sections required
- All content sections are free-form (string or structured)
- Optional stats block for tooling

**Tasks:**
- Create template with inline documentation
- Provide minimal and verbose examples
- **Files created:** context-handoff-template.yaml

### 4.4: Create changelog.yaml template

**File:** `agent-skills/skills/dev-contribute/templates/changelog-template.yaml`

**Minimal structure:**
```yaml
# REQUIRED: Contribution identification
contribution:
  number: <number>
  phase: "<phase>"
  type: "<type>"

# REQUIRED: Summary (one-liner)
summary: "<what was done>"

# REQUIRED: Changes (free-form list)
changes:
  - "<description of change>"
  - "<description of change>"

# REQUIRED: Verification status
verification: |
  <test results, compilation status>

# OPTIONAL: Stats
stats:
  lines_added: <number>
  lines_removed: <number>
  test_failures: <number>
```

**Tasks:**
- Create template with documentation
- Provide examples
- **Files created:** changelog-template.yaml

### 4.5: Update skill instructions to reference YAML templates

**Files to update:**
- `agent-skills/skills/dev-contribute/skill.md` - Reference YAML templates instead of markdown
- `agent-skills/skills/dev-strategy/skill.md` - Reference YAML templates instead of markdown

**Tasks:**
- Update instructions to generate YAML instead of markdown
- Update examples to show YAML structure
- Keep instructions otherwise unchanged

**Verification:**
- Templates exist and are valid YAML
- Templates have clear inline documentation
- Templates demonstrate both minimal and verbose styles

---

## Rollout Plan

### Phase 1: Entity updates
- **Estimated time:** 30 minutes
- **Risk:** Low (isolated to one file)
- **Verification:** `cargo check --package diffviz-review` compiles

### Phase 2: TUI rendering updates
- **Estimated time:** 20 minutes
- **Risk:** Low (isolated to rendering logic)
- **Verification:** `cargo check --package diffviz-review-tui` compiles

### Phase 3a: diffviz-review test updates
- **Estimated time:** 15 minutes
- **Risk:** Low (test-only changes)
- **Verification:** `cargo test --package diffviz-review` passes

### Phase 3b: diffviz-review-tui test updates
- **Estimated time:** 45 minutes (10 files)
- **Risk:** Low (test-only changes)
- **Verification:** `cargo test --workspace` passes

### Phase 4: Agent skills YAML templates
- **Estimated time:** 60 minutes
- **Risk:** Low (new files, no existing code affected)
- **Verification:** Templates are valid YAML, have clear documentation

### Final verification
- **Estimated time:** 10 minutes
- Run full workspace checks (Phases 1-3)
- Verify templates are valid YAML (Phase 4)
- Verify zero clippy warnings

**Total estimated time:** ~3 hours (2 hours for Phases 1-3, 1 hour for Phase 4)

---

## Success Criteria

### Track A: diffviz-review (Phases 1-3)

✅ **Compilation:**
- `cargo check --workspace` succeeds with zero errors

✅ **Tests:**
- `cargo test --workspace` all tests pass

✅ **Linting:**
- `cargo clippy --workspace` zero warnings

✅ **Functionality:**
- TUI renders decisions correctly with simplified structure
- Decision indexing still works (build_index_from_review_state)
- Decision approvals still work
- Navigation (decision tree → files → chunks) still works

✅ **Serialization:**
- Decision structs can serialize to/from YAML using serde_yaml
- Optional rationale field omitted when None

### Track B: agent-skills (Phase 4)

✅ **Templates Created:**
- decision-log-template.yaml exists in dev-contribute/templates/
- decision-log-template.yaml exists in dev-strategy/templates/
- context-handoff-template.yaml exists in dev-contribute/templates/
- changelog-template.yaml exists in dev-contribute/templates/

✅ **Template Quality:**
- All templates are valid YAML
- Templates have inline documentation
- Templates show both minimal and verbose examples

✅ **Skill Instructions Updated:**
- dev-contribute/skill.md references YAML templates
- dev-strategy/skill.md references YAML templates

---

## Rollback Strategy

If issues arise during implementation:

**After Phase 1:** Revert decision.rs changes, restore old struct definitions
**After Phase 2:** Revert decision_details_panel.rs changes
**After Phase 3a/3b:** Revert specific test file changes

**Git strategy:** Create feature branch, commit after each phase for easy rollback

---

## Post-Implementation

### Track A: diffviz-review (After Phases 1-3)

1. **Verify YAML serialization:**
   - Create example Decision in Rust
   - Serialize to YAML string
   - Verify clean output (no change_type, no confidence, rationale optional)

2. **Update documentation:**
   - Update decision.rs module doc comments
   - Note breaking changes in CHANGELOG (if exists)

3. **Consider next steps:**
   - Add YAML import/export helper methods to Decision struct?
   - Create example decision-log.yaml files for diffviz-review?
   - Update TUI help text if needed?

### Track B: agent-skills (After Phase 4)

1. **Test templates with real contribution:**
   - Use dev-contribute skill to create an actual contribution
   - Generate YAML artifacts using new templates
   - Verify agent can parse and understand the structure

2. **Gather feedback:**
   - Are templates too rigid or too flexible?
   - Do agents naturally use minimal or verbose style?
   - Are there common fields agents want to add?

3. **Consider tooling:**
   - Build stats aggregation tool (parse YAML, compute metrics)
   - Build CLI viewer to render YAML nicely for humans
   - Add YAML validation during contribution creation?

### Integration Opportunities

**Potential future connection:**
- Could dev-contribute eventually generate diffviz-review Decision YAML files?
- Contribution decision-log.yaml could map to code review Decision YAML
- This would connect contribution decisions → code review decisions
- Not in scope for this implementation, but architecturally possible
