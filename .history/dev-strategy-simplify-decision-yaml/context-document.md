# Context Document: Simplify Decision Structs for YAML-based Decision Logs

## Behavioral Specification

Two-track implementation to enable YAML-based decision logs across the codebase:

**Track A: diffviz-review (Code Review System)**
Simplify the `Decision` and `CodeImpact` data structures to support cleaner YAML serialization. The current structures contain fields (`summary`, `change_type`, `confidence`) that are either redundant or add unnecessary complexity for the YAML workflow.

**Track B: agent-skills (Contribution Documentation)**
Create YAML templates for dev-strategy and dev-contribute artifacts, replacing markdown templates with structured YAML that enables stats computation and tooling while preserving agent flexibility.

**What needs to change:**
1. Remove `summary` field from `Decision` struct (redundant with new `rationale` field)
2. Add optional `rationale` field to `Decision` struct (free-form explanation of why/alternatives/trade-offs)
3. Remove `change_type` field from `CodeImpact` struct (not needed for basic mapping)
4. Remove `confidence` field from `CodeImpact` struct (not essential for core functionality)
5. Remove `ChangeType` and `Confidence` enums (no longer needed)
6. Update all tests to match the simplified structure
7. Update TUI rendering to handle simplified Decision structure

**What stays the same:**
- Decision-based review workflow continues to work
- ReviewDecisions indexing algorithm unchanged (only uses file + line_ranges)
- All decision approval logic preserved
- TUI navigation and interaction unchanged

**Relationship between tracks:**
- Track A (diffviz-review) and Track B (agent-skills) both involve "decision logs" but serve different purposes:
  - **diffviz-review Decision** = Runtime entity for mapping architectural decisions → code changes during code review
  - **agent-skills decision-log.yaml** = Documentation artifact recording decisions made during a contribution
- Both benefit from simpler YAML structure
- Independent implementations, no direct code dependency
- Potential future integration: agent-skills could generate diffviz-review Decision YAML files

## Architecture Summary

### Current System Architecture

**diffviz-review** (Review orchestration layer):
- Contains core domain entities: `Decision`, `CodeImpact`, `ReviewDecisions`
- Uses Serde for serialization (already supports JSON/YAML via serde_yaml)
- Decision system maps architectural decisions → code line ranges
- Reverse indexing: ReviewableDiffId → affecting decision numbers
- Critical workflow: add_decision() → build_index_from_review_state() → query

**diffviz-review-tui** (TUI presentation layer):
- Consumes Decision entities from review engine
- Renders decision details panel showing: title, summary, code impacts, change_type, confidence
- Navigation: Decision tree (depth 0) → File list (depth 1) → Chunk list
- Test harness: 10 test files construct Decision structs with full field sets

### Key Integration Points

1. **Decision Entity Flow:**
   ```
   YAML string → serde_yaml::from_str() → Decision struct
     ↓
   ReviewDecisions.add_decision()
     ↓
   ReviewDecisions.build_index_from_review_state() [uses file + line_ranges only]
     ↓
   TUI queries via ReviewEngine.get_decision()
     ↓
   decision_details_panel.rs renders Decision fields
   ```

2. **Serialization Layer:**
   - decision.rs:12-28 defines ChangeType and Confidence enums with serde derives
   - decision.rs:40-46 defines CodeImpact with change_type and confidence fields
   - decision.rs:50-57 defines Decision with summary field
   - All structs use `#[derive(Serialize, Deserialize)]`

3. **TUI Rendering Dependencies:**
   - decision_details_panel.rs:75-79 displays `decision.summary`
   - decision_details_panel.rs:139-162 displays `impact.change_type` and `impact.confidence`
   - Confidence mapped to colors: High=green, Medium=yellow, Low=red

4. **Test Construction:**
   - decision_approval_tests.rs:39-76 constructs Decision with full fields
   - All 10 TUI test files create CodeImpact with change_type + confidence
   - decision.rs tests (270+ lines) create test fixtures with all fields

