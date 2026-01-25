# Code Context

This document provides line-specific references to the codebase for context expansion and folding implementation.

---

## Primary Implementation File

### `diffviz-core/src/reviewable_diff.rs`

**Function to replace**: `expand_changes_to_reviewable_diffs()` (lines 288-319)

Current trivial implementation:
```rust
// Line 288-319
pub fn expand_changes_to_reviewable_diffs<'source>(
    changes: &[ASTChange<'source>],
    parser: &dyn crate::common::LanguageParser,
    old_source: &'source dyn SourceProvider,
    new_source: &'source dyn SourceProvider,
    language: ProgrammingLanguage,
) -> Vec<ReviewableDiff> {
    let reviewable_diffs: Vec<_> = changes
        .iter()
        .map(|change| {
            // Creates SIMPLE ChangeWithContext - NO ACTUAL EXPANSION
            let change_with_context = ChangeWithContext {
                original_changes: vec![change.clone()],
                context_boundary: *change.primary_node(),  // Just uses change node!
                context_tree: ContextNode::new(*change.primary_node(), ESSENTIAL),  // Single node!
            };
            ReviewableDiff::from_change_with_context(
                change_with_context,
                language,
                old_source,
                new_source,
                parser,
                start_time,
            )
        })
        .collect();
    reviewable_diffs
}
```

**Conversion logic**: `convert_context_node_to_diff_node()` (lines 126-158)
- This function already handles ContextNode → DiffNode conversion
- Correctly overrides relevance to ESSENTIAL for nodes with changes
- Should NOT need modification

---

## Core Data Structures

### `diffviz-core/src/ast_diff/changes.rs`

**ASTChange enum** (lines 14-38):
```rust
pub enum ASTChange<'result> {
    Addition(NodeRef<'result>),
    Deletion(NodeRef<'result>),
    ContentChange { old: NodeRef, new: NodeRef },
    StructuralChange { old: NodeRef, new: NodeRef },
    KindChange { old: NodeRef, new: NodeRef },
    Reorder { parent: NodeRef },
}
```

**Key method**: `primary_node()` (lines 52-62)
- Returns the primary node for context boundary detection
- Uses new version for modifications

**ChangeWithContext struct** (lines 65-74):
```rust
pub struct ChangeWithContext<'source> {
    pub original_changes: Vec<ASTChange<'source>>,  // Multiple changes can share a boundary
    pub context_boundary: NodeRef<'source>,          // The boundary node defining scope
    pub context_tree: ContextNode<'source>,          // Tree with relevance scores
}
```

**ContextNode struct** (lines 76-100):
```rust
pub struct ContextNode<'source> {
    pub node: NodeRef<'source>,              // TreeSitter node reference
    pub relevance: RelevanceScore,           // 0-3 (ESSENTIAL to NOISE)
    pub children: Vec<ContextNode<'source>>, // Recursive structure
}

impl<'source> ContextNode<'source> {
    pub fn new(node: NodeRef<'source>, relevance: RelevanceScore) -> Self;
    pub fn add_child(&mut self, child: ContextNode<'source>);
}
```

**Relevance scoring constants** (lines 8-12):
```rust
pub const ESSENTIAL: RelevanceScore = 0;   // Contains or is the actual change
pub const IMPORTANT: RelevanceScore = 1;   // Direct semantic container of change
pub const BACKGROUND: RelevanceScore = 2;  // Sibling context (collapsible in UI)
pub const NOISE: RelevanceScore = 3;       // Unrelated context (hideable in UI)
```

---

## LanguageParser Trait Interface

### `diffviz-core/src/common.rs`

**Trait definition** (lines 118-251):

**Key method 1**: `classify_node_kind()` (lines 165-166)
```rust
fn classify_node_kind(&self, node_kind: &str) -> SemanticNodeKind;
```
- Maps TreeSitter node kinds to semantic kinds
- Example: "function_item" → `SemanticNodeKind::Function`

**Key method 2**: `get_context_boundaries()` (lines 168-216)
```rust
fn get_context_boundaries(
    &self,
    change_type: &ASTChangeType,
    _change_node_kind: &SemanticNodeKind,
) -> Vec<SemanticNodeKind>;
```
- Returns priority-ordered list of boundary types
- Example for Content changes: `[Function, Class, SourceFile]`
- First matching parent becomes the boundary

**Key method 3**: `classify_leaf_relevance()` (lines 218-248)
```rust
fn classify_leaf_relevance(&self, node_kind: &SemanticNodeKind) -> RelevanceScore;
```
- Default relevance for node types
- Function/Class/Struct → ESSENTIAL
- Module/Import → BACKGROUND
- Comment/Statement → NOISE

**SemanticNodeKind enum** (lines 35-67):
```rust
pub enum SemanticNodeKind {
    Function,
    Class,
    Struct,
    Enum,
    Interface,
    ImplBlock,
    Module,
    Import,
    Variable,
    Statement,
    Expression,
    TypeDefinition,
    Comment,
    SourceFile,
    Other(String),
}
```

---

## Language Parser Implementations

### `diffviz-core/src/parsers/rust.rs`

**Context boundaries for Rust** (lines 177-206):
```rust
fn get_context_boundaries(
    &self,
    change_type: &ASTChangeType,
    _change_node_kind: &SemanticNodeKind,
) -> Vec<SemanticNodeKind> {
    match change_type {
        ASTChangeType::Content => vec![
            SemanticNodeKind::Function,
            SemanticNodeKind::Struct,
            SemanticNodeKind::Module,
            SemanticNodeKind::SourceFile,
        ],
        ASTChangeType::Structural => vec![
            SemanticNodeKind::Module,
            SemanticNodeKind::Struct,
            SemanticNodeKind::SourceFile,
        ],
        // ... other change types
    }
}
```

