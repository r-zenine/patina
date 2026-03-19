# Active Bugs - diffviz-core

## 🐛 Bug: Rust Parser Does Not Classify `impl` Blocks as Semantic Units

**Issue**: The Rust parser flattens `impl` blocks: methods are extracted and promoted as direct
children of the module, but the `impl_item` node itself is discarded from the SemanticTree.
This means byte ranges that start anywhere in the `impl` header (e.g. `impl Foo {`) or in doc
comments preceding a method do not fall inside any SemanticNode smaller than the module root.

When `find_semantic_unit_at_range()` is used (decision-based diff expansion), any input range
that covers the impl header escalates all the way to the Module node, producing catastrophic
expansion factors (100–150×).

**Repro**:
```
cargo run --bin diffviz -- debug --file <file> --from <commit> --line-range <start-end>
```
Input range covering `impl Foo { ... }` → expands to entire file.

**Root Cause**: `build_source_file_node()` in `src/parsers/rust.rs` calls `build_impl_items()`
which extracts only `function_item` children and adds them as siblings at module level.
The `impl_item` tree-sitter node is never wrapped in a `SemanticNode`.

**Impact**:
- Decision-based context expansion is unusable for any range touching an impl block header,
  doc comments before a method, or any gap between methods within an impl block.
- The expansion reports `Unit type: Module` and 100× expansion factors.

**Affected Languages**: Rust only (other languages don't have impl blocks)

**Test Location**: `tests/bug_rust_impl_block_not_classified.rs`

**Fix Required**: Represent `impl_item` as a `SemanticNode` (likely as a `DataStructure` or
dedicated `Module` subtype) with its methods as children, so that byte ranges inside impl
blocks resolve to the impl node rather than the file-level module.

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
