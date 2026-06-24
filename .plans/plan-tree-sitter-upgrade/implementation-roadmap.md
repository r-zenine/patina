# Implementation Roadmap — tree-sitter Upgrade

**Strategy**: TDD  
**Target**: tree-sitter 0.24 (Phase 0–1)

---

## Phase 0 — Mechanical Upgrade to 0.24

**Deliverable**: workspace compiles on tree-sitter 0.24, all tests pass, zero warnings.

### 0.1 — Cargo.toml version bumps

Files: `diffviz-core/Cargo.toml`, `diffviz-cli/Cargo.toml`, `Cargo.toml` (workspace)

Move `tree-sitter` and grammar crates into `[workspace.dependencies]`:
```toml
tree-sitter = "0.24"
tree-sitter-rust = "0.24"
tree-sitter-c = "0.24"
tree-sitter-python = "0.24"
tree-sitter-javascript = "0.24"
tree-sitter-go = "0.23"
tree-sitter-typescript = "0.23"
tree-sitter-java = "0.23"
tree-sitter-cpp = "0.23"
# verify latest for: tree-sitter-json, tree-sitter-css, tree-sitter-toml
```

Remove duplicate `tree-sitter*` entries from `diffviz-cli/Cargo.toml` (replace with `{ workspace = true }`).

### 0.2 — Fix language acquisition (8 one-liners)

For each parser file, change `ts_language()`:

| File | Old | New |
|------|-----|-----|
| `parsers/rust.rs:187` | `tree_sitter_rust::language()` | `tree_sitter_rust::LANGUAGE.into()` |
| `parsers/python.rs:127` | `tree_sitter_python::language()` | `tree_sitter_python::LANGUAGE.into()` |
| `parsers/go.rs:125` | `tree_sitter_go::language()` | `tree_sitter_go::LANGUAGE.into()` |
| `parsers/typescript.rs:134` | `tree_sitter_typescript::language_typescript()` | `tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()` |
| `parsers/javascript.rs:128` | `tree_sitter_javascript::language()` | `tree_sitter_javascript::LANGUAGE.into()` |
| `parsers/java.rs:202` | `tree_sitter_java::language()` | `tree_sitter_java::LANGUAGE.into()` |
| `parsers/c.rs:152` | `tree_sitter_c::language()` | `tree_sitter_c::LANGUAGE.into()` |
| `parsers/cpp.rs:186` | `tree_sitter_cpp::language()` | `tree_sitter_cpp::LANGUAGE.into()` |

### 0.3 — `cargo check` clean pass

After 0.1 + 0.2, run `cargo check --workspace`. No further compile errors expected (no Query API, no deprecated node methods used). Fix any that appear.

### 0.4 — Test suite: detect and fix kind-name drift

Run `cargo test --package diffviz-core`. Grammar version bumps may have renamed node kinds. Failures indicate drift in `SEMANTIC_KIND_MAP` or `TRIVIAL_KINDS`. Fix table entries to match new grammar kind names.

Risk table (grammars most likely to have renamed kinds):
| Grammar | Risk | Watch for |
|---------|------|-----------|
| `tree-sitter-typescript` 0.20→0.23 | High | Several expression node names changed |
| `tree-sitter-go` 0.20→0.23 | Medium | Short var declaration, type node names |
| `tree-sitter-java` 0.20→0.23 | Medium | Class body, annotation nodes |
| `tree-sitter-rust` 0.20→0.24 | Low | Grammar is stable |
| `tree-sitter-python` 0.20→0.24 | Low | Grammar is stable |

### 0.5 — Zero-warning pass

`cargo fmt --all && cargo clippy --workspace` — fix all warnings before considering phase done.

**Estimated effort**: 1–3 hours (30 min mechanical + up to 2.5 hours on kind-name drift)

---

## Phase 1 — Simplify Identifier Extraction with `field_name_for_named_child`

**Deliverable**: `extract_identifier` overrides simplified where the 0.24 grammar exposes named fields; no behavioral change.

### 1.1 — Audit `mut_pattern` in the updated Rust grammar

`field_name_for_named_child` (new in 0.24): `Node::field_name_for_child(idx: u32) -> Option<&str>`

In `rust.rs:230-237`, `mut_pattern` is walked manually to find an `identifier` child because 0.20 had no named fields there. Check the 0.24 Rust grammar:

```rust
// Diagnostic: print field names for mut_pattern children
let lang = tree_sitter_rust::LANGUAGE.into();
for i in 0..node.child_count() as u32 {
    println!("{i}: {:?}", lang.field_name_for_id(node.child(i as usize).unwrap().kind_id()));
}
```

