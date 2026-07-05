# Context Document for patina-detect

> Immutable reference. Derived from `docs/patina-detect/spec.md` (design spec,
> nothing implemented yet) plus a working-tree audit on 2026-07-05 that
> surfaced three gaps the spec's own sequencing rationale doesn't account for:
> the TUI framework split is further along than the spec assumes, `lspkit` is
> a complete skeleton, and `ast-grep` isn't wired into the workspace yet.

## Behavioral Specification

Build `patina-detect`: deterministic, LLM-free detectors that surface
review-worthy symptoms in agent-generated code (re-implemented helpers, dead
exports, delegation chains, complexity extremes, near-duplicate structs,
parallel dispatch, data clumps, single-impl traits, house-rule violations),
each backed by typed evidence and content-addressed so triage verdicts survive
re-runs and rebases. Ship `patina-detect-tui`, a triage UI on the same
framework crates `diffviz-review-tui` uses, reusing the Decision UX's
navigate→drill→verdict flow with the semantics reversed (detector→triager
instead of author→reviewer). Ship them incrementally, one small detector or
one small infrastructure slice at a time, rather than as few large phases.

## Scope

**In scope** — all 9 detectors from `docs/patina-detect/spec.md` ("The nine
symptoms" section), the `patina-detect` domain crate (entities, detector
engine, baseline persistence), `patina-detect-tui` (triage UI), the `lspkit`
implementation work required to unblock detectors 3/4/6/7/9, the `ast-grep`
adapter for detector 1, and a Phase 0 that generalizes
`diffviz-tui-contribution` into a framework-wide skill plus extracts the
remaining domain-coupled generic UI components out of `diffviz-review-tui`.

**Out of scope (deferred, see spec + decision log)**

- Everything under the spec's "Deferred (and why)" section: feature envy,
  test theater, naming drift, arbitrary-block clones, churn hotspots as a
  standalone detector (churn stays a ranking input only).
- Spec open question 3 (instruction cascade auto-resolve vs confirm) —
  deferred until `patina-detect-tui` has working `Fix` verdicts; not a
  blocker for any phase in this plan.
- Any change to `diffviz-core`, `diffviz-review`, or `diffviz-review-tui`'s
  *behavior* beyond the Phase 0 component extraction (which must be behavior-
  preserving for `diffviz-review-tui`).

## Constraints already resolved this session (do not re-litigate)

1. **lspkit build order**: incremental. Each lspkit method (or small group,
   e.g. the three call-hierarchy methods) gets wired in its own phase,
   immediately before the first detector phase that consumes it — not one
   upfront "finish lspkit" phase.
2. **ast-grep integration**: no new crate. Rule-pack loading lives in an
   adapter module inside `patina-detect` itself; `ast-grep-*` crates are a
   dependency of that one crate only (design rule 6 in the spec already
   requires ast-grep's tree-sitter world stay out of `diffviz-core`).
3. **Audit-mode rendering**: confirmed working today.
   `create_reviewable_diff_from_range()` accepts `old_source:
   Option<&dyn SourceProvider>`; passing `None` is a real, tested path
   (`diffviz-core/tests/pin_added_boundary_rendering.rs:17-36`) producing
   correct `Addition`-classified rendering. No diffviz-core changes needed.
   Remaining unknown (checked in Phase 4, not before): side-by-side rendering
   of two independent clone-group members is a different call shape than the
   single-`None`-old-source case and isn't covered by the existing test.
4. **Fingerprint design per detector** (spec open question 1): treated as a
   mandatory deliverable of each detector's own phase's acceptance criteria,
   not a separate phase. Phase 1 only defines the `SymptomId` type/contract.
5. **Framework split reality check**: `tui-harness`, `tui-elm`, `tui-design`
   already exist as separate `libs/*` crates and `diffviz-review-tui` already
   depends on them (registry, leader state, which-key/help logic, card/
   scroll/theme primitives are already generic). Phase 0's scope is narrower
   than "extract a framework from scratch" — it's the drillnav pattern plus
   four render-only components, plus generalizing the governing skill.

## Codebase Patterns to Follow

- **Elm architecture** (`diffviz-tui-contribution` skill, generalized in
  Phase 0): pure views, `Command` for I/O, registry-only key matching, state
  encapsulation via methods. Applies to `patina-detect-tui` from its first
  line of code.
- **Descriptor pattern** (`diffviz-core`): per-language behavior via
  `LanguageDescriptor` trait + generic builder; detectors 1, 5, 8 (tree-sitter
  only) should extend this pattern (e.g. per-language complexity increment
  tables) rather than hardcoding per-language `match`es in the detector.
- **Layering** (spec design rule 6): detection lives in `patina-detect`;
  nothing detector-related enters `diffviz-core`. `patina-detect` consumes
  `diffviz-core` (`SemanticTree`) and `lspkit`.
- **Zero-warning rule** (root `CLAUDE.md`): `cargo fmt` / `clippy` / `check`
  clean after every phase, workspace-wide.
- **thiserror errors with `#[source]` chains** (root `CLAUDE.md`) for every
  new error type in `patina-detect` and `lspkit`.

## Technical Constraints

- `patina-detect` depends on `diffviz-core` and `lspkit`; must not depend on
  `diffviz-review` (that crate's Decision/Reviewable model is a UX ancestor,
  not a dependency — Symptom is modeled first-class per the spec's Concept
  section).
- `patina-detect-tui` depends on `tui-harness`, `tui-elm`, `tui-design`, and
  `patina-detect`; must not depend on `diffviz-review-tui`.
- `lspkit` stays domain-free: "pure LSP client infrastructure ... no
  dependency on any review/diff-orchestration domain layer" (its own module
  doc comment, `libs/lspkit/src/lib.rs:1-3`). Detector-specific interpretation
  of LSP results (e.g. "read/write split for fields") lives in `patina-detect`.
- Workspace must stay green (build + tests + clippy + fmt) at the end of
  every phase, including the 15 detector/infra phases after Phase 0 — this
  plan has many small phases specifically so that bar is always cheap to hit.

## Interaction with in-flight work

- `plan-core-hardening` (in progress, `.plans/plan-core-hardening/`) is
  deleting the legacy semantic-pairing API from `diffviz-core`, including
  `is_semantically_identical`. Detector 2 (Type-2 clones, Phase 4 here) reuses
  that function's *normalization logic* (placeholder substitution over
  identifiers/literals), not the function itself — re-derive it fresh in
  `patina-detect` rather than depending on code `plan-core-hardening` removes.
  Verify `plan-core-hardening`'s Phase 1 has landed before starting Phase 4;
  if not, read the pre-deletion version for the normalization approach.

## Local Repository Skills

- `diffviz-tui-contribution` — generalized into a framework-wide skill in
  Phase 0; the generalized skill governs every `patina-detect-tui` phase
  after that (3, and any TUI work embedded in later detector phases).
- `filing-bugs` — not expected to apply (this is new-feature work, not a bug
  fix), but if a detector phase's implementation surfaces a `diffviz-core` or
  `lspkit` defect unrelated to this plan's own code, file it there rather than
  fixing it inline.
