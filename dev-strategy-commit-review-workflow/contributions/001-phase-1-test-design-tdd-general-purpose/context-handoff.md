# Context Handoff - Phase 1 Test Design (TDD)

## 🎯 Core Result (What agents get from this work)
**Built**: Three failing unit tests in `diffviz-review/src/entities/decision.rs` that specify the new `DecisionLog` API.
**Key insight**: All three tests fail with E0609 (no field on type `Vec<Decision>`) — this is the compile-time red state for Rust TDD. The existing passing test `test_decision_log_parse_deserializes_correctly` also calls `decisions.len()` directly on the parse result and will need updating during implementation.

## 🚦 Current State (Agent decision points)
**✅ Solid foundation**: Red state is confirmed. Three tests added, four compile errors emitted. The test names clearly express the behavioral requirements.
**⚠️ Needs attention**: The existing test `test_decision_log_parse_deserializes_correctly` (line ~295) calls `DecisionLog::parse(yaml).unwrap()` and then uses `.len()`, `.[]` indexing on the result directly. When `parse()` is changed to return `Result<DecisionLog>`, that test will also break and must be updated by the implementation contributor.
**⏸️ Deferred**: No tests written for the DiffQuery routing logic in `diffviz-cli/src/main.rs` — that logic is CLI glue code with no unit test surface. The routing is verified by the end-to-end criterion in the roadmap.

## 👥 Next Agent Guidance (Specific handoff)
**Test Validation Agent (002)**: Confirm the 4 compile errors are present and cover the three behavioral requirements: (1) `parse()` returns `DecisionLog`, (2) `base_commit` is `None` without the field in YAML, (3) `base_commit` is `Some(hash)` with the field. Check that the existing test at line ~295 is also a known breakage point for implementation.
**Implementation Agent (003)**: Make the three new tests green by:
1. Adding `base_commit: Option<String>` with `#[serde(default, skip_serializing_if = "Option::is_none")]` to `DecisionLog` struct
2. Changing `parse()` return type from `Result<Vec<Decision>>` to `Result<DecisionLog>` (remove the `.decisions` extraction)
3. Updating both callers in `diffviz-cli/src/main.rs` (`run_contribution_review` at L171 and `run_debug_expansion` at L228): use `log.decisions` for `build_from_decisions` and `log.base_commit` for query routing
4. Updating `test_decision_log_parse_deserializes_correctly` to call `log.decisions.len()` etc.

---
## 🔗 Integration Points (Technical context)
**Expects**: Clean baseline — all tests passed before this contribution (confirmed).
**Provides**: Precise compile-time specification of the `DecisionLog` API contract needed for Phase 1.

## 📋 Reference Links
- [decision-log.yaml](decision-log.yaml) - Technical choices made
