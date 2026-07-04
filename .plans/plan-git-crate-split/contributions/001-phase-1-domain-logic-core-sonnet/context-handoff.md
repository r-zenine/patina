# Context Handoff - Phase 1 Domain Logic (gitkit extraction)

## 🎯 Core Result
**Built**: `libs/gitkit`, a standalone crate wrapping `git2` directly — `GitRepository::open`, `get_file_content_at_commit` (commit/`working-directory`/`index` modes), `get_diff_files`/`get_working_directory_files` (→ `RawFileStatus`), `get_file_stats_for_commits` (→ `RawFileStats`), `resolve_parent_commit`. Zero dependency on `diffviz-review`/`diffviz-core`. `diffviz-git` is completely untouched — the workspace stays green throughout.
**Key insight**: The four near-identical diff-iteration loops in the original `diffviz-git/src/lib.rs` (two for stats counting, two for file-status collection) were consolidated into two shared private helpers (`count_additions_deletions`, `collect_diff_files`) rather than literally duplicated per-method. Same behavior, less code to keep in sync — see decision-log #1.

## 🚦 Current State
**✅ Solid foundation**: `cargo build/test/clippy/fmt --package gitkit` all clean. 14 tests cover repo-open failure modes, all three `get_file_content_at_commit` ref modes (including not-found), diff-file status mapping (added + deleted), working-directory file listing, stats counting, and `resolve_parent_commit` (including the no-parent-commit error case).
**⚠️ Needs attention**: None — Phase 1's scope is fully self-contained and doesn't touch any existing crate.
**⏸️ Deferred**: Everything in `diffviz-git` itself (Phase 2) — slimming it to delegate to `gitkit`, the local `GitRepository`/`GitError` newtypes (orphan-rule wrappers per decision-log D002/D003), and the `RawFileStats`/`RawFileStatus` → `FileStats`/`FileStatus` mapping at the adapter boundary.

## 👥 Next Agent Guidance
**Phase 2 (adapter) implementer**: `gitkit::GitRepository`'s public method set (see decision-log #1's code_impacts) is the exact contract to delegate to. `gitkit::Error` variants are named identically to today's `diffviz_git::GitError` variants, so the `#[from]` wrapper conversion in the adapter should be close to mechanical. Watch the note in the roadmap about `StagingFailed`/`PatchCreationFailed`/`ValidationFailed` — check whether the adapter still constructs any of these directly (it currently doesn't, so they likely just flow through `gitkit::Error::Core`... actually all of `GitError`'s current variants moved into `gitkit::Error` verbatim; the adapter needs its own local `GitError` per D003, wrapping `gitkit::Error` via `#[from]`).

## 🔗 Integration Points
**Expects**: Nothing — `gitkit` has no dependents yet.
**Provides**: `libs/gitkit` crate, ready for `diffviz-git/Cargo.toml` to add `gitkit = { path = "../libs/gitkit" }` in Phase 2. Root `Cargo.toml` now has the `libs/*` member glob per the plan's Definition of Done.

## 📋 Reference Links
- [decision-log.yaml](decision-log.yaml) - Technical choices made
- Commit: `ee6c832bbdaa7bc27861bd0f0f31dcafe577cccb`
