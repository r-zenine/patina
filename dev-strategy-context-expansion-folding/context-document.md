# Context Expansion and Folding Implementation

## Behavioral Specification

### What We're Building

Implement **context expansion and folding** for code review diffs to make reviewing large changes manageable.

**Current Problem:**
- `expand_changes_to_reviewable_diffs()` doesn't actually expand context - it just marks the changed node as ESSENTIAL
- No parent boundary detection happens
- No sibling context classification occurs
- All lines get ESSENTIAL relevance → nothing can be folded
- Result: Large functions with 1 changed line show all 50+ lines as equally important

**Desired Behavior:**
When a single line changes in a 50-line function:
1. **Context Expansion**: Expand the change boundary to the entire parent function
2. **Relevance Classification**:
   - Changed line + function signature + parent blocks: ESSENTIAL (always visible)
   - Import statements in function scope: BACKGROUND (foldable)
   - Comments, docstrings: NOISE (foldable)
3. **Folding**: UI can hide ~30-40 lines of BACKGROUND/NOISE while keeping structural understanding intact

**Success Criteria:**
- Given a 1-line change in a 50-line function, the system:
  - Expands boundary to include the entire function
  - Marks changed line + ancestors as ESSENTIAL
  - Marks imports/siblings as BACKGROUND
  - Marks comments as NOISE
  - Enables 60-80% of lines to be hidden when folding is enabled
  - Never folds lines that contain actual changes

---

## Architecture Summary

### Data Flow Pipeline

```
ASTChange → ChangeWithContext → ReviewableDiff → RenderableDiff
    ↓              ↓                   ↓               ↓
(detection)  (enrichment)        (structure)      (display)
```

### Key Types

**ASTChange** (`diffviz-core/src/ast_diff/changes.rs`):
- Atomic unit of change (Addition, Deletion, ContentChange, StructuralChange, KindChange, Reorder)
- `primary_node()` method returns the primary TreeSitter node

**ChangeWithContext** (`diffviz-core/src/ast_diff/changes.rs:65-74`):
- Enriched change with semantic context
- `original_changes: Vec<ASTChange>` - Multiple changes can share a boundary
- `context_boundary: NodeRef` - The boundary node defining scope
- `context_tree: ContextNode` - Tree with relevance scores

**ContextNode** (`diffviz-core/src/ast_diff/changes.rs:76-100`):
- Node in context tree with relevance classification
- `node: NodeRef` - TreeSitter node reference
- `relevance: RelevanceScore` - 0-3 (ESSENTIAL to NOISE)
- `children: Vec<ContextNode>` - Recursive structure

**Relevance Scoring** (`diffviz-core/src/ast_diff/changes.rs:8-12`):
- `ESSENTIAL = 0` - Contains or is the actual change
- `IMPORTANT = 1` - Direct semantic container of change
- `BACKGROUND = 2` - Sibling context (collapsible in UI)
- `NOISE = 3` - Unrelated context (hideable in UI)

### TreeSitter Navigation

Key methods available on `tree_sitter::Node`:
- `parent()` - Get parent node (returns `Option<Node>`)
- `children(&mut cursor)` - Iterator over children
- `next_sibling()` / `prev_sibling()` - Navigate siblings
- `kind()` - Node type string (e.g., "function_item", "impl_item")
- `start_byte()` / `end_byte()` - Byte positions

### LanguageParser Trait

Provides language-specific context boundary selection:

**`get_context_boundaries()`** (`diffviz-core/src/common.rs:168-216`):
- Returns priority-ordered list of semantic kinds for boundaries
- Example for Content changes: `[Function, Class, SourceFile]`
- First matching parent becomes the context boundary

**`classify_node_kind()`** (`diffviz-core/src/common.rs:165-166`):
- Maps TreeSitter node kinds to SemanticNodeKind enum
- Example: "function_item" → `SemanticNodeKind::Function`

**`classify_leaf_relevance()`** (`diffviz-core/src/common.rs:218-248`):
- Default relevance for node types
- Function/Class/Struct → ESSENTIAL
- Module/Import → BACKGROUND
- Comment/Statement → NOISE

### Current Implementation Gap

Location: `diffviz-core/src/reviewable_diff.rs:288-319`

```rust
pub fn expand_changes_to_reviewable_diffs<'source>(...) -> Vec<ReviewableDiff> {
    let reviewable_diffs: Vec<_> = changes
        .iter()
        .map(|change| {
            // Creates SIMPLE ChangeWithContext - NO ACTUAL EXPANSION
            let change_with_context = ChangeWithContext {
                original_changes: vec![change.clone()],
                context_boundary: *change.primary_node(),  // Just uses change node!
                context_tree: ContextNode::new(*change.primary_node(), ESSENTIAL),  // Single node!
            };
            // Convert to ReviewableDiff...
        })
        .collect();
    reviewable_diffs
}
```

**This is where context expansion needs to be implemented.**

### Folding Infrastructure (Already Working)

Location: `diffviz-core/src/renderable_diff/mod.rs:320-322`

```rust
pub fn should_fold(&self) -> bool {
    self.max_relevance() >= BACKGROUND && !self.has_changes()
}
```

The folding logic already works correctly - it just needs proper input (ContextNode trees with varied relevance scores).

---

## Technical Research Findings

### TreeSitter Parent Navigation Pattern

