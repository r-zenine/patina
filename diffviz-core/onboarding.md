# diffviz-core - Orientation Guide

Updated: 2026-01-31 - Added comprehensive pipeline documentation and recent parent-child deletion overlap fix

## What This Module Does
Transforms raw source code diffs into semantic, reviewable changes through Tree-sitter AST analysis and intelligent pairing algorithms.

## Before You Code Here
**Existing Patterns:**
- **Fail-fast semantic pairing**: `build_semantic_pairs()` uses exhaustive coverage tracking to ensure ALL nodes are accounted for (no silent failures)
- **Child marking after parent operations**: When a parent node is marked as deletion/addition, its children MUST be marked to prevent duplicate pairs
- **Special handling for full-file modules**: Root modules at byte 0 allow children to create their own pairs, preventing overly broad diffs

**Reusable DTOs/Types:**
- `SemanticPair`: The pairing result enum (Matched/Addition/Deletion) - don't create custom pair types
- `SemanticNode`: Universal semantic AST node - reuse for all languages, don't create language-specific variants
- `ReviewableDiff`: Self-contained diff boundary with metadata - don't scatter diff data across multiple structures
- `RenderableDiff`: Line-oriented display format - use for all rendering, don't bypass this layer

**Integration Points:**
- Language parsers implement `LanguageParser` trait (`parsers/rust.rs`, etc.) for semantic tree building
- `semantic_pairs_to_reviewable_diffs()` bridges semantic analysis to review layer
- `RenderableDiff::from()` converts tree-based diffs to line-based display

## The Complete Pipeline: Source Code → ReviewableDiff → RenderableDiff

### Phase 1: Raw Code → Tree-sitter AST
**Entry Point:** Language-specific parser (e.g., `RustParser::try_parse()`)
- Parses source code into Tree-sitter AST using language grammar
- Returns `tree_sitter::Tree` with full parse information
- **Location:** `src/parsers/{language}.rs`, trait method `LanguageParser::try_parse()`

### Phase 2: Tree-sitter AST → SemanticTree
**Entry Point:** `LanguageParser::build_semantic_tree()`
- Transforms low-level Tree-sitter nodes into meaningful semantic constructs
- Each language parser walks the AST and creates `SemanticNode` instances
- Filters trivial syntax tokens (punctuation, keywords) to focus on semantic units
- Captures metadata nodes (attributes, decorators) and associates them with their targets
- **Key Types:**
  - `SemanticTree`: Root container with language and source ranges
  - `SemanticNode`: Universal semantic construct with 5 unit types (DataStructure, Callable, Variable, Import, Module)
  - `SemanticUnitType`: Enum discriminating semantic categories with rich metadata
- **Location:** `src/parsers/rust.rs::build_semantic_node()` (example implementation)
- **Output:** Complete semantic tree ensuring exhaustive source coverage

### Phase 3: SemanticTree Pair → SemanticPairs
**Entry Point:** `build_semantic_pairs()` or `build_semantic_pairs_with_coverage()`
- Compares old and new semantic trees to identify meaningful changes
- **Algorithm (3 phases):**
  1. **Exact Matching**: Pair nodes by name and type using `can_pair_with()`
  2. **Parent-Child Marking**: After creating deletion/addition pairs, mark children as used via `mark_node_and_children_as_used()` to prevent duplicate pairs (CRITICAL FIX from 2026-01-31)
  3. **Orphan Processing**: Remaining nodes become `SemanticPair::Addition` or `SemanticPair::Deletion`
- **Special Cases:**
  - Full-file modules (byte 0): Children get their own pairs via `should_mark_children_as_used()`
  - Import units: Optimized with `is_semantically_identical()` check before expensive comparison
- **Location:** `src/semantic_ast.rs::build_semantic_pairs()` (lines 928-1020)
- **Output:** Vector of `SemanticPair` representing all changes

### Phase 4: SemanticPairs → ReviewableDiffs
**Entry Point:** `semantic_pairs_to_reviewable_diffs()`
- Converts semantic pairs into self-contained diff boundaries
- Filters out full-file module pairs via `should_create_diff_for_pair()`
- **Key Functions:**
  - `create_matched_diff()`: Handles modified semantic units with similarity analysis
  - `create_addition_diff()`: Handles newly added units
  - `create_deletion_diff()`: Handles removed units
  - `build_child_nodes_with_context()`: Walks Tree-sitter children and assigns relevance scores for folding