### Critical Algorithm: build_index_from_review_state()

**Purpose:** Maps ReviewableDiffId → decision numbers via line range overlap detection

**Algorithm (unchanged by this refactor):**
```
For each Decision:
  For each CodeImpact:
    For each ReviewableDiff in ReviewState:
      if diff.file_path == impact.file:
        if ranges_overlap(diff.line_range, impact.line_ranges):
          decision_index[diff.id].push(decision.number)
```

**Fields used:** Only `CodeImpact.file` and `CodeImpact.line_ranges`
**Fields NOT used:** change_type, confidence, reasoning

**Conclusion:** Removing change_type and confidence has ZERO impact on indexing logic.

## Research Findings

### Serde YAML Support

**serde_yaml crate (already in Cargo.toml):**
- Full round-trip serialization for Rust structs
- `#[serde(default)]` provides default values for missing fields during deserialization
- `#[serde(skip_serializing_if = "Option::is_none")]` omits None values from output
- Handles Option<T> naturally for optional fields

**Example transformation:**
```rust
// Before
pub struct Decision {
    pub number: u32,
    pub title: String,
    pub summary: String,  // REMOVE
    pub code_impacts: Vec<CodeImpact>,
}

// After
pub struct Decision {
    pub number: u32,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,  // ADD
    pub code_impacts: Vec<CodeImpact>,
}
```

### Field Removal Impact Analysis

**Decision.summary → Decision.rationale:**
- Current usage: TUI displays at decision_details_panel.rs:75-79
- New behavior: Display rationale if present, otherwise show nothing
- Migration: All test fixtures need `rationale` instead of `summary`
- **Impact:** Medium - straightforward find-replace in tests + UI update

**CodeImpact.change_type:**
- Current usage: TUI displays as "Addition/Modification/Deletion" string
- Justification: Git diffs already show add/modify/delete context
- Migration: Remove enum definition + all references in tests
- **Impact:** Medium - remove rendering logic from TUI

**CodeImpact.confidence:**
- Current usage: TUI displays with color styling (green/yellow/red)
- Justification: Not essential for core workflow, adds subjective complexity
- Migration: Remove enum definition + all references + color styling
- **Impact:** Medium - simplify TUI rendering

### Breaking Changes Assessment

**Serialization compatibility:**
- Removing fields: NOT backward compatible
- Adding optional fields: Forward compatible (old YAML can be read)
- **Decision:** This is acceptable (pre-production, no data to migrate)

**Affected code locations:**
1. **diffviz-review/src/entities/decision.rs:**
   - Lines 12-28: Remove ChangeType and Confidence enums
   - Lines 40-46: Update CodeImpact (remove 2 fields)
   - Lines 50-57: Update Decision (remove summary, add rationale)
   - Lines 270-923: Update all test fixtures

2. **diffviz-review-tui/src/ui/components/decision_details_panel.rs:**
   - Lines 75-79: Change from summary to rationale
   - Lines 139-162: Remove change_type and confidence rendering

3. **diffviz-review-tui/tests/*.rs (10 files):**
   - Update all Decision/CodeImpact construction (remove old fields)

## Constraints

### Technical Constraints
1. **Zero warnings:** Must pass `cargo clippy --workspace` and `cargo check --workspace`
2. **Serde for YAML:** Use serde_yaml (already in dependencies)
3. **Indexing preserved:** build_index_from_review_state() logic unchanged
4. **Test coverage:** All tests pass after changes

### Architectural Constraints
1. **Layer boundaries:** Changes to diffviz-review (entities) + diffviz-review-tui (rendering)
2. **No infrastructure deps:** Cannot depend on diffviz-git or other infra layers
3. **Read onboarding.md first:** Per CLAUDE.md mandatory rule

### Business Constraints
1. **Review workflow intact:** Decision-based navigation works
2. **Approval preserved:** Decision approval + cascade logic unchanged
3. **TUI interactions:** All keyboard shortcuts and features work