From `diffviz-core/examples/treesitter_ast_explorer.rs`, the standard pattern for walking up the AST:

```rust
fn walk_children(node: Node, source: &str, depth: usize) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        // Process each child
        let text = child.utf8_text(source.as_bytes()).unwrap_or("<error>");
        println!("{}├─ {} → \"{}\"", indent, child.kind(), preview);
    }
}
```

Key insights:
- Always use a cursor for child iteration
- `node.kind()` returns the TreeSitter node type
- Parent navigation: `if let Some(parent) = node.parent() { ... }`

### Boundary Detection Strategy

From `diffviz-core/src/common.rs`, each language parser defines context boundaries:

**Rust example**:
- Content changes: Look for Function → Struct → Module → SourceFile
- Structural changes: Look for Module → Struct → SourceFile
- First matching parent becomes the boundary

**TypeScript example**:
- Content changes: Look for Function → Class → Module → SourceFile
- Structural changes: Look for Module → Class → SourceFile

### Relevance Classification Strategy

Two-phase approach:
1. **Default relevance**: Based on node semantic kind (`classify_leaf_relevance()`)
2. **Override for change path**: Nodes containing changes get ESSENTIAL

---

## Code Context

### Files to Modify

**Primary implementation file**:
- `diffviz-core/src/reviewable_diff.rs:288-319` - `expand_changes_to_reviewable_diffs()`

**Supporting files** (context only, no changes):
- `diffviz-core/src/ast_diff/changes.rs` - ContextNode, ChangeWithContext
- `diffviz-core/src/common.rs` - LanguageParser trait methods
- `diffviz-core/src/parsers/rust.rs` - Example language parser

### Test Files to Modify

**Existing fixtures to enhance**:
- `diffviz-review/tests/fixtures/rust_trait_impl.json` - Expand to 50+ lines
- `diffviz-review/tests/fixtures/typescript_react_component.json` - Add realistic imports/comments

**Integration test location**:
- `diffviz-core/tests/` - Add context expansion verification tests

### Examples for Reference

- `diffviz-core/examples/boundary_merging_demo.rs` - Shows multi-change handling
- `diffviz-core/examples/treesitter_ast_explorer.rs` - TreeSitter navigation patterns

---

## Constraints and Requirements

### Mandatory Constraints (from CLAUDE.md)

1. **Tree-sitter Only**: No string/regex operations for code analysis
2. **Fail Fast**: No defensive programming or fallbacks
3. **Zero Warnings**: All compiler and clippy warnings must be resolved
4. **TDD for diffviz-core**: Must run full test suite after changes
5. **No Backward Compatibility**: Not needed unless explicitly requested

### Performance Considerations

- Merkle tree hashing already optimized (O(log n) comparison)
- Context expansion should be O(h) where h = tree height
- Avoid redundant AST traversals (single upward walk per change)

### Testing Requirements

From CLAUDE.md:
> When asked to debug an issue occurring in this crate, your first mission is to understand why the test suite did not capture the regression already and figure out a way to update it to capture the regression.

Must update test suite to verify:
- Context boundaries are correctly identified
- Relevance scores are properly assigned
- Folding behavior works end-to-end
- Multi-change boundary merging still functions

---

## Development Strategy: Core-then-Integrate

### Why This Strategy?

**Core-then-Integrate** chosen because:
- Complex algorithm requiring focused development
- Clear interface (LanguageParser trait provides all needed hooks)
- Can test algorithm in isolation before wiring into pipeline
- Aligns with clean architecture (pure function for core logic)

### Implementation Approach

1. **Build Core Algorithm** (Pure function, no side effects)
   - Create context expansion algorithm in separate function
   - Unit tests with synthetic TreeSitter nodes
   - Focus: Correct boundary detection and relevance classification

2. **Wire into Pipeline** (Integration point)
   - Replace trivial logic in `expand_changes_to_reviewable_diffs()`
   - Minimal changes to existing code
   - Preserve multi-change merging behavior

3. **Integration Testing** (End-to-end verification)
   - Enhance existing fixtures to exercise folding
   - Add dedicated context expansion tests
   - Verify ReviewableDiff → RenderableDiff pipeline

4. **TUI Validation** (Visual confirmation)
   - Test with enhanced fixtures in diffviz-review-tui
   - Verify folding works with Space+t+c toggle
   - Confirm visual rendering correctness

---

## Implementation Scope

### Deliverable 1: Context Expansion + TUI Validation

**In Scope:**
- Implement context expansion algorithm
- Wire into `expand_changes_to_reviewable_diffs()`
- Enhance 2-3 existing fixtures to test folding
- Add integration tests for context expansion
- Verify folding works in TUI

**Out of Scope:**
- Creating brand new fixture infrastructure
- Performance optimization (unless critical)
- Edge case handling beyond core languages (Rust, TypeScript, Python)

### Success Metrics

1. **Functional**: Context expansion creates rich ContextNode trees with varied relevance
2. **Testable**: Test suite verifies boundary detection and relevance scoring
3. **Visual**: TUI shows foldable content for enhanced fixtures
4. **Quality**: Zero compiler warnings, all tests pass

---

## Next Steps

After consulting with user on implementation strategy:
1. Review and approve this context document
2. Create implementation roadmap with detailed phases
3. Begin implementation following Core-then-Integrate approach
