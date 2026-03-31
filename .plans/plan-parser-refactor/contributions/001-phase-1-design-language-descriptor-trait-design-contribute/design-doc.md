# Design Document - LanguageDescriptor Trait API

## Decision: Data Tables + Targeted Override Methods

The `LanguageDescriptor` trait exposes language-specific behavior as static data slices (for the common cases) plus two targeted override methods with sensible defaults (for the genuinely unique cases: Go naming visibility and unusual metadata positioning).

## Why This Design

**Constraints That Led Here:**
- 7 parsers share identical skeleton logic — the differences are purely data (which node kinds, which field names)
- Go naming-convention visibility and unusual metadata positions are real but narrow deviations
- Impl blocks are a current bug (not classified) — the new design fixes this by adding `impl_item` to the kind map, not by special-casing it

**User Priorities:**
Fine-grained control per concern: targeted overrides scoped to specific behaviors (visibility, metadata), not a single catch-all escape hatch.

**Simplicity Rationale:**
Static slices are zero-boilerplate to implement. The two override methods cover all real deviations seen in 7 parsers. No generic hook avoids the temptation to put arbitrary logic there.

## How It Works

**The Trait:**
```
LanguageDescriptor {
  // Identity
  ts_language() -> tree_sitter::Language

  // Classification tables (static slices)
  semantic_kind_map() -> &[(&'static str, SemanticNodeKind)]
  trivial_kinds()     -> &[&'static str]

  // Structural config
  container_body_field(kind: &str) -> Option<&'static str>
  metadata_kind()     -> Option<&'static str>

  // Targeted overrides (with defaults)
  extract_visibility(node, source) -> String     // default: keyword sibling lookup
  collect_metadata(node, source)   -> Vec<MetadataNode>  // default: preceding metadata_kind siblings
}
```

**Builder Consumes Descriptor:**
`GenericSemanticTreeBuilder<D: LanguageDescriptor>` implements `LanguageParser`. Callers see no change — they still hold `&dyn LanguageParser`.

**Integration Points:**
- Builder's `classify_node_kind` scans `semantic_kind_map` (builds HashMap on construction)
- Builder's `build_semantic_tree` walks AST, dispatches via kind map, calls `extract_visibility` + `collect_metadata` for each semantic unit
- `container_body_field` tells builder where children live inside containers

## What We're NOT Doing

**Rejected Alternatives:**
- **Generic override hook**: `override_build() -> Option<Result<Vec<SemanticNode>>>` — too open-ended, hides where overrides actually occur
- **Pure data with flatten config**: `flatten_node_kinds + flatten_child_kinds` — impl blocks aren't special cases anymore (they're just correctly classified now)

**Out of Scope:**
- Phase 2 language descriptors (Python, Go, TS, Java, C, C++)
- `get_context_boundaries` and `classify_leaf_relevance` — handled in builder with existing defaults

## Implementation Guidance

**For Next Contributor:**
- Define `LanguageDescriptor` trait in `diffviz-core/src/parsers/descriptor.rs`
- `GenericSemanticTreeBuilder<D>` in `diffviz-core/src/parsers/generic_builder.rs`
- `RustDescriptor` in `diffviz-core/src/parsers/rust.rs` (replaces 882-line `RustParser`)
- Map `impl_item` in RustDescriptor's `semantic_kind_map` — this fixes the impl block classification bug

**Testing Strategy:**
- Assert `GenericSemanticTreeBuilder<RustDescriptor>` output equals current `RustParser` output for all Rust fixtures
- Un-ignore `bug_rust_impl_block_not_classified.rs` and `bug_struct_range_expansion.rs` tests

**Success Criteria:**
- All 4 existing Rust parser unit tests pass
- Byte coverage invariant holds (sum of child ranges == parent range at every node)
- Both bug test files pass (un-ignored)
- `RustDescriptor` implementation is ~100 lines
