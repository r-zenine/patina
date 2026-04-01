# Active Bugs - diffviz-core

*(no active bugs)*

---

# Fixed Bugs

## ✅ Bug: Parent-Child Node Deletion Overlap (FIXED)

**Issue**: When a parent AST node is deleted (e.g., a class declaration), the algorithm reported BOTH:
1. The parent node deletion as a separate semantic pair
2. All child node deletions (e.g., method definitions) as individual semantic pairs

This created redundant/overlapping semantic pairs that represented the same structural change.

**Impact**:
- Review systems showed the same deletion at multiple nesting levels
- Duplicate review items for logically identical changes
- Confused reviewers about what actually changed

**Affected Languages**: All languages (language-agnostic Tree-sitter bug)

**Test Location**: `tests/bug_parent_child_deletion_overlap.rs`
- `typescript_class_to_functional_refactor_overlap()` - [PASSING] ✅
- `rust_struct_impl_deletion_overlap()` - [PASSING] ✅

**Solution**: Modified `build_semantic_pairs()` and `build_semantic_pairs_with_coverage()` in `semantic_ast.rs` to call `mark_node_and_children_as_used()` after creating deletion/addition pairs in Phase 2. This ensures that when a parent node is processed as a deletion/addition, all its children are marked as used and won't create separate redundant pairs.

**Changes**:
- `semantic_ast.rs` lines 990-1001: Added child marking for `build_semantic_pairs()`
- `semantic_ast.rs` lines 1089-1100: Added child marking for `build_semantic_pairs_with_coverage()`

**Behavior Change**:
- Before: Multiple overlapping deletion pairs for parent + all children
- After: Single deletion pair for parent node (children implicitly included)

---

## ✅ Bug: Rust `impl` Blocks Not Classified as Semantic Units (FIXED)

**Issue**: The old `RustParser` flattened `impl` blocks — methods were promoted as module-level
siblings and the `impl_item` node itself was never wrapped in a `SemanticNode`. Byte ranges
covering an impl header escalated to the file-level Module node (100–150× expansion).

**Affected Languages**: Rust only

**Test Location**: `tests/bug_rust_impl_block_not_classified.rs`
- `impl_block_range_should_not_expand_to_module()` — [PASSING] ✅
- `impl_block_method_should_resolve_to_impl_not_module()` — [PASSING] ✅

**Fix**: Phase 1 of the parser refactor. `RustDescriptor` maps `impl_item` → `ImplBlock` in
`RUST_SEMANTIC_KIND_MAP` and sets `container_body_field("impl_item") = Some("body")`.
`GenericSemanticTreeBuilder` recurses into the body, producing an `ImplBlock` node that wraps
its `function_item` children. Ranges inside impl blocks now resolve to the `ImplBlock` node,
not the file-level module.

---

## ✅ Bug: Struct Declaration Range Expansion (FIXED)

**Issue**: `SemanticNode` for a struct reported a byte range covering only its name/fields,
not the full struct including attributes and `pub` keyword. `find_semantic_unit_at_range()`
missed ranges that touched the keyword or leading attribute.

**Affected Languages**: Rust

**Test Location**: `tests/bug_struct_range_expansion.rs`
- `test_struct_declaration_range_should_expand_to_full_struct()` — [PASSING] ✅
- `test_decision_log_scenario_struct_range_should_expand()` — [PASSING] ✅

**Fix**: Phase 1 of the parser refactor. `GenericSemanticTreeBuilder::build_semantic_node()`
uses the tree-sitter node's own `.byte_range()`, which covers the complete source span including
leading modifiers and decorators already gathered as `metadata_nodes`.

---

## ✅ Bug: TypeScript Files Classified as "New File" Instead of "Modified" (Parser Layer) (FIXED)

**Issue**: `TypeScriptParser::build_semantic_tree()` returned an error for valid TypeScript source,
causing the review layer to fall back to a "new file" classification.

**Affected Languages**: TypeScript, TSX

**Test Location**: `tests/bug_typescript_file_classification.rs`
- `test_typescript_modified_file_classification()` — [PASSING] ✅
- `test_typescript_file_type_detection()` — [PASSING] ✅
- `test_typescript_new_files_work_correctly()` — [PASSING] ✅

**Fix**: Phase 2 of the parser refactor. `TypeScriptDescriptor` correctly maps TypeScript
tree-sitter node kinds (`function_declaration`, `class_declaration`, `interface_declaration`,
`type_alias_declaration`, etc.) and `GenericSemanticTreeBuilder` produces a valid `SemanticTree`
for all TypeScript source.

---

## ✅ Bug: JavaScript Files Shown as Unsupported Language Error (FIXED)

**Issue**: `JavaScriptParser` was a stub that returned `SemanticError::UnsupportedLanguage` for
all input, causing the review layer to surface a spurious "unsupported language" error for `.js`
and `.jsx` files.

**Affected Languages**: JavaScript, JSX

**Test Location**: `tests/bug_javascript_error_message.rs`
- `test_javascript_modified_files_should_not_show_error()` — [PASSING] ✅
- `test_javascript_new_files_work_correctly()` — [PASSING] ✅
- `test_cross_language_modified_file_error_pattern()` — [PASSING] ✅

**Fix**: Phase 2 of the parser refactor. `JavaScriptDescriptor` implements the full
`LanguageDescriptor` trait mapping JS node kinds (`function_declaration`, `class_declaration`,
`arrow_function`, `import_statement`, `export_statement`, etc.). JavaScript is now a fully
supported language in the semantic pipeline.
