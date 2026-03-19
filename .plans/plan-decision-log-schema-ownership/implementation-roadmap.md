# Implementation Roadmap: Decision-Log Schema Ownership

## Strategy: Steel Thread (Foundation → Expansion → Enhancement)

Each phase delivers working end-to-end capability. Can stop after Phase 1 if needed.

---

## Phase 1: Foundation - CLI Subcommands (Manual Templates)

**Goal**: Working `diffviz templates` and `diffviz validate` commands. Agents can get current schema and validate files without macro implementation.

**Duration estimate**: Low (1-2 days)

### Phase 1.1: Prepare Structs with Documentation

**Objective**: Add rustdoc comments to Decision entities (will become schema descriptions in Phase 2)

**Work:**
1. Add rustdoc to `DecisionLog` struct
   - Explain overall purpose
   - Explain commit field: "Git hash of commit containing these code changes. Optional during strategy phase, required during implementation."

2. Add rustdoc to `Decision` struct
   - Explain: "An architectural decision and its code impacts"
   - Field docs for: number, title, rationale, code_impacts

3. Add rustdoc to `CodeImpact` struct
   - Explain: "How a single decision affects a specific file"
   - Field docs for: file, reasoning, line_ranges

4. Add rustdoc to `DecisionLineRange` struct
   - Explain: "Inclusive range of lines affected by decision"
   - Field docs for: start, end

**Files to modify:**
- `diffviz-review/src/entities/decision.rs` (Lines 14-49)

**Testing:**
- Run `cargo doc --package diffviz-review` to verify rustdoc renders correctly
- No test code changes needed

### Phase 1.2: Create Templates Module in diffviz-review

**Objective**: Implement manual schema export logic (will be replaced by macro in Phase 2)

**Create new file**: `diffviz-review/src/templates.rs`

**What it exports:**
```rust
pub trait SchemaTemplate {
    fn yaml_template() -> String;  // Returns YAML with placeholders
}

impl SchemaTemplate for DecisionLog {
    fn yaml_template() -> String {
        // Returns the YAML template
    }
}
```

**Manual template content:**
```yaml
# Decision Log - Schema Template
# Use this file to document architectural decisions made in this contribution.
# See diffviz-review/onboarding.md for detailed explanation.

commit: "git-hash-here"  # Git hash of commit containing these code changes
                         # Required during implementation, optional during strategy phase

decisions:
  # Each decision maps architectural choice to actual code changes
  - number: 1
    title: "[Decision made in one sentence]"
    rationale: "[Why this choice - constraints, priorities, trade-offs]"  # Optional
    code_impacts:
      # One or more files affected by this decision
      - file: "[path/to/file.rs]"
        reasoning: "[Why this file is affected by this decision]"
        line_ranges:
          # One or more line ranges in this file affected
          - start: 10
            end: 50

  - number: 2
    title: "[Next decision]"
    rationale: "[Rationale]"  # Optional
    code_impacts:
      - file: "[another/file.rs]"
        reasoning: "[Why affected]"
        line_ranges:
          - start: 100
            end: 150
```

**Testing:**
- Unit test: `SchemaTemplate::yaml_template()` parses correctly as DecisionLog
- Unit test: Template contains all required fields
- Unit test: Template structure matches struct hierarchy

**Files to create/modify:**
- Create: `diffviz-review/src/templates.rs`
- Modify: `diffviz-review/src/lib.rs` (export SchemaTemplate trait)

### Phase 1.3: Implement 'diffviz templates' Command

**Objective**: Create `diffviz templates decision-log` subcommand in CLI

**Create new file**: `diffviz-cli/src/commands/templates.rs`

**Handler structure:**
```rust
pub struct TemplatesCommand {
    artifact: String,  // "decision-log"
}

impl TemplatesCommand {
    pub fn new(artifact: String) -> Self { ... }
}

impl CommandExecutor for TemplatesCommand {
    fn execute(&self, _env: Environment) -> Result<()> {
        match self.artifact.as_str() {
            "decision-log" => {
                let template = DecisionLog::yaml_template();
                println!("{}", template);
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Unknown artifact: {}", self.artifact))
        }
    }
}
```

**CLI integration:**
1. Modify `diffviz-cli/src/main.rs`:
   - Add to `Commands` enum: `Templates { artifact: String }`
   - Add match arm in main(): `Commands::Templates { artifact } => TemplatesCommand::new(artifact).execute(env)?`

2. Modify `diffviz-cli/src/commands/mod.rs`:
   - Add `pub mod templates;`
   - Re-export `TemplatesCommand` if needed

**Testing:**
- Integration test: `diffviz templates decision-log` outputs valid YAML
- Integration test: Output parses as DecisionLog struct
- Output contains expected field names and structure

