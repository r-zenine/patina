# Code Context — tree-sitter upgrade

## Cargo files

| File | Relevance |
|------|-----------|
| `Cargo.toml` (workspace) | Add `tree-sitter`, grammar crates to `[workspace.dependencies]` |
| `diffviz-core/Cargo.toml` | All `tree-sitter*` version constraints live here |
| `diffviz-cli/Cargo.toml` | Duplicate `tree-sitter*` deps to align with core |

## `diffviz-core/src/parsers/`

### descriptor.rs (125 lines)
- `LanguageDescriptor` trait — the contract all 8 language parsers implement
- **L19**: `fn ts_language(&self) -> tree_sitter::Language` — return type stays, implementations change
- **L30**: `fn semantic_kind_map(&self) -> &[(&'static str, SemanticNodeKind)]` — Phase 2 target: complement with grammar-derived supertypes
- **L36**: `fn trivial_kinds(&self) -> &[&'static str]` — Phase 2 target: replace partial lists with grammar-aware derivation
- **L64–68**: `fn extract_identifier` default — uses `child_by_field_name("name")`, correct, unchanged

### generic_builder.rs (504 lines)
- `GenericSemanticTreeBuilder<D: LanguageDescriptor>` — consumes any descriptor
- **L33–44**: `new()` — builds `kind_map: HashMap` and `trivial_set: HashSet` from descriptor tables; Phase 2 will augment these at construction time from `Language::supertypes()`
- **L86–110**: `build_container_children` — main tree-walk loop; unchanged by upgrade
- **L191**: `node.child_by_field_name("name")` in `build_callable` — Phase 1 target for audit
- **L236**: `node.child_by_field_name("name")` in `build_data_structure` — Phase 1 audit
- **L438–444**: `try_parse` — calls `parser.set_language(self.descriptor.ts_language())`, API stable across 0.20–0.24
- **L452**: `get_language` — returns `self.descriptor.ts_language()`, unchanged

### rust.rs (439 lines)
- **L32–69**: `RUST_SEMANTIC_KIND_MAP` — 20 entries; Phase 2: Expression/Statement subset derivable from supertypes
- **L78–183**: `RUST_TRIVIAL_KINDS` — 70+ entries; punctuation, keywords, literals, identifiers; Phase 2: validate/trim using grammar introspection
- **L186–188**: `ts_language()` → **Phase 0 fix**: `tree_sitter_rust::language()` → `tree_sitter_rust::LANGUAGE.into()`
- **L221–247**: `extract_identifier` override — handles `let_declaration`; **L230–237**: `mut_pattern` case walks children manually for `identifier` — Phase 1: evaluate `field_name_for_named_child` simplification

### python.rs (260 lines)
- **L18–53**: `PYTHON_SEMANTIC_KIND_MAP` — 15 entries
- **L54–125**: `PYTHON_TRIVIAL_KINDS` — ~50 entries  
- **L126–128**: `ts_language()` → **Phase 0 fix**: `tree_sitter_python::language()` → `tree_sitter_python::LANGUAGE.into()`
- **L150**: `extract_identifier` override — custom logic for `decorated_definition`

### go.rs (226 lines)
- **L18–51**: `GO_SEMANTIC_KIND_MAP` — 14 entries
- **L52–123**: `GO_TRIVIAL_KINDS` — ~50 entries
- **L124–126**: `ts_language()` → **Phase 0 fix**: `tree_sitter_go::language()` → `tree_sitter_go::LANGUAGE.into()`
- **L148**: `extract_identifier` override — handles `short_var_declaration`, `var_declaration`

### typescript.rs (272 lines)
- **L16–49**: `TYPESCRIPT_SEMANTIC_KIND_MAP` — 13 entries
- **L50–132**: `TYPESCRIPT_TRIVIAL_KINDS` — ~60 entries
- **L133–135**: `ts_language()` → **Phase 0 fix**: `tree_sitter_typescript::language_typescript()` → `tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()`
- **L157**: `extract_identifier` override — handles arrow functions, class expressions

### javascript.rs (244 lines)
- **L17–42**: `JAVASCRIPT_SEMANTIC_KIND_MAP` — 11 entries (thinnest coverage)
- **L43–126**: `JAVASCRIPT_TRIVIAL_KINDS` — ~55 entries
- **L127–129**: `ts_language()` → **Phase 0 fix**: `tree_sitter_javascript::language()` → `tree_sitter_javascript::LANGUAGE.into()`
- No `extract_identifier` override — uses default `child_by_field_name("name")`

### java.rs (262 lines)
- **L16–52**: `JAVA_SEMANTIC_KIND_MAP` — 14 entries
- **L53–200**: `JAVA_TRIVIAL_KINDS` — ~100 entries (largest list; Java grammar is verbose)
- **L201–203**: `ts_language()` → **Phase 0 fix**: `tree_sitter_java::language()` → `tree_sitter_java::LANGUAGE.into()`
- No `extract_identifier` override

### c.rs (212 lines)
- **L16–46**: `C_SEMANTIC_KIND_MAP` — 9 entries (least complete)
- **L47–150**: `C_TRIVIAL_KINDS` — ~80 entries
- **L151–153**: `ts_language()` → **Phase 0 fix**: `tree_sitter_c::language()` → `tree_sitter_c::LANGUAGE.into()`

### cpp.rs (246 lines)
- **L16–36**: `CPP_SEMANTIC_KIND_MAP` — 9 entries
- **L37–184**: `CPP_TRIVIAL_KINDS` — ~100 entries
- **L185–187**: `ts_language()` → **Phase 0 fix**: `tree_sitter_cpp::language()` → `tree_sitter_cpp::LANGUAGE.into()`

## Test suite (diffviz-core/tests/)

| File | Covers |
|------|--------|
| `bug_*.rs` (5 files) | Regression tests for known bugs; primary safety net for kind-name drift |
| `regression_rust_parser_visibility_modifier_classification.rs` | Rust-specific classifier regression |
| `renderable_diff_anchor_tests.rs` | Semantic anchor integration tests |
| `test_utils.rs` | Shared test helpers, realistic fixture loading |
| `tests/fixtures/` | Realistic source file fixtures across multiple languages |
