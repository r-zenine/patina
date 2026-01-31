# diffviz-review - Orientation Guide

## What This Module Does
Orchestrates the code review workflow by managing review state, decisions, approvals, and instructions for ReviewableDiffs produced by diffviz-core.

## Before You Code Here
**Existing Patterns:**
- ReviewableDiff wrapper pattern: wraps core ReviewableDiff with review-layer metadata (id, file_path)
- State mutation through builder pattern: methods return `&mut Self` for chaining
- DiffProvider trait for dependency inversion: infrastructure provides git capabilities to review layer
- Decision-based review system: maps architectural decisions to code changes with reverse indexing
- Fixture-based testing: JSON fixture files loaded via MockDiffProvider for predictable testing

**Reusable DTOs/Types:**
- `ReviewableDiffId`: Universal identifier combining (DiffQuery, file_path, LineRange) - use this everywhere for identifying review items
- `ReviewState`: Central state container with approvals, instructions, decisions, and reviewable_diffs
- `Decision`, `Approval`, `Instruction`: Core review entities organized by ReviewableDiffId
- `DiffQuery` and `GitRef`: Type-safe git reference modeling (avoid hardcoded strings)
- `ReviewFixture`: Test fixture structure for loading curated test data

**Integration Points:**
- Depends on diffviz-core for semantic analysis (ReviewableDiff, RenderableDiff, AST parsing)
- Provides DiffProvider trait that diffviz-git implements
- ReviewEngineBuilder bridges git layer → core analysis → review state

## Key Abstractions to Reuse

### ReviewEngineBuilder (review_engine_builder.rs) - CRITICAL PIPELINE ORCHESTRATOR

**Purpose:** Transforms git diffs into ReviewEngine with semantic ReviewableDiffs.

**Construction:**
```rust
let builder = ReviewEngineBuilder::new(
    diff_provider: Box<dyn DiffProvider>,
    author: String
);
```

**The Complete Pipeline (build method):**

1. **DiffProvider → Changed Files**
   - Calls `diff_provider.get_changed_files(&query)`
   - Returns `Vec<(String, FileStatus)>` of changed files
   - FileStatus: Added, Modified, Deleted, Renamed, Copied, Untracked

2. **File Filtering**
   - Skips unsupported file types (only processes: rs, py, go, java, ts, tsx, js, jsx, c, h, cpp, cxx, hpp, hxx)
   - Skips deleted files (semantic analysis requires both old and new content)
   - Uses `is_supported_file()` helper for extension matching

3. **Semantic Analysis Per File** (create_semantic_reviewable_diffs)
   - **Step 3a: Get Parser** - `get_language_parser_for_file()` returns language-specific parser
   - **Step 3b: Fetch Content** - DiffProvider gets old/new source code via `get_source_code(file, &query.from/to)`
   - **Step 3c: Parse AST** - `parser.try_parse()` builds TreeSitter AST for old and new
   - **Step 3d: Build Semantic Trees** - `parser.build_semantic_tree()` converts AST to semantic representation
   - **Step 3e: Build Semantic Pairs** - `build_semantic_pairs(old, new, old_source, new_source)` detects changes
   - **Step 3f: Convert to ReviewableDiffs** - `semantic_pairs_to_reviewable_diffs()` with context expansion
   - **Step 3g: Wrap with Review Layer** - Create ReviewableDiffId and wrap core diff

4. **ReviewEngine Creation**
   - Collects all ReviewableDiffs from all files
   - Calls `ReviewEngine::new(all_diffs, author, diff_provider)`
   - Returns ready-to-use ReviewEngine

**Special Handling:**
- **Added/Untracked Files**: Old content = empty string (special method `create_semantic_reviewable_diffs_for_added_file`)
- **Line Range Extraction**: Uses `extract_line_range_from_core_diff()` which:
  - Inspects boundary node's change status (Unchanged/Added/Deleted/Modified/Moved/Reordered)
  - Selects appropriate source provider (old or new)
  - Calls `source_provider.line_range(node)` to get actual line range
  - Converts core LineRange to review layer LineRange

**ReviewableDiffId Generation:**
- Format: `{file_path}#{index}` for uniqueness (e.g., "src/auth.rs#0", "src/auth.rs#1")
- Index ensures multiple semantic pairs from same file get unique IDs
- LineRange extracted from core diff boundary node

### ReviewState (state/mod.rs) - CENTRAL STATE CONTAINER

**Structure:**
```rust
pub struct ReviewState {
    reviewable_diffs: BTreeMap<ReviewableDiffId, ReviewableDiff>,
    approvals: ReviewApprovals,
    instructions: ReviewInstructions,
    decisions: ReviewDecisions,
    decision_approvals: DecisionApprovals,
    journey: ReviewJourney,
    author: String,
    session_metadata: Option<SessionMetadata>,
}
```