**Files to create/modify:**
- Create: `diffviz-cli/src/commands/templates.rs`
- Modify: `diffviz-cli/src/commands/mod.rs`
- Modify: `diffviz-cli/src/main.rs` (add command enum variant and handler)

### Phase 1.4: Implement 'diffviz validate' Command

**Objective**: Create `diffviz validate decision-log file.yaml` subcommand

**Create new file**: `diffviz-cli/src/commands/validate.rs`

**Handler structure:**
```rust
pub struct ValidateCommand {
    artifact: String,       // "decision-log"
    file: String,          // path to YAML file
}

impl CommandExecutor for ValidateCommand {
    fn execute(&self, _env: Environment) -> Result<()> {
        match self.artifact.as_str() {
            "decision-log" => {
                let content = std::fs::read_to_string(&self.file)?;
                match DecisionLog::parse(&content) {
                    Ok(_) => {
                        println!("✓ {} is valid", self.file);
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("✗ {} is invalid: {}", self.file, e);
                        eprintln!("\nRun: diffviz templates decision-log");
                        eprintln!("to see the expected schema.");
                        Err(e.into())
                    }
                }
            }
            _ => Err(anyhow::anyhow!("Unknown artifact: {}", self.artifact))
        }
    }
}
```

**CLI integration:**
1. Modify `diffviz-cli/src/main.rs`:
   - Add to `Commands` enum: `Validate { artifact: String, file: String }`
   - Add match arm in main()

2. Modify `diffviz-cli/src/commands/mod.rs`:
   - Add `pub mod validate;`

**Error handling improvement:**
- Catch serde_yaml::Error and provide actionable message
- Show what field is invalid
- Suggest running `diffviz templates decision-log` for reference

**Testing:**
- Test: Valid YAML file passes validation
- Test: Missing required field fails with clear error
- Test: Unknown field fails with clear error
- Test: Invalid YAML structure fails with clear error

**Files to create/modify:**
- Create: `diffviz-cli/src/commands/validate.rs`
- Modify: `diffviz-cli/src/commands/mod.rs`
- Modify: `diffviz-cli/src/main.rs` (add command enum variant and handler)

### Phase 1.5: Testing & Documentation

**Objective**: Validate Phase 1 works end-to-end, document for users

**Work:**
1. Create test fixtures:
   - Valid decision-log.yaml example
   - Invalid examples (missing fields, wrong types, etc.)
   - Edge cases (empty decisions, no code_impacts)

2. Integration tests in `diffviz-cli/tests/`:
   - Test `diffviz templates decision-log` output
   - Test `diffviz validate` with various inputs
   - Verify error messages are helpful

3. Update CLI onboarding documentation:
   - Document new subcommands
   - Show workflow: generate template → fill in → validate

4. Add error message improvements:
   - Better serde error context
   - Suggestions for fixing common mistakes

**Files to create/modify:**
- Create: `diffviz-cli/tests/templates_tests.rs` or add to existing
- Create: `diffviz-cli/tests/fixtures/` with YAML examples
- Modify: `diffviz-cli/onboarding.md` or README

### Phase 1 Completion Criteria

- ✅ `cargo build --workspace` succeeds with zero warnings
- ✅ `cargo test --package diffviz-review --package diffviz-cli` passes
- ✅ `diffviz templates decision-log` outputs valid YAML
- ✅ `diffviz validate decision-log file.yaml` validates correctly
- ✅ Error messages are clear and actionable
- ✅ Documentation updated explaining new commands
- ✅ No divergence: template from command matches Rust struct

---

## Phase 2: Expansion - Derive Macro (Automate Template Generation)

**Goal**: Replace manual template generation with `#[derive(SchemaTemplate)]` macro. Auto-generate from struct definitions.

**Duration estimate**: Medium (2-3 days)

### Phase 2.1: Create diffviz-schema-macro Crate

**Objective**: Set up new proc-macro crate

**Create new crate**: `diffviz-schema-macro/`

**Structure:**
```
diffviz-schema-macro/
├── Cargo.toml
└── src/
    └── lib.rs
```

**Cargo.toml setup:**
```toml
[package]
name = "diffviz-schema-macro"
version = "0.1.0"
edition = "2021"

[lib]
proc-macro = true

[dependencies]
proc-macro2 = "1.0"
quote = "1.0"
syn = { version = "2.0", features = ["full"] }
darling = "0.20"
```

**Add to workspace Cargo.toml:**
```toml
members = [
    ...,
    "diffviz-schema-macro",
]
```

**Testing:**
- Cargo check succeeds
- All dependencies resolve

**Files to create:**
- Create: `diffviz-schema-macro/Cargo.toml`
- Create: `diffviz-schema-macro/src/lib.rs`
- Modify: `/Cargo.toml` workspace members

