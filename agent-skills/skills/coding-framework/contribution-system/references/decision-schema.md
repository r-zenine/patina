# Unified Decision Schema

All `decision-log.yaml` files (strategy, implementation, design) use the **same schema**, matching the `diffviz-review::Decision` struct:

```yaml
commit: null  # Strategy: null. Implementation: git hash of commit containing code changes.

decisions:
  - number: 1                          # Decision identifier (u32)
    title: "[One sentence summary]"    # What was decided (required)
    rationale: "[Why...]"              # Why this choice (optional)
    code_impacts: []                   # Code changes (empty for planning, populated for implementation)
      # - file: "path/to/file.rs"
      # - reasoning: "[Why affected]"
      # - line_ranges:
      #     - start: 10
      #       end: 50
```

## Key Insight: Code Impacts Distinguish Phases

The distinction between strategy, design, and implementation is the `code_impacts` field:

- **Strategy-level** (`dev-strategy` output): `code_impacts: []` — empty, decisions made before coding
- **Design-level** (design contributions): `code_impacts: []` — empty, design specs with no code yet
- **Implementation-level** (contributions): `code_impacts: [...]` — populated, decisions + actual code changes

Same YAML structure, different semantic meaning determined by phase.

## Why Unified?

The unified schema directly matches the `diffviz-review::Decision` struct in the codebase:

```rust
pub struct Decision {
    pub number: u32,
    pub title: String,
    pub rationale: Option<String>,
    pub code_impacts: Vec<CodeImpact>,
}
```

**Benefits:**
1. **Direct deserialization** — YAML parses straight to Rust struct, no translation
2. **Same structure everywhere** — reduces cognitive load; one schema to learn
3. **Progressive population** — strategy decisions start with `code_impacts: []`, implementation fills them in
4. **Testability** — the struct defines the contract; tests verify it works
5. **Traceability** — decisions flow from planning → implementation without reformatting
