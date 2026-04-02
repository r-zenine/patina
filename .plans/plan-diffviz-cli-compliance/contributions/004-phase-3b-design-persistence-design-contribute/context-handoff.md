# Context Handoff — Phase 3b Design: State Persistence

## What Was Decided

Design for moving `load_review_state` / `save_review_state` + 3 private DTOs from
`diffviz-cli/src/main.rs` into a new `diffviz-review/src/persistence.rs` module.

See [design-doc.md](design-doc.md) for the full spec (< 60 lines, implementation-ready).

## Implementer Checklist

1. Create `diffviz-review/src/persistence.rs` — move the 5 items verbatim, add `PersistenceError`
2. Add `pub mod persistence;` + re-exports to `diffviz-review/src/lib.rs`
3. In `diffviz-cli/src/main.rs`: delete lines 100–203, add `use diffviz_review::persistence::*`
4. `cargo clippy --workspace` — zero warnings required (ZERO WARNING RULE)
5. `cargo test --workspace` — all tests pass

## No Open Questions

All design decisions were made interactively — no follow-up design needed.
