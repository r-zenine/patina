# Context Document: Decision-Log Schema Ownership Solution

## Problem Statement

The decision-log schema definition is **split across two independently versioned artifacts**:

1. **Rust Struct** (source of truth for parsing)
   - Location: `diffviz-review/src/entities/decision.rs`
   - Changes via: Code review process
   - Authority: Serialization/deserialization behavior

2. **YAML Template** (what agents write)
   - Location: `agent-skills/skills/contribution-system/assets/templates/decision-log-template.yaml`
   - Changes via: Documentation updates
   - Authority: Agent instruction source

**Root cause of divergence:**
- No automated test validates template matches struct
- No process ensures both are updated together
- Different teams/repos maintain each
- Template is static documentation; struct is live code

**Impact when they diverge:**
- Agents write YAML following outdated template
- CLI tries to parse with current struct expectations
- Serde deserialization fails with cryptic error
- User workflow blocked until someone manually fixes template

**Historical pattern:** This has happened at least 3 times in recent commits:
- Missing required fields (commit `8a0b5b7`)
- Malformed files (commit `f61f187`)
- Field renames without template update (commit `602e96b`)

## Solution Architecture

### Core Insight: Invert Ownership

Instead of agents reading a static template, they query the **live schema** from diffviz:

```
OLD MODEL (Static Template):
  agent-skills/template.yaml → agents write YAML → CLI tries to parse
  (divergence risk)

NEW MODEL (Live Query):
  agents run: diffviz templates decision-log → get current YAML → fill in values
  (always synchronized)
```

### Three-Phase Approach Using Steel Thread Strategy

**Phase 1: Foundation - CLI Subcommands**
- Add `diffviz templates decision-log` command
- Add `diffviz validate decision-log` command
- Templates generated manually (no macro yet)
- Validation uses existing DecisionLog::parse()

**Phase 2: Expansion - Derive Macro**
- Implement `#[derive(SchemaTemplate)]` proc macro
- Auto-generate templates from struct definitions
- Update structs with rustdoc comments (become schema descriptions)
- Remove manual template generation

**Phase 3: Enhancement - Customization**
- Add optional `#[schema_template(...)]` attributes
- Customize example values per field
- Extend to other artifacts (context-handoff, design-doc, etc.)

### Why Derive Macro Solves Ownership

1. **Single Source of Truth**: Rust struct is the only schema definition
2. **Zero Manual Sync**: Change struct → template changes automatically
3. **Compile-Time Derivation**: Cannot diverge; they're derived together
4. **Inspectable**: Can read doc comments, serde attributes, field types
5. **Proven Pattern**: serde, schemars, other derive macros use this approach

## Key Decisions Made

**Requirement Clarifications (from AskUserQuestion):**

1. **Artifact Scope**: Decision-log only (MVP)
   - Why: Core ownership problem, highest ROI
   - Future: Extend to context-handoff, design-doc, etc.

2. **Output Formats**: YAML template only (MVP)
   - Why: Solves primary pain point (agents need template)
   - Future: Add markdown reference, JSON Schema export

3. **Skills Integration**: Direct command invocation
   - Agents run: `diffviz templates decision-log > decision-log.yaml`
   - Skills document this approach
   - No embedded/hidden invocation

4. **Breaking Changes**: Fail fast with clear error
   - Old YAML that doesn't match new schema → parse failure
   - Error message clearly explains what changed
   - Users manually update files (clean approach)

5. **Validation**: Include in Phase 1
   - `diffviz validate decision-log file.yaml` command
   - Uses same struct for validation
   - Enables early error detection

6. **Implementation Strategy**: Steel Thread
   - Phase 1: End-to-end working capability
   - Phase 2: Derive macro automation
   - Phase 3: Customization and extension
   - Each phase delivers value; can stop after Phase 1 if needed

## Architecture Design Decisions

### Decision 1: Where to Place Template Generation

**Options:**
- A) In CLI layer (main.rs) - direct YAML string generation
- B) In diffviz-review (business logic) - trait + implementations
- C) In proc-macro (compile-time generation)

**Selected: B (Phase 1) → C (Phase 2)**
- Phase 1: Manual implementation in diffviz-review module
- Phase 2: Replace with `#[derive(SchemaTemplate)]` macro
- Allows validation/testing without macro complexity
- Macro automatically replaces manual code

### Decision 2: Validation Approach

**Options:**
- A) Fail at parse time only (current behavior)
- B) Add explicit `diffviz validate` command
- C) Add JSON Schema validator with detailed errors
- D) All of the above (phased)

**Selected: B (Phase 1) + path to C (Phase 3)**
- Phase 1: Simple `diffviz validate` that uses DecisionLog::parse()
- Phase 2: Could extend with better error messages
- Phase 3: Export JSON Schema for external validators

**Rationale:**
- Phase 1 validation catches errors before running diffviz
- Reuses existing DecisionLog::parse() logic
- Provides user-friendly early feedback

### Decision 3: Backward Compatibility During Transition