### Phase 2.2: Implement SchemaTemplate Derive Macro

**Objective**: Create `#[derive(SchemaTemplate)]` that generates Phase 1 template code

**In `diffviz-schema-macro/src/lib.rs`:**

**Macro signature:**
```rust
#[proc_macro_derive(SchemaTemplate)]
pub fn derive_schema_template(input: TokenStream) -> TokenStream {
    // Parse struct
    // Inspect fields, doc comments, serde attributes
    // Generate yaml_template() implementation
    // Return generated code
}
```

**What the macro does:**
1. Parse input struct (using syn)
2. For each field:
   - Read rustdoc comment (becomes YAML comment)
   - Detect Option<T> → optional field
   - Read serde attributes (default, skip, alias)
   - Generate YAML template line with comment
3. For nested structs (Vec<T>):
   - Recursively process (Example: Vec<Decision> → generate decisions array structure)
4. Return generated impl SchemaTemplate for struct

**Output example for DecisionLog:**
```rust
impl SchemaTemplate for DecisionLog {
    fn yaml_template() -> String {
        r#"commit: "git-hash-here"  # Git hash of commit containing these code changes
decisions:
  - number: 1
    title: "[Decision made in one sentence]"
    rationale: "[Why this choice]"  # Optional
    code_impacts:
      - file: "[path/to/file.rs]"
        reasoning: "[Why affected]"
        line_ranges:
          - start: 10
            end: 50
"#.to_string()
    }
}
```

**Implementation details:**
- Use syn to parse TokenStream
- Use darling to extract attributes
- Use quote! to generate code
- Handle Option<T> for optionality
- Handle Vec<T> for arrays
- Handle nested structs

**Testing:**
- Unit tests in diffviz-schema-macro/src/lib.rs
- Test with mock structs
- Verify generated YAML structure correct
- Verify doc comments included

**Files to modify:**
- Create: `diffviz-schema-macro/src/lib.rs` (full macro impl)

### Phase 2.3: Integrate Macro into diffviz-review

**Objective**: Use macro instead of manual template implementation

**Remove from diffviz-review/src/lib.rs:**
- Remove manual `SchemaTemplate` implementation for DecisionLog

**Add to DecisionLog struct in decision.rs:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize, SchemaTemplate)]
pub struct DecisionLog {
    ...
}
```

**Add dependency to Cargo.toml:**
```toml
[dependencies]
diffviz-schema-macro = { path = "../diffviz-schema-macro" }
```

**Testing:**
- `cargo build --package diffviz-review` succeeds
- `cargo test --package diffviz-review` passes (existing tests still work)
- `diffviz templates decision-log` output matches Phase 1 output exactly

**Files to modify:**
- Modify: `diffviz-review/src/entities/decision.rs` (add derive attribute)
- Modify: `diffviz-review/Cargo.toml` (add dependency)
- Delete: `diffviz-review/src/templates.rs` (manual impl no longer needed)
- Modify: `diffviz-review/src/lib.rs` (remove manual impl exports)

### Phase 2.4: Validation & Testing

**Objective**: Ensure macro-generated templates match Phase 1

**Work:**
1. Compare outputs:
   - Run Phase 1 template logic manually
   - Run macro-generated template
   - Assert they're identical

2. Test struct changes automatically update template:
   - Add new field to Decision struct
   - Regenerate (cargo build)
   - Template automatically includes new field
   - No manual template update needed

3. Test serde attribute inspection:
   - Change `#[serde(default)]` on a field
   - Verify template generation respects this
   - Test `skip_serializing_if` handling
   - Test `alias` handling

4. Backward compatibility:
   - Old YAML with `base_commit` still parses
   - Error message helpful for new schema

**Testing:**
- Integration test: macro output is valid YAML
- Integration test: macro output parses as DecisionLog
- Integration test: adding field to struct updates template
- All existing tests pass

**Files to create/modify:**
- Create: `diffviz-review/tests/schema_template_tests.rs`
- Modify: `diffviz-cli/tests/templates_tests.rs` (verify macro output)

### Phase 2.5: Documentation & Skills Update

**Objective**: Document new approach, update agent-skills to use command

**Update agent-skills documentation:**

1. Update `agent-skills/skills/contribution-system/SKILL.md`:
   - Replace inline schema example with: "Run `diffviz templates decision-log` to see current schema"
   - Remove static decision-log-template.yaml reference

2. Update `agent-skills/skills/dev-contribute/reference.md`:
   - Step 2.1: "Generate decision-log template: `diffviz templates decision-log > decision-log.yaml`"
   - Step 4: "Fill in your template and save"
   - Step 4.5: "Validate with: `diffviz validate decision-log decision-log.yaml`"