**Relationship with ReviewableDiff:**
- BTreeMap ensures ordered iteration by file path → query → line range
- ReviewableDiff = wrapper around core ReviewableDiff with review metadata (id, file_path)
- Each ReviewableDiff has unique ReviewableDiffId that serves as key for all review operations

**Query Methods:**
- `is_approved(&ReviewableDiffId)` - check approval status
- `get_instructions(&ReviewableDiffId)` - get instructions for specific diff
- `approval_progress()` - (approved, total, percentage)
- `get_reviewable_diff(&ReviewableDiffId)` - retrieve specific diff
- `get_reviewable_diffs_by_file()` - group diffs by file path
- `get_ordered_reviewable_ids()` - ordered iteration of all IDs

**Mutation Methods (return &mut Self for chaining):**
- `approve(ReviewableDiffId, reviewer)` - approve a diff
- `unapprove(&ReviewableDiffId)` - remove approval
- `add_instruction(Instruction)` - add instruction
- `approve_all_in_file(file_path, reviewer)` - bulk approve file

### ReviewableDiff Wrapper (state/mod.rs)

**Purpose:** Review layer wrapper around core ReviewableDiff from diffviz-core.

**Structure:**
```rust
pub struct ReviewableDiff {
    pub id: ReviewableDiffId,           // Review layer identifier
    pub core_diff: CoreReviewableDiff,  // Core semantic diff from diffviz-core
    pub file_path: String,              // Convenience field for review layer
}
```

**Key Methods:**
- `language()` - get programming language
- `total_changes()` - get change count
- `boundary_name()` - display name for boundary node

**Relationship to ReviewableDiffId:**
- ReviewableDiffId is constructed from: DiffQuery + file_path + LineRange
- LineRange comes from core_diff.boundary node's position in source
- file_path with #index suffix ensures uniqueness when multiple diffs from same file

### ReviewableDiffId (entities/reviewable_diff_id.rs)

