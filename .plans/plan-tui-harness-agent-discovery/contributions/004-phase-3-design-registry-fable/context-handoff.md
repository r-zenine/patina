# Context Handoff - Phase 3 Design (Keybinding Registry)

## What Problem Are We Solving
Bindings live in four drifting copies (dispatch match arms, which-key tuples,
help overlay, skill docs), and `affordances()` cannot be implemented against
opaque match arms. The roadmap deferred the registry's exact shape to this
design gate: how scoping, aliases, parametric events, and descriptions are
represented so all five consumers read the same rows without post-processing.

## Design Overview
One `static BINDINGS: &[Binding]` table in `events/bindings.rs`; rows carry
`scope` (`Navigation | LeaderRoot | LeaderSubmenu(char) | Input`), `KeySpec`
aliases, a concrete `UiEvent`, notation strings, and a description. Matching
semantics are per-scope inside `lookup()` (exact modifiers for
Navigation/Input, code-only for Leader — reproducing the Phase 0
characterized quirks). Truly parametric behavior (`InputChar(c)`, leader
unknown-key deactivation) stays as coded fallbacks with doc-only `CATCH_ALLS`
rows so manifest and affordances still document them. Rejected: per-row match
strategies, HashMap indexing, proc-macros (D004), and fixing the leader
modifier quirk (behavior change forbidden by Phase 0).

## Reading Guide
Implementers: read "How It Works" for the interfaces and the five consumer
integration points, then "Implementation Guidance" for build order (dispatch
first against the Phase 0 oracle, overlays second, manifest/affordances
last). "What We're NOT Doing" prevents re-litigating settled trade-offs.
