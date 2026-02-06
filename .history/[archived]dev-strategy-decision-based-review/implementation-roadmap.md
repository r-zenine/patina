# Implementation Roadmap: Decision-Based Review Pipeline

**Strategy**: Core-then-Integrate

**Status**: Phase 5 Complete ✅

---

## Phase 1: Core Module (diffviz-core) ✅ COMPLETE

Build `decision_based_diff.rs` with pure business logic and comprehensive tests.

### 1.1 Semantic unit lookup by range
- Given a `SemanticTree` and a target line range, find the smallest semantic unit that fully contains that range
- If the range spans multiple units, return the parent unit containing all of them
- Expand the returned range to cover the complete unit boundaries (start of first line to end of last line)
- Test: fixture with known unit boundaries, verify correct unit is found for various ranges (exact, partial, spanning)

### 1.2 Semantic unit lookup by name
- Given a `SemanticTree` and a unit name + type, find the matching unit
- Simple linear scan of `tree.all_units()` comparing name text and unit type
- Return `Option<&SemanticNode>` — None means the unit doesn't exist in this tree
- Test: find existing unit, miss for renamed unit, miss for deleted unit

### 1.3 Change classification
- Define a `ChangeClassification` enum: Addition, Deletion, Modification
- Given: new file semantic tree, old file semantic tree (optional), target range from CodeImpact
  - Find unit at range in new tree (step 1.1)
  - If old file doesn't exist → Addition
  - Look up same-named unit in old tree (step 1.2)
  - If not found in old → Addition
  - If found in old → Modification
- Handle edge case: file existed before but unit is new (addition within existing file)
- Test: each classification path with real fixture data

### 1.4 ReviewableDiff construction
- For **Addition**: build DiffNode tree from new unit only, all nodes marked Added
- For **Deletion**: build DiffNode tree from old unit only, all nodes marked Deleted
- For **Modification**: build DiffNode tree spanning both old and new, nodes marked appropriately
- Reuse `build_child_nodes_with_context()` pattern for relevance assignment
- Reuse existing `DiffNode`, `DiffMetadata`, `SourceProvider` types
- Test: verify produced ReviewableDiff has correct structure, can convert to RenderableDiff

### 1.5 Public API
- Single entry point function:
  ```
  fn create_reviewable_diff_from_range(
      file_path: &str,
      target_range: (usize, usize),  // start_line, end_line from CodeImpact
      old_source: Option<&str>,       // None if file is new
      new_source: &str,
      language: ProgrammingLanguage,
      parser: &dyn LanguageParser,
  ) -> Result<ReviewableDiff, ...>
  ```
- Orchestrates steps 1.1 through 1.4
- Test: end-to-end with the existing calculator.rs fixture data

---

## Phase 2: Integration (diffviz-review) ✅ COMPLETE

Wire the new module into ReviewEngineBuilder.

### 2.1 New build path in ReviewEngineBuilder
- Add method that accepts decisions as input instead of discovering diffs from git
- For each Decision → for each CodeImpact:
  - Fetch old/new source via DiffProvider
  - Call `create_reviewable_diff_from_range()` from diffviz-core
  - Wrap result in review-layer ReviewableDiff with ReviewableDiffId
- Decision-to-diff relationship is now implicit (we know which decision created which diff)

### 2.2 Decision index simplification
- Since decisions drive creation, the reverse index can be built at creation time
- No need for `build_index_from_review_state()` overlap detection
- Each ReviewableDiffId is directly associated with its source decision number

### 2.3 Remove unmapped decision logic
- Remove `create_unmapped_decision()` calls
- Remove Decision 0 concept
- If a file change isn't in a decision, it's not reviewed

### 2.4 Update MockDiffProvider / test harness
- Ensure the TUI's `create_test_review_engine()` in main.rs works with the new path
- Decisions are already hardcoded there — they become the primary input
- Fixture files still provide old/new content via MockDiffProvider

---

## Phase 3: TUI Validation ✅ COMPLETE

Use the diffviz-tui-contribution skill's test harness to verify the TUI works end-to-end.

### 3.1 Test harness verification
- Run `--test-input` and `--test-full` sequences to validate:
  - Decision tree navigation still works
  - File expansion shows chunks under each file
  - Chunk selection displays actual diff content
  - Approval workflows function correctly

### 3.2 Visual inspection
- Verify calculator.rs, api.ts, Greeting.tsx, fetcher.py, client.rs, reader.rs all show code
- Verify diff rendering (added/deleted/modified lines) looks correct
- Verify status bar counts are accurate

### 3.3 Regression check
- Run existing TUI test suite: `cargo test --package diffviz-review-tui --features test-harness`
- Run decision approval tests: `cargo test --test decision_approval_tests`

---

## Phase 4: Remove Old Path ⏳ NEXT

Delete `build_semantic_pairs()` and its direct callers.

### 4.1 Remove pairing entry points
- Delete `build_semantic_pairs()` from `semantic_ast.rs`
- Delete `build_semantic_pairs_with_coverage()` from `semantic_ast.rs`
- Delete helper functions: `mark_node_and_children_as_used()`, `should_mark_children_as_used()`

### 4.2 Remove semantic-to-reviewable bridge
- Delete `reviewable_diff_from_semantic.rs` (or gut it, keeping only reusable helpers if any were extracted)
- Delete `semantic_pairs_to_reviewable_diffs()` and its helper functions

### 4.3 Remove old builder path
- Remove `create_semantic_reviewable_diffs()` from ReviewEngineBuilder
- Remove `create_semantic_reviewable_diffs_for_added_file()`
- Remove the old `build()` method if fully replaced

### 4.4 Verify compilation and tests
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace`
- Fix any breakage from removed code

---

## Phase 5: Cleanup ✅ COMPLETE

Sweep all remaining dead code that supported semantic pairing.

### 5.1 Remove unused types
- `SemanticPair` enum (if no longer referenced)
- `SemanticSimilarity` struct
- `CoverageStats` struct
- Any test utilities specific to pairing

### 5.2 Remove unused trait methods
- `LanguageParser::compare_semantic_units()` — only used by pairing
- Remove from trait definition and all language parser implementations

### 5.3 Remove deprecated modules
- Delete `semantic_unit_partitioner.rs` (already deprecated)

### 5.4 Remove unused decision infrastructure
- `build_index_from_review_state()` (if fully replaced by creation-time association)
- `create_unmapped_decision()`
- `ranges_overlap()` helper (if no longer needed)

### 5.5 Clean up tests
- Remove pairing-specific test files
- Remove fixture tests that only validated pairing behavior
- Update `bugs.md` to reflect that pairing-related bugs are no longer applicable

### 5.6 Final verification
- `cargo fmt --all`
- `cargo clippy --workspace` — zero warnings
- `cargo test --workspace` — all pass
- `cargo check --workspace` — clean compilation
- TUI test harness validation one final time
