# Known Bugs - diffviz-core

This file tracks known bugs in the diffviz-core crate. Each bug has a corresponding test case that reproduces the issue.

## Active Bugs

### Bug #1: Phantom changes detected in unchanged functions
- **GitHub Issue**: https://github.com/r-zenine/diffviz/issues/1
- **Test Location**: `tests/bug_issue_1.rs`
- **Description**: DiffViz incorrectly identifies unchanged functions as having changes when only one function in a file is modified. For example, when modifying only `compare_semantic_units`, other functions like `build_impl_items` are incorrectly marked as changed.
- **Status**: Active (test is ignored)
- **Impact**: Creates false positive "phantom changes" that add noise to code reviews

### Bug #2: Python semantic analysis creates excessive false positive boundaries
- **GitHub Issue**: _To be created_
- **Test Location**: `tests/bug_issue_2.rs`
- **Description**: DiffViz Python parser over-decomposes semantic units, creating excessive boundaries with duplicates. Issues include: (1) enum values treated as separate functions and duplicated, (2) individual statements treated as separate boundaries, (3) file modifications treated as delete+add instead of modifications, (4) class definitions not properly grouped semantically.
- **Status**: Active (test is ignored)
- **Impact**: Creates 20+ semantic boundaries for simple changes that should have ~5-6, making reviews overwhelming with false positives
- **Example**: Adding a class and enum to a basic Python script produces 20 boundaries instead of expected 5-6

### Bug #3: Go semantic analysis creates duplicate boundaries and over-decomposition
- **GitHub Issue**: _To be created_
- **Test Location**: `tests/bug_issue_3.rs`
- **Description**: DiffViz Go parser creates duplicate and over-decomposed semantic boundaries. Issues include: (1) package declarations shown as deleted+added instead of unchanged, (2) struct definitions decomposed into multiple separate boundaries (type+struct+name), (3) modules duplicated as separate boundaries, (4) "Unknown" semantic nodes created inappropriately.
- **Status**: Active (test is ignored)
- **Impact**: Creates 8+ semantic boundaries for changes that should have ~5, with duplicate modules and over-decomposed structs
- **Example**: Adding struct and methods to basic Go program produces 8 boundaries instead of expected 5-6

## Fixed Bugs

_No fixed bugs yet - once a bug is fixed, remove the #[ignore] attribute from its test and move this entry to the Fixed Bugs section_

---

## Test Commands

- Run all ignored bug tests: `cargo test --package diffviz-core -- --ignored`
- Run specific bug test: `cargo test --package diffviz-core bug_1_phantom_changes -- --ignored`
- Run Bug #2 tests: `cargo test --package diffviz-core bug_2_python -- --ignored`
- Run Bug #3 tests: `cargo test --package diffviz-core bug_3_go -- --ignored`
- Check all tests (including fixed): `cargo test --package diffviz-core`