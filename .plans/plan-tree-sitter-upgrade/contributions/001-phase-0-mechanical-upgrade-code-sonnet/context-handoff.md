# Context Handoff - Phase 0 Mechanical Upgrade

## 🎯 Core Result
**Built**: tree-sitter upgraded from 0.20 to 0.23 across the workspace; all 8 parsers on LANGUAGE constant; 57 tests green.
**Key insight**: The plan targeted 0.24 but the crate landscape forces 0.23 — python/javascript/go have no 0.24.x releases. 0.23 provides the LANGUAGE constant (the key Phase 0 goal) and is the highest version where all 8 grammars coexist.

## 🚦 Current State
**✅ Solid foundation**: Workspace compiles clean. Grammar crates at 0.23.x (latest). Zero kind-name drift detected in tests — all 57 tests pass without any SEMANTIC_KIND_MAP or TRIVIAL_KINDS fixups.
**⚠️ Needs attention**: Phase 0.5 is next — gut anonymous entries from TRIVIAL_KINDS using `!child.is_named()` guard in `generic_builder.rs:86–110`.
**⏸️ Deferred**: Phase 1 (`field_name_for_named_child`) requires tree-sitter 0.24. The API is not available at 0.23. Upgrading to 0.24 will need the python/javascript/go grammar crates to release 0.24.x versions, or we target 0.25 once typescript/java/cpp catch up.

## 👥 Next Agent Guidance
**Phase 0.5 implementer**: Work in `diffviz-core/src/parsers/generic_builder.rs`. Add `if !child.is_named() { continue; }` before the `trivial_set.contains(child.kind())` check in `build_container_children` (L86–110) and the same guard in `build_node` (L114–143). Then strip anonymous-only entries from all 8 `TRIVIAL_KINDS` statics (rust.rs, python.rs, go.rs, typescript.rs, javascript.rs, java.rs, c.rs, cpp.rs). Anonymous entries are punctuation like `{`, `}`, `;`, keywords, operators — anything where the grammar node has no name.
**Phase 1 implementer**: Verify tree-sitter 0.24 is reachable before starting (check if python/go/javascript have 0.24.x releases). If not, Phase 1 is blocked.

## 🔗 Integration Points
**Expects**: Nothing new — this was a pure mechanical upgrade.
**Provides**: `tree_sitter_<lang>::LANGUAGE.into()` pattern in all 8 `ts_language()` impls. `set_language(&lang)` (by reference) at all call sites.

## 📋 Reference Links
- [decision-log.yaml](decision-log.yaml) - Version choices and API changes documented