- **Relevance Assignment:**
  - DataStructure/Callable/Module: ESSENTIAL
  - Variable/Import: IMPORTANT
  - Unknown nodes: BACKGROUND or NOISE (if error nodes)
- **Location:** `src/reviewable_diff_from_semantic.rs::semantic_pairs_to_reviewable_diffs()` (lines 21-49)
- **Output:** Vector of `ReviewableDiff` with DiffNode trees and metadata

### Phase 5: ReviewableDiff Structure
**Container:** `ReviewableDiff` (src/reviewable_diff.rs)
- **Fields:**
  - `language`: Programming language enum
  - `boundary`: Root `DiffNode` representing the changed semantic unit
  - `old_source`/`new_source`: Boxed source providers for lazy text access
  - `metadata`: Statistics (total_changes, change_summary, essential_node_count, analysis_duration)
- **DiffNode Hierarchy:**
  - Preserves AST structure as tree of `DiffNode` instances
  - Each node has: `node_type`, `semantic_kind`, `change_status`, `relevance`, `children`
  - `NodeChangeStatus` enum: Unchanged/Added/Deleted/Modified/Moved/Reordered
  - Enables context-aware folding via relevance scores

### Phase 6: ReviewableDiff → RenderableDiff
**Entry Point:** `RenderableDiff::from(&ReviewableDiff)`
- Converts tree-based diffs to line-oriented display format
- **For Modified Changes:**
  - Uses Myers diff algorithm via `create_line_by_line_diff_for_modified()`
  - Extracts old/new text and splits into lines
  - Applies semantic anchors (function signatures, variable assignments, etc.)
  - Produces `DiffOp` sequence (Keep/Delete/Add/Modify)
  - Each line gets relevance score from byte range annotations
- **For Other Changes (Added/Deleted):**
  - Uses `create_single_source_lines()` for direct line extraction
- **Output:** `RenderableDiff` with vector of `RenderableLine` instances
- **Location:** `src/renderable_diff/mod.rs::From<&ReviewableDiff>` (lines 375-426)

### Phase 7: RenderableDiff Structure
**Container:** `RenderableDiff` (src/renderable_diff/mod.rs)
- **Fields:**
  - `lines`: Vector of `RenderableLine` for display
  - `metadata`: Simplified rendering metadata with boundary name and line ranges
  - `language`: Programming language for syntax highlighting
- **RenderableLine:**
  - `line_number`: Sequential line number for display
  - `content`: Actual line text with proper lifetime
  - `byte_range`: Position in source (for highlighting)
  - `annotations`: Vector of `LineAnnotation` with relevance and change type
  - `semantic_anchor`: Optional anchor for navigation (function name, variable, etc.)
- **Purpose:** Ready for TUI/CLI rendering with folding, syntax highlighting, and navigation

## Key Abstractions to Reuse

### 1. SemanticPair - Don't Reinvent Pairing Logic
```rust
pub enum SemanticPair<'a> {
    Matched { old_unit, new_unit, similarity },
    Addition { unit },
    Deletion { unit },
}
```
**Reuse for:** Any old/new comparison needs. Don't create custom Result/Change/Diff enums.

### 2. SemanticNode - Universal Semantic Construct
```rust
pub struct SemanticNode<'a> {
    tree_sitter_node: Node<'a>,
    metadata_nodes: Vec<MetadataNode<'a>>,  // Attributes/decorators
    children: Vec<SemanticNode<'a>>,
    name_node: Option<Node<'a>>,
    unit_type: SemanticUnitType<'a>,
}
```
**Reuse for:** All semantic analysis across languages. Language-specific details go in `SemanticUnitType` metadata, not new types.

### 3. ReviewableDiff - Self-Contained Diff Boundary
```rust
pub struct ReviewableDiff {
    language: ProgrammingLanguage,
    boundary: DiffNode,                    // Root of context tree
    old_source: Box<dyn SourceProvider>,   // Lazy source access
    new_source: Box<dyn SourceProvider>,
    metadata: DiffMetadata,
}
```
**Reuse for:** All diff rendering pipelines. Don't scatter diff data across multiple structures.

