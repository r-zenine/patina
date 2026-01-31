# Active Bugs - diffviz-core

(No active bugs tracked - see Fixed Bugs section below)

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
