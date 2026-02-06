# Decision Log: Phase 2 - TUI Rendering Updates

## Phase Context
Contribution 002: Implement Phase 2 of the core-then-integrate strategy. Phase 1 (entity structure updates) was completed successfully - ChangeType and Confidence enums removed, Decision.summary replaced with Decision.rationale, all diffviz-review tests updated.

Goal: Update TUI rendering layer to work with simplified Decision/CodeImpact structure while maintaining all navigation and approval workflows.

---

## D1: Rendering Strategy for Optional Rationale

**Decision**: Display optional rationale inline in decision details panel with safe Some/None handling.

**Choice**: Use `if let Some(rationale)` pattern to conditionally render the rationale section.

**Rationale**:
The Decision.rationale field is now Option<String> to support architectural decisions without code context. The TUI must gracefully handle both cases:
- Some(text): Display the rationale in the decision details
- None: Skip the rationale section entirely

Using `if let` pattern provides clear intent: rationale display is optional and conditional. Alternatives considered:
- Unwrap with default: Loses information when None (unacceptable)
- Match statement: More verbose for simple Some/None case
- Using unwrap_or: Still renders something even when no rationale exists

The `if let` approach is idiomatic Rust and makes the optional nature explicit without noise.

**Impact**: Lines 75-79 in decision_details_panel.rs. Test fixture still passes because rationale is set to Some(...) for all test decisions.

---

## D2: Code Impact Rendering Simplification

**Decision**: Remove change_type and confidence rendering; keep only file, line ranges, and reasoning.

**Choice**: Delete the entire change_type and confidence matching logic; keep file/line rendering and add safety check for reasoning.

**Rationale**:
Phase 1 removed ChangeType and Confidence from the CodeImpact struct entirely. These were rendering-only fields - not used by any business logic, approval operations, or persistence. The TUI rendering code must be updated to reflect this.

Two implementation approaches:
1. Delete all change_type/confidence rendering code (chosen)
2. Keep stubs that render "unknown" for missing fields

Option 1 is cleaner and forces correctness: if a type is removed from the entity, it can't appear anywhere. This prevents accidental regressions if someone tries to add it back without updating business logic.

Safety improvement: Added `if !impact.reasoning.is_empty()` check before rendering reasoning. While reasoning is currently always populated in test fixtures, this defensive programming prevents blank reasoning lines in UI if a future change allows empty reasoning.

**Impact**: Lines 132-168 in decision_details_panel.rs. Removes 37 lines of code (match statements for change_type and confidence with color styling).

---

## D3: Test Fixture Update Strategy

**Decision**: Update hardcoded decisions in main.rs to remove change_type/confidence fields and replace summary with rationale.

**Choice**: Search and replace all three Decision construction blocks to use new schema.

**Rationale**:
The diffviz-review-tui/src/main.rs file contains three hardcoded Decision structs used for TUI testing. These must be updated to match the new schema or compilation fails.

Process:
1. Remove ChangeType and Confidence from imports
2. Update each Decision.summary → Decision.rationale: Some(...)
3. Remove change_type and confidence fields from all CodeImpact structs
4. Keep all reasoning text unchanged

This follows the same pattern used in Phase 1 for diffviz-review tests - mechanical schema updates with zero behavior changes.

**Impact**: Lines 10-168 in main.rs. Three decisions with total 8 CodeImpact structs updated.

---

## D4: Verification Scope

**Decision**: Run full workspace tests and clippy to verify no regressions.

**Choice**: Execute full test suite including decision_approval_tests and clippy across all crates.

**Rationale**:
Phase 2 touches only rendering code in the TUI layer. However, the TUI layer integrates with ReviewEngine which was already tested in Phase 1. To ensure no integration issues:
- Run diffviz-review-tui tests (rendering layer)
- Run decision_approval_tests (TUI + ReviewEngine integration)
- Run full workspace tests (complete integration)
- Run clippy (no warnings rule enforcement)

Results show all tests pass with zero warnings, confirming:
- Rendering changes work correctly with simplified data model
- ReviewEngine integration still functions
- No compilation issues anywhere in workspace
- Code quality maintained

**Impact**: Verification step only, no code changes.

---

## D5: Documentation Approach

**Decision**: Create comprehensive Phase 2 changelog and decision log for knowledge transfer.

**Choice**: Document all changes with clear file locations, behavioral changes, and breaking API changes.

**Rationale**:
Following dev-strategy pattern of mandatory documentation for each contribution:
- Changelog: High-level what/where/why for handoff to next phase
- Decision log: Technical rationale for architectural choices made during phase
- Context handoff: Exploration process, fragile areas, guidance for Phase 3

This enables Phase 3 contributors to understand the design intent and avoid introducing regressions. Specifically for Phase 3:
- Test fixture updates are a large task (10+ test files)
- Understanding why changes were made prevents defensive workarounds
- Breaking changes documented help future debugging

**Impact**: Documentation only, enables Phase 3 execution.

---

## Summary

Phase 2 completed successfully with:
- ✅ TUI rendering updated to optional rationale pattern
- ✅ Simplified code impact display (removed change_type/confidence rendering)
- ✅ All test fixtures updated to new schema
- ✅ Zero compilation warnings
- ✅ 140+ tests passing
- ✅ Full workspace verification clean

Key architectural insight: Rendering layer adapts to domain model changes without business logic modifications. Approval operations, navigation, and state management remain completely unchanged - only what displays to users was updated.
