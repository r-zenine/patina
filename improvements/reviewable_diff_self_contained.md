# Make ReviewableDiff Self-Contained with File Content Hash

## Problem Statement

**Crash:** TUI crashes when adding instructions to deleted files
**Error:** `Failed to get file content: File not found: src/config/reader.rs#0 at ref Unstaged`
**Root Cause:** File content hash and snapshot are computed too late in the lifecycle - at instruction-creation time instead of diff-analysis time.

## Current Architecture Issues

### Issue 1: Late Hash Computation

Currently, `file_content_hash` and `content_snapshot` are computed in `ReviewEngine.add_instruction()` when the instruction is created:

```rust
// diffviz-review/src/engines/review_engine.rs:315-319
let file_path = reviewable_id.file_path();
let git_ref = &reviewable_id.query().to;  // ← Uses 'to' ref
let file_content_hash = self.calculate_file_hash(file_path, git_ref)?;  // ← Can fail!
let content_snapshot = self.extract_content_snapshot(file_path, git_ref, &reviewable_id.line_range())?;
```

**Problems:**
1. File may not exist at `query.to` ref (e.g., deleted files)
2. File state may have changed between diff analysis and instruction creation
3. For deletions, file exists at `query.from` but not at `query.to`
4. Timing window: file can be modified/deleted after diff analysis

### Issue 2: ReviewableDiff Lacks File Context

```rust
// diffviz-review/src/state/mod.rs:75-79
pub struct ReviewableDiff {
    pub id: ReviewableDiffId,
    pub core_diff: CoreReviewableDiff,
    pub file_path: String,
    // ❌ NO file_content_hash
    // ❌ NO content_snapshot
}
```

ReviewableDiff is not self-contained - it cannot provide the file hash needed for instruction staleness tracking without re-fetching file content.

### Issue 3: Wrong Git Ref for Deleted Files

When adding instruction to deleted file:
- `query.to = GitRef::Unstaged` (working directory)
- File doesn't exist in working directory (it was deleted)
- Should use `query.from` (e.g., HEAD) instead where file still exists
- Current code blindly uses `query.to` → crash

## Crash Scenario Breakdown

### User Actions:
1. Navigate to a chunk in a deleted file (file exists in HEAD, not in working directory)
2. Press `Space+i+i` to open instruction modal
3. Type message and press Enter

### What Happens:
```
ReviewableDiffId {
  query: DiffQuery { from: HEAD, to: Unstaged },
  file_path: "src/config/reader.rs#0",
  line_range: LineRange { ... }
}

ReviewEngine.add_instruction()
├─ git_ref = reviewable_id.query().to  // GitRef::Unstaged
├─ calculate_file_hash("src/config/reader.rs", Unstaged)
│  └─ diff_provider.get_source_code("src/config/reader.rs", Unstaged)
│     └─ std::fs::read_to_string(workdir + "src/config/reader.rs")
│        └─ ERROR: File not found!
└─ CRASH
```

## Proposed Solution: Self-Contained ReviewableDiff

### Design Principle

**ReviewableDiff should capture file state at diff-analysis time, not instruction-creation time.**

The file hash represents a snapshot of the file content when the diff was analyzed. This is the correct reference point for:
- Detecting staleness (has file changed since review?)
- Preserving context (what did the code look like when reviewed?)
- Instruction validation (is instruction still applicable?)

### Updated Structure

```rust
// diffviz-review/src/state/mod.rs
pub struct ReviewableDiff {
    pub id: ReviewableDiffId,
    pub core_diff: CoreReviewableDiff,
    pub file_path: String,

    // NEW: File state at diff-analysis time
    pub file_content_hash: String,        // SHA-256 hash for staleness detection
    pub content_snapshot: Option<String>,  // Actual code lines for context preservation
}
```

### Correct Git Ref Selection Logic

When computing hash/snapshot, use the git ref where the content actually exists:

