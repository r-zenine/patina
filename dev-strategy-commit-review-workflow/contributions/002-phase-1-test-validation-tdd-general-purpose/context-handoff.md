# Context Handoff - Phase 1 Test Validation (TDD)

## 🎯 Core Result (What agents get from this work)
**Built**: Validation that the 3 new tests are red and correctly specify the required Phase 1 contract.
**Key insight**: All 4 compile errors map cleanly to the 3 behavioral requirements. No gaps found. One existing test (`test_decision_log_parse_deserializes_correctly`) is a known casualty of the return type change — document it as a migration obligation for the implementation contributor.

## 🚦 Current State (Agent decision points)
**✅ Solid foundation**: Red state verified. 4 compile errors:
- `no field 'decisions' on type Vec<Decision>` — 2 errors from tests 1 and 2
- `no field 'base_commit' on type Vec<Decision>` — 2 errors from tests 2 and 3

**⚠️ Needs attention**: `test_decision_log_parse_deserializes_correctly` (line 279) uses the old `Vec<Decision>` API directly. It will break when `parse()` return type changes. The implementation contributor must update it to use `log.decisions.len()`, `log.decisions[0].number`, etc.

**⏸️ Deferred**: No DiffQuery routing tests — that code lives in `diffviz-cli/src/main.rs` which has no unit test surface for this path. Verified by end-to-end criterion in the roadmap.

## 👥 Next Agent Guidance (Specific handoff)
**Implementation Agent (003)**: Proceed with full implementation. Exact steps:
1. `diffviz-review/src/entities/decision.rs`:
   - Add `pub base_commit: Option<String>` with `#[serde(default, skip_serializing_if = "Option::is_none")]` to `DecisionLog` struct
   - Change `parse()` signature to `pub fn parse(content: &str) -> Result<DecisionLog>`
   - Remove the `.decisions` extraction from `parse()` body — just `Ok(serde_yaml::from_str(content)?)`
   - Update `test_decision_log_parse_deserializes_correctly`: replace `decisions.len()` with `log.decisions.len()` etc.
2. `diffviz-cli/src/main.rs` line 171 (`run_contribution_review`):
   - Change `let decisions = DecisionLog::parse(&content)?` to `let log = DecisionLog::parse(&content)?`
   - Replace `decisions` with `log.decisions` in `build_from_decisions`
   - Replace `DiffQuery::head_to_unstaged()` with `log.base_commit.clone().map(DiffQuery::commit_to_head).unwrap_or_else(DiffQuery::head_to_unstaged)`
3. `diffviz-cli/src/main.rs` line 228 (`run_debug_expansion`):
   - Same pattern: `let log = ...`, use `log.decisions`, use routing on `log.base_commit`
   - Remove the local `let query = DiffQuery::head_to_unstaged()` line at ~249
4. Run `cargo fmt --all && cargo clippy --workspace` — zero warnings
5. Run `cargo test --workspace` — all tests green

---
## 🔗 Integration Points (Technical context)
**Expects**: Contribution 001 tests are in place (confirmed).
**Provides**: Confirmed coverage map for the implementation contributor.

## 📋 Reference Links
- [decision-log.yaml](decision-log.yaml) - Technical choices made
