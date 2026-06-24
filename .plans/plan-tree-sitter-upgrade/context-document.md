# Context Document — tree-sitter Upgrade (0.20 → 0.24, then 0.25)

## Behavioral Spec

Upgrade the `tree-sitter` dependency from 0.20 to 0.24, leveraging new 0.24 APIs to:
1. Eliminate the single breaking change (language acquisition API)
2. Gut TRIVIAL_KINDS anonymous entries using `Node::is_named()` (available since 0.1)
3. Simplify `extract_identifier` overrides using `field_name_for_named_child` (0.24)

The Supertype API (tree-sitter 0.25) was evaluated and dropped — see D9 in decision log.

At no point should behaviour observable from outside `diffviz-core` change. The entire test suite must pass green throughout.

## Architecture Summary

`diffviz-core` uses tree-sitter in one pattern: parse source text into a `Tree`, then walk `Node`s to build a `SemanticTree`. There is no Query API usage.

The descriptor pattern (`LanguageDescriptor` trait + `GenericSemanticTreeBuilder<D>`) is the key abstraction:
- Each of 8 languages provides a descriptor with static `SEMANTIC_KIND_MAP` and `TRIVIAL_KINDS` arrays
- `GenericSemanticTreeBuilder::new()` materialises these into a `HashMap` and `HashSet` at construction time
- All tree-walking is done against `Node` methods that are stable across 0.20–0.24

The only instability across versions is:
1. How language handles are obtained from grammar crates (`language()` fn → `LANGUAGE` constant)
2. Node kind names emitted by updated grammar crates (grammar-specific, discovered through tests)

## Research Findings

### Breaking Change: Language Acquisition (0.23+)
Grammar crates dropped `language()` in favour of a `LANGUAGE: LanguageFn` constant:
```rust
// 0.20
tree_sitter_rust::language()
tree_sitter_typescript::language_typescript()

// 0.24
tree_sitter_rust::LANGUAGE.into()
tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
```
8 call sites, one per language parser. The `LanguageDescriptor::ts_language()` return type (`tree_sitter::Language`) does not change.

### Grammar Crate Version Matrix
| Grammar | Current | Target (Phase 0) |
|---------|---------|-----------------|
| `tree-sitter` core | 0.20 | **0.24** |
| `tree-sitter-rust` | 0.20 | **0.24** |
| `tree-sitter-c` | 0.20 | **0.24** |
| `tree-sitter-python` | 0.20 | **0.24** |
| `tree-sitter-javascript` | 0.20 | **0.24** |
| `tree-sitter-go` | 0.20 | **0.23** (latest) |
| `tree-sitter-typescript` | 0.20 | **0.23** (latest) |
| `tree-sitter-java` | 0.20 | **0.23** (latest) |
| `tree-sitter-cpp` | 0.20 | **0.23** (latest) |
| `tree-sitter-json` | 0.20 | verify latest |
| `tree-sitter-css` | 0.20 | verify latest |
| `tree-sitter-toml` | 0.20 | verify latest |

Not targeting 0.25 in Phase 0: `tree-sitter-go`, `tree-sitter-java`, `tree-sitter-cpp` top out at 0.23.x and the supertype API (Phase 2) requires grammar crates to support ABI 15.

### New API: `field_name_for_named_child` (0.24)
`Node::field_name_for_child(idx: u32) -> Option<&str>` — returns the grammar field name for a child at a given index. Currently `rust.rs:230-237` walks `mut_pattern.children()` searching for an `identifier` child because `mut_pattern` has no named fields in the 0.20 grammar. This API (combined with grammar updates) may allow simplification — evaluate during Phase 1.

### Supertype Introspection (0.25) — evaluated, dropped
`Language::supertypes()` was researched and probed across all 8 grammar crates. Findings: Java, C, and C++ — the grammars with the most incomplete kind maps — expose zero `_expression` or `_statement` supertypes. Go/Java/C++ are also stuck at grammar ABI 14 (0.23), making `Language::supertypes()` (ABI 15 / 0.25) unqueryable at installable versions. Net benefit reduced to ~11 entries in Rust only. Dropped (see D9).

## Constraints

- CLAUDE.md: zero warnings rule — `cargo fmt + clippy + check` must be clean after every phase
- CLAUDE.md: no string/regex operations in diffviz-core — tree-sitter only
- CLAUDE.md: no fallbacks — fail fast; no defensive programming
- The `LanguageDescriptor` trait is a public API consumed by all 8 parsers; signature changes require updating all implementations
- Phase 2 (supertype API) depends on grammar crates releasing 0.25-compatible versions — defer until that is true
