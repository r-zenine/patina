# Architecture Context: Debug Subcommand Implementation

## Codebase Architecture Overview

The debug subcommand leverages DiffViz's clean architecture with clear separation of concerns across layers:

```
CLI Layer (diffviz-cli)
    ↓ CommandExecutor pattern
Review Layer (diffviz-review)
    ↓ ReviewEngineBuilder orchestrates 7-phase pipeline
    ↓ DiffProvider abstraction (trait-based)
Core Layer (diffviz-core)
    ↓ Semantic analysis (Tree-sitter, ReviewableDiff)
Infrastructure (diffviz-git, diffviz-llm, diffviz-utils)
    ↓ Git operations, LLM calls, utilities
```

The debug command will **reuse existing abstractions** rather than duplicate logic:
- ReviewEngineBuilder handles 7-phase orchestration
- DiffProvider handles git interaction
- Existing serialization patterns via serde
- CommandExecutor trait for CLI integration

---

## Key Types & Serialization Patterns

### 1. ReviewEngineBuilder: The Pipeline Orchestrator

**Location**: `diffviz-review/src/review_engine_builder.rs`

**What it does**: Takes Decisions (user-specified code impacts) and produces ReviewEngine containing semantic analysis results.

**Input**:
```rust
ReviewEngineBuilder::new(Box<dyn DiffProvider>, author: String)
    .build_from_decisions(decisions: Vec<Decision>, query: DiffQuery)
    → Result<ReviewEngine>
```

**Output**: ReviewEngine containing ReviewState with:
- `reviewable_diffs: BTreeMap<ReviewableDiffId, ReviewableDiff>` — core semantic diffs
- `decisions: ReviewDecisions` — associated decisions
- Other metadata (approvals, instructions, etc.)

**For the debug command**: You don't need to create Decisions manually. Instead:
1. Create minimal Decision with single CodeImpact for the file/line-range
2. Let ReviewEngineBuilder run phases 1-7
3. Extract ReviewState and serialize phases

**Phase Breakdown** (from codebase analysis):
- **Phase 1**: Decision input
- **Phase 2-3**: Build ReviewDecisions
- **Phase 4-5**: Get source code via DiffProvider, parse with Tree-sitter
- **Phase 6-7**: Create ReviewableDiff, wrap with metadata

---

### 2. DiffProvider: Git Abstraction

**Location**: `diffviz-review/src/providers/mod.rs` (trait), `diffviz-git/src/lib.rs` (impl: GitRepository)

**Key methods** (all you need):
```rust
pub trait DiffProvider {
    fn get_source_code(&self, file_path: &str, git_ref: &GitRef)
        → Result<String>;
    fn get_file_stats(&self, file_path: &str, query: &DiffQuery)
        → Result<FileStats>;
}
```

**GitRef enum**:
```rust
pub enum GitRef {
    Commit(String),  // SHA, tag, branch name
    Head,            // Current HEAD
    Staged,          // git add'ed changes
    Unstaged,        // Working directory
}
```

**For debug command**: Use `get_source_code()` to fetch old/new code for fixture export. DiffProvider is already injected via Environment.

---

### 3. CommandExecutor Pattern

**Location**: `diffviz-cli/src/commands/mod.rs` (trait), various commands implement it

**Pattern**:
```rust
pub trait CommandExecutor {
    fn execute(&self, environment: Environment) → Result<()>;
}

struct DebugCommand {
    file_path: String,
    from_ref: GitRef,
    to_ref: GitRef,
    // ... other flags
}

impl CommandExecutor for DebugCommand {
    fn execute(&self, environment: Environment) → Result<()> {
        // Access config, logger, diff_provider via environment
        // Perform operation
        // Output results (print JSON, etc.)
        Ok(())
    }
}
```

**In main.rs**:
```rust
#[derive(Subcommand)]
enum Commands {
    Debug {
        #[arg(long)]
        file: String,
        // ... other args
    },
}

// In match statement:
Commands::Debug { file, ... } => {
    let cmd = DebugCommand::new(file, ...);
    cmd.execute(environment)?;
}
```

---

### 4. Environment Pattern: Dependency Injection

**Location**: `diffviz-cli/src/environment.rs`

**What it provides**:
```rust
pub struct Environment {
    config: Config,  // Contains: author, repo_path, verbose, terminal_backend
    // Implicitly: diff_provider (GitRepository), logger, etc.
}
```

**Access from command**:
```rust
fn execute(&self, environment: Environment) → Result<()> {
    let author = environment.config.author.clone();
    let repo_path = environment.config.repo_path.clone();
    let verbose = environment.config.verbose;

    // Create DiffProvider for this repo
    let diff_provider = GitRepository::new(repo_path)?;

    // Use diff_provider...
}
```

**Key insight**: You don't create dependencies; Environment provides them. This enables testing and swapping implementations.

---

### 5. Serialization Patterns

The codebase uses **serde** throughout. Key patterns:

