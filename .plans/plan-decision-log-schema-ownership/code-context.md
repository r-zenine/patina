# Code Context: Decision-Log Schema Ownership

## Problem Domain

Decision-log YAML schema currently lives in two places that diverge:
1. **Rust Struct** (`diffviz-review/src/entities/decision.rs`): Source of truth for parsing
2. **YAML Template** (`agent-skills/skills/contribution-system/assets/templates/decision-log-template.yaml`): What agents are instructed to write

When these diverge, agents write YAML that won't parse, causing user-facing failures.

**Historical incidents:**
- Commit `8a0b5b7`: Missing required `number` field in template
- Commit `f61f187`: Malformed decision-log files requiring rewrite
- Commit `602e96b`: Field rename (base_commit → commit) required manual updates to 6 files

## Key Classes & Structures

### Decision Entity Hierarchy

**Location:** `diffviz-review/src/entities/decision.rs`

**DecisionLog (Lines 40-49)**
```rust
pub struct DecisionLog {
    pub decisions: Vec<Decision>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "base_commit"  // Backward compat
    )]
    pub commit: Option<String>,
}
// Parse method: Line 54-56
pub fn parse(content: &str) -> Result<DecisionLog> {
    Ok(serde_yaml::from_str(content)?)
}
```

**Decision (Lines 30-37)**
```rust
pub struct Decision {
    pub number: u32,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    pub code_impacts: Vec<CodeImpact>,
}
```

**CodeImpact (Lines 22-27)**
```rust
pub struct CodeImpact {
    pub file: String,
    pub line_ranges: Vec<DecisionLineRange>,
    pub reasoning: String,
}
```

**DecisionLineRange (Lines 14-18)**
```rust
pub struct DecisionLineRange {
    pub start: usize,
    pub end: usize,
}
```

### CLI Entry Point

**Location:** `diffviz-cli/src/main.rs`

**Commands Enum (Lines 48-86)**
- Pattern: `#[derive(Subcommand)]` enum with variants
- Each variant maps to handler in `commands/` module
- Main flow (Line 218): `run_contribution_review()` reads decision-log.yaml and parses via `DecisionLog::parse()`

**Handler Pattern (Lines 9, 240-241)**
- Create handler struct implementing `CommandExecutor` trait
- Add match arm in main()
- Example: `ReviewCommand::new(...).execute(environment)`

### Error Handling

**Location:** `diffviz-review/src/errors.rs`
```rust
#[error("YAML parse error: {0}")]
YamlParse(#[from] serde_yaml::Error),
```

**CLI Integration (main.rs, Lines 167-168)**
```rust
let log = DecisionLog::parse(&content)
    .map_err(|e| anyhow::anyhow!("Failed to parse decision-log.yaml: {e}"))?;
```

## Architecture Patterns

### Serde Usage Pattern
- All decision structs use `#[derive(Serialize, Deserialize)]`
- Field-level attributes control optionality and serialization
- No custom serializers, purely declarative

### Backward Compatibility Pattern
```rust
#[serde(alias = "base_commit")]
pub commit: Option<String>,
```
Allows old YAML using `base_commit` field to parse as `commit`.

### CLI Subcommand Pattern
```
enum Commands { Variant { args } }
↓
struct VaraintCommand { fields }
impl CommandExecutor for VariantCommand { execute() }
↓
main() matches and calls command.execute(env)
```

## Dependencies

**Current decision-log related:**
- `serde` v1.0 (with derive feature)
- `serde_yaml` v0.9
- `anyhow` for error handling

**Needed for macro approach:**
- `syn` (AST parsing) - standard
- `quote` (code generation) - standard
- `proc-macro2` (token handling) - standard
- `darling` (attribute parsing) - ~20KB, no transitive cost

## Testing Infrastructure

**Location:** `diffviz-review/src/entities/decision.rs` (Lines 276-967)

**Test coverage includes:**
- YAML parsing with all fields
- Backward compatibility (`base_commit` alias)
- Invalid YAML rejection
- Decision index building with overlap detection
- Approval lifecycle
- Unmapped decision creation

**Test pattern:** Unit tests embedded in same file, use `create_test_decision()` and `create_test_reviewable_diff()` helpers

## Files to Modify/Create

| File | Change | Phase |
|------|--------|-------|
| `diffviz-review/src/entities/decision.rs` | Add rustdoc comments to structs/fields | Phase 1 |
| `diffviz-cli/src/commands/templates.rs` | New: Templates command handler | Phase 1 |
| `diffviz-cli/src/commands/validate.rs` | New: Validate command handler | Phase 1 |
| `diffviz-cli/src/commands/mod.rs` | Wire up new command handlers | Phase 1 |
| `diffviz-cli/src/main.rs` | Add Templates + Validate subcommands | Phase 1 |
| `diffviz-review/src/lib.rs` | Export new SchemaExportable trait | Phase 2 |
| `diffviz-schema-macro/src/lib.rs` | New crate: Implement #[derive(SchemaTemplate)] | Phase 2 |
| `diffviz-schema-macro/Cargo.toml` | New: Proc-macro crate setup | Phase 2 |
| Agent-skills templates | Update to reference `diffviz templates decision-log` | Phase 2 |

## Related Documentation

**Current schema documentation:**
- Template: `agent-skills/skills/contribution-system/assets/templates/decision-log-template.yaml`
- Docs: `agent-skills/skills/contribution-system/references/implementation-artifacts.md`
- SKILL docs: `agent-skills/skills/contribution-system/SKILL.md` (Lines 127-150)
- Onboarding: `diffviz-review/onboarding.md` (Lines 146-205)

**Will be deprecated/replaced by:**
- `diffviz templates decision-log` command output
- Validation via `diffviz validate decision-log`