```rust
fn select_git_ref_for_content(reviewable_id: &ReviewableDiffId, change_type: &ChangeType) -> GitRef {
    match change_type {
        // For deletions: content exists in 'from' ref (e.g., HEAD)
        ChangeType::Deletion => reviewable_id.query().from.clone(),

        // For additions/modifications: content exists in 'to' ref (e.g., Unstaged)
        _ => reviewable_id.query().to.clone(),
    }
}
```

This ensures we always read from a ref where the file exists.

## Implementation Plan

### Step 1: Add Fields to ReviewableDiff

**File:** `diffviz-review/src/state/mod.rs`

```rust
pub struct ReviewableDiff {
    pub id: ReviewableDiffId,
    pub core_diff: CoreReviewableDiff,
    pub file_path: String,
    pub file_content_hash: String,
    pub content_snapshot: Option<String>,
}

impl ReviewableDiff {
    pub fn new(
        id: ReviewableDiffId,
        core_diff: CoreReviewableDiff,
        file_path: String,
        file_content_hash: String,
        content_snapshot: Option<String>,
    ) -> Self {
        Self {
            id,
            core_diff,
            file_path,
            file_content_hash,
            content_snapshot,
        }
    }
}
```

### Step 2: Compute Hash in ReviewEngineBuilder

**File:** `diffviz-review/src/review_engine_builder.rs`

**Current code (around line 191-195):**
```rust
let reviewable_id = ReviewableDiffId::new(
    query.clone(),
    format!("{file_path}#{index}"),
    line_range,
);
let reviewable_diff = ReviewableDiff::new(reviewable_id.clone(), core_diff, file_path.clone());
```

**New code:**
```rust
let reviewable_id = ReviewableDiffId::new(
    query.clone(),
    format!("{file_path}#{index}"),
    line_range,
);

// Determine correct git ref based on change type
let change_type = extract_change_type(&core_diff);  // Helper to get change type
let git_ref = match change_type {
    ChangeType::Deletion => &query.from,  // Deleted file exists in 'from' ref
    _ => &query.to,                        // Added/modified exist in 'to' ref
};

// Compute hash and snapshot at diff-analysis time
let file_content_hash = calculate_file_hash(&self.diff_provider, &file_path, git_ref)?;
let content_snapshot = extract_content_snapshot(&self.diff_provider, &file_path, git_ref, &line_range)?;

let reviewable_diff = ReviewableDiff::new(
    reviewable_id.clone(),
    core_diff,
    file_path.clone(),
    file_content_hash,
    content_snapshot,
);
```

**Add helper functions:**
```rust
fn extract_change_type(core_diff: &CoreReviewableDiff) -> ChangeType {
    use diffviz_core::reviewable_diff::NodeChangeStatus;

    match core_diff.boundary.change_status {
        NodeChangeStatus::Added => ChangeType::Addition,
        NodeChangeStatus::Deleted => ChangeType::Deletion,
        NodeChangeStatus::Modified => ChangeType::Modification,
        _ => ChangeType::Modification,  // Default to modification
    }
}

enum ChangeType {
    Addition,
    Deletion,
    Modification,
}

fn calculate_file_hash(
    diff_provider: &dyn DiffProvider,
    file_path: &str,
    git_ref: &GitRef,
) -> Result<String> {
    let content = diff_provider
        .get_source_code(file_path, git_ref)
        .map_err(|e| DiffVizError::Git(format!("Failed to get file content: {e}")))?;

    // Normalize line endings (CRLF → LF)
    let normalized = content.replace("\r\n", "\n");

    // Calculate SHA256
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    let result = hasher.finalize();

    Ok(format!("{result:x}"))
}

fn extract_content_snapshot(
    diff_provider: &dyn DiffProvider,
    file_path: &str,
    git_ref: &GitRef,
    line_range: &LineRange,
) -> Result<Option<String>> {
    let content = diff_provider
        .get_source_code(file_path, git_ref)
        .map_err(|e| DiffVizError::Git(format!("Failed to get file content: {e}")))?;

    let lines: Vec<&str> = content.lines().collect();

    // 1-based to 0-based index conversion
    let start_idx = (line_range.start_line.saturating_sub(1)).min(lines.len());
    let end_idx = line_range.end_line.min(lines.len());

    if start_idx >= lines.len() {
        return Ok(None);
    }

    let snapshot = lines[start_idx..end_idx].join("\n");
    Ok(Some(snapshot))
}
```

