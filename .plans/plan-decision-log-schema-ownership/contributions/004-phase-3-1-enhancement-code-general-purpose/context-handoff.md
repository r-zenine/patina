# Context Handoff: Phase 3.1 Complete - Dynamic #[schema(...)] Attributes

## What Works

**Phase 3.1 objectives achieved:**
- ✅ Proper `#[schema(example = "...", comment = "...")]` attributes on struct fields
- ✅ Macro dynamically parses attributes using `darling::FromField`
- ✅ Generated YAML templates include examples from attribute values
- ✅ Zero hardcoding - templates fully derived from struct definitions
- ✅ All 194 tests pass, zero warnings
- ✅ Verified: `diffviz templates decision-log` outputs examples from attributes

**Example of what now works:**
```rust
#[derive(SchemaTemplate)]
pub struct Decision {
    #[schema(
        example = "Add authentication middleware",
        comment = "One-sentence summary of the architectural decision"
    )]
    pub title: String,
}
```

Generated template output:
```yaml
title: "Add authentication middleware"  # One-sentence summary of the architectural decision
```

**Architecture:**
- `SchemaAttr` struct with `#[derive(FromField)]` from darling parses field attributes
- `SchemaTemplate` derive macro extracts parsed schemas from fields
- `build_yaml_template` uses field schemas to dynamically construct YAML
- Comments and examples flow directly from code to templates

## What's Solid

**Idiomatic Rust pattern:**
- Matches style of `thiserror` attribute usage (cleaner than rustdoc parsing)
- Uses darling for battle-tested attribute parsing
- Helper attributes on struct fields (standard proc-macro pattern)
- No external configuration or YAML files required

**Single source of truth maintained:**
- Struct definitions contain example values AND schema generation rules
- Changes to struct fields automatically update templates
- No divergence possible - code and templates generated together

**Flexibility for future:**
- Easy to extend to DecisionLineRange (already has derive)
- Can add more attribute fields (`example`, `comment`, others) without macro changes
- Framework supports optional fields, arrays, nested structures

## Key Discovery

**Helper attributes require containing struct to have the derive macro:**
- `#[proc_macro_derive(SchemaTemplate, attributes(schema))]` declares what attributes the macro handles
- But `#[schema(...)]` helper attributes only work on fields within structs that have `#[derive(SchemaTemplate)]`
- Applied this to CodeImpact which previously didn't need SchemaTemplate
- Pattern proven: any struct with the derive can use its helper attributes

This is why CodeImpact needed the derive macro added even though it's a nested struct.

## What's Missing (Intentionally Deferred)

**Phase 3 future work:**
- ❌ Generic field processing (currently pattern-matches on struct names)
- ❌ Extended to context-handoff and design-doc (Phase 3.2+)
- ❌ Attribute-based customization beyond example/comment (Phase 3.3)
- ❌ Schema versioning (Phase 3.4)

**Not blockers:** The core pattern works perfectly. Other artifacts can follow the same approach.

## Guidance for Next Phase (Phase 3.2: Extend to context-handoff)

### Starting Point

You have:
- Working darling-based attribute parsing infrastructure
- Proven pattern for extracting field-level examples/comments
- YAML template generation that reuses parsed schemas
- Test infrastructure showing macro works correctly

### Phase 3.2 Direction

Extend template generation to context-handoff (markdown instead of YAML):

1. **Create ContextHandoff struct** with appropriate fields
2. **Apply #[schema(...)] attributes** to each field
3. **Update build_yaml_template** to detect struct type and generate markdown instead
4. **Add `diffviz templates context-handoff` command**
5. **Test agent workflow**: agents run command, get current template, fill in, validate

Example:
```rust
#[derive(SchemaTemplate)]
pub struct ContextHandoff {
    #[schema(
        example = "Built: Phase 3.1 macro-based template generation",
        comment = "One sentence describing what was built"
    )]
    pub built: String,

    #[schema(
        example = "Key insight: darling helper attributes work within structs that have the derive",
        comment = "Most important technical discovery"
    )]
    pub key_insight: String,
}
```

### Why This Order?

1. **Same pattern proven:** darling + attributes + dynamic generation
2. **Simpler than decision-log:** No nested arrays, cleaner structure
3. **High value for agents:** Current context-handoff docs are manual/static
4. **Incremental progress:** Each artifact extends the pattern

### Risk Assessment

- **Zero macro complexity increase:** Reuse darling infrastructure
- **Template generation:** Add markdown formatting (trivial)
- **Integration:** Same CLI pattern as decision-log
- **Adoption:** Agents already familiar with command-based workflow

## Key Insights for Contributors

### Why #[schema(...)] is Better Than Rustdoc

1. **Structured data:** Example and comment are distinct fields, not parsed strings
2. **Type-safe:** Darling validates attribute syntax at compile time
3. **Explicit:** Readers see immediately that these values are for schema
4. **Idiomatic:** Matches Rust ecosystem conventions (thiserror, serde, etc)
5. **No parsing:** No regex/heuristics to extract from rustdoc comments

### Darling's FromField Pattern

This macro uses a powerful pattern from darling:

```rust
#[derive(FromField)]
#[darling(attributes(schema))]
struct SchemaAttr {
    #[darling(default)]
    example: Option<String>,
    #[darling(default)]
    comment: Option<String>,
}
```

This automatically:
- Parses `#[schema(example = "...", comment = "...")]` syntax
- Handles optional fields with `#[darling(default)]`
- Returns error if unknown attributes used
- Works seamlessly with `proc_macro_derive` helper attributes

### What Makes This Work

Three pieces together:
1. **Macro declaration:** `#[proc_macro_derive(SchemaTemplate, attributes(schema))]`
2. **Darling struct:** `#[derive(FromField)]` with `#[darling(attributes(schema))]`
3. **Containing struct:** Must have `#[derive(SchemaTemplate)]` for helper attributes to work

Remove any one piece and it breaks. All three are essential.

## Next Steps

1. **Mark Phase 3.1 complete** - Commit contribution folder
2. **Plan Phase 3.2** - Analyze ContextHandoff schema requirements
3. **Extend macro** - Add markdown template generation
4. **Integrate CLI** - Add context-handoff commands
5. **Phase 3.3+** - Design-doc, versioning, optional attributes
