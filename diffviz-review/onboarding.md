# diffviz-review - Orientation Guide

## What This Module Does
Orchestrates the code review workflow by managing review state, decisions, approvals, and instructions for ReviewableDiffs produced by diffviz-core.

## Before You Code Here
**Existing Patterns:**
- ReviewableDiff wrapper pattern: wraps core ReviewableDiff with review-layer metadata (id, file_path)
- State mutation through builder pattern: methods return `&mut Self` for chaining
- DiffProvider trait for dependency inversion: infrastructure provides git capabilities to review layer
- Decision-based review system: maps architectural decisions to code changes with reverse indexing

**Reusable DTOs/Types:**
- `ReviewableDiffId`: Universal identifier combining (DiffQuery, file_path, LineRange) - use this everywhere for identifying review items
- `ReviewState`: Central state container with approvals, instructions, decisions, and reviewable_diffs
- `Decision`, `Approval`, `Instruction`: Core review entities organized by ReviewableDiffId
- `DiffQuery` and `GitRef`: Type-safe git reference modeling (avoid hardcoded strings)

**Integration Points:**
- Depends on diffviz-core for semantic analysis (ReviewableDiff, RenderableDiff, AST parsing)
- Provides DiffProvider trait that diffviz-git implements
- ReviewEngineBuilder bridges git layer → core analysis → review state

## Key Abstractions to Reuse

### ReviewState (state/mod.rs)
Central state container for all review data. Contains:
- `reviewable_diffs: BTreeMap<ReviewableDiffId, ReviewableDiff>` - ordered by file and line range
- `approvals: ReviewApprovals` - approval tracking by ReviewableDiffId
- `instructions: ReviewInstructions` - instruction collection by ReviewableDiffId
- `decisions: ReviewDecisions` - decision-to-code mapping with reverse index
- `journey: ReviewJourney` - placeholder for review journey tracking (currently minimal)

Query methods: `is_approved()`, `get_instructions()`, `approval_progress()`, `check_instruction_overlap()`
Mutation methods: `approve()`, `unapprove()`, `add_instruction()` (return `&mut Self`)

### ReviewableDiffId (entities/reviewable_diff_id.rs)
Universal identifier for review items. Triplet of:
- `query: DiffQuery` - what comparison (HEAD..unstaged, commit..commit, etc.)
- `file_path: String` - which file
- `line_range: LineRange` - which lines (1-based start/end, 0-based columns)

Implements Ord for BTreeMap ordering (file → query → line range). Use `same_file_and_query()` to check overlap candidates.

### Review Entities (entities/)

**Approval (approval.rs)**: Simple approved/rejected state per ReviewableDiffId
- Fields: `reviewable_id`, `approved`, `approved_by`, `approval_timestamp`
- Collection: `ReviewApprovals` with HashMap indexing
- Methods: `approve()`, `unapprove()`, `is_approved()`, `total_approved()`, `approval_percentage()`

**Instruction (instruction.rs)**: Actionable feedback with status tracking
- Status: Active (valid), Stale (file changed), Addressed (user marked as done)
- Fields: `id`, `reviewable_id`, `content`, `author`, `timestamp`, `status`, `file_content_hash`, `content_snapshot`
- Collection: `ReviewInstructions` with HashMap<ReviewableDiffId, Vec<Instruction>>
- Methods: `add_instruction()`, `get_instructions()`, `get_instructions_by_status()`, `remove_instruction_by_id()`

**Decision (decision.rs)**: Architectural decision mapping with code impact tracking
- Structure: Decision has title, summary, and `code_impacts: Vec<CodeImpact>`
- CodeImpact: Maps to file + line_ranges + change_type + confidence + reasoning
- Collection: `ReviewDecisions` with:
  - `decisions: HashMap<u32, Decision>` - indexed by decision number
  - `decision_index: HashMap<ReviewableDiffId, Vec<u32>>` - reverse index (which decisions affect this code)