**Node kind classification** (lines 208-244):
```rust
fn classify_node_kind(&self, node_kind: &str) -> SemanticNodeKind {
    match node_kind {
        "function_item" => SemanticNodeKind::Function,
        "impl_item" => SemanticNodeKind::ImplBlock,
        "struct_item" => SemanticNodeKind::Struct,
        "mod_item" => SemanticNodeKind::Module,
        "use_declaration" => SemanticNodeKind::Import,
        "line_comment" | "block_comment" => SemanticNodeKind::Comment,
        // ... many more mappings
        _ => SemanticNodeKind::Other(node_kind.to_string()),
    }
}
```

**Relevance classification** (lines 246-279):
```rust
fn classify_leaf_relevance(&self, node_kind: &SemanticNodeKind) -> RelevanceScore {
    match node_kind {
        SemanticNodeKind::Function | SemanticNodeKind::Struct => ESSENTIAL,
        SemanticNodeKind::ImplBlock | SemanticNodeKind::TypeDefinition => IMPORTANT,
        SemanticNodeKind::Module | SemanticNodeKind::Import => BACKGROUND,
        SemanticNodeKind::Comment | SemanticNodeKind::Statement => NOISE,
        // ... other kinds
        _ => BACKGROUND,
    }
}
```

### `diffviz-core/src/parsers/typescript.rs`

Similar structure to Rust parser. Key differences in boundaries:
- Uses `Class`, `Interface` instead of `Struct`, `ImplBlock`
- Different TreeSitter node kind mappings

---

## Folding Infrastructure (Already Working)

### `diffviz-core/src/renderable_diff/mod.rs`

**RenderableLine struct** (lines 254-281):
```rust
pub struct RenderableLine<'source> {
    pub line_number: usize,
    pub content: &'source str,
    pub byte_range: (usize, usize),
    pub annotations: Vec<LineAnnotation>,
    pub semantic_anchor: Option<SemanticAnchor>,
}
```

**Folding logic** (lines 320-322):
```rust
pub fn should_fold(&self) -> bool {
    self.max_relevance() >= BACKGROUND && !self.has_changes()
}
```

**Helper methods**:
- `max_relevance()` - Returns highest (worst) relevance score from annotations
- `has_changes()` - Returns true if line contains any ChangeType annotation

---

## TreeSitter Node API

### `diffviz-core/src/ast_diff/nodes.rs`

**NodeRef wrapper** (lines 8-29):
```rust
pub struct NodeRef<'tree> {
    pub node: Node<'tree>,  // TreeSitter's native node
}
```

**Key TreeSitter Node methods** (from `tree_sitter` crate):
- `parent() -> Option<Node>` - Get parent node
- `children(&mut cursor) -> impl Iterator<Item = Node>` - Iterate children
- `kind() -> &str` - Node type string
- `start_byte() -> usize` - Start position in bytes
- `end_byte() -> usize` - End position in bytes
- `utf8_text(source: &[u8]) -> Result<&str>` - Get source text

**Navigation pattern** (from examples):
```rust
let mut cursor = node.walk();
for child in node.children(&mut cursor) {
    // Process each child
}
```

---

## Test Files

### Existing Fixtures to Enhance

**`diffviz-review/tests/fixtures/rust_trait_impl.json`**:
- Current: 11 lines (old), 20 lines (new)
- Target: 50+ lines with imports, comments, multiple methods
- Should exercise: Function-level folding, import classification

**`diffviz-review/tests/fixtures/typescript_react_component.json`**:
- Current: 10 lines (old), 16 lines (new)
- Target: 50+ lines with imports, type definitions, JSX
- Should exercise: Component-level folding, import/type classification

### Integration Test Location

**New file**: `diffviz-core/tests/context_expansion_tests.rs`
- Verify context boundary detection
- Verify relevance score assignment
- Verify ContextNode tree structure
- Verify multi-change merging still works

---

## Reference Examples

### `diffviz-core/examples/boundary_merging_demo.rs`

Shows complete pipeline:
1. Parse AST trees (lines 92-106)
2. Detect changes (lines 108-118)
3. Expand to ReviewableDiffs (lines 120-126)
4. Convert to RenderableDiffs (lines 128-133)

**Key insight**: Shows multi-change handling within same function

### `diffviz-core/examples/treesitter_ast_explorer.rs`

Shows TreeSitter navigation patterns:
- Walking children with cursor (lines 76-99)
- Getting node text (line 81)
- Node kind inspection (line 91)

**Key insight**: Standard pattern for AST traversal

---

## Summary of Code Locations

| Component | File | Lines |
|-----------|------|-------|
| Function to modify | `diffviz-core/src/reviewable_diff.rs` | 288-319 |
| ContextNode struct | `diffviz-core/src/ast_diff/changes.rs` | 76-100 |
| ChangeWithContext struct | `diffviz-core/src/ast_diff/changes.rs` | 65-74 |
| Relevance constants | `diffviz-core/src/ast_diff/changes.rs` | 8-12 |
| LanguageParser trait | `diffviz-core/src/common.rs` | 168-248 |
| Rust parser implementation | `diffviz-core/src/parsers/rust.rs` | 177-279 |
| TypeScript parser implementation | `diffviz-core/src/parsers/typescript.rs` | Similar structure |
| Folding logic | `diffviz-core/src/renderable_diff/mod.rs` | 320-322 |
| TreeSitter examples | `diffviz-core/examples/` | Multiple files |
