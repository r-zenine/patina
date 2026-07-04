# Implementation Roadmap: VCS History Metrics

**Strategy**: Steel Thread — each phase is a shippable increment visible to the user.

---

## Phase 1: Hotspots + Logical Coupling

**Deliverable**: `patina-cli` computes and prints hotspots and logical coupling for a given repo path and time window.

### Step 1.0 — Rename diffviz-git → git-provider

- Rename directory: `diffviz-git/` → `git-provider/`
- Update `git-provider/Cargo.toml`: `name = "git-provider"`
- Update workspace `Cargo.toml` members list
- Update all `diffviz-git` references in dependent crates (`diffviz-cli`, `diffviz-review-tui`, `Cargo.toml` files)
- `cargo check --workspace` to confirm no breakage

### Step 1.1 — Scaffold patina-analysis and patina-cli crates

Create two new workspace members:

`patina-analysis/Cargo.toml` dependencies: `git-provider`, `diffviz-core`, `chrono`, `thiserror`

`patina-cli/Cargo.toml` dependencies: `patina-analysis`, `git-provider`, `clap`, `thiserror`

Add both to workspace `Cargo.toml` members list.

### Step 1.2 — HistoryProvider trait + types in patina-analysis

New file: `patina-analysis/src/provider.rs`

```rust
pub struct CommitRecord {
    pub hash: String,
    pub date: chrono::DateTime<chrono::Utc>,
    pub files: Vec<String>,
}

pub trait HistoryProvider {
    fn commits_for_files(
        &self,
        files: &[String],
        window_days: u32,
    ) -> Result<Vec<CommitRecord>, Box<dyn std::error::Error>>;
}
```

New file: `patina-analysis/src/signals.rs`

```rust
pub struct RevisionHotspot {
    pub file: String,
    pub commit_count: u32,
    pub window_days: u32,
}

pub struct LogicalCoupling {
    pub file: String,
    pub coupled_file: String,
    pub degree: u8,
    pub shared_commits: u32,
}

pub struct HistorySignals {
    pub hotspots: Vec<RevisionHotspot>,
    pub couplings: Vec<LogicalCoupling>,
}
```

### Step 1.3 — GitHistory implementation in git-provider

New file: `git-provider/src/history.rs`

Implement `HistoryProvider` for `GitRepository`:
- Use `repo.revwalk()` with `push_head()` and `set_sorting(TOPOLOGICAL | TIME)`
- For each commit in the window: collect the list of changed files via `repo.diff_tree_to_tree()`
- Return `Vec<CommitRecord>` bounded by `window_days`

Add `chrono = { workspace = true }` to `git-provider/Cargo.toml`.

### Step 1.4 — Metric computation

New file: `patina-analysis/src/metrics.rs`

`compute_hotspots(records: &[CommitRecord], files: &[String], window_days: u32) -> Vec<RevisionHotspot>`
- Count records where `files` intersects the target file set

`compute_coupling(records: &[CommitRecord], files: &[String], min_degree: u8) -> Vec<LogicalCoupling>`
- Build co-occurrence map: `HashMap<(String, String), u32>` (shared commit count)
- Build per-file total: `HashMap<String, u32>`
- Apply coupling formula: `shared / avg(total_A, total_B) * 100`
- Filter to pairs involving at least one file from the current diff

### Step 1.5 — patina-cli entry point

`patina-cli/src/main.rs`: takes a repo path and optional `--days` flag (default 90).
- Opens repo via `GitRepository`
- Fetches all changed files from the current diff as the file set of interest
- Calls `HistoryProvider::commits_for_files`
- Runs `compute_hotspots` and `compute_coupling`
- Prints results to stdout (table format)

No TUI, no diffviz-review dependency. Standalone tool.

### Tests (TDD)
- Unit test `compute_hotspots` and `compute_coupling` against a fixed `Vec<CommitRecord>` — no git required
- Integration test `GitRepository as HistoryProvider` against a temp git repo (follow pattern in `git-provider/tests/` if it exists)

---

## Phase 2: Non-uniform Modules

**Deliverable**: Files with high intra-file churn variance get an advisory note: which functions are stable vs. churning, with a split recommendation.

**Prerequisite**: Phase 1 complete.

### Design deferred to Last Responsible Moment

The key unknowns to resolve at Phase 2 design time:
- What does the AST node → commit frequency map look like in practice? Need to run Phase 1 on a real codebase first to see if the line range → node mapping is tractable.
- How to define "high variance" threshold? Calibrate against real data from Phase 1.
- Performance: walking history per-file AND parsing each historical version of that file may be slow. May need to limit to the N most recent commits or only walk files that are already hotspots.

### Known approach

1. Extend `HistoryProvider` with a second method:
   ```rust
   fn line_diffs_for_file(
       &self,
       file: &str,
       window_days: u32,
   ) -> Result<Vec<FileDiff>, Box<dyn std::error::Error>>;
   // FileDiff = { commit_hash, changed_line_ranges: Vec<(u32, u32)> }
   ```

2. For each historical version of the file (at commits that touched it):
   - Fetch file content at that commit (`get_file_content_at_commit`)
   - Parse with `diffviz-core` tree-sitter parser for the file's language
   - Map changed line ranges → intersecting `SemanticNode`s by `source_range.start_row/end_row`

3. Compute per-node commit frequency → variance across nodes → `ModuleCoherence` signal

4. Add `coherence: Option<ModuleCoherence>` to `HistorySignals` (None for unsupported languages)

5. TUI: advisory note below the file header when coherence flag is set.

---

## Out of Scope

- Code age (last modification date) — deferred; lower priority than the 3 agreed metrics
- Commit message mining — deferred
- Cross-repo or monorepo aggregation
- Persistent caching of history metrics across sessions (compute fresh each time for now)