- Key methods:
  - `add_decision()` - add a decision
  - `build_index_from_review_state()` - build reverse index by detecting overlaps
  - `create_unmapped_decision()` - synthetic Decision 0 for unmapped code (only if unmapped diffs exist)
  - `get_decisions_for_diff()` - find all decisions affecting a ReviewableDiffId
- Overlap detection: ranges overlap if `start1 <= end2 && start2 <= end1`

### ReviewEngine (engines/review_engine.rs)
Main business logic orchestrator. Contains:
- `state: ReviewState` - current review state
- `renderable_cache: HashMap<ReviewableDiffId, String>` - rendering cache (placeholder, would be RenderableDiff)
- `diff_provider: Box<dyn DiffProvider>` - git operations abstraction

Key behaviors:
- **Approve/reject**: invalidates renderable cache for affected ReviewableDiffId
- **Instruction overlap auto-merge**: when overlap detected, extends to union range and concatenates content with separator
- **File hash tracking**: calculates file_content_hash and content_snapshot for staleness detection
- **Export/import**: JSON format with metadata for agent understanding (git context, query formats, examples)

Export/Import capabilities:
- Export scopes: SingleFile, SingleInstruction, All
- ExportedInstruction format includes: file, query, line_range, content, author, timestamp, status, file_content_hash, content_snapshot
- ImportSummary tracks: total_imported, active_count, stale_count, errors

### ReviewEngineBuilder (review_engine_builder.rs)
Orchestrates the complete pipeline from git → ReviewEngine:
1. Get changed files via DiffProvider (`get_changed_files(&DiffQuery)`)
2. Filter to supported languages: Rust, Python, Go, Java, TypeScript, JavaScript, C, C++
3. Run diffviz-core semantic analysis pipeline:
   - Parse AST with TreeSitter using language-specific parser
   - Build semantic trees with `parser.build_semantic_tree()`
   - Build semantic pairs with `build_semantic_pairs()` (change detection)
   - Convert to ReviewableDiffs with `semantic_pairs_to_reviewable_diffs()`
4. Create ReviewEngine with populated state

Special handling:
- Added/untracked files: use empty string as old content
- Deleted files: skipped (no new content to analyze)
- Unsupported file types: skipped with error message
- Line range extraction: uses `extract_line_range_from_core_diff()` based on boundary node's change status

### DiffProvider Trait (providers/mod.rs)
Interface for git operations needed by review layer:
- `get_changed_files(&DiffQuery) -> Vec<(String, FileStatus)>` - list files with changes
- `get_file_stats(&file, &DiffQuery) -> FileStats` - git diffstat (additions/deletions/total_changes)
- `get_source_code(&file, &GitRef) -> String` - file content at specific git ref

FileStatus enum: Added, Modified, Deleted, Renamed, Copied, Untracked
FileStats helpers: `is_creation()`, `is_deletion()`, `is_modification()`, `is_unchanged()`

Implemented by diffviz-git infrastructure layer. Use MockDiffProvider for testing.

## Architectural Constraints

**Layer Dependencies (CRITICAL):**
- This is the review orchestration layer
- Depends on diffviz-core for semantic analysis (ReviewableDiff, semantic AST, parsers)
- Infrastructure layers (diffviz-git, diffviz-llm) depend on this layer
- Never introduce dependencies on infrastructure layers

**ReviewableDiffId as Universal Identifier:**
- All review operations use ReviewableDiffId, never legacy chunk IDs
- BTreeMap ordering ensures consistent file/line traversal
- Line ranges are 1-based (lines) and 0-based (columns) - follow this convention
- Display format: `{query}:{file_path}:L{start}-{end}` (e.g., "working:src/main.rs:L10-20")

**State Immutability Pattern:**
- Mutation methods return `&mut Self` for builder-style chaining
- Example: `state.approve(id, reviewer).add_instruction(instruction)`
- Never expose raw HashMap mutations - use provided methods

**Decision Index Building (Critical Workflow):**
1. Add all decisions with `add_decision()`
2. Call `build_index_from_review_state()` to populate reverse index
3. Call `create_unmapped_decision()` to create Decision 0 (only if unmapped diffs exist)
4. Now `get_decisions_for_diff()` returns all affecting decisions

