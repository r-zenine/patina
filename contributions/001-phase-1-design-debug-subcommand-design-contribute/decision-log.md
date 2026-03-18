# Debug Subcommand - Design Decision Log

## Decisions

### D1: Full Pipeline Exposure (All 7 Phases)
**Decision:** Run all phases by default; use `--phase` flags for selective inspection.

**Rationale:** Agents may not know which phase caused unexpected behavior. Full transparency is the core value. Filtering optional for advanced use. JSON cost is negligible.

**Alternative rejected:** Start with minimal phases. Would require agents to discover which phase matters.

---

### D2: Line Range as Filter (Not Constraint)
**Decision:** Process entire file, then filter ReviewableDiffs to those overlapping the line range.

**Rationale:** Follows TUI code-impact processing logic. Lets agents see contextually-related diffs outside the exact range. Prevents silently dropping semantic relationships.

**Alternative rejected:** Pre-filter before processing. Would hide important context.

---

### D3: Git-Only Input
**Decision:** Accept only git refs + file paths; no stdin/inline code samples.

**Rationale:** Agents work with real diffs in real repos. Eliminates language inference. Reuses existing DiffProvider/ReviewEngineBuilder. Aligns with TUI workflow.

**Alternative rejected:** Support abstract code samples. Adds complexity; less useful.

---

### D4: JSON Output Default
**Decision:** Default to JSON; add `--human` flag for readable output.

**Rationale:** Agents parse JSON, not ANSI text. Structured output enables programmatic analysis. Can add formatters later.

**Alternative rejected:** Human-readable first. Would require agents to parse prose.

---

### D5: Minimal Fixture Export
**Decision:** Export only old_code, new_code, file_path, language in ReviewFixture format.

**Rationale:** Sufficient for test fixture creation. Tests rebuild semantic analysis (that's what we test). Keeps export small and focused.

**Alternative rejected:** Export intermediate phases. Would encourage testing pipeline internals instead of code.

---

### D6: --explain-folding Optional
**Decision:** Add folding explanations only when flag is passed.

**Rationale:** Keeps baseline output lean. Relevance scores already present in JSON. Complex logic worth deferring. Agents ask for it on demand.

**Alternative rejected:** Always include. Adds implementation complexity; not always needed.

---

### D7: Phase 1 AST as Summary
**Decision:** Show Tree-sitter AST as structure outline (not full dump).

**Rationale:** Full AST too verbose and low-level. Outline sufficient for parsing issues. Agents rarely care about raw AST; they care about semantic tree.

**Alternative rejected:** Full AST tree. Would bloat output; rarely useful.

---

## Architecture Alignment

- ✅ Reuses ReviewEngineBuilder (no pipeline duplication)
- ✅ Works through DiffProvider abstraction
- ✅ ReviewableDiff as contract with review layer
- ✅ Serde for domain type serialization
- ✅ Clean CLI composition in Environment pattern