**Simple types with Serialize derive**:
```rust
#[derive(Serialize, Deserialize)]
pub struct ReviewableDiffId {
    pub query: DiffQuery,
    pub file_path: String,
    pub line_range: LineRange,
}

// Serializes naturally to JSON
let json = serde_json::to_string_pretty(&id)?;
```

**Complex types without Serialize** → Create wrapper structs:
```rust
// If DiffNode doesn't derive Serialize:
#[derive(Serialize)]
struct SerializableDiffNode {
    kind: String,
    start: usize,
    end: usize,
    relevance_score: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    explanation: Option<String>,
}

// Map domain → wrapper in serialize function
fn serialize_diff_node(node: &DiffNode) -> SerializableDiffNode {
    SerializableDiffNode {
        kind: format!("{:?}", node.semantic_kind),
        start: node.range.start,
        end: node.range.end,
        relevance_score: node.relevance_score,
        explanation: None,
    }
}
```

**Container pattern** (from ExportedInstructions):
```rust
#[derive(Serialize)]
pub struct DebugOutput {
    pub file_path: String,
    pub language: String,
    pub query: DiffQuery,
    pub line_range_filter: Option<LineRange>,
    pub phases: Phases,
    pub metadata: Metadata,
}

#[derive(Serialize)]
pub struct Phases {
    pub phase_2_semantic_tree: Option<Vec<SerializableNode>>,
    pub phase_3_semantic_pairs: Option<SemanticPairsOutput>,
    pub phase_4_reviewable_diffs: Option<Vec<ReviewableDiffOutput>>,
    // ...
}

// Serialize container:
let output = DebugOutput { ... };
println!("{}", serde_json::to_string_pretty(&output)?);
```

---

## Implementation Patterns from Codebase

### Reading source code from git

From DebugExpansion function (main.rs:237-346):
```rust
let old_source = diff_provider.get_source_code(&file_path, &GitRef::Head)?;
let new_source = diff_provider.get_source_code(&file_path, &GitRef::Unstaged)?;
```

### Creating ReviewableDiff

Also from DebugExpansion:
```rust
let core_diff = create_reviewable_diff_from_range(
    &old_source,
    &new_source,
    &file_path,
    language,
    LineRange { start: 1, end: 100 },
)?;
```

This function is in `diffviz-core` and returns semantic analysis result. Use this instead of duplicating logic.

### Detecting language from file

Pattern used throughout (review_engine_builder.rs:223-245):
```rust
fn get_language_parser_for_file(file_path: &str) -> Result<ProgrammingLanguage> {
    match std::path::Path::new(file_path)
        .extension()
        .and_then(|ext| ext.to_str())
    {
        Some("rs") => Ok(ProgrammingLanguage::Rust),
        Some("py") => Ok(ProgrammingLanguage::Python),
        Some("go") => Ok(ProgrammingLanguage::Go),
        Some("ts") | Some("tsx") => Ok(ProgrammingLanguage::TypeScript),
        Some("js") | Some("jsx") => Ok(ProgrammingLanguage::JavaScript),
        // ... more
        _ => Err(Error::UnsupportedLanguage),
    }
}
```

Reuse this function rather than reimplementing.

---

## Data Flow for Debug Command

### High-level flow:
```
1. Parse CLI args (--file, --from, --to, --line-range, etc.)
2. Validate: file exists, language supported, git refs valid
3. Create DiffQuery from --from/--to refs
4. Create minimal Decision for the file
5. Get DiffProvider from Environment
6. Call ReviewEngineBuilder::build_from_decisions()
   ↓ Returns ReviewEngine with ReviewState
7. Extract ReviewState
8. Filter reviewable_diffs by line_range (if provided)
9. Walk ReviewState phases and serialize to JSON
10. Apply --explain-folding (if requested)
11. Apply --human formatting (if requested)
12. Output or write to --export-fixture
```

### Code skeleton:
```rust
impl CommandExecutor for DebugCommand {
    fn execute(&self, environment: Environment) -> Result<()> {
        // 1. Validate inputs
        let language = detect_language(&self.file_path)?;
        validate_git_refs(&self.from_ref, &self.to_ref)?;

        // 2. Create DiffQuery
        let query = DiffQuery {
            from: self.from_ref.clone(),
            to: self.to_ref.clone(),
        };

        // 3. Get DiffProvider
        let diff_provider = GitRepository::new(environment.config.repo_path.clone())?;

        // 4. Build ReviewEngine
        let builder = ReviewEngineBuilder::new(Box::new(diff_provider), author);
        let decisions = vec![/* minimal decision for this file */];
        let engine = builder.build_from_decisions(decisions, query.clone())?;

        // 5. Extract and filter
        let mut state = engine.state;
        if let Some(range) = self.line_range {
            state.reviewable_diffs.retain(|_, diff| {
                diff.id.line_range.overlaps(&range)
            });
        }

        // 6. Serialize phases
        let output = serialize_phases(&state, &query)?;

        // 7-12. Apply flags and output
        if self.explain_folding {
            add_explanations(&mut output)?;
        }

        let final_output = if self.human {
            format_as_human(&output)?
        } else {
            serde_json::to_string_pretty(&output)?
        };

        println!("{}", final_output);

        if let Some(fixture_path) = &self.export_fixture {
            export_fixture(&state, &query, &self.file_path, fixture_path)?;
        }

        Ok(())
    }
}
```