**Overlap Semantics:**
- Instruction overlap detection: ranges overlap if `start1 <= end2 && start2 <= end1`
- Decision overlap detection: same algorithm, maps ReviewableDiffId to affecting decisions
- Overlap types: Exact (same range), Partial (some overlap), Nested (one contains other)
- Instruction auto-merge: calculate union range, remove old, create new with merged content

**File Hash and Content Snapshot:**
- Instructions track `file_content_hash` (SHA-256 of file content) for staleness detection
- `content_snapshot` captures actual code lines for context preservation
- Must use DiffProvider to fetch content at specific GitRef for accurate snapshots
- Status transitions: Active → Stale when file hash changes (future capability)

## Directory Map
```
diffviz-review/
├── src/
│   ├── entities/          # Core domain entities
│   │   ├── approval.rs    # Approval tracking with ReviewApprovals collection
│   │   ├── decision.rs    # Decision-to-code mapping with reverse index
│   │   ├── instruction.rs # Review instructions with status (Active/Stale/Addressed)
│   │   ├── reviewable_diff_id.rs  # Universal identifier (DiffQuery + file + LineRange)
│   │   └── git_ref.rs     # Type-safe git references (Commit/Head/Staged/Unstaged)
│   ├── state/             # Centralized review state
│   │   └── mod.rs         # ReviewState (approvals/instructions/decisions) + ReviewableDiff wrapper
│   ├── engines/           # Business logic orchestration
│   │   └── review_engine.rs  # Review operations + overlap auto-merge + export/import
│   ├── providers/         # Infrastructure interfaces
│   │   ├── mod.rs         # DiffProvider trait (get_changed_files/get_file_stats/get_source_code)
│   │   └── mock_provider.rs  # Test provider
│   ├── review_engine_builder.rs  # Pipeline orchestration (git → core → review)
│   ├── errors.rs          # Structured errors (DiffVizError + ReviewError)
│   └── lib.rs             # Public API exports
```

## Development Rules

**When Adding Review Operations:**
- Always operate on ReviewableDiffId, never raw strings or indices
- Invalidate renderable_cache entries when state changes affect rendering
- Use DiffProvider for all git operations - never call git directly
- Return `&mut Self` from mutation methods for builder pattern consistency

**When Working with Decisions:**
- After bulk-adding decisions, call `build_index_from_review_state()`
- Call `create_unmapped_decision()` only after index is built (creates Decision 0 if needed)
- Use `get_decisions_for_diff()` to find which decisions affect a ReviewableDiffId
- Decision numbers start at 1 (Decision 0 is synthetic for unmapped code)
- Use `all_decisions()` for ordered iteration (sorted by number)

**When Adding New Review Entities:**
- Index by ReviewableDiffId in HashMap for fast lookup
- Provide query methods that don't expose internal HashMap
- Implement serialization with serde for persistence
- Add convenience constructors to ReviewState for manipulation
- Follow existing patterns: entity struct + collection struct with HashMap

**When Handling Overlaps:**
- Use `check_instruction_overlap()` to detect conflicts before adding
- Review engine auto-merges overlapping instructions (extends range + concatenates)
- Decision overlaps populate the reverse index (multiple decisions can affect same code)
- Use `detect_range_overlap()` for custom overlap logic

**Testing Strategy:**
- Unit tests in entity modules verify serialization and business logic
- State tests verify overlap detection and instruction management
- Builder tests use MockDiffProvider to simulate git operations
- Avoid testing git/core integration - that's in higher layers
- Test fixtures should create realistic ReviewableDiffs with proper core diffs

## Common Patterns

### Creating ReviewableDiffId
```rust
use crate::entities::reviewable_diff_id::{ReviewableDiffId, LineRange};
use crate::entities::git_ref::{DiffQuery, GitRef};

let id = ReviewableDiffId::new(
    DiffQuery::head_to_unstaged(),
    "src/main.rs".to_string(),
    LineRange { start_line: 10, end_line: 20, start_column: 0, end_column: 0 }
);
```