**Pattern: Alias Migration**
- Keep `#[serde(alias = "base_commit")]` on commit field
- Old YAML using `base_commit` continues to work
- New templates use `commit`
- Eventually deprecate alias in major version

**For breaking changes:**
- Fail fast with clear error
- Include schema version in error message
- Document migration path in error

### Decision 4: Proc Macro Scope (Phase 2)

**What the macro inspects:**
- Struct and field names
- Doc comments (`///`)
- Serde attributes (`#[serde(...)]`)
- Type optionality (Option<T>)
- Nested struct composition

**What the macro generates:**
- YAML template with:
  - Proper field structure and nesting
  - Field comments from rustdoc
  - Proper optionality handling
  - Example/placeholder values

**What requires supplementary attributes:**
- Custom placeholder values
- Complex examples
- Business logic descriptions beyond doc comments

## Research Findings

### Derive Macro Feasibility: High (70-80%)

**Proven tools:**
- `syn` + `quote`: Standard Rust macro infrastructure
- `darling`: Attribute parsing (used by serde patterns)
- `schemars`: Generates JSON schemas from structs (reference implementation)

**What works well:**
- Inspecting field names, types, and serde attributes
- Detecting optionality (Option<T>)
- Reading rustdoc comments
- Generating YAML structure
- Handling nested structs (Vec<Decision>, etc.)

**Limitations:**
- Custom examples require `#[schema_template(example = "...")]` attributes
- Cannot inspect runtime values or business logic
- No automatic "smart" defaults

**Recommendation:**
- Phase 1 macro: Basic structure generation
- Phase 2 attributes: Custom examples and descriptions
- This matches effort vs. value curve

### Serde Attribute Inspection

The Decision struct uses serde attributes that the macro can inspect:

```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub rationale: Option<String>,

#[serde(alias = "base_commit")]
pub commit: Option<String>,
```

Macro can:
- Detect `default` → field is optional
- Detect `skip_serializing_if = "Option::is_none"` → field can be omitted
- Extract `alias` value → support old field names in template documentation
- Generate correct YAML optionality handling

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|-----------|
| **Proc-macro complexity delays Phase 2** | Medium | High | Phase 1 works standalone; Phase 2 is optimization |
| **New proc-macro crate adds maintenance** | Low | Medium | Standard tools (syn, quote); well-documented patterns |
| **Users expect auto-migration of old YAML** | Medium | Medium | Clear error messages document what changed |
| **Schema changes in future break Phase 1** | Low | Low | Phase 2 macro eliminates this risk entirely |
| **Validation catches edge cases we missed** | Medium | Low | Include comprehensive test fixtures |

## Success Criteria

### Phase 1 Success:
- ✅ `diffviz templates decision-log` outputs valid YAML template
- ✅ Agent can use output → fill in values → runs `diffviz validate decision-log file.yaml`
- ✅ Validation catches real errors (missing fields, wrong types)
- ✅ Error messages are clear and actionable
- ✅ Template includes all required fields with doc comments

### Phase 2 Success:
- ✅ `#[derive(SchemaTemplate)]` auto-generates template matching Phase 1 output
- ✅ Struct changes automatically update template
- ✅ No manual template file maintenance needed
- ✅ Zero divergence between struct and template

### Phase 3 Success:
- ✅ Custom examples work via `#[schema_template(...)]` attributes
- ✅ Extended to context-handoff and design-doc artifacts
- ✅ Skills use command instead of reading static templates

## What Changes for Users/Skills

### Before (Current State)
```
Skills document:
  "Use this template: [static YAML file in agent-skills]"
```

### After (All Phases)
```
Skills document:
  "Generate your template: diffviz templates decision-log > decision-log.yaml"

  Agents run this command once, get current schema, fill in values
```

### Error Handling Improvement

**Before:**
```
Failed to parse decision-log.yaml: missing field `number` at line 5
[Cryptic serde error]
```

**After (Phase 1+):**
```
Failed to parse decision-log.yaml: missing required field 'number'
Expected schema:
  decisions:
    - number: 1 (required)
      title: "..." (required)
      rationale: "..." (optional)
      code_impacts: [...]

Run 'diffviz templates decision-log' to see the full template.
```

## Dependencies Added

**Phase 1:**
- None (uses existing diffviz-review, serde, serde_yaml)

**Phase 2:**
- `syn` (AST parsing) - standard, widely used
- `quote` (code generation) - standard, widely used
- `proc-macro2` (token handling) - standard, widely used
- `darling` (attribute parsing) - ~20KB, no transitive bloat

**All are standard Rust macro dependencies**, well-maintained by ecosystem.

## Next Steps

1. **Review and Approve Plan** - Confirm Phase 1/2/3 approach
2. **Implement Phase 1** - CLI subcommands (foundation)
3. **Validate with Skills Team** - Confirm command invocation pattern works
4. **Implement Phase 2** - Derive macro (automation)
5. **Update Agent Skills** - Reference new command instead of static template