### 4. Context Expansion with Relevance Scores
**Pattern:** `build_child_nodes_with_context()` walks Tree-sitter children and assigns:
- ESSENTIAL: Core changes (DataStructure, Callable)
- IMPORTANT: Supporting elements (Variable, Import)
- BACKGROUND: Context (Unknown nodes)
- NOISE: Error nodes
**Reuse for:** Any context tree building. Enables intelligent folding in UIs.

## Architectural Constraints

### 1. Tree-sitter Only - NO String/Regex Operations
All code analysis MUST use Tree-sitter AST traversal. String operations forbidden in semantic analysis.
**Rationale:** Ensures language-agnostic correctness and handles edge cases (comments, string literals, complex syntax).

### 2. Exhaustive Coverage - No Silent Node Loss
Both `build_semantic_pairs()` variants ensure ALL nodes are accounted for:
```rust
assert_eq!(
    stats.matched_pairs + stats.deletions,
    old_units.len(),
    "Not all old nodes covered!"
);
```
**Rationale:** Prevents bugs where changes are silently dropped. Every node becomes a pair.

### 3. Parent-Child Marking Pattern (CRITICAL)
After creating deletion/addition pairs in Phase 2, MUST mark children:
```rust
if should_mark_children_as_used(unit) {
    mark_node_and_children_as_used(unit, &units, &mut used_flags);
}
```
**Rationale:** Prevents duplicate pairs for parent + all children. Fixed bug where class deletion created N+1 pairs (class + each method).

### 4. Fail-Fast - No Defensive Programming
Errors should panic or return `Result::Err`, not fall back to degraded behavior.
**Example:** If semantic tree building fails, return `SemanticError::TreeBuildError`, don't create partial trees.

## Directory Map
```
diffviz-core/src/
├── semantic_ast.rs              # Phase 3: Semantic pairing algorithm
├── reviewable_diff_from_semantic.rs  # Phase 4: Semantic→Reviewable conversion
├── semantic_unit_partitioner.rs # DEPRECATED: Old pairing approach
├── reviewable_diff.rs           # Phase 5: ReviewableDiff structure
├── renderable_diff/
│   ├── mod.rs                   # Phase 6-7: Renderable conversion
│   ├── myers_diff.rs            # Line-by-line diff algorithm
│   ├── semantic_anchors.rs      # Navigation anchor extraction
│   └── line_utils.rs            # Line extraction utilities
├── parsers/
│   ├── rust.rs                  # Phase 1-2: Rust semantic tree building
│   ├── python.rs                # Phase 1-2: Python semantic tree building
│   ├── typescript.rs            # Phase 1-2: TypeScript semantic tree building
│   └── ...                      # Other language parsers
├── ast_diff/                    # Low-level AST diffing (Merkle trees)
└── common.rs                    # Shared traits and types
```

## Recent Changes (2026-01-31)

### Bug Fix: Parent-Child Deletion Overlap
**Problem:** When a parent AST node was deleted (e.g., class declaration), the algorithm created BOTH:
1. Parent node deletion as a semantic pair
2. All child deletions (methods) as individual semantic pairs

This caused redundant/overlapping pairs representing the same structural change.

**Solution:** Modified `build_semantic_pairs()` (lines 990-1017) and `build_semantic_pairs_with_coverage()` (lines 1123-1134):
- After creating deletion/addition pairs in Phase 2, call `mark_node_and_children_as_used()`
- Exception: Full-file modules at byte 0 use `should_mark_children_as_used()` to allow children their own pairs
- Children are recursively marked via pointer equality check in `mark_node_and_children_as_used()`

**Impact:**
- Before: Class deletion → N+1 pairs (class + N methods)
- After: Class deletion → 1 pair (class only, children implicitly included)

**Test Coverage:** `tests/bug_parent_child_deletion_overlap.rs` (passing)