### Step 3: Simplify ReviewEngine.add_instruction()

**File:** `diffviz-review/src/engines/review_engine.rs`

**Current code (lines 239-342):**
```rust
pub fn add_instruction(
    &mut self,
    reviewable_id: ReviewableDiffId,
    content: String,
    author: String,
    on_result: OperationCallback,
) -> Result<()> {
    // ... overlap detection ...

    // ❌ Remove this computation - use ReviewableDiff fields instead
    let file_path = reviewable_id.file_path();
    let git_ref = &reviewable_id.query().to;
    let file_content_hash = self.calculate_file_hash(file_path, git_ref)?;
    let content_snapshot = self.extract_content_snapshot(file_path, git_ref, &reviewable_id.line_range())?;

    // ... create instruction ...
}
```

**New code:**
```rust
pub fn add_instruction(
    &mut self,
    reviewable_id: ReviewableDiffId,
    content: String,
    author: String,
    on_result: OperationCallback,
) -> Result<()> {
    // Get ReviewableDiff to access file hash and snapshot
    let reviewable_diff = self.state.reviewable_diffs.get(&reviewable_id)
        .ok_or_else(|| DiffVizError::Review("ReviewableDiff not found".to_string()))?;

    // ... overlap detection (use reviewable_diff.file_content_hash for extended range) ...

    // ✅ Use pre-computed values from ReviewableDiff
    let file_content_hash = reviewable_diff.file_content_hash.clone();
    let content_snapshot = reviewable_diff.content_snapshot.clone();

    let instruction = Instruction {
        id: uuid::Uuid::new_v4().to_string(),
        reviewable_id: reviewable_id.clone(),
        author,
        timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        content,
        status: InstructionStatus::Active,
        file_content_hash,
        content_snapshot,
    };

    self.state.add_instruction(instruction);
    // ... rest of method ...
}
```

**Remove these methods (no longer needed):**
- `calculate_file_hash()` (lines 345-366)
- `extract_content_snapshot()` (lines 368-399)

### Step 4: Handle Overlap Auto-Merge

For overlapping instructions that get merged to extended ranges, we need to compute hash for the new extended range:

```rust
// When overlap is detected and range is extended
let extended_id = ReviewableDiffId::new(
    reviewable_id.query().clone(),
    reviewable_id.file_path().to_string(),
    union_range,
);

// Find which ReviewableDiff contains the extended range or create synthetic hash
// Option 1: Use hash from one of the overlapping ReviewableDiffs (simpler)
let file_content_hash = reviewable_diff.file_content_hash.clone();

// Option 2: Compute new hash for extended range (more accurate but requires DiffProvider)
// This requires keeping diff_provider reference in ReviewEngine
```

**Recommendation:** Use Option 1 initially (use hash from original ReviewableDiff). The hash is used for staleness detection - if any of the overlapping ranges' files change, they all become stale.

### Step 5: Update Tests

**Files to update:**
- `diffviz-review/src/state/mod.rs` - Update test fixtures
- `diffviz-review/src/engines/review_engine.rs` - Update test cases
- `diffviz-review/src/review_engine_builder.rs` - Update builder tests