### Building Review State with Chaining
```rust
use crate::state::ReviewState;

let mut state = ReviewState::new(reviewable_diffs, "author".to_string());
state
    .approve(reviewable_id.clone(), "reviewer".to_string())
    .add_instruction(instruction);
```

### Building ReviewEngine from Git Query
```rust
use crate::review_engine_builder::ReviewEngineBuilder;
use crate::entities::git_ref::{DiffQuery, GitRef};

let builder = ReviewEngineBuilder::new(diff_provider, "author".to_string());
let engine = builder.build(DiffQuery::head_to_unstaged())?;
```

### Adding Decisions with Index (Critical Workflow)
```rust
use crate::entities::ReviewDecisions;

let mut decisions = ReviewDecisions::new();
decisions.add_decision(decision1);
decisions.add_decision(decision2);

// CRITICAL: Must build index after adding all decisions
decisions.build_index_from_review_state(&review_state);

// CRITICAL: Only creates Decision 0 if unmapped diffs exist
decisions.create_unmapped_decision(&review_state);

// Now can query which decisions affect specific code
let affecting_decisions = decisions.get_decisions_for_diff(&reviewable_id);
```

### Instruction Auto-merge on Overlap
```rust
// When adding instruction that overlaps existing one
engine.add_instruction(
    reviewable_id,  // Overlaps with existing
    "New feedback".to_string(),
    "reviewer".to_string(),
    Some(Box::new(|success, msg| {
        // Callback receives: "Instruction extended to L10-25"
    }))
)?;
// Auto-extends to union range and merges content with "---" separator
```

## Testing Patterns

### Using MockDiffProvider
```rust
use crate::providers::{DiffProvider, FileStatus};

struct MockDiffProvider {
    files: Vec<(String, FileStatus)>,
}

impl DiffProvider for MockDiffProvider {
    fn get_changed_files(&self, _query: &DiffQuery)
        -> Result<Vec<(String, FileStatus)>> {
        Ok(self.files.clone())
    }

    fn get_source_code(&self, _file_path: &str, git_ref: &GitRef)
        -> Result<String> {
        match git_ref {
            GitRef::Head => Ok("old content".to_string()),
            GitRef::Unstaged => Ok("new content".to_string()),
            _ => Ok("test content".to_string()),
        }
    }

    // ... implement get_file_stats
}
```

### Creating Test ReviewableDiffs
See `state/mod.rs` tests for complete examples. Basic pattern:
```rust
use diffviz_core::{
    ast_diff::{OwnedNodeData, SourceCode},
    common::{ProgrammingLanguage, SemanticNodeKind},
    reviewable_diff::{DiffMetadata, DiffNode, NodeChangeStatus, ReviewableDiff as CoreReviewableDiff},
};

// Create core diff with placeholder data
let core_diff = CoreReviewableDiff {
    language: ProgrammingLanguage::Rust,
    boundary: DiffNode { /* ... */ },
    old_source: Box::new(SourceCode::new("old".to_string())),
    new_source: Box::new(SourceCode::new("new".to_string())),
    metadata: DiffMetadata { /* ... */ },
};

// Wrap with review layer
let reviewable_diff = ReviewableDiff::new(reviewable_id, core_diff, file_path);
```

## Known Patterns and Anti-patterns

**DO:**
- Use ReviewableDiffId for all review item identification
- Call `build_index_from_review_state()` after adding decisions
- Invalidate cache when state changes
- Return `&mut Self` from mutation methods
- Use DiffProvider for git operations

**DON'T:**
- Don't use legacy chunk IDs or ad-hoc string identifiers
- Don't forget to build decision index (reverse lookup won't work)
- Don't skip `create_unmapped_decision()` (unmapped code becomes invisible)
- Don't expose internal HashMap - provide query methods
- Don't call git directly - use DiffProvider abstraction

---

**Updated:** 2026-01-23 - Comprehensive analysis of current decision-based review system, instruction auto-merge, export/import capabilities, and ReviewEngineBuilder pipeline details.
