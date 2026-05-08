# Simplification Audit

Opportunities to remove duplication and consolidate utilities across the workspace.

---

## 1. Language Detection Duplication ~~(Critical)~~ ✅ Done

**Problem**: Two implementations mapping the same file extensions to language names.

**Fix applied**: Added `Display` impl to `ProgrammingLanguage` in `diffviz-core/src/common.rs`. Deleted `detect_language()` from `diffviz-cli/src/commands/debug.rs` and replaced call sites with `ProgrammingLanguage::from_file_path(&path).to_string()`.

---

## 2. Git Reference Formatting Duplication ~~(Critical)~~ ✅ Done

**Problem**: Two near-identical implementations of `GitRef → String` conversion.

| Location | Function | Notes |
|---|---|---|
| `diffviz-review/src/entities/reviewable_diff_id.rs:197-210` | `format_git_ref()` + `shorten_ref()` | Private; display strings ("HEAD", "STAGED", "UNSTAGED") |
| `diffviz-git/src/lib.rs:879-886` | `git_ref_to_string()` | Git operation markers ("--staged", "--unstaged") |

Both shorten commit hashes to 7 chars. `shorten_ref` is private so diffviz-git can't reuse it.

**Fix applied**: Added `Display` impl to `GitRef` in `git_ref.rs` (truncates commit hashes to 7 chars, maps others to `HEAD`/`STAGED`/`UNSTAGED`). Deleted `format_git_ref()` from `reviewable_diff_id.rs` and replaced its call site with `query.from` via `Display`. The git-operation variant (`git_ref_to_string` returning `--staged` etc.) stays in `diffviz-git` where it belongs.

**Estimated savings**: ~20 LOC, better cohesion

---

## 3. Language Support Check Duplicated in review_engine_builder ~~(High)~~ ✅ Done

**Problem**: `diffviz-review/src/review_engine_builder.rs` re-implements language-from-path logic that diffviz-core already owns.

- `is_supported_file(file_path: &str) -> bool` — checks extension against a hardcoded list
- `get_language_parser_for_file()` — factory that duplicates what `ProgrammingLanguage::from_file_path()` + the parser registry already do

**Fix applied**: Added `tsx`/`jsx`/`hxx` to `ProgrammingLanguage::from_file_path()`. Added `parser_for_language(ProgrammingLanguage) -> Option<Box<dyn LanguageParser>>` factory in `diffviz-core/src/parsers/mod.rs`. Deleted both private functions from `review_engine_builder.rs` and replaced the call sites with `from_file_path()` + `parser_for_language()`.

---

## 4. Private Utility Functions Trapped in reviewable_diff_id.rs ~~(High)~~ ✅ Done

**Problem**: Five pure utility functions were private to `reviewable_diff_id.rs`, making them unreusable.

**Fix applied**: Moved `categorize_query`, `compare_queries`, `format_diff_query`, and `shorten_ref` into `git_ref.rs` as `Display` and `Ord`/`PartialOrd` impls on `DiffQuery`. `ReviewableDiffId`'s `Display` now delegates to `self.query` and its `Ord` uses `self.query.cmp()`.

---

## 5. TempFile / TempDirectory Reimplement the `tempfile` Crate ~~(Medium)~~ ✅ Done

**Problem**: `sam-utils/src/fsutils.rs` had hand-rolled `TempFile` and `TempDirectory` structs using UUID-generated names and manual `Drop` cleanup.

**Fix applied**: Deleted both structs and removed `rand`/`uuid` deps from `sam-utils`. Replaced all 4 test usages in `sam-persistence` with `tempfile::NamedTempFile` (using `.path()` instead of `.path`). Updated `sam-persistence` dev-dependency to use the workspace `tempfile = "3.8"` version.

---

## 6. walk_dir Has Misleading Documentation ~~(Medium)~~ ✅ Done

**Problem**: `sam-utils/src/fsutils.rs:54-69` — `walk_dir()` is documented as a recursive directory walker but only lists entries one level deep.

**Fix**: Either fix the implementation to actually recurse (using `walkdir` crate already available), or rename to `list_dir()` to match the actual behavior.

---

## 7. CLI Parsing Utilities Belong Closer to Their Types ~~(Low)~~ ✅ Done

**Problem**: `diffviz-cli/src/commands/debug.rs` had two utility methods trapped on `DebugCommand`.

**Fix applied**: Implemented `FromStr for GitRef` (with `Infallible` error) in `git_ref.rs`; call sites now use `.parse().unwrap()`. Converted `parse_line_range` from a `&self` method to a module-level free function in `debug.rs` — it's only used there so no shared location is needed yet.

---

## Suggested Order of Execution

1. **Language detection** — smallest, cleanest win; adds `Display` to `ProgrammingLanguage`, deletes CLI duplicate
2. **Git ref consolidation** — unify `format_git_ref`/`shorten_ref`/`git_ref_to_string`
3. **review_engine_builder cleanup** — delete `is_supported_file` and `get_language_parser_for_file`, use core types directly
4. **reviewable_diff_id.rs extraction** — move private fns to the types they operate on
5. **TempFile/TempDirectory** — replace with `tempfile` crate
6. **walk_dir** — fix or rename
7. **CLI parsing** — implement `FromStr` for `GitRef`
