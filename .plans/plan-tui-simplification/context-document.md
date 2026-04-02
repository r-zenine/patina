# Context Document — TUI Simplification

## Behavioral Specification

Remove dead code, disabled features, and an unsafe workaround from `diffviz-review-tui` that were identified as violations of YAGNI, Kent Beck's 4 Simple Design Rules, and Sufficient Implementation. The codebase should compile clean with zero warnings after each change, and all existing behaviour must be preserved — nothing functional is being removed, only code that is already inactive or unreachable.

## Codebase Patterns to Follow

- **Error handling**: fail-fast with `expect()` for invariants (in-process, not at system boundaries)
- **State management**: Elm-inspired pure `UiState` + side-effect `Command` enum — keep all changes consistent with this model
- **Testing**: zero-warning rule applies to all targets including `--all-features` (test harness is feature-gated under `test-harness`)
- **Unsafe**: none allowed; the `into_review_engine()` unsafe block is the one exception being removed

## Technical Constraints

- **ZERO WARNING RULE**: after every change run `cargo fmt --all && cargo clippy --workspace -- -D warnings && cargo check --workspace`
- **diffviz-core boundary**: no string/regexp operations, no fallbacks — not directly affected by this plan but must not be violated in the process
- **Test harness is intentionally excluded**: the `test_harness` module (feature-gated) serves agentic TUI testing and must not be touched beyond removing fields that become dead due to state.rs changes
- **No functional regressions**: all 6 changes are dead-code removals or safe refactors — verified by the build passing

## Local Repository Skills

- **`diffviz-tui-contribution`** (project skill) — mandatory skill for any interaction with `diffviz-review-tui`; load before contributing code or tests

## Research Findings

No research conducted — all changes involve existing Rust patterns (`Option::take`, named constants, module deletion).