If `mut_pattern` now has a named `value` or `pattern` field: replace the cursor walk with `node.child_by_field_name("value")`.

If the field is still unnamed: keep the cursor walk (see D4).

### 1.2 — Audit other `extract_identifier` overrides

Review Go `short_var_declaration` and Python `decorated_definition` handling in their respective parsers for similar opportunities. Apply `field_name_for_named_child` where grammar fields are now named.

### 1.3 — Test + zero-warning pass

All existing tests must continue to pass. `cargo clippy` clean.

**Estimated effort**: 1–2 hours

---

## Phase 0.5 — Gut TRIVIAL_KINDS with `Node::is_named()`

**Deliverable**: anonymous node entries removed from all 8 `TRIVIAL_KINDS` tables; `build_container_children` skips unnamed nodes without a table lookup.

### 0.5.1 — Filter anonymous nodes in `GenericSemanticTreeBuilder`

In `generic_builder.rs`, `build_container_children` (L86–110) currently checks `self.trivial_set.contains(kind)`. Add an early-exit before the set lookup:

```rust
for child in container.children(&mut cursor) {
    if !child.is_named() {
        continue; // skip all anonymous nodes: punctuation, keywords, operators
    }
    // existing trivial_set check for named-but-trivial nodes
    if self.trivial_set.contains(child.kind()) {
        continue;
    }
    // ...
}
```

Apply the same guard in `build_node` (L114–143).

### 0.5.2 — Strip anonymous entries from all 8 TRIVIAL_KINDS tables

Remove every anonymous entry from each language's `TRIVIAL_KINDS` static. What stays (named-but-trivial nodes):
- `identifier`, `field_identifier`, `type_identifier`
- `string_literal`, `integer_literal`, `float_literal`, `boolean_literal`, `char_literal`, `raw_string_literal`
- `line_comment`, `block_comment`, `doc_comment`
- `visibility_modifier`, `function_modifiers` (also in SEMANTIC_KIND_MAP — keep)
- `primitive_type`, `reference_type`, `pointer_type`, `array_type`, `tuple_type`
- `binary_operator`, `unary_operator`, `assignment_operator`
- `ERROR`, `MISSING`

### 0.5.3 — Test + zero-warning pass

Tests must stay green (no behaviour change — anonymous nodes were never semantic). `cargo clippy` clean.

**Estimated effort**: 1 hour

---

## Phase 1 — Simplify Identifier Extraction with `field_name_for_named_child`

**Deliverable**: `extract_identifier` overrides simplified where the 0.24 grammar exposes named fields; no behavioral change.

### 1.1 — Audit `mut_pattern` in the updated Rust grammar

`field_name_for_named_child` (new in 0.24): `Node::field_name_for_child(idx: u32) -> Option<&str>`

In `rust.rs:230-237`, `mut_pattern` is walked manually to find an `identifier` child because 0.20 had no named fields there. Check the 0.24 Rust grammar:

```rust
// Diagnostic: print field names for mut_pattern children
let lang = tree_sitter_rust::LANGUAGE.into();
for i in 0..node.child_count() as u32 {
    println!("{i}: {:?}", lang.field_name_for_id(node.child(i as usize).unwrap().kind_id()));
}
```

If `mut_pattern` now has a named `value` or `pattern` field: replace the cursor walk with `node.child_by_field_name("value")`.

If the field is still unnamed: keep the cursor walk (see D4).

### 1.2 — Audit other `extract_identifier` overrides

Review Go `short_var_declaration` and Python `decorated_definition` handling in their respective parsers for similar opportunities. Apply `field_name_for_named_child` where grammar fields are now named.

### 1.3 — Test + zero-warning pass

All existing tests must continue to pass. `cargo clippy` clean.

**Estimated effort**: 1–2 hours

---

<!-- Phase 2 (Supertype API) dropped. Probe confirmed: Java/C/C++ — the grammars Phase 2 was
     designed for — expose zero _expression/_statement supertypes. The remaining benefit
     (~11 entries in Rust only) does not justify the grammar-introspection mechanism.
     Deferred note: if the project moves to tree-sitter 0.25 for other reasons, revisit
     _literal auto-trivialization for Rust only (tree_sitter_rust::LANGUAGE exposes _literal
     with ~6 subtypes). -->

## Completion Criteria

| Phase | Done when |
|-------|-----------|
| 0 | `cargo test --package diffviz-core` green, `cargo clippy` clean, all 8 parsers on 0.23/0.24 grammars |
| 0.5 | `build_container_children` uses `!is_named()` guard; anonymous entries gone from all 8 TRIVIAL_KINDS; tests green |
| 1 | `mut_pattern` and at least one other override simplified or documented-as-unchanged; tests green |
