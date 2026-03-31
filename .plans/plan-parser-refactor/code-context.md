# Code Context for Parser Refactor

## Core Trait & Data Model

- **LanguageParser trait** (`diffviz-core/src/common.rs:126`) - Core trait: `try_parse`, `get_language`, `build_semantic_tree`, `classify_node_kind`, `get_context_boundaries`, `classify_leaf_relevance`
- **SemanticTree** (`diffviz-core/src/semantic_ast.rs:28`) - Root node + language + source_ranges
- **SemanticNode** (`diffviz-core/src/semantic_ast.rs:62`) - tree_sitter_node, metadata_nodes, children, name_node, unit_type
- **SemanticUnitType** (`diffviz-core/src/semantic_ast.rs:81`) - 6 variants: DataStructure, Callable, Variable, Import, Module, Unknown
- **SemanticNodeKind** (`diffviz-core/src/common.rs:36`) - 14-category classification enum used for node dispatch
- **ProgrammingLanguage** (`diffviz-core/src/common.rs:76`) - Language enum with `from_file_path`
- **MetadataNode / MetadataPosition** (`diffviz-core/src/semantic_ast.rs:42-58`) - Preceding/following attribute tracking

## Parser Implementations (to be replaced)

- **RustParser** (`diffviz-core/src/parsers/rust.rs:16`, 882 lines) - Full impl with `build_source_file_node`, `build_function_node`, `build_struct_node`, `build_enum_node`, `build_module_node`, `build_import_node`, `build_variable_node`, `build_impl_items`
- **PythonParser** (`diffviz-core/src/parsers/python.rs`, 713 lines) - Full impl, decorator handling
- **GoParser** (`diffviz-core/src/parsers/go.rs`, 577 lines) - Full impl, naming-convention visibility
- **TypeScriptParser** (`diffviz-core/src/parsers/typescript.rs`, 821 lines) - Full impl, type system handling
- **JavaParser** (`diffviz-core/src/parsers/java.rs`, 884 lines) - Full impl, annotation handling
- **CParser** (`diffviz-core/src/parsers/c.rs`, 700 lines) - Full impl, preprocessor handling
- **CppParser** (`diffviz-core/src/parsers/cpp.rs`, 721 lines) - Full impl, template/namespace handling
- **JavaScriptParser** (`diffviz-core/src/parsers/javascript.rs`, ~50 lines) - Stub, returns UnsupportedLanguage from build_semantic_tree
- **CssParser** (`diffviz-core/src/parsers/css.rs`, ~35 lines) - Stub
- **JsonParser** (`diffviz-core/src/parsers/json.rs`, ~35 lines) - Stub
- **TomlParser** (`diffviz-core/src/parsers/toml.rs`, ~35 lines) - Stub

## Callers of LanguageParser (must be ported)

### Production Code
- **decision_based_diff.rs** (`diffviz-core/src/decision_based_diff.rs`) - Uses `&dyn LanguageParser` at lines 241, 285, 316, 348, 380, 387, 492; calls `try_parse` + `build_semantic_tree` + `classify_node_kind`
- **reviewable_diff.rs** (`diffviz-core/src/reviewable_diff.rs`) - Uses `&dyn LanguageParser` at lines 111, 150, 311, 332, 362, 438; calls `classify_node_kind` + `get_context_boundaries` + `classify_leaf_relevance`
- **review_engine_builder.rs** (`diffviz-review/src/review_engine_builder.rs:225`) - Factory: maps file extension → `Box<dyn LanguageParser>`, instantiates all 6 big parsers (lines 227-232)

### Test Code
- **test_utils.rs** (`diffviz-core/tests/test_utils.rs:17-30`) - `parser_for_language()` factory returning `Box<dyn LanguageParser>`
- **ast_diff/tests.rs** (`diffviz-core/src/ast_diff/tests.rs`) - Direct RustParser instantiation in demo tests
- **semantic_ast.rs tests** (`diffviz-core/src/semantic_ast.rs:861-984`) - Multiple RustParser::new() in unit tests
- **parsers/rust.rs tests** (`diffviz-core/src/parsers/rust.rs:820-865`) - Parser-specific unit tests
- **4 bug test files** (`diffviz-core/tests/bug_*.rs`) - 9 ignored tests across 4 files

## Key Algorithms Consuming SemanticTree

- **build_semantic_pairs** (`diffviz-core/src/semantic_ast.rs`) - Pairs old/new semantic nodes for diff
- **build_semantic_pairs_with_coverage** (`diffviz-core/src/semantic_ast.rs`) - Same with coverage tracking
- **find_semantic_unit_at_range** (`diffviz-core/src/decision_based_diff.rs`) - Range → semantic node lookup (where impl block bug manifests)

## Test Infrastructure

- **44 passing tests**, **9 ignored** (4 bug files)
- **100+ structured test fixtures** in `tests/fixtures/{language}/` organized by change type
- Fixture format: JSON with old_code, new_code, expected_changes, performance_expectations
