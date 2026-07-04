# Context Handoff - Phase 5 Design (sam-tui Entry Point)

## What Problem Are We Solving
sam-tui must adopt the agent CLI to prove the abstraction generalizes
(D006), but it's a library — the production binary is sam-cli's `sam`,
whose arg surface must not grow agent flags. The roadmap deferred where the
entry point lives to this gate.

## Design Overview
A fixture-driven example binary (`sam-tui/examples/modal_agent.rs`) calling
`run_agent_cli` with `HeadlessModalView<MockValue>` — the same fixtures the
test suite already uses. Discovery data comes from one hand-written const
binding list in `headless.rs` feeding both `describe()` and `affordances()`
(filtered by `has_options` / `allow_multi_select`), so sam stays
registry-free (D006) but internally consistent. Rejected: hidden sam-cli
subcommand (production surface), feature-gated bin (ceremony), registry
(speculative for ~12 keys). Skill-doc updates land in the same phase.

## Reading Guide
"How It Works" names the two methods and the const-list pattern;
"Success Criteria" is the plan's end-to-end gate (describe → drive →
observe with no source reading), to be run against both apps.