**Test helper update:**
```rust
// In test fixtures
fn create_test_reviewable_diff() -> ReviewableDiff {
    ReviewableDiff::new(
        reviewable_id,
        core_diff,
        "src/test.rs".to_string(),
        "abc123...".to_string(),  // test hash
        Some("fn test() {}\n".to_string()),  // test snapshot
    )
}
```

## Impact Analysis

### Benefits

1. **Crash Prevention:** Deleted files can be handled correctly by using appropriate git ref
2. **Timing Independence:** Hash computed once at analysis time, not re-fetched later
3. **Self-Contained:** ReviewableDiff contains all context needed for instruction operations
4. **Correct Semantics:** Hash represents "state when diff was analyzed" not "state when instruction was added"
5. **Performance:** Eliminates redundant file reads when adding multiple instructions to same diff

### Breaking Changes

1. **ReviewableDiff constructor signature changes:** Requires `file_content_hash` and `content_snapshot` parameters
2. **Serialization format changes:** Adds new fields to ReviewableDiff (impacts JSON export/import if ReviewableDiff is serialized)

### Migration Strategy

Since ReviewableDiff is created programmatically (not loaded from disk in current implementation), no data migration needed. Just update all construction sites.

## Testing Strategy

### Unit Tests

1. **ReviewableDiff creation with hash/snapshot**
   - Test hash computation for various file types
   - Test snapshot extraction for different line ranges
   - Test correct git ref selection (deletions vs additions)

2. **Instruction creation without file access**
   - Mock ReviewableDiff with pre-computed hash
   - Verify no DiffProvider calls during add_instruction()
   - Test overlap merge using cached hashes

### Integration Tests

1. **Deleted file scenario**
   - Create diff with deleted file (exists in HEAD, not in Unstaged)
   - Verify ReviewableDiff creation succeeds with hash from HEAD
   - Add instruction - should succeed without crash

2. **File modification timing**
   - Create ReviewableDiff with file in state A
   - Modify file to state B in working directory
   - Add instruction - should use hash from state A (when analyzed)

### TUI Integration Test

1. **Reproduce original crash scenario**
   - Setup: Create diff with deleted file
   - Navigate: Tab+j to expand decision, Tab+j to expand file, select chunk
   - Action: Space+i+i, type message, press Enter
   - Expected: No crash, instruction created successfully

## Open Questions

1. **Overlap auto-merge hash:** When merging overlapping instructions into extended range, should we:
   - Use hash from one of the original ReviewableDiffs? (simpler)
   - Compute new hash for extended range? (requires keeping DiffProvider in ReviewEngine)
   - Skip hash for synthetic extended ReviewableDiffs? (mark as special case)

2. **Staleness detection:** When should hashes be re-computed to detect stale instructions?
   - On ReviewEngine load?
   - On-demand when viewing instructions?
   - Background process?

3. **Memory impact:** Adding hash/snapshot to every ReviewableDiff increases memory usage. Is this acceptable?
   - Estimate: ~100 bytes per ReviewableDiff (64-char hash + small snapshot)
   - For 1000 diffs: ~100KB additional memory
   - Seems acceptable for better correctness

## Implementation Order

1. ✅ Write this design document
2. Add fields to ReviewableDiff struct
3. Add helper functions to ReviewEngineBuilder
4. Update ReviewEngineBuilder to compute hash/snapshot
5. Simplify ReviewEngine.add_instruction() to use cached values
6. Remove calculate_file_hash() and extract_content_snapshot() methods
7. Update all tests and fixtures
8. Test deleted file scenario in TUI
9. Run full test suite

## References

- **Crash location:** `diffviz-review-tui/src/app.rs:429`
- **Current hash computation:** `diffviz-review/src/engines/review_engine.rs:345-399`
- **ReviewableDiff struct:** `diffviz-review/src/state/mod.rs:75-94`
- **ReviewEngineBuilder:** `diffviz-review/src/review_engine_builder.rs:191-195`
- **Git ref handling:** `diffviz-git/src/lib.rs:941-981`
