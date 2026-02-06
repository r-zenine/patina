# Decision Log: Phase 1 Implementation

## Contribution Context
- **Phase:** 1 - Entity Structure Updates
- **Title:** Remove ChangeType/Confidence, implement optional rationale field
- **Type:** Implementation (structural refactoring, no behavior change)
- **Strategy:** Core-then-Integrate (update entities first, then consumers)

---

## D1: Approach for removing enums vs deprecation

**Decision:** Remove ChangeType and Confidence enums completely rather than deprecating

**Rationale:**
- Project is pre-production with no external users
- Cleaner approach: full removal > deprecation > migration
- Tests can be updated atomically with entity changes
- No data migration needed (tests are the only source of Decision construction)

**Alternatives considered:**
- Keep enums as deprecated: Rejected, adds complexity for pre-production code
- Keep as optional fields: Rejected, still maintains unused code paths

**Trade-offs:**
- Breaking change to public API (acceptable for pre-production)
- Clear semver signal (major version bump when released)

---

## D2: Ordering of changes - entities before exports

**Decision:** Update entity definitions first, then fix export statements, then update tests

**Rationale:**
- Reduces scope of compiler errors at each step
- Each stage is independently verifiable
- Makes debugging easier if issues arise

**Sequence followed:**
1. Remove ChangeType and Confidence enums from decision.rs
2. Update CodeImpact and Decision structs in decision.rs
3. Fix entities/mod.rs re-exports
4. Fix lib.rs public API re-exports
5. Update all test fixtures in decision.rs
6. Update test helpers in review_engine.rs

---

## D3: Rationale field design - Option<String> vs required

**Decision:** Make rationale Optional with serde defaults rather than required

**Rationale:**
- Some decisions may not need detailed rationale (title is sufficient)
- Aligns with YAML artifact philosophy: concise by default, verbose when needed
- Option<String> with skip_serializing_if reduces YAML bloat
- Test code already uses this pattern successfully

**Alternatives considered:**
- Make rationale required: Rejected, forces verbosity even for simple decisions
- Keep both summary and rationale: Rejected, creates redundancy

**Design details:**
```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub rationale: Option<String>,
```
- `default` allows deserialization of old YAML without rationale field
- `skip_serializing_if` keeps YAML clean when rationale is None

---

## D4: Create_unmapped_decision behavior with new schema

**Decision:** Update Decision 0 (unmapped) to use rationale instead of summary

**Rationale:**
- Consistency: all Decision structs use same schema
- Unmapped decision is still a valid decision with structured impact data
- The rationale explains why diffs are unmapped

**Implementation:**
- Change from `summary: "..."` to `rationale: Some("...")`
- Maintains the same informational content
- Semantically correct (rationale explains the unmapping)

---

## D5: Test fixture update strategy - systematically per location

**Decision:** Update test fixtures in groups: helper functions first, then test functions

**Rationale:**
- Helper functions are reused (e.g., create_test_decision)
- Updating helpers first reduces duplicate changes
- Systematic approach reduces missed updates

**Groups updated:**
1. Test helper create_test_decision() in decision.rs
2. 15+ test functions in decision.rs
3. 2 test helper functions in review_engine.rs

---

## D6: Export removal - public re-exports vs internal usage

**Decision:** Remove ChangeType and Confidence from all public re-exports immediately

**Rationale:**
- These are foundational entity types used in public API
- No internal uses of ChangeType or Confidence remain (all removed from code)
- Clean break: don't expose types that don't exist in entities

**Files updated:**
- entities/mod.rs: Re-exports for internal use
- lib.rs: Public API re-exports

**Note:** diffviz-review-tui imports still need updates in Phase 2

---

## D7: Serialization tests - removal vs preservation

**Decision:** Remove the serialization tests for ChangeType and Confidence

**Rationale:**
- These types no longer exist, tests cannot compile
- No need to preserve tests for removed functionality
- Entity serialization still tested via Decision serialization test

**Tests removed:**
- test_confidence_serialization()
- test_change_type_serialization()

**Tests preserved:**
- test_decision_serialization() - still validates Decision serde behavior

---

## Implementation Notes

### File Organization
All changes kept localized to entity definitions, exports, and tests. No changes to business logic or algorithms. The decision indexing algorithm in `build_index_from_review_state()` required no changes because it only uses `file` and `line_ranges` fields (not the removed fields).

### Compiler-Guided Refactoring
Let compiler errors guide the update path:
1. Remove enums → compilation error in exports
2. Fix exports → compilation error in tests
3. Update tests → all green

### Verification Strategy
After each edit, ran:
- `cargo check --package diffviz-review` (quick compilation check)
- `cargo test --package diffviz-review` (full test suite)
- `cargo clippy --package diffviz-review` (linting)
- `cargo fmt --package diffviz-review` (formatting)

---

## Risks Addressed

**Risk: Missing test fixture updates**
- Mitigation: Compiler errors guide all necessary changes
- Verification: All 140 tests pass

**Risk: Inconsistent structure across decisions**
- Mitigation: All Decision construction updated atomically
- Verification: Single schema consistently applied

**Risk: Breaking downstream consumers**
- Note: Only diffviz-review-tui and tests consume Decisions
- Next phase (Phase 2) will fix TUI consumption
- This phase only fixed test consumption

---

## Quality Metrics
- **Files modified:** 4
- **Lines removed:** ~50 (enums + test serialization)
- **Struct fields changed:** 2 (CodeImpact and Decision)
- **Test fixtures updated:** 17
- **Compilation warnings:** 0
- **Clippy warnings:** 0
- **Test pass rate:** 100% (140/140)