3. Update `agent-skills/skills/contribution-system/references/implementation-artifacts.md`:
   - Schema section: "See `diffviz templates decision-log` for authoritative schema"
   - Remove hard-coded YAML example (or mark as reference only)

4. Consider deprecating `agent-skills/skills/contribution-system/assets/templates/decision-log-template.yaml`:
   - Mark as deprecated in comments
   - Note: Use `diffviz templates decision-log` instead
   - Keep for reference/backup only

**Testing:**
- Skills team reviews and confirms approach works for agents
- Test agent workflow: run command → fill template → validate

**Files to modify:**
- Modify: `agent-skills/skills/contribution-system/SKILL.md`
- Modify: `agent-skills/skills/dev-contribute/reference.md`
- Modify: `agent-skills/skills/contribution-system/references/implementation-artifacts.md`
- Modify: `agent-skills/skills/contribution-system/assets/templates/decision-log-template.yaml` (deprecation notice)

### Phase 2 Completion Criteria

- ✅ `#[derive(SchemaTemplate)]` macro implemented and tested
- ✅ Macro-generated template identical to Phase 1 manual template
- ✅ All existing tests pass
- ✅ Changing struct automatically updates template output
- ✅ Zero manual template maintenance needed
- ✅ Agent-skills documentation updated
- ✅ Skills tested with new command-based workflow

---

## Phase 3: Enhancement - Customization & Extension

**Goal**: Add advanced features (custom examples, extend to other artifacts)

### Phase 3.1: Attribute-Based Customization

**Objective**: Allow `#[schema_template(...)]` attributes for custom examples

**Example usage:**
```rust
#[derive(SchemaTemplate)]
pub struct Decision {
    pub number: u32,

    #[schema_template(example = "Add authentication middleware")]
    pub title: String,

    #[schema_template(example = "Middleware must validate tokens for security")]
    pub rationale: Option<String>,
}
```

**Macro enhancement:**
- Parse `#[schema_template(...)]` attributes
- Use custom example if provided; otherwise generate default `[placeholder]`
- Include custom example in YAML output

### Phase 3.2: Extend to context-handoff

**Objective**: Apply schema template pattern to context-handoff markdown

**Work:**
- Implement similar SchemaTemplate for context-handoff
- Add `diffviz templates context-handoff` command
- Update dev-contribute to use command instead of static template

### Phase 3.3: Extend to design-doc

**Objective**: Apply schema template pattern to design-doc

**Work:**
- Implement SchemaTemplate for design-doc
- Add `diffviz templates design-doc` command
- Update design-contribute to use command

### Phase 3.4: Schema Versioning & Migration

**Objective**: Track schema version, support migration

**Work:**
- Add version to template output
- Document schema changes per version
- Add optional `diffviz migrate` command for bulk conversions

---

## Implementation Order & Dependencies

### Phase 1 (No dependencies):
1. **Phase 1.1**: Add rustdoc to structs
2. **Phase 1.2**: Create templates module
3. **Phase 1.3**: Implement `diffviz templates` command
4. **Phase 1.4**: Implement `diffviz validate` command
5. **Phase 1.5**: Testing & documentation

### Phase 2 (Depends on Phase 1):
1. **Phase 2.1**: Create macro crate
2. **Phase 2.2**: Implement derive macro
3. **Phase 2.3**: Integrate into diffviz-review
4. **Phase 2.4**: Validation & testing
5. **Phase 2.5**: Documentation & skills update

### Phase 3 (Depends on Phase 2):
1. **Phase 3.1**: Attribute customization
2. **Phase 3.2**: Extend to context-handoff
3. **Phase 3.3**: Extend to design-doc
4. **Phase 3.4**: Schema versioning

---

## Success Metrics

**Phase 1:**
- Command works: `diffviz templates decision-log`
- Validation works: `diffviz validate decision-log file.yaml`
- Template matches struct
- No schema divergence

**Phase 2:**
- Struct changes → template changes automatically
- Zero manual template maintenance
- Macro-generated == Phase 1 output

**Phase 3:**
- Agents use `diffviz templates decision-log` for all artifacts
- Custom examples work via attributes
- Migration support available

---

## Risk & Mitigation

| Risk | Phase | Mitigation |
|------|-------|-----------|
| Phase 1 too tightly coupled to manual code | Phase 1 | Design Phase 1 templates.rs as Phase 2 template (to be replaced) |
| Macro doesn't generate correct YAML | Phase 2 | Comprehensive macro tests; Phase 1 as fallback |
| Rustdoc comments insufficient | Phase 3 | Add `#[schema_template(...)]` attributes as supplement |
| Agents don't adopt new workflow | Phase 2 | Work with skills team during Phase 2.5 |
| Breaking schema changes | All | Fail-fast with clear error; schema versioning in Phase 3 |