Universal identifier for review items. Triplet of:
- `query: DiffQuery` - what comparison (HEAD..unstaged, commit..commit, etc.)
- `file_path: String` - which file (includes #index for uniqueness)
- `line_range: LineRange` - which lines (1-based start/end, 0-based columns)

Implements Ord for BTreeMap ordering (file → query → line range). Use `same_file_and_query()` to check overlap candidates.

### Decision System (entities/decision.rs) - CRITICAL WORKFLOW

**Decision Structure:**
```rust
pub struct Decision {
    number: u32,
    title: String,
    summary: String,
    decision_log_line: Option<usize>,
    code_impacts: Vec<CodeImpact>,
}

pub struct CodeImpact {
    file: String,
    line_ranges: Vec<DecisionLineRange>,
    change_type: ChangeType,
    confidence: Confidence,
    reasoning: String,
}
```

**ReviewDecisions Collection:**
```rust
pub struct ReviewDecisions {
    decisions: HashMap<u32, Decision>,
    decision_index: HashMap<ReviewableDiffId, Vec<u32>>,  // REVERSE INDEX
}
```

**The build_index_from_review_state() Method - CRITICAL:**

**Purpose:** Builds reverse index mapping ReviewableDiffId → decision numbers that affect it.

**Algorithm:**
1. Clear existing decision_index
2. Get all decision numbers, sort for consistent ordering
3. For each decision and its code_impacts:
   - For each ReviewableDiff in review_state.reviewable_diffs:
     - Check if diff.file_path == impact.file
     - Check if diff.line_range overlaps with any impact.line_ranges
     - If overlap: add decision_number to decision_index[reviewable_diff_id]
4. Result: decision_index maps each ReviewableDiffId to affecting decisions

**Overlap Detection:**
- Formula: `start1 <= end2 && start2 <= end1` (inclusive ranges)
- Allows: exact matches, partial overlaps, nested ranges
- Multiple decisions can affect same ReviewableDiffId

**Critical Workflow Sequence:**
```rust
// 1. Add all decisions
decisions.add_decision(decision1);
decisions.add_decision(decision2);

// 2. Build index - MUST happen before querying
decisions.build_index_from_review_state(&review_state);

// 3. Create Decision 0 for unmapped diffs (optional, only if unmapped exist)
decisions.create_unmapped_decision(&review_state);

// 4. Now can query
let affecting = decisions.get_decisions_for_diff(&reviewable_id);
```

**create_unmapped_decision() Method:**
- Finds ReviewableDiffs NOT in decision_index
- Only creates Decision 0 if unmapped diffs exist
- Adds all unmapped diffs to Decision 0's code_impacts
- Updates decision_index to map unmapped diffs to Decision 0

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

### ReviewEngine (engines/review_engine.rs)

Main business logic orchestrator. Contains:
- `state: ReviewState` - current review state
- `renderable_cache: HashMap<ReviewableDiffId, String>` - rendering cache (placeholder, would be RenderableDiff)
- `diff_provider: Box<dyn DiffProvider>` - git operations abstraction

**How ReviewEngine Creates ReviewableDiffs:**
ReviewEngine itself doesn't create ReviewableDiffs - that's done by ReviewEngineBuilder. ReviewEngine receives pre-built ReviewableDiffs via constructor:
```rust
ReviewEngine::new(reviewable_diffs: Vec<ReviewableDiff>, author, diff_provider)
```

Key behaviors:
- **Approve/reject**: invalidates renderable cache for affected ReviewableDiffId
- **Reverse cascade**: when all chunks for a decision are approved, auto-approve the decision
- **Instruction overlap auto-merge**: when overlap detected, extends to union range and concatenates content with separator
- **File hash tracking**: calculates file_content_hash and content_snapshot for staleness detection
- **Export/import**: JSON format with metadata for agent understanding (git context, query formats, examples)

Export/Import capabilities:
- Export scopes: SingleFile, SingleInstruction, All
- ExportedInstruction format includes: file, query, line_range, content, author, timestamp, status, file_content_hash, content_snapshot
- ImportSummary tracks: total_imported, active_count, stale_count, errors

### DiffProvider Trait (providers/mod.rs)

Interface for git operations needed by review layer:
- `get_changed_files(&DiffQuery) -> Vec<(String, FileStatus)>` - list files with changes
- `get_file_stats(&file, &DiffQuery) -> FileStats` - git diffstat (additions/deletions/total_changes)
- `get_source_code(&file, &GitRef) -> String` - file content at specific git ref

FileStatus enum: Added, Modified, Deleted, Renamed, Copied, Untracked
FileStats helpers: `is_creation()`, `is_deletion()`, `is_modification()`, `is_unchanged()`

Implemented by diffviz-git infrastructure layer. Use MockDiffProvider for testing.

### MockDiffProvider (providers/mock_provider.rs) - TEST INFRASTRUCTURE

**Two Creation Patterns:**

**Pattern 1: Manual Construction**
```rust
let mut provider = MockDiffProvider::new();
provider.add_file_content("file.rs", &GitRef::Head, "old content");
provider.add_file_content("file.rs", &GitRef::Unstaged, "new content");
```

**Pattern 2: Fixture-Based (PREFERRED for integration tests)**
```rust
// Loads all JSON fixtures from tests/fixtures/
let provider = MockDiffProvider::from_review_fixtures()?;
```

**ReviewFixture Structure:**
```rust
pub struct ReviewFixture {
    name: String,                    // Fixture identifier
    file_path: String,               // File path for this fixture
    language: String,                // Programming language
    description: String,             // What this fixture tests
    old_code: String,                // Before state
    new_code: String,                // After state
    expected_line_stats: LineStats,  // Expected +/- lines
    metadata: FixtureMetadata,       // complexity_level, tags
}
```

**How from_review_fixtures() Works:**
1. Scans `{CARGO_MANIFEST_DIR}/tests/fixtures/` for .json files
2. Deserializes each JSON into ReviewFixture
3. Populates `changed_files` with (file_path, FileStatus::Modified)
4. Stores fixtures in HashMap indexed by file_path
5. DiffProvider methods use fixtures to return old_code/new_code based on GitRef

**Usage Pattern:**
```rust
// Load all fixtures
let provider = MockDiffProvider::from_review_fixtures()?;

// Build ReviewEngine with fixture data
let builder = ReviewEngineBuilder::new(Box::new(provider), "test_author");
let engine = builder.build(DiffQuery::head_to_unstaged())?;

// All fixtures are now processed into ReviewableDiffs
let diffs = &engine.state().reviewable_diffs;
```

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
- File path includes #index suffix for uniqueness (e.g., "src/auth.rs#0")

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
│   │   └── mock_provider.rs  # Test provider with fixture loading
│   ├── review_engine_builder.rs  # Pipeline orchestration (git → core → review)
│   ├── errors.rs          # Structured errors (DiffVizError + ReviewError)
│   └── lib.rs             # Public API exports
├── tests/
│   ├── fixtures/          # JSON fixture files for testing
│   │   ├── *.json        # ReviewFixture format: name, file_path, language, old_code, new_code
│   ├── fixture_semantic_pair_validation.rs  # Validates all fixtures produce semantic pairs
│   └── semantic_pair_counter.rs            # Counts and analyzes semantic pairs per fixture
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
- Integration tests use `MockDiffProvider::from_review_fixtures()` for realistic data
- Fixture validation tests ensure all fixtures produce semantic pairs
- Avoid testing git/core integration - that's in higher layers

## Common Patterns

### Creating ReviewableDiffId
```rust
use crate::entities::reviewable_diff_id::{ReviewableDiffId, LineRange};
use crate::entities::git_ref::{DiffQuery, GitRef};

let id = ReviewableDiffId::new(
    DiffQuery::head_to_unstaged(),
    "src/main.rs#0".to_string(),  // Include #index for uniqueness
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

### Building ReviewEngine from Git Query (Full Pipeline)
```rust
use crate::review_engine_builder::ReviewEngineBuilder;
use crate::entities::git_ref::{DiffQuery, GitRef};

// Creates ReviewEngine with complete semantic analysis pipeline
let builder = ReviewEngineBuilder::new(diff_provider, "author".to_string());
let engine = builder.build(DiffQuery::head_to_unstaged())?;

// Engine now contains ReviewableDiffs from semantic analysis
let state = engine.state();
let diffs = &state.reviewable_diffs;
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

### Using MockDiffProvider with Fixtures (PREFERRED)
```rust
use crate::providers::mock_provider::MockDiffProvider;
use crate::review_engine_builder::ReviewEngineBuilder;
use crate::entities::git_ref::DiffQuery;

// Load all JSON fixtures from tests/fixtures/
let provider = MockDiffProvider::from_review_fixtures()
    .expect("Failed to load fixtures");

// Build ReviewEngine with fixture data
let builder = ReviewEngineBuilder::new(Box::new(provider), "test_author".to_string());
let engine = builder.build(DiffQuery::head_to_unstaged())
    .expect("Failed to build review engine");

// Access ReviewableDiffs created from fixtures
let diffs = &engine.state().reviewable_diffs;

// Group by file for analysis
let mut files: HashMap<String, Vec<_>> = HashMap::new();
for (id, diff) in diffs.iter() {
    let file = id.file_path.split('#').next().unwrap_or("unknown");
    files.entry(file.to_string()).or_default().push((id, diff));
}
```

### Using MockDiffProvider Manually
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

### Creating ReviewFixture JSON Files
```json
{
  "name": "rust_function_modification",
  "file_path": "src/example.rs",
  "language": "rust",
  "description": "Simple function modification to test semantic pairing",
  "old_code": "fn hello() {\n    println!(\"old\");\n}",
  "new_code": "fn hello() {\n    println!(\"new\");\n}",
  "expected_line_stats": {
    "additions": 1,
    "deletions": 1
  },
  "metadata": {
    "complexity_level": "simple",
    "tags": ["function", "modification"]
  }
}
```

## Known Patterns and Anti-patterns

**DO:**
- Use ReviewableDiffId for all review item identification
- Call `build_index_from_review_state()` after adding decisions
- Invalidate cache when state changes
- Return `&mut Self` from mutation methods
- Use DiffProvider for git operations
- Use `MockDiffProvider::from_review_fixtures()` for integration tests
- Include #index suffix in file_path for ReviewableDiffId uniqueness

**DON'T:**
- Don't use legacy chunk IDs or ad-hoc string identifiers
- Don't forget to build decision index (reverse lookup won't work)
- Don't skip `create_unmapped_decision()` (unmapped code becomes invisible)
- Don't expose internal HashMap - provide query methods
- Don't call git directly - use DiffProvider abstraction
- Don't manually construct ReviewableDiffs in tests - use ReviewEngineBuilder with fixtures

## Pipeline Flow Diagram

```
DiffProvider (git)
    ↓ get_changed_files(&DiffQuery)
    ↓ get_source_code(&file, &GitRef)
ReviewEngineBuilder
    ↓ Filter supported files
    ↓ For each file:
    ├─→ Parse AST (TreeSitter)
    ├─→ Build semantic tree (diffviz-core)
    ├─→ Build semantic pairs (change detection)
    ├─→ Convert to ReviewableDiffs (context expansion)
    └─→ Create ReviewableDiffId (file#index + line_range)
ReviewEngine
    ↓ ReviewState created with all ReviewableDiffs
    ↓ Decisions added
    ↓ build_index_from_review_state()
    ↓ create_unmapped_decision()
Ready for TUI/Review Operations
```

---

**Updated:** 2026-01-31 - Enhanced with detailed ReviewEngineBuilder pipeline flow, decision system index building, ReviewableDiff creation process, fixture-based testing patterns, and relationship between ReviewState/ReviewableDiff/ReviewableDiffId.
