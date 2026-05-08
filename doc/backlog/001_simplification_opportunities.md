# Simplification Audit

Opportunities to remove duplication and consolidate utilities across the workspace.

---

## 1. Language Detection Duplication (Critical)

**Problem**: Two implementations mapping the same file extensions to language names.

| Location | Function | Returns |
|---|---|---|
| `diffviz-core/src/common.rs:90-106` | `ProgrammingLanguage::from_file_path()` | `ProgrammingLanguage` enum |
| `diffviz-cli/src/commands/debug.rs:110-121` | `detect_language()` | `Result<String>` |

**Fix**: Add `Display` impl (or `display_name()`) to `ProgrammingLanguage` in diffviz-core, then delete `detect_language()` from diffviz-cli.

**Estimated savings**: ~30 LOC

---

## 2. Git Reference Formatting Duplication (Critical)

**Problem**: Two near-identical implementations of `GitRef → String` conversion.

| Location | Function | Notes |
|---|---|---|
| `diffviz-review/src/entities/reviewable_diff_id.rs:197-210` | `format_git_ref()` + `shorten_ref()` | Private; display strings ("HEAD", "STAGED", "UNSTAGED") |
| `diffviz-git/src/lib.rs:879-886` | `git_ref_to_string()` | Git operation markers ("--staged", "--unstaged") |

Both shorten commit hashes to 7 chars. `shorten_ref` is private so diffviz-git can't reuse it.

**Fix**: Make `shorten_ref` public and shared. Consider a `Display` impl on `GitRef` for the display case, and keep the git-operation variant in diffviz-git where it belongs. Remove the private duplication.

**Estimated savings**: ~20 LOC, better cohesion

---

## 3. Language Support Check Duplicated in review_engine_builder (High)

**Problem**: `diffviz-review/src/review_engine_builder.rs` re-implements language-from-path logic that diffviz-core already owns.

- `is_supported_file(file_path: &str) -> bool` — checks extension against a hardcoded list
- `get_language_parser_for_file()` — factory that duplicates what `ProgrammingLanguage::from_file_path()` + the parser registry already do

**Fix**: Delete both functions, call `ProgrammingLanguage::from_file_path()` directly and route to the existing parser infrastructure.

---

## 4. Private Utility Functions Trapped in reviewable_diff_id.rs (High)

**Problem**: Five pure utility functions are private to `reviewable_diff_id.rs`, making them unreusable.

- `categorize_query()`
- `compare_queries()`
- `format_diff_query()`
- `format_git_ref()`
- `shorten_ref()`

These operate on `DiffQuery` and `GitRef` — types that live elsewhere. They should be methods on those types or live in the module where those types are defined (`git_ref.rs`), not buried in the ID module.

**Fix**: Move to `diffviz-review/src/entities/git_ref.rs` as inherent methods or a shared formatting module. Expose where appropriate.

---

## 5. TempFile / TempDirectory Reimplement the `tempfile` Crate (Medium)

**Problem**: `sam-utils/src/fsutils.rs` has hand-rolled `TempFile` and `TempDirectory` structs using UUID-generated names and manual `Drop` cleanup.

The `tempfile` crate is **already a workspace dependency** and provides `TempDir` and `NamedTempFile` with the same semantics plus better error handling and cross-platform support.

**Fix**: Replace `TempFile`/`TempDirectory` usages (3 in sam-persistence) with `tempfile::NamedTempFile` / `tempfile::TempDir`, then delete the custom implementations.

---

## 6. walk_dir Has Misleading Documentation (Medium)

**Problem**: `sam-utils/src/fsutils.rs:54-69` — `walk_dir()` is documented as a recursive directory walker but only lists entries one level deep.

**Fix**: Either fix the implementation to actually recurse (using `walkdir` crate already available), or rename to `list_dir()` to match the actual behavior.

---

## 7. CLI Parsing Utilities Belong Closer to Their Types (Low)

**Problem**: `diffviz-cli/src/commands/debug.rs` contains:

- `parse_git_ref(ref_str: &str) -> GitRef` — String → enum; belongs on `GitRef` as `FromStr`
- `parse_line_range(range: &str) -> Result<(usize, usize)>` — generic enough to live in diffviz-review or diffviz-core

**Fix**: Implement `FromStr` for `GitRef` in diffviz-review. Move `parse_line_range` to a shared location if ever needed beyond debug.

---

## Suggested Order of Execution

1. **Language detection** — smallest, cleanest win; adds `Display` to `ProgrammingLanguage`, deletes CLI duplicate
2. **Git ref consolidation** — unify `format_git_ref`/`shorten_ref`/`git_ref_to_string`
3. **review_engine_builder cleanup** — delete `is_supported_file` and `get_language_parser_for_file`, use core types directly
4. **reviewable_diff_id.rs extraction** — move private fns to the types they operate on
5. **TempFile/TempDirectory** — replace with `tempfile` crate
6. **walk_dir** — fix or rename
7. **CLI parsing** — implement `FromStr` for `GitRef`
