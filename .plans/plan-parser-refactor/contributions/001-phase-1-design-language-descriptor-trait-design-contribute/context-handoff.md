# Context Handoff - Phase 1 Design

## What Problem Are We Solving

The 7 bespoke parsers share identical skeleton logic — only the data differs (node kind names, field
names, trivial token sets). This session resolved the core design question: **what API should a
`LanguageDescriptor` expose so a single `GenericSemanticTreeBuilder` can replace all 7 parsers?**
Design was deferred from planning to learn the real divergences between parsers before committing
to the trait shape.

## Design Overview

**Chosen approach:** Static data slices (kind maps, trivial sets, structural config) plus two
targeted override methods with sensible defaults: `extract_visibility` and `collect_metadata`.

**Key decisions:**
- Generic escape hatch rejected — targeted overrides keep language-specific logic visible and named
- Impl blocks are NOT a special case: mapping `impl_item` in `semantic_kind_map` fixes the
  classification bug; the generic builder handles it via byte-coverage traversal
- `GenericSemanticTreeBuilder<D>` implements `LanguageParser` — zero caller changes in Phase 1

**What was rejected:** a single `override_build() -> Option<Result>` hook (too open-ended).

## Reading Guide

Start with `design-doc.md` § "How It Works" for the full trait signature.
§ "Implementation Guidance" gives the exact files to create and the testing strategy.
`decision-log.yaml` explains the three key decisions (targeted overrides, impl_item, LanguageParser wrapper).
