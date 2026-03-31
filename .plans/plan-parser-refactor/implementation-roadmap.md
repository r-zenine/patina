# Implementation Roadmap for Parser Refactor

## Execution Strategy

**Strategy**: Core-then-Integrate
**Approach**: Build the generic semantic tree builder (domain core) first, then implement language descriptors (adapters), then port callers and remove old code (integration). Existing 44 passing tests + 100+ fixtures serve as correctness oracle throughout.

---

## Phase 1: Generic Semantic Tree Builder (Domain Core)

**Description**: Create the `LanguageDescriptor` trait and `GenericSemanticTreeBuilder` that encapsulate all shared parser logic. This is the foundation — tested in isolation against Rust fixtures before any old parser is touched.

**Objectives**:
- **Design**: Define the `LanguageDescriptor` trait API — what methods does a language descriptor need to provide? (node kind mappings, trivial tokens, metadata kinds, visibility strategy, container body fields, override hooks for unique constructs)
- **Implementation**: Implement `GenericSemanticTreeBuilder` that consumes any `LanguageDescriptor` to produce a `SemanticTree` with complete byte coverage invariant
- **Implementation**: Implement `RustDescriptor` as the first language descriptor (Rust is the most complex with impl blocks, making it the best test case)
- **Implementation**: Wire `RustDescriptor` through `LanguageParser` trait so existing callers work unchanged
- **Implementation**: Add structural invariant test: assert every byte in source maps to exactly one `SemanticNode`

**Testing Criteria**:
- `GenericSemanticTreeBuilder` + `RustDescriptor` produces identical `SemanticTree` output as current `RustParser` for all Rust fixtures
- All 4 existing Rust parser unit tests pass (`test_parse_simple_rust`, `test_classify_node_kinds`, `test_context_boundaries`, `test_rust_parser_creation`)
- Byte coverage invariant test passes: sum of child ranges == parent range for every node
- The 2 Rust impl block bug tests (`bug_rust_impl_block_not_classified.rs`) pass (un-ignore them)
- The 2 struct range expansion bug tests (`bug_struct_range_expansion.rs`) pass (un-ignore them)
- `cargo clippy --package diffviz-core` and `cargo fmt --all` clean

**Dependencies**: None

**Relevant Local Skills**: `design-contribute` (for LanguageDescriptor trait design), `dev-contribute` (for implementation)

**Files to Modify**:
- `diffviz-core/src/parsers/mod.rs` — Add new modules for descriptor trait and generic builder
- `diffviz-core/src/parsers/` — New file(s) for `LanguageDescriptor` trait + `GenericSemanticTreeBuilder`
- `diffviz-core/src/parsers/rust.rs` — Replace 882-line bespoke parser with ~100-line `RustDescriptor` + thin `LanguageParser` wrapper
- `diffviz-core/tests/bug_rust_impl_block_not_classified.rs` — Un-ignore tests
- `diffviz-core/tests/bug_struct_range_expansion.rs` — Un-ignore tests

---

## Phase 2: Migrate All Language Descriptors

**Description**: Convert the remaining 6 big parsers (Python, Go, TypeScript, Java, C, C++) to language descriptors, and promote JavaScript from stub to full descriptor. Each conversion is independent and validated against that language's fixtures.

**Objectives**:
- **Implementation**: Implement `PythonDescriptor` replacing `PythonParser` (713 lines → ~80 lines)
- **Implementation**: Implement `GoDescriptor` replacing `GoParser` (577 lines → ~60 lines)
- **Implementation**: Implement `TypeScriptDescriptor` replacing `TypeScriptParser` (821 lines → ~90 lines)
- **Implementation**: Implement `JavaDescriptor` replacing `JavaParser` (884 lines → ~90 lines)
- **Implementation**: Implement `CDescriptor` replacing `CParser` (700 lines → ~70 lines)
- **Implementation**: Implement `CppDescriptor` replacing `CppParser` (721 lines → ~80 lines)
- **Implementation**: Implement `JavaScriptDescriptor` as new full parser (promoting from stub)
- **Implementation**: Un-ignore TypeScript and JavaScript bug tests

**Testing Criteria**:
- All 44 previously passing tests still pass
- All language-specific fixtures produce identical `SemanticTree` output
- TypeScript bug tests pass: `test_typescript_modified_file_classification`, `test_typescript_file_type_detection` (un-ignored)
- JavaScript bug tests pass: `test_javascript_modified_files_should_not_show_error`, `test_cross_language_modified_file_error_pattern` (un-ignored)
- All 9 previously-ignored bug tests now pass (0 ignored tests remaining)
- Byte coverage invariant holds for all languages
- `cargo clippy --package diffviz-core` clean

**Dependencies**: Phase 1 complete

**Relevant Local Skills**: `dev-contribute`

**Files to Modify**:
- `diffviz-core/src/parsers/python.rs` — Replace with `PythonDescriptor` (~80 lines)
- `diffviz-core/src/parsers/go.rs` — Replace with `GoDescriptor` (~60 lines)
- `diffviz-core/src/parsers/typescript.rs` — Replace with `TypeScriptDescriptor` (~90 lines)
- `diffviz-core/src/parsers/java.rs` — Replace with `JavaDescriptor` (~90 lines)
- `diffviz-core/src/parsers/c.rs` — Replace with `CDescriptor` (~70 lines)
- `diffviz-core/src/parsers/cpp.rs` — Replace with `CppDescriptor` (~80 lines)
- `diffviz-core/src/parsers/javascript.rs` — Replace stub with `JavaScriptDescriptor` (~70 lines)
- `diffviz-core/tests/bug_typescript_file_classification.rs` — Un-ignore tests
- `diffviz-core/tests/bug_javascript_error_message.rs` — Un-ignore tests

---

## Phase 3: Port Callers and Cleanup

**Description**: Update all code outside diffviz-core/src/parsers/ that references old parser types. Remove dead code. Update the parser factory. Validate the full workspace compiles and all tests pass.

**Objectives**:
- **Implementation**: Update `review_engine_builder.rs` parser factory to instantiate new descriptor-based parsers
- **Implementation**: Update `test_utils.rs` parser factory to use new types
- **Implementation**: Remove any dead code from old parser implementations that wasn't carried forward
- **Implementation**: Update `diffviz-core/CLAUDE.md` and `onboarding.md` to reflect new architecture
- **Implementation**: Run full workspace build + test + clippy to ensure zero regressions

**Testing Criteria**:
- `cargo build --workspace` succeeds with zero warnings
- `cargo test --workspace` passes all tests (including previously-ignored bug tests)
- `cargo clippy --workspace` clean
- No references to old bespoke parser internals remain (grep for old method names like `build_source_file_node`, `build_function_node`, etc.)
- `review_engine_builder.rs` correctly instantiates parsers for all supported file extensions

**Dependencies**: Phase 2 complete

**Relevant Local Skills**: `dev-contribute`

**Files to Modify**:
- `diffviz-review/src/review_engine_builder.rs` — Update parser factory (lines 225-232)
- `diffviz-core/tests/test_utils.rs` — Update `parser_for_language()` factory
- `diffviz-core/src/parsers/mod.rs` — Update public exports
- `diffviz-core/CLAUDE.md` — Document new architecture
- `diffviz-core/onboarding.md` — Update onboarding guide
- `diffviz-core/bugs.md` — Mark resolved bugs as fixed