---

## Critical Architectural Constraints

**From CLAUDE.md**:

1. **No fallbacks in core layer** — Fail fast, no defensive programming
2. **Zero warning rule** — All compiler + clippy warnings must be fixed
3. **Architecture rules enforced**:
   - diffviz-core: Tree-sitter only (no string/regex analysis)
   - No circular dependencies between layers
   - DiffProvider is the git abstraction boundary
4. **Technical and functional changes separate** — Don't refactor and feature-gate in same commit

**For debug command**:
- Don't add error handling for hypothetical scenarios
- Reuse existing abstractions (DiffProvider, ReviewEngineBuilder)
- Don't introduce new ways to do things if a pattern already exists
- Match existing code style and patterns

---

## File Locations Reference

| Component | Path | Key Items |
|-----------|------|-----------|
| ReviewEngineBuilder | `diffviz-review/src/review_engine_builder.rs` | `build_from_decisions()`, `get_language_parser_for_file()` |
| DiffProvider | `diffviz-review/src/providers/mod.rs` | Trait definition |
| GitRepository | `diffviz-git/src/lib.rs` | Implementation of DiffProvider |
| ReviewableDiff | `diffviz-review/src/entities/reviewable_diff_id.rs` | ReviewableDiffId struct |
| DiffQuery/GitRef | `diffviz-review/src/entities/git_ref.rs` | Query and ref enums |
| ReviewState | `diffviz-review/src/state/mod.rs` | Contains all review data |
| ReviewEngine | `diffviz-review/src/engines/review_engine.rs` | Wraps ReviewState |
| CommandExecutor | `diffviz-cli/src/commands/mod.rs` | Trait |
| CLI Entry | `diffviz-cli/src/main.rs` | Command registration |
| Environment | `diffviz-cli/src/environment.rs` | DI container |

---

## Implementation Precedents

**Similar functionality already exists**:

1. **DebugExpansion function** (main.rs:225-346)
   - Gets source code from DiffProvider
   - Calls semantic analysis (create_reviewable_diff_from_range)
   - Outputs structured data
   - Good reference for pattern

2. **ReviewStateFile loading/saving** (main.rs:113-166)
   - Demonstrates serde::to_string_pretty() and serde_json usage
   - Shows how to handle optional fields with `#[serde(default)]`

3. **ExportedInstructions** (review_engine.rs:116-121)
   - Container pattern for complex nested data
   - Custom serialization helpers with default values
   - Good reference for JSON output structure

Use these as templates; don't reinvent serialization or file I/O patterns.

---

## Testing Strategy Implications

**Where to test**:
- Unit tests in `diffviz-cli/src/commands/debug.rs` for input validation
- Integration tests in `diffviz-cli/tests/` for end-to-end CLI invocation
- Possibly tests in `diffviz-review/src/` if you enhance ReviewEngineBuilder for debug use

**What to test**:
- Invalid inputs (missing file, bad git ref, unsupported language)
- JSON schema validity (parse output with serde)
- Line-range filtering correctness (compare with TUI behavior)
- Fixture export format and content

**Don't test**:
- ReviewEngineBuilder internals (already tested elsewhere)
- DiffProvider implementation (already tested in diffviz-git)
- Semantic analysis (already tested in diffviz-core)

---

## No-Go Zone: Architecture Violations

**Don't do**:
- ❌ String-based code analysis (Tree-sitter only in diffviz-core)
- ❌ Circular dependencies (CLI → Review → Git/Core, not reverse)
- ❌ Bypass DiffProvider with direct git2 calls
- ❌ Add fallback/"safe" paths for error cases
- ❌ Duplicate ReviewEngineBuilder logic (reuse it)
- ❌ Create domain types without proper error variants (use thiserror)

**Do**:
- ✅ Reuse existing abstractions
- ✅ Follow CommandExecutor pattern
- ✅ Use serde for serialization
- ✅ Fail fast on invalid input
- ✅ Match existing code style
- ✅ Zero warnings guarantee

---

## Summary for Implementers

1. **Don't overthink it**: The command is mostly orchestration + serialization
2. **Reuse, don't reinvent**: ReviewEngineBuilder, DiffProvider, serde patterns already exist
3. **Follow patterns**: CommandExecutor, Environment injection, wrapper structs for serialization
4. **Test early**: Validate inputs, serialize output, test end-to-end
5. **Zero warnings**: Run clippy after every phase; fix immediately
6. **Architecture compliance**: Respect layer boundaries, don't add fallbacks, fail fast

The design is solid. The codebase has all the building blocks. Execution is straightforward orchestration + JSON output.