### Modified Filter: Full-File Module Pairs
**Change:** `reviewable_diff_from_semantic.rs::should_create_diff_for_pair()` (line 60)
- Updated comment: "Skip full-file module pairs - their unmatched children have their own pairs now"
- Reflects that children of full-file modules now create independent pairs due to special handling in `should_mark_children_as_used()`

## Development Rules

### Zero Warning Rule
After every change:
1. `cargo fmt --all` - Format code
2. `cargo clippy --workspace` - Fix all clippy warnings
3. `cargo check --workspace` - Verify compilation
**NO warnings allowed in commits.**

### Test Suite Strategy
**Comprehensive Coverage:** diffviz-core has extensive test suite (100+ fixtures) that MUST pass on every change.
**Bug Discovery Workflow:**
1. When debugging, first check why test suite didn't catch the regression
2. Add failing test to `tests/` if missing
3. Fix code to pass test
4. Update `bugs.md` with fix details

**Prefer Realistic Fixtures:** When adding tests, prefer `tests/realistic_fixtures` with real-world code over synthetic examples.

### TDD for Bug Fixes
1. Write failing test reproducing the bug
2. Verify test fails
3. Implement fix
4. Verify test passes
5. Check that fix doesn't break other tests

## Performance Characteristics

**Optimized for Large Codebases:**
- **Phase 1-2:** O(n) Tree-sitter parsing and semantic tree building
- **Phase 3:** O(n²) worst-case pairing (O(n) with name-based matching)
- **Phase 4:** O(n) ReviewableDiff conversion with tree walking
- **Phase 6:** O(n) Myers diff for modified nodes
- **Merkle Optimization:** SHA-256 hashing enables O(log n) structural comparison in ast_diff module (not currently used in semantic pipeline)

**Memory Efficiency:**
- Lazy source access via `SourceProvider` trait - text extracted on demand
- Owned node data stores only byte ranges, not full text
- Reference-based Tree-sitter nodes with explicit lifetimes

## Testing Infrastructure

### Test Organization
```
diffviz-core/tests/
├── realistic_fixtures.rs        # Real-world test cases (preferred)
├── bug_parent_child_deletion_overlap.rs  # Recent fix validation
├── fixtures/                    # 100+ structured test fixtures
│   ├── rust/                    # Language-specific fixtures
│   ├── python/
│   ├── typescript/
│   └── ...
└── integration/                 # End-to-end pipeline tests
```

### Running Tests
```bash
# Full test suite (MUST pass before commits)
cargo test --package diffviz-core

# Specific module tests
cargo test --package diffviz-core semantic_ast
cargo test --package diffviz-core reviewable_diff_from_semantic

# Test with output for debugging
cargo test --package diffviz-core -- --nocapture
```

## Examples for Learning the Pipeline

### Core Pipeline Demonstrations
- `examples/semantic_partitioning_demo.rs` - Phase 3 pairing visualization
- `examples/reviewable_diff_demo.rs` - Complete Phase 1-5 pipeline
- `examples/treesitter_ast_explorer.rs` - Phase 1 AST structure analysis

### Language-Specific Examples
- `examples/rust_reviewable_diff_demo.rs` - Rust pipeline end-to-end
- `examples/python_reviewable_diff_demo.rs` - Python pipeline
- `examples/typescript_reviewable_diff_demo.rs` - TypeScript pipeline

### Advanced Features
- `examples/boundary_merging_demo.rs` - Context expansion algorithms
- `examples/renderable_line_range_demo.rs` - Phase 6-7 rendering pipeline

Run with: `cargo run --package diffviz-core --example <name>`

## Common Pitfalls to Avoid

1. **Creating custom pairing logic** - Use `build_semantic_pairs()`, don't reinvent
2. **Forgetting child marking** - After parent deletion/addition, MUST mark children
3. **Ignoring full-file module special case** - Root modules need `should_mark_children_as_used()` check
4. **Bypassing semantic layer** - Don't go directly from Tree-sitter to ReviewableDiff
5. **String-based analysis** - MUST use Tree-sitter, not regex/string matching
6. **Partial semantic trees** - Ensure exhaustive coverage, validate with assertions
7. **Ignoring test failures** - All tests MUST pass, investigate why suite didn't catch bugs
