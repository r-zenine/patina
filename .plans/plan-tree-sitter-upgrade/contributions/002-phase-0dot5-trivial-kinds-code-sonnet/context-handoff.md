# Context Handoff - Phase 0.5 Gut TRIVIAL_KINDS with is_named()

## Core Result
**Built**: Anonymous node filtering via `!child.is_named()` guard in `GenericSemanticTreeBuilder`. Stripped 558 lines of punctuation/keyword entries from all 8 TRIVIAL_KINDS tables.
**Key insight**: The anonymous-node entries (punctuation, keywords, operators) were dead weight — tree-sitter's `is_named()` is a free boolean check that handles all of them in one shot. Named-but-trivial nodes (identifiers, literals, comments, type nodes) remain in the tables because `is_named()` cannot distinguish them from semantic nodes.

## Current State
**✅ Solid**: 57 tests green, zero clippy warnings. All 8 TRIVIAL_KINDS tables are now lean — only named node kinds remain.
**⏸️ Blocked**: Phase 1 (`field_name_for_named_child`) requires tree-sitter 0.24. Currently on 0.23. Python, JavaScript, and Go grammar crates have no 0.24.x releases as of 2026-06. Phase 1 is deferred until those crates release 0.24.x or we target 0.25.

## Next Agent Guidance
**Phase 1 implementer**: Before starting, verify tree-sitter 0.24 is reachable:
```
cargo add tree-sitter@0.24 --dry-run  # or check crates.io
```
Check if `tree-sitter-python`, `tree-sitter-javascript`, `tree-sitter-go` have 0.24.x on crates.io. If not, Phase 1 is blocked.

If 0.24 is reachable: work in `diffviz-core/src/parsers/rust.rs:230-237` (the `mut_pattern` case in `extract_identifier`). Check if `mut_pattern` now has a named field in the 0.24 Rust grammar using `Node::field_name_for_child`. Then audit Go `short_var_declaration` and Python `decorated_definition` for similar simplifications (roadmap 1.2).

## Integration Points
**Provides**: `GenericSemanticTreeBuilder` now skips anonymous nodes before any table lookup. `build_container_children` and `build_node` both have the guard.
**Table state**: All 8 TRIVIAL_KINDS tables contain only named-node kinds. The `trivial_set` HashSet built at construction time is now much smaller.

## Reference Links
- [decision-log.yaml](decision-log.yaml) — Decisions on guard placement, Java literal ambiguity, dead Rust derive entries
